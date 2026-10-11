//! Continue at Reset for a Codex its usage limit stopped (#715).
//!
//! A limited Codex seat (`codex_events::limit`) can be armed, from Needs you or by
//! `marley.continue_at_reset`, to continue once its limit resets: at the reset Marley sends the
//! lead thread `turn/start` with [`marley_agent::RESUME_PROMPT`] through the terminal's App Server
//! follow, which sends it only while the same limit still holds on an idle thread. An arm names
//! the connection's generation, the thread, the failed turn and the reset, and drops, sending
//! nothing, once the seat moves on: a turn starts, the thread waits, closes or changes, or the
//! follow or the terminal ends. Arms live in memory and are checked against the wall clock, so a
//! reset that passed while the machine slept fires at the next check, once.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use gpui::{App, EntityId, Global};
use marley_agent::codex_events;
use marley_fleet::Session;
use settings::Settings as _;

use crate::agent_events::now_ms;
use crate::{ContinueAtReset, MarleySettings};

/// The longest wait between two checks of the arms against the wall clock.
const CHECK_AT_MOST: Duration = Duration::from_secs(30);

/// A limit to continue past: the follow's generation, the lead thread, the turn the limit failed,
/// and when the limit resets.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Limited {
    pub(crate) generation: u64,
    pub(crate) thread: String,
    pub(crate) turn: String,
    pub(crate) resets_at_ms: Option<u64>,
}

impl Limited {
    /// The limit `seat` stopped at, on the follow of `generation`; `None` while it is not limited.
    pub(crate) fn of(seat: &Session, generation: u64) -> Option<Self> {
        let limit = codex_events::limit(seat)?;
        Some(Self {
            generation,
            thread: seat.labels.get(codex_events::THREAD_LABEL)?.clone(),
            turn: limit.turn,
            resets_at_ms: limit.resets_at_ms,
        })
    }
}

/// The armed limits by terminal, the ones already continued or cancelled, which the setting never
/// arms again, and whether a check is running.
#[derive(Default)]
pub(crate) struct Arms {
    armed: HashMap<EntityId, Limited>,
    handled: HashSet<(EntityId, Limited)>,
    checking: bool,
}

impl Global for Arms {}

pub(crate) fn init(cx: &mut App) {
    cx.set_global(Arms::default());
}

/// When `terminal`'s armed limit continues, in milliseconds since the epoch; `None` while nothing
/// is armed.
pub(crate) fn armed_at(terminal: EntityId, cx: &App) -> Option<u64> {
    cx.try_global::<Arms>()?.armed.get(&terminal)?.resets_at_ms
}

/// Arms `terminal` to continue past `limited` at its reset; a limit with no reset is not armed.
pub(crate) fn arm(terminal: EntityId, limited: Limited, cx: &mut App) {
    let Some(reset) = limited.resets_at_ms else {
        return;
    };
    log::info!("limits: the Codex of {terminal:?} continues at {reset}");
    let arms = cx.global_mut::<Arms>();
    let _previous = arms.armed.insert(terminal, limited);
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
        if let Some(limited) = arms.armed.remove(&terminal) {
            let _new = arms.handled.insert((terminal, limited));
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
pub(crate) fn continue_now(terminal: EntityId, limited: Limited, cx: &mut App) {
    cancel(terminal, cx);
    send(terminal, limited, cx);
}

/// After each fold of `terminal`'s Codex seat on the follow of `generation`: an arm the seat no
/// longer matches drops, and with `marley.continue_at_reset` on, a newly limited seat with a known
/// reset is armed.
pub(crate) fn on_seat(terminal: EntityId, seat: Option<&Session>, generation: u64, cx: &mut App) {
    let limited = seat.and_then(|seat| Limited::of(seat, generation));
    let Some(arms) = cx.try_global::<Arms>() else {
        return;
    };
    let armed = arms.armed.get(&terminal);
    if armed.is_some_and(|armed| limited.as_ref() != Some(armed)) {
        cancel(terminal, cx);
    }
    let Some(limited) = limited else {
        return;
    };
    let arms = cx.global::<Arms>();
    let fresh =
        !arms.armed.contains_key(&terminal) && !arms.handled.contains(&(terminal, limited.clone()));
    if fresh
        && limited.resets_at_ms.is_some()
        && MarleySettings::get_global(cx).continue_at_reset == ContinueAtReset::Always
    {
        arm(terminal, limited, cx);
    }
}

/// Sends the continuation through `terminal`'s follow and remembers it, so the setting never arms
/// the same limit again.
fn send(terminal: EntityId, limited: Limited, cx: &mut App) {
    crate::codex_server::continue_lead(terminal, &limited, cx);
    let _new = cx.global_mut::<Arms>().handled.insert((terminal, limited));
}

/// Checks the arms again at the earliest reset, or in [`CHECK_AT_MOST`].
fn check_later(cx: &App) {
    let now = now_ms();
    let wait = cx
        .global::<Arms>()
        .armed
        .values()
        .filter_map(|limited| limited.resets_at_ms)
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
    let due: Vec<(EntityId, Limited)> = cx
        .global::<Arms>()
        .armed
        .iter()
        .filter(|(_, limited)| limited.resets_at_ms.is_some_and(|reset| reset <= now))
        .map(|(terminal, limited)| (*terminal, limited.clone()))
        .collect();
    for (terminal, limited) in due {
        let _armed = cx.global_mut::<Arms>().armed.remove(&terminal);
        send(terminal, limited, cx);
    }
    if cx.global::<Arms>().armed.is_empty() {
        cx.global_mut::<Arms>().checking = false;
    } else {
        check_later(cx);
    }
}
