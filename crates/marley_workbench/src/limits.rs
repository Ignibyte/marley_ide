//! Continue at Reset for an agent its usage limit stopped (#715, #765).
//!
//! A limited seat (`marley_agent::usage::limit`) can be armed, from Needs you or by
//! `marley.continue_at_reset`, to continue once its limit resets with
//! [`marley_agent::RESUME_PROMPT`]: a Codex through its App Server follow, which sends `turn/start`
//! only while the same limit still holds on an idle thread (#715), and a Claude Code typed into its
//! terminal, as #750's Continue types it, only while its seat still reads the same limit (#765).
//! An arm names the agent, the connection's generation (Codex's), the session, the failed turn
//! and the reset, and drops, sending nothing, once the seat moves on: a turn starts, the agent
//! waits, its thread or session closes or changes, or the terminal ends. Arms live in memory and
//! are checked against the wall clock, so a reset that passed while the machine slept fires at the
//! next check, once.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use gpui::{App, EntityId, Global, WeakEntity};
use marley_agent::{AgentKind, claude_events, codex_events, usage};
use marley_fleet::Session;
use settings::Settings as _;
use terminal::Terminal;

use crate::agent_events::now_ms;
use crate::{ContinueAtReset, MarleySettings};

/// The longest wait between two checks of the arms against the wall clock.
const CHECK_AT_MOST: Duration = Duration::from_secs(30);

/// A limit to continue past: the agent, the Codex follow's generation, the session (Codex's
/// thread, Claude Code's session id), the turn the limit failed, and when the limit resets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Limited {
    pub(crate) agent: AgentKind,
    /// The Codex follow's generation; 0 for Claude Code, whose limit no connection carries.
    pub(crate) generation: u64,
    pub(crate) session: String,
    pub(crate) turn: String,
    pub(crate) resets_at_ms: Option<u64>,
}

/// What names one limit across its reset's readings: the generation, the session and the turn.
type LimitKey = (u64, String, String);

impl Limited {
    /// The limit a Codex `seat` stopped at, on the follow of `generation`; `None` while it is not
    /// limited.
    pub(crate) fn of_codex(seat: &Session, generation: u64) -> Option<Self> {
        let limit = usage::limit(seat)?;
        Some(Self {
            agent: AgentKind::Codex,
            generation,
            session: seat.labels.get(codex_events::THREAD_LABEL)?.clone(),
            turn: limit.turn,
            resets_at_ms: limit.resets_at_ms,
        })
    }

    /// The limit a Claude Code `seat` stopped at; `None` while it is not limited.
    pub(crate) fn of_claude(seat: &Session) -> Option<Self> {
        let limit = usage::limit(seat)?;
        Some(Self {
            agent: AgentKind::Claude,
            generation: 0,
            session: seat.labels.get(claude_events::SESSION_LABEL)?.clone(),
            turn: limit.turn,
            resets_at_ms: limit.resets_at_ms,
        })
    }

    fn key(&self) -> LimitKey {
        (self.generation, self.session.clone(), self.turn.clone())
    }
}

/// An armed limit, with the view whose seat holds it and the terminal it continues in.
struct Armed {
    limited: Limited,
    view: EntityId,
    terminal: WeakEntity<Terminal>,
}

/// The armed limits by terminal, the ones already continued or cancelled, which the setting never
/// arms again, and whether a check is running.
#[derive(Default)]
pub(crate) struct Arms {
    armed: HashMap<EntityId, Armed>,
    handled: HashSet<(EntityId, LimitKey)>,
    checking: bool,
}

impl Global for Arms {}

pub(crate) fn init(cx: &mut App) {
    cx.set_global(Arms::default());
}

/// When `terminal`'s armed limit continues, in milliseconds since the epoch; `None` while nothing
/// is armed.
pub(crate) fn armed_at(terminal: EntityId, cx: &App) -> Option<u64> {
    cx.try_global::<Arms>()?
        .armed
        .get(&terminal)?
        .limited
        .resets_at_ms
}

/// Arms `terminal`, whose view `view` holds the seat, to continue past `limited` at its reset; a
/// limit with no reset is not armed.
pub(crate) fn arm(terminal: WeakEntity<Terminal>, view: EntityId, limited: Limited, cx: &mut App) {
    let Some(reset) = limited.resets_at_ms else {
        return;
    };
    let id = terminal.entity_id();
    log::info!(
        "limits: the {} of {id:?} continues at {reset}",
        limited.agent.display_name()
    );
    let arms = cx.global_mut::<Arms>();
    let _previous = arms.armed.insert(
        id,
        Armed {
            limited,
            view,
            terminal,
        },
    );
    if !arms.checking {
        arms.checking = true;
        check_later(cx);
    }
}

/// Drops `terminal`'s arm, sending nothing; with the setting on, the same limit is not armed again.
pub(crate) fn cancel(terminal: EntityId, cx: &mut App) {
    if cx
        .try_global::<Arms>()
        .is_some_and(|arms| arms.armed.contains_key(&terminal))
    {
        log::info!("limits: the arm of {terminal:?} dropped; nothing sent");
        let arms = cx.global_mut::<Arms>();
        if let Some(armed) = arms.armed.remove(&terminal) {
            let _new = arms.handled.insert((terminal, armed.limited.key()));
        }
    }
}

/// Forgets `terminal`: its follow or the terminal itself ended, so its arm drops, sending nothing.
pub(crate) fn forget(terminal: EntityId, cx: &mut App) {
    cancel(terminal, cx);
    if cx
        .try_global::<Arms>()
        .is_some_and(|arms| arms.handled.iter().any(|(handled, _)| *handled == terminal))
    {
        cx.global_mut::<Arms>()
            .handled
            .retain(|(handled, _)| *handled != terminal);
    }
}

/// Continues `terminal` past `limited` now: Continue in Needs you, once the reset has passed.
pub(crate) fn continue_now(
    terminal: WeakEntity<Terminal>,
    view: EntityId,
    limited: Limited,
    cx: &mut App,
) {
    cancel(terminal.entity_id(), cx);
    send(
        &Armed {
            limited,
            view,
            terminal,
        },
        cx,
    );
}

/// After a seat of `terminal`'s view `view` moved, with the limit it now reads (`limited`): an arm
/// the seat no longer matches drops, and with `marley.continue_at_reset` on, a newly limited seat
/// with a known reset is armed.
pub(crate) fn on_seat(
    terminal: &WeakEntity<Terminal>,
    view: EntityId,
    limited: Option<Limited>,
    cx: &mut App,
) {
    let id = terminal.entity_id();
    let Some(arms) = cx.try_global::<Arms>() else {
        return;
    };
    let stale = arms.armed.get(&id).is_some_and(|armed| {
        limited
            .as_ref()
            .is_none_or(|limited| limited.key() != armed.limited.key())
    });
    if stale {
        cancel(id, cx);
    }
    let Some(limited) = limited else {
        return;
    };
    let arms = cx.global::<Arms>();
    let fresh = !arms.armed.contains_key(&id) && !arms.handled.contains(&(id, limited.key()));
    if fresh
        && limited.resets_at_ms.is_some()
        && MarleySettings::get_global(cx).continue_at_reset == ContinueAtReset::Always
    {
        arm(terminal.clone(), view, limited, cx);
    }
}

/// Sends the continuation, through the follow for Codex or typed for Claude Code, and remembers
/// it, so the setting never arms the same limit again.
fn send(armed: &Armed, cx: &mut App) {
    let id = armed.terminal.entity_id();
    match armed.limited.agent {
        AgentKind::Codex => crate::codex_server::continue_lead(id, &armed.limited, cx),
        _ => type_continuation(armed, cx),
    }
    let _new = cx
        .global_mut::<Arms>()
        .handled
        .insert((id, armed.limited.key()));
}

/// Types the resume prompt and Enter into a Claude Code's terminal, as #750's Continue does, while
/// its seat still reads the same limit; otherwise types nothing (#765).
fn type_continuation(armed: &Armed, cx: &mut App) {
    let still = crate::agent_events::seat_of(armed.view, AgentKind::Claude, cx)
        .and_then(Limited::of_claude)
        .is_some_and(|now| now.key() == armed.limited.key());
    let Some(terminal) = armed.terminal.upgrade().filter(|_| still) else {
        log::info!("limits: the limited Claude Code moved on; nothing typed");
        return;
    };
    log::info!("limits: continued the limited Claude Code");
    crate::terminal_drive::paste_then(
        &terminal,
        marley_agent::RESUME_PROMPT,
        |terminal| terminal.input(b"\r".to_vec()),
        cx,
    )
    .detach();
}

/// Checks the arms again at the earliest reset, or in [`CHECK_AT_MOST`].
fn check_later(cx: &App) {
    let now = now_ms();
    let wait = cx
        .global::<Arms>()
        .armed
        .values()
        .filter_map(|armed| armed.limited.resets_at_ms)
        .min()
        .map_or(CHECK_AT_MOST, |reset| {
            Duration::from_millis(reset.saturating_sub(now))
                .clamp(Duration::from_secs(1), CHECK_AT_MOST)
        });
    cx.spawn(async move |cx| {
        cx.background_executor().timer(wait).await;
        cx.update(fire_due);
    })
    .detach();
}

/// Sends each arm whose reset has passed by the wall clock, then checks again while any is left.
fn fire_due(cx: &mut App) {
    let now = now_ms();
    let due: Vec<EntityId> = cx
        .global::<Arms>()
        .armed
        .iter()
        .filter(|(_, armed)| armed.limited.resets_at_ms.is_some_and(|reset| reset <= now))
        .map(|(terminal, _)| *terminal)
        .collect();
    for terminal in due {
        if let Some(armed) = cx.global_mut::<Arms>().armed.remove(&terminal) {
            send(&armed, cx);
        }
    }
    if cx.global::<Arms>().armed.is_empty() {
        cx.global_mut::<Arms>().checking = false;
    } else {
        check_later(cx);
    }
}
