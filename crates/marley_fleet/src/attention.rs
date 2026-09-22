//! Read-time staleness + attention derivation. NO clock lives here — `now_ms` and `stale_after_ms` are
//! always injected, so a snapshot's attention is a pure function of (snapshot, now, threshold) and the
//! snapshot itself stays replay-deterministic.

use crate::reducer::FleetSnapshot;
use crate::session::{Session, State};

/// Why a seat is flagged for attention — its single HIGHEST reason. Priority = declaration order
/// (`Error` outranks `Question` outranks `Stale`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttentionReason {
    /// Stopped on an error — stopped AND silent (the 15-minute-unnoticed API-529 incident). Highest.
    Error,
    /// `Waiting` with a structured question — an articulated, bounded need.
    Question,
    /// Silent past the staleness threshold — suspicion, not declaration.
    Stale,
}

/// A seat needing attention, paired with its highest reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attention<'a> {
    /// The seat.
    pub session: &'a Session,
    /// Why it needs attention.
    pub reason: AttentionReason,
}

/// Is this seat stale? — not `Done`, and silent for at least `stale_after_ms`. A `Done` seat's silence
/// is legitimate and never flags. `saturating_sub` tolerates a minor `now < last_event` clock skew
/// without panicking.
///
/// `stale_after_ms` is a caller-owned threshold (no default is baked in — see the spec's
/// D-OPEN-STALE-DEFAULT). Note the degenerate `stale_after_ms == 0` flags EVERY non-`Done` seat
/// (silence of "at least 0ms" is always true); the consumer (e.g. #369's rail) is responsible for a
/// sensible positive value.
pub fn is_stale(session: &Session, now_ms: u64, stale_after_ms: u64) -> bool {
    session.state != State::Done && now_ms.saturating_sub(session.last_event_ms) >= stale_after_ms
}

/// The single highest attention reason for a seat, or `None` if it is calm.
fn reason_for(session: &Session, now_ms: u64, stale_after_ms: u64) -> Option<AttentionReason> {
    if session.state == State::Error {
        Some(AttentionReason::Error)
    } else if session.state == State::Waiting && session.question.is_some() {
        Some(AttentionReason::Question)
    } else if is_stale(session, now_ms, stale_after_ms) {
        Some(AttentionReason::Stale)
    } else {
        None
    }
}

/// The rank used to order the attention set: `Error` before `Question` before `Stale`. This MUST stay
/// in sync with `reason_for`'s branch priority (they currently agree); `AttentionReason` deliberately
/// does not derive `Ord`, so this fn is the single source of the sort order.
fn rank(reason: AttentionReason) -> u8 {
    match reason {
        AttentionReason::Error => 0,
        AttentionReason::Question => 1,
        AttentionReason::Stale => 2,
    }
}

/// The seats needing attention, each at its single highest reason, ordered `Error` → `Question` →
/// `Stale` with ties in snapshot (first-seen) order. `slice::sort_by_key` is a stable sort, so
/// within-reason first-seen order is preserved.
pub fn attention<'a>(
    snapshot: &'a FleetSnapshot,
    now_ms: u64,
    stale_after_ms: u64,
) -> Vec<Attention<'a>> {
    let mut out: Vec<Attention<'a>> = snapshot
        .seats()
        .iter()
        .filter_map(|session| {
            reason_for(session, now_ms, stale_after_ms).map(|reason| Attention { session, reason })
        })
        .collect();
    out.sort_by_key(|a| rank(a.reason));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reducer::{FleetSnapshot, SessionEvent, reduce};
    use crate::session::{Question, Transport};
    use std::collections::BTreeMap;

    fn upsert(id: &str, ts: u64, state: State) -> SessionEvent {
        SessionEvent::Upsert {
            id: id.into(),
            ts_ms: ts,
            title: id.into(),
            state,
            labels: BTreeMap::new(),
            transport: Some(Transport::Local),
        }
    }

    // Build a snapshot of (id, state, last_event_ms) seats via the reducer (seats is private).
    fn snap(seats: &[(&str, State, u64)]) -> FleetSnapshot {
        let events: Vec<SessionEvent> = seats
            .iter()
            .map(|(id, state, ts)| upsert(id, *ts, *state))
            .collect();
        reduce(FleetSnapshot::default(), &events)
    }

    // REQ-010: staleness boundary (>= at threshold, < below), Done never stale even when ancient,
    // clock-skew tolerance. Both boundary orientations per the two-sided-boundary lesson.
    #[test]
    fn t367_req010_staleness_boundaries() {
        let threshold = 500;
        let s = snap(&[("w", State::Working, 1000)]);
        let w = s.get("w").unwrap();
        assert!(is_stale(w, 1500, threshold)); // now-last == 500 >= 500 → stale
        assert!(!is_stale(w, 1499, threshold)); // 499 < 500 → not stale
        assert!(is_stale(w, 1501, threshold)); // 501 → stale
        assert!(!is_stale(w, 900, threshold)); // now < last → saturating_sub 0 → not stale

        // B5: a Done seat is never stale even far past the threshold.
        let sd = snap(&[("d", State::Done, 1000)]);
        assert!(!is_stale(sd.get("d").unwrap(), 100_000, threshold));
    }

    // REQ-011 + B5: attention orders Error → Question → Stale with ties in first-seen order, each
    // seat once. The fleet is built OUT of that order [stale, question, error, calm] to prove the
    // reorder (kills rank→0/→1), and the calm Idle seat is asserted ABSENT (kills reason_for's
    // Error-check ==→!= + covers the else-None branch).
    #[test]
    fn t367_req011_attention_reason_order() {
        let events = vec![
            upsert("stale", 100, State::Working),
            upsert("quest", 1000, State::Waiting),
            SessionEvent::QuestionRaised {
                id: "quest".into(),
                ts_ms: 1000,
                question: Question {
                    prompt: "p".into(),
                    options: vec![],
                    context_refs: vec![],
                },
            },
            upsert("err", 1000, State::Error),
            upsert("calm", 1000, State::Idle),
        ];
        let s = reduce(FleetSnapshot::default(), &events);
        let (now, threshold) = (1000, 500);
        let att = attention(&s, now, threshold);

        let expected = vec![
            Attention {
                session: s.get("err").unwrap(),
                reason: AttentionReason::Error,
            },
            Attention {
                session: s.get("quest").unwrap(),
                reason: AttentionReason::Question,
            },
            Attention {
                session: s.get("stale").unwrap(),
                reason: AttentionReason::Stale,
            },
        ];
        assert_eq!(att, expected); // full Vec<Attention> equality (exercises Attention's derive)
    }

    // B3 (inspect): a fresh Waiting seat with NO question is calm — attention() is empty (kills
    // attention.rs:40 &&→||, which would flag it as Question).
    #[test]
    fn t367_b3_fresh_waiting_without_question_is_calm() {
        let s = snap(&[("w", State::Waiting, 1000)]);
        assert!(attention(&s, 1000, 500).is_empty());
    }

    // A stale seat with no first-class reason still surfaces as Stale (covers the reason_for
    // fall-through and the Stale branch end-to-end).
    #[test]
    fn t367_stale_working_seat_surfaces_as_stale() {
        let s = snap(&[("w", State::Working, 0)]);
        let att = attention(&s, 1000, 500);
        assert_eq!(att.len(), 1);
        assert_eq!(att[0].reason, AttentionReason::Stale);
        assert_eq!(att[0].session.id, "w");
    }
}
