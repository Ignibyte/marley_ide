//! Claude Code's hook events, for the rail (#519).
//!
//! Marley's plugin makes Claude Code print each hook event into its terminal as an OSC 777
//! notify titled [`marley_terminal::AGENT_EVENT_TITLE`]. `on_frame` folds each one into the
//! app's fleet snapshot, a seat per terminal view, and the rail reads the seat of each agent
//! row's terminal. A frame counts only while Claude Code is the terminal's foreground program,
//! so a `cat` of an old log moves no row.

use std::time::{SystemTime, UNIX_EPOCH};

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
}

/// A terminal view's seat id: its entity id, as `terminal_list` gives it.
fn seat_id(view: EntityId) -> String {
    view.as_u64().to_string()
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
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|since| u64::try_from(since.as_millis()).ok())
        .unwrap_or(0);
    let previous = cx
        .try_global::<AgentEvents>()
        .and_then(|events| events.snapshot.get(&seat));
    let events = claude_events::fold(&seat, previous, &event, now_ms);
    if events.is_empty() {
        return;
    }
    let agent_events = cx.default_global::<AgentEvents>();
    for event in &events {
        marley_fleet::apply(&mut agent_events.snapshot, event);
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
