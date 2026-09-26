//! Claude Code's hook events, for the rail (#519).
//!
//! Marley's plugin makes Claude Code print each hook event into its terminal as an OSC 777
//! notify titled [`marley_terminal::AGENT_EVENT_TITLE`]. `on_frame` folds each one into the
//! app's fleet snapshot, a seat per terminal view, and the rail reads the seat of each agent
//! row's terminal. A frame counts only while Claude Code is the terminal's foreground program,
//! so a `cat` of an old log moves no row. The MCP server serves the same snapshot as
//! `fleet_snapshot` (#547).

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gpui::{App, Context, EntityId, Global};
use marley_agent::AgentKind;
use marley_agent::claude_events;
use marley_fleet::{FleetSnapshot, Session, SessionEvent, State};
use terminal_view::TerminalView;

/// The Claude Code sessions Marley's terminals run, a seat per terminal view that has sent an
/// event. The rail observes it.
#[derive(Debug, Default)]
pub struct AgentEvents {
    snapshot: FleetSnapshot,
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

/// Now, in the fleet's epoch milliseconds.
pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|since| u64::try_from(since.as_millis()).ok())
        .unwrap_or(0)
}

/// Folds the body of a `marley-event` frame from `view`'s terminal into the view's seat, while
/// Claude Code is the terminal's foreground program.
pub(crate) fn on_frame(view: &TerminalView, body: &str, cx: &mut Context<TerminalView>) {
    if crate::agent_bar::agent_in(view.terminal().read(cx)) != Some(AgentKind::Claude) {
        return;
    }
    let event = match claude_events::decode(body) {
        Ok(event) => event,
        Err(error) => {
            log::debug!("a marley-event frame that is not an event: {error}");
            return;
        }
    };
    let seat = seat_id(cx.entity_id());
    let previous = cx
        .try_global::<AgentEvents>()
        .and_then(|events| events.snapshot.get(&seat));
    let events = claude_events::fold(&seat, previous, &event, now_ms());
    if events.is_empty() {
        return;
    }
    let agent_events = cx.default_global::<AgentEvents>();
    for event in &events {
        marley_fleet::apply(&mut agent_events.snapshot, event);
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
    let ts_ms = now_ms();
    let agent_events = cx.default_global::<AgentEvents>();
    for id in live {
        marley_fleet::apply(
            &mut agent_events.snapshot,
            &SessionEvent::Ended { id, ts_ms },
        );
    }
}

/// Forgets the seat of the terminal view `view`, which is closing.
pub(crate) fn forget(view: EntityId, cx: &mut App) {
    let seat = seat_id(view);
    let known = cx
        .try_global::<AgentEvents>()
        .is_some_and(|events| events.snapshot.get(&seat).is_some());
    if known {
        let agent_events = cx.default_global::<AgentEvents>();
        agent_events.snapshot = without(&agent_events.snapshot, &seat);
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
