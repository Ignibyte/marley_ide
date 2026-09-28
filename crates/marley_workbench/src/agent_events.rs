//! Claude Code's hook events, for the rail (#519).
//!
//! Marley's plugin makes Claude Code print each hook event into its terminal as an OSC 777
//! notify titled [`marley_terminal::AGENT_EVENT_TITLE`]. `on_frame` folds each one into the
//! app's fleet snapshot, a seat per terminal view, and the rail reads the seat of each agent
//! row's terminal. A frame counts only while Claude Code is the terminal's foreground program,
//! so a `cat` of an old log moves no row. The MCP server serves the same snapshot as
//! `fleet_snapshot` (#547).
//!
//! When the lead's turn stops, the stop kind (#566) says what the stop needs: the rules of
//! `marley_agent::stop_kind` settle the clear cases, and the System One layer is asked the rest.
//! With the use in `suggest` or `act` the answer lands on the seat as labels, which the rail's
//! row reads; `shadow` only logs. The user's next prompt logs the stop's outcome.

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gpui::{App, Context, EntityId, Global};
use marley_agent::AgentKind;
use marley_agent::claude_events::{
    self, HookEvent, MESSAGE_LABEL, PROMPT_ID_LABEL, PROMPT_LABEL, SESSION_LABEL, TurnFacts,
};
use marley_agent::stop_kind::{self, Kind, Source, StopKindShown};
use marley_fleet::{FleetSnapshot, Session, SessionEvent, State};
use marley_system_one::reading::{Reading, Signal};
use marley_system_one::state::Detail;
use settings::SystemOneMode;
use terminal_view::TerminalView;

use crate::system_one::{self, Asked, Asking};

/// The state's labels for the prompt's parts, in order.
const PART_LABELS: [&str; marley_system_one::MAX_PARTS] =
    ["part 1", "part 2", "part 3", "part 4", "part 5", "part 6"];

/// How soon a prompt after a stop that asked or was blocked counts as its answer, for the
/// outcome.
const ANSWERED_WITHIN_MS: u64 = 60_000;

/// The Claude Code sessions Marley's terminals run, a seat per terminal view that has sent an
/// event. The rail observes it.
#[derive(Debug, Default)]
pub struct AgentEvents {
    snapshot: FleetSnapshot,
    /// Each seat's last stop the System One layer logged, by seat, for the outcome its next
    /// prompt gives (#566).
    last_stops: HashMap<String, LastStop>,
}

/// A stop the layer logged.
#[derive(Debug)]
struct LastStop {
    /// The call's row.
    call: String,
    /// The Claude Code session it stopped in.
    session: Option<String>,
    /// When the stop's answer came, in the fleet's epoch milliseconds.
    at_ms: u64,
    /// The kind it read, when it read one.
    kind: Option<Kind>,
}

impl Global for AgentEvents {}

impl AgentEvents {
    /// The seat of the terminal view `view`, while its session has not ended.
    #[must_use]
    pub fn seat(&self, view: EntityId) -> Option<&Session> {
        self.snapshot
            .get(&seat_id(view))
            .filter(|session| session.state != State::Done)
    }

    /// Every seat, for the MCP server's `fleet_snapshot`.
    #[must_use]
    pub const fn snapshot(&self) -> &FleetSnapshot {
        &self.snapshot
    }

    /// How long until a working seat's row next changes at `now_ms` (#547): when it has gone
    /// `no_update_after_ms` without an event and reads `no update in N m`, then at each whole
    /// minute after, when N moves. `None` while no seat works, or with the form turned off.
    #[must_use]
    pub fn next_quiet_change(&self, now_ms: u64, no_update_after_ms: u64) -> Option<Duration> {
        if no_update_after_ms == 0 {
            return None;
        }
        self.snapshot
            .seats()
            .iter()
            .filter(|session| session.state == State::Working)
            .map(|session| {
                let quiet = now_ms.saturating_sub(session.last_event_ms);
                if quiet < no_update_after_ms {
                    no_update_after_ms - quiet
                } else {
                    60_000 - quiet % 60_000
                }
            })
            .min()
            .map(Duration::from_millis)
    }
}

/// A terminal view's seat id: its entity id, as `terminal_list` gives it.
fn seat_id(view: EntityId) -> String {
    view.as_u64().to_string()
}

/// Whether the Claude Code in the terminal `view` waits on the user: a permission or a question
/// (#508), which a paste into it would answer.
pub(crate) fn waiting(view: EntityId, cx: &App) -> bool {
    cx.try_global::<AgentEvents>()
        .and_then(|events| events.seat(view))
        .is_some_and(|seat| seat.state == State::Waiting)
}

/// Now, in the fleet's epoch milliseconds.
pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|since| u64::try_from(since.as_millis()).ok())
        .unwrap_or(0)
}

/// Folds the body of a `marley-event` frame from `view`'s terminal into the view's seat, while
/// Claude Code is the terminal's foreground program, and gives the seat's state before the frame
/// and the seat after it, for the push (#535).
pub(crate) fn on_frame(
    view: &TerminalView,
    body: &str,
    cx: &mut Context<TerminalView>,
) -> Option<(State, Session)> {
    if crate::agent_bar::agent_in(view.terminal().read(cx)) != Some(AgentKind::Claude) {
        return None;
    }
    let event = match claude_events::decode(body) {
        Ok(event) => event,
        Err(error) => {
            log::debug!("a marley-event frame that is not an event: {error}");
            return None;
        }
    };
    let seat = seat_id(cx.entity_id());
    let previous = cx
        .try_global::<AgentEvents>()
        .and_then(|events| events.snapshot.get(&seat));
    let before = previous.map_or(State::Starting, |session| session.state);
    let events = claude_events::fold(&seat, previous, &event, now_ms());
    if events.is_empty() {
        return None;
    }
    let agent_events = cx.default_global::<AgentEvents>();
    for event in &events {
        marley_fleet::apply(&mut agent_events.snapshot, event);
    }
    if let Some(after) = agent_events.snapshot.get(&seat).cloned() {
        // The stall watch (#569): the event is the outcome of a call waiting for one, and a
        // working seat is watched.
        crate::stall::moved(&after, &event.event, cx);
        if after.state == State::Working {
            crate::stall::watch(cx);
        }
        after_fold(view, &event, before, &after, cx);
    }
    cx.try_global::<AgentEvents>()
        .and_then(|events| events.snapshot.get(&seat))
        .cloned()
        .map(|after| (before, after))
}

/// The stop kind's part after `event` moved `seat` on from `before` (#566): the user's prompt
/// logs the outcome of the seat's last stop, and a lead `Stop`, or the interrupt that ended the
/// lead's turn, asks what the stop needs. The lead's turns open and close with it (#509).
fn after_fold(
    view: &TerminalView,
    event: &HookEvent,
    before: State,
    seat: &Session,
    cx: &mut Context<TerminalView>,
) {
    if event.agent_id.is_some() {
        return;
    }
    crate::turns::on_event(view, cx.entity_id().as_u64(), event, seat, cx);
    match event.event.as_str() {
        "UserPromptSubmit" => {
            let prompt = event.prompt.as_deref().unwrap_or_default();
            if !claude_events::is_harness_injected(prompt)
                && !claude_events::is_compact_continuation(prompt)
            {
                note_outcome(seat, prompt, cx);
            }
        }
        "Stop" => ask_stop_kind(view, seat, cx),
        // A lead tool's end may make a loop (#569).
        "PostToolUse" | "PostToolUseFailure" if seat.state == State::Working => {
            crate::stall::note_tool_end(view, seat, cx);
        }
        "PostToolUseFailure"
            if event.is_interrupt == Some(true)
                && before != State::Idle
                && seat.state == State::Idle =>
        {
            ask_stop_kind(view, seat, cx);
        }
        // A new session's first prompt answers nothing the last one stopped on.
        "SessionStart" => forget_last_stop(&seat.id, cx),
        _ => {}
    }
}

/// Forgets seat `seat`'s last stop, whose outcome no prompt will give.
fn forget_last_stop(seat: &str, cx: &mut App) {
    let known = cx
        .try_global::<AgentEvents>()
        .is_some_and(|events| events.last_stops.contains_key(seat));
    if known {
        let _forgotten = cx.default_global::<AgentEvents>().last_stops.remove(seat);
    }
}

/// Logs what the user's `prompt` says of `seat`'s last stop: how long it came after, how long it
/// is, and whether it answered a stop that asked or was blocked. A prompt in another session of
/// the same terminal answers nothing the last one stopped on.
fn note_outcome(seat: &Session, prompt: &str, cx: &mut App) {
    let Some(last) = cx
        .default_global::<AgentEvents>()
        .last_stops
        .remove(&seat.id)
    else {
        return;
    };
    if last.session.as_ref() != seat.labels.get(SESSION_LABEL) {
        return;
    }
    let after_ms = now_ms().saturating_sub(last.at_ms);
    let answered = last
        .kind
        .filter(|kind| matches!(kind, Kind::AsksYou | Kind::Blocked))
        .filter(|_| after_ms < ANSWERED_WITHIN_MS)
        .map(|kind| format!(", within a minute of {}", kind.words()))
        .unwrap_or_default();
    let outcome = format!(
        "next prompt after {}, {} characters{answered}",
        delay_words(after_ms),
        prompt.chars().count()
    );
    system_one::outcome(&last.call, outcome, cx);
}

/// A delay as an outcome says it: `12 s`, `4 min`, `2 h`.
fn delay_words(milliseconds: u64) -> String {
    let seconds = milliseconds / 1000;
    if seconds < 60 {
        format!("{seconds} s")
    } else if seconds < 3600 {
        format!("{} min", seconds / 60)
    } else {
        format!("{} h", seconds / 3600)
    }
}

/// Asks what seat `seat`'s stop needs, with the use on: the rules first, and the layer when they
/// cannot tell. The view's terminal names the project; its workspace is not being updated here,
/// since a terminal's event reaches the view outside any other entity's update.
fn ask_stop_kind(view: &TerminalView, seat: &Session, cx: &mut Context<TerminalView>) {
    let spec = marley_system_one::stop_kind(0);
    if system_one::use_mode(spec.name, cx) == SystemOneMode::Off {
        return;
    }
    let facts = TurnFacts::of(&seat.labels);
    let message = seat.labels.get(MESSAGE_LABEL).cloned();
    let verdict = stop_kind::rules(message.as_deref(), &facts);
    let kind = verdict.kind();
    // A stop with no message and no verdict leaves nothing to read.
    if kind.is_none() && message.is_none() {
        return;
    }
    let Some(workspace) = view.marley_workspace().upgrade() else {
        return;
    };
    let (folders, local) = system_one::project_of(workspace.read(cx), cx);
    let project = system_one::project_name(&folders);
    let mut asking = Asking {
        subject: seat.id.clone(),
        facts: std::iter::once(("project", project.clone()))
            .chain(stop_kind::state_facts(&facts, now_ms()))
            .collect(),
        texts: Vec::new(),
        verdict: None,
        project,
        folders,
        local,
    };
    let prompt = seat.labels.get(PROMPT_LABEL).cloned().unwrap_or_default();
    let parts = match system_one::detail(&asking, cx) {
        Ok(Detail::Full) if kind.is_none() => stop_kind::parts(&prompt),
        _ => Vec::new(),
    };
    asking.texts = [("prompt", Some(prompt)), ("message", message)]
        .into_iter()
        .filter_map(|(label, text)| {
            text.filter(|text| !text.is_empty())
                .map(|text| (label, text))
        })
        .chain(PART_LABELS.into_iter().zip(parts.iter().cloned()))
        .collect();
    let landing = Landing {
        seat: seat.id.clone(),
        session: seat.labels.get(SESSION_LABEL).cloned(),
        prompt_id: seat.labels.get(PROMPT_ID_LABEL).cloned(),
        stops: facts.stops,
        parts,
        facts,
    };
    if let Some(kind) = kind {
        asking.verdict = Some(system_one::choice_verdict("kind", kind.value()));
        let asked = system_one::record(*spec, &asking, cx);
        landing.land(&asked, cx);
        return;
    }
    let asked = system_one::ask(
        *marley_system_one::stop_kind(landing.parts.len()),
        &asking,
        cx,
    );
    cx.spawn(async move |_, cx| {
        let asked = asked.await;
        cx.update(|cx| landing.land(&asked, cx));
    })
    .detach();
}

/// Where a stop's answer lands, and what it is read against.
struct Landing {
    seat: String,
    session: Option<String>,
    prompt_id: Option<String>,
    stops: u32,
    parts: Vec<String>,
    facts: TurnFacts,
}

impl Landing {
    /// Keeps the stop for its outcome and, with the use in `suggest` or `act`, puts the kind and
    /// the parts not covered on the seat.
    fn land(&self, asked: &Asked, cx: &mut App) {
        let (kind, missing) = self.read(&asked.reading);
        // A refused or failed call read nothing, so what the user does next says nothing of it.
        if let Some(row) = asked.row.as_ref().filter(|_| !asked.reading.failed()) {
            let last = LastStop {
                call: row.id.clone(),
                session: self.session.clone(),
                at_ms: now_ms(),
                kind: kind.map(|(kind, _, _)| kind),
            };
            let _previous = cx
                .default_global::<AgentEvents>()
                .last_stops
                .insert(self.seat.clone(), last);
        }
        if !matches!(asked.mode, SystemOneMode::Suggest | SystemOneMode::Act) {
            return;
        }
        let labels = stop_kind::labels(kind, &missing);
        if !labels.is_empty() {
            land_stop_kind(
                &self.seat,
                self.prompt_id.as_deref(),
                self.stops,
                labels,
                cx,
            );
        }
    }

    /// The kind `reading` gives, held to what code saw (D1), and the parts it says the message
    /// does not cover. A refusal, no answer, or no signal gives neither.
    fn read(&self, reading: &Reading) -> (Option<(Kind, Source, f64)>, Vec<String>) {
        let (reads, source) = match reading {
            Reading::Rules(reads) => (reads, Source::Rules),
            Reading::Model(reads) => (reads, Source::Model),
            Reading::Off | Reading::Refused(_) | Reading::Unavailable(_) => {
                return (None, Vec::new());
            }
        };
        let mut kind = None;
        let mut missing = Vec::new();
        for read in reads {
            match &read.signal {
                Signal::Choice { option, confidence } if read.key == "kind" => {
                    kind = Kind::from_value(option).map(|kind| {
                        (
                            stop_kind::apply_evidence(kind, &self.facts),
                            source,
                            *confidence,
                        )
                    });
                }
                Signal::Noul { holds: false, .. } => {
                    if let Some(part) = part_number(&read.key)
                        .and_then(|number| number.checked_sub(1))
                        .and_then(|index| self.parts.get(index))
                    {
                        missing.push(part.clone());
                    }
                }
                _ => {}
            }
        }
        (kind, missing)
    }
}

/// The part a `part_N_done` question asks about, from 1.
fn part_number(key: &str) -> Option<usize> {
    key.strip_prefix("part_")?
        .strip_suffix("_done")?
        .parse()
        .ok()
}

/// Puts a stop's `labels` on seat `session` while it is still at that stop: idle, on the prompt
/// `prompt_id`, with no stop since (`stops`).
fn land_stop_kind(
    session: &str,
    prompt_id: Option<&str>,
    stops: u32,
    labels: Vec<(&'static str, String)>,
    cx: &mut App,
) {
    let _landed = land_labels(
        session,
        |seat| {
            seat.state == State::Idle
                && seat.labels.get(PROMPT_ID_LABEL).map(String::as_str) == prompt_id
                && TurnFacts::of(&seat.labels).stops == stops
        },
        labels,
        cx,
    );
}

/// Puts `labels` on seat `session` while `holds` says it is still where the answer found it, and
/// says whether they landed. An answer that comes later is dropped, since an `Upsert` would bring
/// back a seat the terminal forgot, or move one that went on. The upsert keeps the seat's own
/// time, so the landing is not an event of the agent's.
pub(crate) fn land_labels(
    session: &str,
    holds: impl FnOnce(&Session) -> bool,
    labels: Vec<(&'static str, String)>,
    cx: &mut App,
) -> bool {
    let Some(upsert) = cx
        .try_global::<AgentEvents>()
        .and_then(|events| events.snapshot.get(session))
        .filter(|seat| holds(seat))
        .map(|seat| {
            let mut kept = seat.labels.clone();
            kept.extend(
                labels
                    .into_iter()
                    .map(|(key, value)| (key.to_string(), value)),
            );
            SessionEvent::Upsert {
                id: seat.id.clone(),
                ts_ms: seat.last_event_ms,
                title: seat.title.clone(),
                state: seat.state,
                labels: kept,
                transport: seat.transport,
            }
        })
    else {
        return false;
    };
    marley_fleet::apply(&mut cx.default_global::<AgentEvents>().snapshot, &upsert);
    true
}

/// How the stop kind shows on an agent's row, from the use's mode (#566).
pub(crate) fn stop_kind_shown(cx: &App) -> StopKindShown {
    match system_one::use_mode(marley_system_one::stop_kind(0).name, cx) {
        SystemOneMode::Suggest => StopKindShown::Suggest,
        SystemOneMode::Act => StopKindShown::Act,
        SystemOneMode::Off | SystemOneMode::Shadow => StopKindShown::Hidden,
    }
}

/// Ends the seats of the terminals named by their ids, whose Claude Code has left the foreground
/// without a `SessionEnd` (#547). A seat that has ended, or failed, is left as it is: the reducer
/// keeps a failed seat failed, and ending it again would change nothing but notify the rail,
/// which would end it again.
pub(crate) fn end(terminals: &[u64], cx: &mut App) {
    let live: Vec<String> = terminals
        .iter()
        .map(u64::to_string)
        .filter(|seat| {
            cx.try_global::<AgentEvents>()
                .and_then(|events| events.snapshot.get(seat))
                .is_some_and(|session| !matches!(session.state, State::Done | State::Error))
        })
        .collect();
    if live.is_empty() {
        return;
    }
    let ended: Vec<u64> = live.iter().filter_map(|seat| seat.parse().ok()).collect();
    crate::turns::on_end(&ended, cx);
    let ts_ms = now_ms();
    let agent_events = cx.default_global::<AgentEvents>();
    for id in live {
        let _forgotten = agent_events.last_stops.remove(&id);
        marley_fleet::apply(
            &mut agent_events.snapshot,
            &SessionEvent::Ended { id, ts_ms },
        );
    }
}

/// Forgets the seat of the terminal view `view`, which is closing, and its turns (#509).
pub(crate) fn forget(view: EntityId, cx: &mut App) {
    crate::turns::forget(view.as_u64(), cx);
    let seat = seat_id(view);
    let known = cx
        .try_global::<AgentEvents>()
        .is_some_and(|events| events.snapshot.get(&seat).is_some());
    if known {
        let agent_events = cx.default_global::<AgentEvents>();
        agent_events.snapshot = without(&agent_events.snapshot, &seat);
        let _forgotten = agent_events.last_stops.remove(&seat);
    }
}

/// `snapshot` without the seat `id`. The reducer never removes a seat, so the others are folded
/// again from nothing, each as an upsert and its question.
fn without(snapshot: &FleetSnapshot, id: &str) -> FleetSnapshot {
    let events: Vec<SessionEvent> = snapshot
        .seats()
        .iter()
        .filter(|seat| seat.id != id)
        .flat_map(|seat| {
            let upsert = SessionEvent::Upsert {
                id: seat.id.clone(),
                ts_ms: seat.last_event_ms,
                title: seat.title.clone(),
                state: seat.state,
                labels: seat.labels.clone(),
                transport: seat.transport,
            };
            let question = seat
                .question
                .clone()
                .map(|question| SessionEvent::QuestionRaised {
                    id: seat.id.clone(),
                    ts_ms: seat.last_event_ms,
                    question,
                });
            std::iter::once(upsert).chain(question)
        })
        .collect();
    marley_fleet::reduce(FleetSnapshot::default(), &events)
}
