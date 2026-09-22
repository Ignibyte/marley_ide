//! The session-verb request/receipt types as pure data (the MCP tool schema seam). Types only — no
//! transport, no handlers, no I/O; Layer 2 implements the verbs against these shapes.

use serde::{Deserialize, Serialize};

/// A request to send text to a seat (`session.send`). Layer 2 delivers it receipted, Enter as a
/// separate write; a refused send (a dialog is up) is a first-class [`Receipt`] outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendRequest {
    /// The target seat.
    pub id: String,
    /// The text to deliver.
    pub text: String,
}

/// The line-oriented range for `session.read` (byte offsets were rejected — `capture-pane` is
/// line-oriented and byte ranges leak text-encoding into a transport-agnostic envelope).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum ReadRange {
    /// The last `lines` of scrollback — the common "what did it just output?" read.
    Tail {
        /// How many trailing lines to return.
        lines: u32,
    },
    /// An absolute half-open line range `[start, end)` in scrollback coordinates.
    Lines {
        /// First line (inclusive).
        start: u64,
        /// End line (exclusive).
        end: u64,
    },
}

/// A request to read a seat's scrollback (`session.read`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadRequest {
    /// The seat to read.
    pub id: String,
    /// Which lines to return.
    pub range: ReadRange,
}

/// A request to open a new seat (`session.open`). `profile` is OPAQUE — the profile vocabulary is
/// policy (the adapter/manager owns it), not Marley's.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenRequest {
    /// The opaque profile identifier for the seat to open.
    pub profile: String,
}

/// A request to surface a seat to the human (`session.surface_to_human`) — the ONE gated write Layer 1
/// ships: open/focus the seat so the operator can view it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceRequest {
    /// The seat to surface.
    pub id: String,
}

/// A request to answer a seat's standing question (`session.answer`, #377 — L2 gated-writes ①).
/// `choice` is the picked option string VERBATIM (never an index — indices renumber if the question
/// re-renders; the string is what the human saw). `prompt` is the answered question's prompt — the
/// question IDENTITY, so the brain can REFUSE an answer that arrives after the question changed (a
/// stale answer must never land silently; inspect #377-F1). The answer is control-plane DATA, never
/// keystrokes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnswerRequest {
    /// The seat whose question is being answered.
    pub id: String,
    /// The picked option string, verbatim.
    pub choice: String,
    /// The prompt of the question this choice answers (the stale-answer refusal key).
    pub prompt: String,
}

/// The generic receipt for a verb: `Accepted` with the verb's payload, or `Refused` with a reason. A
/// refused verb is a first-class outcome, not an error to swallow (the dialog-up rule).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum Receipt<T> {
    /// The verb was accepted; `value` is its success payload.
    Accepted {
        /// The verb's success payload.
        value: T,
    },
    /// The verb was refused; `reason` says why.
    Refused {
        /// Why the verb was refused.
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rt<T>(v: &T)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let json = serde_json::to_string(v).unwrap();
        let back: T = serde_json::from_str(&json).unwrap();
        assert_eq!(*v, back);
    }

    // REQ-013: every verb request round-trips; ReadRange BOTH arms; tag=mode snake_case.
    #[test]
    fn t367_req013_verb_requests_round_trip() {
        rt(&SendRequest {
            id: "a".into(),
            text: "hello".into(),
        });
        rt(&ReadRequest {
            id: "a".into(),
            range: ReadRange::Tail { lines: 50 },
        });
        rt(&ReadRequest {
            id: "a".into(),
            range: ReadRange::Lines { start: 10, end: 20 },
        });
        rt(&OpenRequest {
            profile: "claude".into(),
        });
        rt(&SurfaceRequest { id: "a".into() });
        // #377: the answer verb — round-trips, and the prompt (the stale-answer refusal key) is on
        // the wire by its own name.
        let answer = AnswerRequest {
            id: "a".into(),
            choice: "yes".into(),
            prompt: "apply the migration?".into(),
        };
        rt(&answer);
        assert!(
            serde_json::to_string(&answer)
                .unwrap()
                .contains(r#""prompt":"apply the migration?""#)
        );
        assert!(
            serde_json::to_string(&ReadRange::Tail { lines: 5 })
                .unwrap()
                .contains("\"mode\":\"tail\"")
        );
        assert!(
            serde_json::to_string(&ReadRange::Lines { start: 1, end: 2 })
                .unwrap()
                .contains("\"mode\":\"lines\"")
        );
    }

    // REQ-013: Receipt<T> round-trips in BOTH arms for a unit, a list, and a string payload; tag
    // = result snake_case. Proves internal tagging is safe for any T (the struct-variant choice).
    #[test]
    fn t367_req013_receipt_both_arms_and_payloads() {
        rt(&Receipt::<()>::Accepted { value: () });
        rt(&Receipt::<()>::Refused {
            reason: "a dialog is up".into(),
        });
        rt(&Receipt::<Vec<String>>::Accepted {
            value: vec!["line1".into(), "line2".into()],
        });
        rt(&Receipt::<String>::Accepted {
            value: "new-seat-id".into(),
        });
        assert!(
            serde_json::to_string(&Receipt::<()>::Refused { reason: "x".into() })
                .unwrap()
                .contains("\"result\":\"refused\"")
        );
    }
}
