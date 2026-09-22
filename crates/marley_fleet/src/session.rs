//! The generic session envelope — the one schema the fleet control plane's project-agnosticism lives
//! or dies on (orchestration-shell §2–3). PURE data; a product-specific string here would be a defect.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The generic lifecycle state of a seat — the CLOSED vocabulary (orchestration-shell §3). `Error` is
/// first-class and distinct from `Idle`: a crashed seat is stopped-and-silent, never "at rest" (the
/// dead-vs-idle blindness the evidence night named).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    /// Coming up — not yet producing.
    Starting,
    /// Actively producing output / running.
    Working,
    /// At rest, alive, no active work.
    Idle,
    /// Blocked needing input (a structured [`Question`] may accompany — see [`Session::question`]).
    Waiting,
    /// Stopped on an error — stopped AND silent; must never read as `Idle` or cleanly `Done`.
    Error,
    /// The seat's process has ended cleanly.
    Done,
}

/// Where a seat's PTY actually lives — a RENDER HINT only (never dispatch policy).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
    /// A tmux window/pane.
    Tmux,
    /// A bridge-owned session.
    Bridge,
    /// A local PTY inside Marley.
    Local,
}

/// A structured question a `Waiting` seat is blocked on (fleet-control-plane §6 `halted-with-question`)
/// — rendered as a form, answered as a receipted reply (Layer 2). No digits pressed at a screenshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Question {
    /// The question text shown to the human.
    pub prompt: String,
    /// The offered choices (may be empty for a free-text answer).
    pub options: Vec<String>,
    /// Opaque references the adapter attached for context (rendered, not interpreted).
    pub context_refs: Vec<String>,
}

/// One agent seat in the fleet — the generic envelope every fleet surface reads (the rail, the tests,
/// the MCP tool schema: one seam, three consumers).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    /// STABLE identity — never a tmux window index (the evidence-night lesson: indices renumber).
    pub id: String,
    /// A human label for the seat.
    pub title: String,
    /// The seat's lifecycle state.
    pub state: State,
    /// Present ⇒ `state == Waiting` (reducer-enforced, one-directional). `Waiting` WITHOUT a question is
    /// legal (a seat gone quiet needing input).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub question: Option<Question>,
    /// Opaque, UNINTERPRETED key/value chips (the adapter writes `ticket` / `phase` /
    /// `capabilities.mode`; Marley renders them, never matches on them). Sorted for a stable render +
    /// serialization.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub labels: BTreeMap<String, String>,
    /// Epoch-millis of the last event that touched this seat. Absence-of-advance past a threshold is
    /// itself a signal (staleness — see [`crate::attention`](mod@crate::attention)).
    pub last_event_ms: u64,
    /// Where the PTY lives — `None` when unknown (an auto-vivified placeholder seat has no hint yet).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport: Option<Transport>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_question() -> Question {
        Question {
            prompt: "continue?".into(),
            options: vec!["y".into(), "n".into()],
            context_refs: vec!["ref-1".into()],
        }
    }

    fn labeled() -> BTreeMap<String, String> {
        // Opaque values only — proves labels ride through uninterpreted (D1/D4).
        let mut m = BTreeMap::new();
        m.insert("ticket".to_string(), "1696".to_string());
        m.insert("capabilities.mode".to_string(), "bypass".to_string());
        m
    }

    // REQ-013: a Session round-trips for every state, every transport (incl. None), question ±,
    // labels ± — covers State×6, Transport×3+None, and both skip_serializing_if branches.
    #[test]
    fn t367_req013_session_round_trip_all_states_transports() {
        let states = [
            State::Starting,
            State::Working,
            State::Idle,
            State::Waiting,
            State::Error,
            State::Done,
        ];
        let transports = [
            None,
            Some(Transport::Tmux),
            Some(Transport::Bridge),
            Some(Transport::Local),
        ];
        for state in states {
            for transport in transports {
                for question in [None, Some(sample_question())] {
                    for labels in [BTreeMap::new(), labeled()] {
                        let s = Session {
                            id: "dev-1/agent1".into(),
                            title: "agent one".into(),
                            state,
                            question: question.clone(),
                            labels,
                            last_event_ms: 1234,
                            transport,
                        };
                        let json = serde_json::to_string(&s).unwrap();
                        let back: Session = serde_json::from_str(&json).unwrap();
                        assert_eq!(s, back);
                    }
                }
            }
        }
    }

    // REQ-013: state / transport serialize as lowercase strings (the tool-schema wire shape).
    #[test]
    fn t367_req013_state_transport_lowercase_wire() {
        assert_eq!(
            serde_json::to_string(&State::Waiting).unwrap(),
            "\"waiting\""
        );
        assert_eq!(serde_json::to_string(&State::Error).unwrap(), "\"error\"");
        assert_eq!(serde_json::to_string(&State::Done).unwrap(), "\"done\"");
        assert_eq!(serde_json::to_string(&Transport::Tmux).unwrap(), "\"tmux\"");
        assert_eq!(
            serde_json::to_string(&Transport::Bridge).unwrap(),
            "\"bridge\""
        );
        assert_eq!(
            serde_json::to_string(&Transport::Local).unwrap(),
            "\"local\""
        );
    }

    // REQ-013 (F3): empty question/labels/transport are OMITTED from the wire yet round-trip via
    // `default` — the skip_serializing_if ⇄ default inverse.
    #[test]
    fn t367_req013_empty_fields_omitted_but_round_trip() {
        let s = Session {
            id: "x".into(),
            title: "t".into(),
            state: State::Idle,
            question: None,
            labels: BTreeMap::new(),
            last_event_ms: 5,
            transport: None,
        };
        let json = serde_json::to_string(&s).unwrap();
        assert!(!json.contains("question"));
        assert!(!json.contains("labels"));
        assert!(!json.contains("transport"));
        let back: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
