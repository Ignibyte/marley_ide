//! The generic event stream + the pure, idempotent fold that reduces it to a [`FleetSnapshot`].
//!
//! The reducer never reads a clock and never panics — an event naming an unknown seat auto-vivifies
//! a placeholder rather than dropping it (a live seat made invisible is the cardinal failure).

use crate::session::{Question, Session, State, Transport};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A generic event in the seat stream — the adapter projects a project's wire feed (e.g. UCSOS
/// `seat_events`) into these.
///
/// Every arm carries the `id` it addresses and the `ts_ms` epoch-millis at which it occurred.
/// Closed v1 set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SessionEvent {
    /// Create-or-replace a seat's descriptive fields (identity/title/labels/transport) and set its
    /// state. NOTE the question lifecycle is owned ONLY by [`SessionEvent::QuestionRaised`] /
    /// [`SessionEvent::QuestionCleared`]: an `Upsert` carries no question and, when it sets
    /// `state == Waiting`, PRESERVES any existing one — so a producer using `Upsert` as a periodic
    /// full-state refresh must clear a resolved question with an explicit `QuestionCleared` (the
    /// adapter contract for #368).
    Upsert {
        /// The seat this event addresses.
        id: String,
        /// Epoch-millis at which the event occurred.
        ts_ms: u64,
        /// The seat's human label.
        title: String,
        /// The seat's state after the upsert.
        state: State,
        /// The seat's opaque labels (defaults to empty when the wire omits them). Omitted from the
        /// serialized event when empty, matching [`crate::session::Session`]'s schema.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        labels: BTreeMap<String, String>,
        /// The seat's transport hint, if known. Omitted when `None`, matching `Session`'s schema.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        transport: Option<Transport>,
    },
    /// Move a seat to a new state.
    StateChange {
        /// The seat this event addresses.
        id: String,
        /// Epoch-millis at which the event occurred.
        ts_ms: u64,
        /// The seat's new state.
        state: State,
    },
    /// Atomically set `Waiting` + the question.
    QuestionRaised {
        /// The seat this event addresses.
        id: String,
        /// Epoch-millis at which the event occurred.
        ts_ms: u64,
        /// The question the seat is now blocked on.
        question: Question,
    },
    /// Clear the question, leaving state untouched (the seat's own next event moves state).
    QuestionCleared {
        /// The seat this event addresses.
        id: String,
        /// Epoch-millis at which the event occurred.
        ts_ms: u64,
    },
    /// A liveness ping — advances `last_event_ms` only.
    Heartbeat {
        /// The seat this event addresses.
        id: String,
        /// Epoch-millis at which the event occurred.
        ts_ms: u64,
    },
    /// The seat's process ended — `Done`, unless it was in `Error` (which is retained).
    Ended {
        /// The seat this event addresses.
        id: String,
        /// Epoch-millis at which the event occurred.
        ts_ms: u64,
    },
}

impl SessionEvent {
    /// The seat id this event addresses.
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Upsert { id, .. }
            | Self::StateChange { id, .. }
            | Self::QuestionRaised { id, .. }
            | Self::QuestionCleared { id, .. }
            | Self::Heartbeat { id, .. }
            | Self::Ended { id, .. } => id,
        }
    }

    /// The epoch-millis at which this event occurred.
    #[must_use]
    pub const fn ts_ms(&self) -> u64 {
        match self {
            Self::Upsert { ts_ms, .. }
            | Self::StateChange { ts_ms, .. }
            | Self::QuestionRaised { ts_ms, .. }
            | Self::QuestionCleared { ts_ms, .. }
            | Self::Heartbeat { ts_ms, .. }
            | Self::Ended { ts_ms, .. } => *ts_ms,
        }
    }
}

/// The reduced fleet — seats in first-seen order (a deterministic function of the stream, a stable
/// rail with no sort jumps). The single model every fleet surface reads.
///
/// **Invariant authority: the reducer.** A `FleetSnapshot` is produced by folding an event stream
/// through [`reduce`]/[`apply`], which maintains `question.is_some() ⇒ state == Waiting`, unique ids,
/// and first-seen order by construction. `Deserialize` exists for the wire round-trip (the tool schema)
/// and rebuilds whatever the bytes hold — it does NOT re-establish those invariants, so a
/// hand-authored / untrusted `{"seats":[…]}` could carry a `Waiting`-less question or a duplicate id.
/// In this design that never happens: the reducer is the sole producer and snapshots flow OUTWARD
/// (serialized to the rail / MCP client). A future untrusted-ingest path would add a `sanitize` step at
/// that seam rather than weaken the reducer (kept out of v1 to avoid an uncalled API).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetSnapshot {
    seats: Vec<Session>,
}

impl FleetSnapshot {
    /// The seats, in first-seen order.
    #[must_use]
    pub fn seats(&self) -> &[Session] {
        &self.seats
    }

    /// The seat with the given id, if present.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Session> {
        self.seats.iter().find(|s| s.id == id)
    }

    /// Index of the seat with the given id, if present.
    fn index_of(&self, id: &str) -> Option<usize> {
        self.seats.iter().position(|s| s.id == id)
    }
}

/// Fold one event into the snapshot in place. Idempotent under re-delivery (assignments + a `max` join),
/// so a cursor'd catch-up that overlaps already-applied events converges to the same snapshot.
pub fn apply(snapshot: &mut FleetSnapshot, event: &SessionEvent) {
    let id = event.id();
    let idx = if let Some(i) = snapshot.index_of(id) {
        i
    } else {
        // Auto-vivify: never drop an event for an unknown seat — a live seat made invisible is the
        // cardinal failure. A placeholder carries no render hint yet (transport `None`).
        snapshot.seats.push(Session {
            id: id.to_string(),
            title: id.to_string(),
            state: State::Starting,
            question: None,
            labels: BTreeMap::new(),
            capabilities: BTreeMap::new(),
            last_event_ms: 0,
            transport: None,
        });
        snapshot.seats.len() - 1
    };
    let seat = &mut snapshot.seats[idx];

    match event {
        SessionEvent::Upsert {
            title,
            state,
            labels,
            transport,
            ..
        } => {
            seat.title.clone_from(title);
            seat.state = *state;
            seat.labels = labels.clone();
            seat.transport = *transport;
            if *state != State::Waiting {
                seat.question = None;
            }
        }
        SessionEvent::StateChange { state, .. } => {
            seat.state = *state;
            if *state != State::Waiting {
                seat.question = None;
            }
        }
        SessionEvent::QuestionRaised { question, .. } => {
            seat.state = State::Waiting;
            seat.question = Some(question.clone());
        }
        SessionEvent::QuestionCleared { .. } => {
            seat.question = None;
        }
        SessionEvent::Heartbeat { .. } => {}
        SessionEvent::Ended { .. } => {
            seat.question = None;
            if seat.state != State::Error {
                seat.state = State::Done;
            }
        }
    }

    // Monotone max-join: an older re-delivered event never regresses the seat's last-event clock, and
    // minor cross-box skew cannot walk it backward.
    seat.last_event_ms = seat.last_event_ms.max(event.ts_ms());
}

/// Fold an ordered event stream into `snapshot`, returning the result.
///
/// Deterministic (a pure function of the starting snapshot and the stream) and replay-safe
/// (re-folding an already-applied suffix in full is a no-op).
#[must_use]
pub fn reduce(mut snapshot: FleetSnapshot, events: &[SessionEvent]) -> FleetSnapshot {
    for event in events {
        apply(&mut snapshot, event);
    }
    snapshot
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q() -> Question {
        Question {
            prompt: "p".into(),
            options: vec!["a".into()],
            context_refs: vec![],
        }
    }

    fn upsert(id: &str, ts: u64, state: State) -> SessionEvent {
        SessionEvent::Upsert {
            id: id.into(),
            ts_ms: ts,
            title: format!("title-{id}"),
            state,
            labels: BTreeMap::new(),
            transport: Some(Transport::Tmux),
        }
    }

    // REQ-001: the fold is a deterministic function of the stream.
    #[test]
    fn t367_req001_fold_is_deterministic() {
        let es = vec![
            upsert("a", 1, State::Working),
            SessionEvent::QuestionRaised {
                id: "a".into(),
                ts_ms: 2,
                question: q(),
            },
            upsert("b", 3, State::Idle),
        ];
        assert_eq!(
            reduce(FleetSnapshot::default(), &es),
            reduce(FleetSnapshot::default(), &es)
        );
    }

    // REQ-002: re-folding an already-applied suffix in full is a no-op (all six kinds).
    #[test]
    fn t367_req002_overlap_replay_is_noop() {
        let es = vec![
            upsert("a", 10, State::Working),
            SessionEvent::StateChange {
                id: "a".into(),
                ts_ms: 20,
                state: State::Waiting,
            },
            SessionEvent::QuestionRaised {
                id: "a".into(),
                ts_ms: 30,
                question: q(),
            },
            SessionEvent::QuestionCleared {
                id: "a".into(),
                ts_ms: 40,
            },
            SessionEvent::Heartbeat {
                id: "a".into(),
                ts_ms: 50,
            },
            SessionEvent::Ended {
                id: "a".into(),
                ts_ms: 60,
            },
        ];
        let once = reduce(FleetSnapshot::default(), &es);
        let twice = reduce(once.clone(), &es);
        assert_eq!(once, twice);
        assert!(!once.seats().is_empty());
    }

    // REQ-003: QuestionRaised atomically holds Waiting + the question.
    #[test]
    fn t367_req003_question_raised_atomic() {
        let s = reduce(
            FleetSnapshot::default(),
            &[
                upsert("a", 1, State::Working),
                SessionEvent::QuestionRaised {
                    id: "a".into(),
                    ts_ms: 2,
                    question: q(),
                },
            ],
        );
        let seat = s.get("a").unwrap();
        assert_eq!(seat.state, State::Waiting);
        assert_eq!(seat.question, Some(q()));
    }

    // REQ-004: a StateChange away from Waiting clears the question (kills reducer.rs:164).
    #[test]
    fn t367_req004_statechange_away_clears_question() {
        let s = reduce(
            FleetSnapshot::default(),
            &[
                SessionEvent::QuestionRaised {
                    id: "a".into(),
                    ts_ms: 1,
                    question: q(),
                },
                SessionEvent::StateChange {
                    id: "a".into(),
                    ts_ms: 2,
                    state: State::Working,
                },
            ],
        );
        let seat = s.get("a").unwrap();
        assert_eq!(seat.state, State::Working);
        assert_eq!(seat.question, None);
    }

    // B1 (inspect): an Upsert away from Waiting clears the question — kills reducer.rs:158, which
    // REQ-004 (StateChange path) does NOT exercise.
    #[test]
    fn t367_b1_upsert_away_from_waiting_clears_question() {
        let s = reduce(
            FleetSnapshot::default(),
            &[
                SessionEvent::QuestionRaised {
                    id: "a".into(),
                    ts_ms: 1,
                    question: q(),
                },
                upsert("a", 2, State::Working),
            ],
        );
        let seat = s.get("a").unwrap();
        assert_eq!(seat.state, State::Working);
        assert_eq!(seat.question, None);
    }

    // An Upsert that stays Waiting PRESERVES the question (F2/D5-consistent; covers the guard's
    // false branch on the Upsert arm).
    #[test]
    fn t367_upsert_waiting_preserves_question() {
        let s = reduce(
            FleetSnapshot::default(),
            &[
                SessionEvent::QuestionRaised {
                    id: "a".into(),
                    ts_ms: 1,
                    question: q(),
                },
                upsert("a", 2, State::Waiting),
            ],
        );
        assert_eq!(s.get("a").unwrap().question, Some(q()));
    }

    // REQ-005: Waiting-without-question is legal; QuestionCleared clears the question, state stays.
    #[test]
    fn t367_req005_waiting_without_question_and_cleared() {
        let s = reduce(
            FleetSnapshot::default(),
            &[
                upsert("a", 1, State::Working),
                SessionEvent::StateChange {
                    id: "a".into(),
                    ts_ms: 2,
                    state: State::Waiting,
                },
            ],
        );
        let seat = s.get("a").unwrap();
        assert_eq!(seat.state, State::Waiting);
        assert_eq!(seat.question, None);

        let s2 = reduce(
            FleetSnapshot::default(),
            &[
                SessionEvent::QuestionRaised {
                    id: "a".into(),
                    ts_ms: 1,
                    question: q(),
                },
                SessionEvent::QuestionCleared {
                    id: "a".into(),
                    ts_ms: 2,
                },
            ],
        );
        let seat2 = s2.get("a").unwrap();
        assert_eq!(seat2.state, State::Waiting);
        assert_eq!(seat2.question, None);
    }

    // REQ-006: Heartbeat advances last_event_ms (forward) via max-join and never regresses (backward);
    // state/question untouched. Uses ts 100/500 (not 0/1) to kill ts_ms→0 and ts_ms→1.
    #[test]
    fn t367_req006_heartbeat_max_join() {
        let s = reduce(
            FleetSnapshot::default(),
            &[
                upsert("a", 100, State::Working),
                SessionEvent::Heartbeat {
                    id: "a".into(),
                    ts_ms: 500,
                },
            ],
        );
        let seat = s.get("a").unwrap();
        assert_eq!(seat.last_event_ms, 500);
        assert_eq!(seat.state, State::Working);
        assert_eq!(seat.question, None);

        let s2 = reduce(
            s.clone(),
            &[SessionEvent::Heartbeat {
                id: "a".into(),
                ts_ms: 200,
            }],
        );
        assert_eq!(s2.get("a").unwrap().last_event_ms, 500);
    }

    // REQ-007: a non-Upsert event for an unknown id auto-vivifies a placeholder then applies. Also
    // kills reducer.rs:141 (the len()-1 index) and index_of mutants (a wrong index panics).
    #[test]
    fn t367_req007_unknown_id_autovivifies() {
        let s = reduce(
            FleetSnapshot::default(),
            &[SessionEvent::StateChange {
                id: "ghost".into(),
                ts_ms: 7,
                state: State::Working,
            }],
        );
        let seat = s.get("ghost").unwrap();
        assert_eq!(seat.title, "ghost");
        assert_eq!(seat.state, State::Working);
        assert!(seat.labels.is_empty());
        assert_eq!(seat.transport, None);
        assert_eq!(seat.last_event_ms, 7);
    }

    // REQ-008: Ended → Done, unless Error (retained); question cleared; seat kept.
    #[test]
    fn t367_req008_ended_done_unless_error() {
        let s = reduce(
            FleetSnapshot::default(),
            &[
                upsert("a", 1, State::Working),
                SessionEvent::Ended {
                    id: "a".into(),
                    ts_ms: 2,
                },
            ],
        );
        assert_eq!(s.get("a").unwrap().state, State::Done);
        assert_eq!(s.seats().len(), 1);

        let s2 = reduce(
            FleetSnapshot::default(),
            &[
                upsert("b", 1, State::Error),
                SessionEvent::Ended {
                    id: "b".into(),
                    ts_ms: 2,
                },
            ],
        );
        assert_eq!(s2.get("b").unwrap().state, State::Error);

        let s3 = reduce(
            FleetSnapshot::default(),
            &[
                SessionEvent::QuestionRaised {
                    id: "c".into(),
                    ts_ms: 1,
                    question: q(),
                },
                SessionEvent::Ended {
                    id: "c".into(),
                    ts_ms: 2,
                },
            ],
        );
        let seat3 = s3.get("c").unwrap();
        assert_eq!(seat3.question, None);
        assert_eq!(seat3.state, State::Done);
    }

    // REQ-009: first-seen order; a re-Upsert neither reorders nor duplicates (kills id() mutants +
    // index_of ==→!= / →None).
    #[test]
    fn t367_req009_first_seen_order() {
        let s = reduce(
            FleetSnapshot::default(),
            &[
                upsert("z", 1, State::Working),
                upsert("a", 2, State::Working),
            ],
        );
        let ids: Vec<&str> = s.seats().iter().map(|x| x.id.as_str()).collect();
        assert_eq!(ids, vec!["z", "a"]);

        let s2 = reduce(s, &[upsert("z", 3, State::Idle)]);
        let ids2: Vec<&str> = s2.seats().iter().map(|x| x.id.as_str()).collect();
        assert_eq!(ids2, vec!["z", "a"]);
        assert_eq!(s2.seats().len(), 2);
        assert_eq!(s2.get("z").unwrap().state, State::Idle);
    }

    // B4 (inspect): get() hits the right seat and misses cleanly (kills get→None and get ==→!=).
    #[test]
    fn t367_b4_get_hits_and_misses() {
        let s = reduce(
            FleetSnapshot::default(),
            &[upsert("a", 1, State::Working), upsert("b", 2, State::Idle)],
        );
        assert_eq!(s.get("b").unwrap().id, "b");
        assert!(s.get("nope").is_none());
    }

    // REQ-013: every SessionEvent kind round-trips; tag=kind snake_case; id()/ts_ms() accessors
    // (the accessor asserts kill id→""/"xyzzy" and ts_ms→0/1).
    #[test]
    fn t367_req013_session_event_round_trip_all_kinds() {
        let labels = BTreeMap::from([("phase".to_string(), "implement".to_string())]);
        let events = vec![
            upsert("a", 1, State::Working),
            SessionEvent::Upsert {
                id: "x".into(),
                ts_ms: 9,
                title: "t".into(),
                state: State::Waiting,
                labels,
                transport: None,
            },
            SessionEvent::StateChange {
                id: "a".into(),
                ts_ms: 2,
                state: State::Idle,
            },
            SessionEvent::QuestionRaised {
                id: "a".into(),
                ts_ms: 3,
                question: q(),
            },
            SessionEvent::QuestionCleared {
                id: "a".into(),
                ts_ms: 4,
            },
            SessionEvent::Heartbeat {
                id: "a".into(),
                ts_ms: 5,
            },
            SessionEvent::Ended {
                id: "a".into(),
                ts_ms: 6,
            },
        ];
        for e in &events {
            let json = serde_json::to_string(e).unwrap();
            let back: SessionEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(*e, back);
        }
        assert!(
            serde_json::to_string(&events[3])
                .unwrap()
                .contains("\"kind\":\"question_raised\"")
        );
        assert_eq!(events[0].id(), "a");
        assert_eq!(events[4].ts_ms(), 4);
    }

    // REQ-013: a populated FleetSnapshot round-trips (private `seats` serializes in-module).
    #[test]
    fn t367_req013_snapshot_round_trip() {
        let s = reduce(
            FleetSnapshot::default(),
            &[
                upsert("a", 1, State::Working),
                upsert("b", 2, State::Waiting),
            ],
        );
        let json = serde_json::to_string(&s).unwrap();
        let back: FleetSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
