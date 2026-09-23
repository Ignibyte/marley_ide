//! The tool handlers' PURE parts (D2/D6/D7): `fleet.snapshot` serialization, `surface_to_human` id
//! resolution + receipt, and the `isError` tool-execution-error envelope. The #367 `marley_fleet` types
//! ARE the schema (D2) — no parallel hand-written schema drifts from them.

use marley_fleet::{FleetSnapshot, Receipt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// A tools/call result envelope: the typed `structuredContent` + the back-compat `text` block, with
/// `isError` set for a business refusal (D6).
///
/// MCP requires `content`; `structuredContent` is the typed channel that conforms to the tool's
/// `outputSchema`.
#[must_use]
pub fn tool_result(structured: &Value, is_error: bool) -> Value {
    let text = structured.to_string();
    json!({
        "content": [ { "type": "text", "text": text } ],
        "structuredContent": structured,
        "isError": is_error,
    })
}

/// The `fleet.snapshot` result (D2/REQ-002): the CURRENT `FleetSnapshot` serialized as
/// `structuredContent` (+ text), never an error.
///
/// `to_value` on a plain data struct (string keys, no floats) is infallible; `unwrap_or_default` is
/// the never-taken fallback (§14 — no panic on any path, no uncovered closure).
#[must_use]
pub fn fleet_snapshot_result(snapshot: &FleetSnapshot) -> Value {
    let structured = serde_json::to_value(snapshot).unwrap_or_default();
    tool_result(&structured, false)
}

/// A tool-execution error result (D6) — `isError:true` carrying the reason. Used for permission denials
/// and any known-tool business refusal (unknown session id, etc.).
#[must_use]
pub fn tool_error(reason: &str) -> Value {
    tool_result(&json!({ "result": "refused", "reason": reason }), true)
}

/// Resolve a session id to its shell surface handle — an OPAQUE `u64` (the app's `PaneId.0`, so
/// `marley_mcp` never depends on `marley_app`).
///
/// `None` when no LOCAL surface hosts that id (D-OPEN-SURFACE-V1: L1 can only surface a session the
/// shell hosts; a remote/demo-only seat refuses). Exact-match; app owns the index.
#[must_use]
pub fn resolve_surface(id: &str, index: &[(String, u64)]) -> Option<u64> {
    index
        .iter()
        .find(|(entry_id, _)| entry_id == id)
        .map(|(_, handle)| *handle)
}

/// The success payload of a `surface_to_human` call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceAck {
    /// The id that was surfaced.
    pub surfaced: String,
}

/// Build the `surface_to_human` receipt (D7).
///
/// `Accepted` when the id RESOLVED to a hosted pane (the focus effect is then applied best-effort
/// on the UI thread — a lost race afterward is not reflected back); `Refused` otherwise (the #367
/// first-class refusal → D6 `isError`). Never panics.
#[must_use]
pub fn surface_receipt(focused: bool, id: &str) -> Receipt<SurfaceAck> {
    if focused {
        Receipt::Accepted {
            value: SurfaceAck {
                surfaced: id.to_string(),
            },
        }
    } else {
        Receipt::Refused {
            reason: format!("no surfaceable session for id '{id}'"),
        }
    }
}

/// Wrap a surface receipt as a tools/call result: `Refused` → `isError:true` (D6); `Accepted` → normal.
#[must_use]
pub fn surface_result(receipt: &Receipt<SurfaceAck>) -> Value {
    let is_error = matches!(receipt, Receipt::Refused { .. });
    // `to_value` on the plain receipt is infallible; `unwrap_or_default` is the never-taken fallback.
    let structured = serde_json::to_value(receipt).unwrap_or_default();
    tool_result(&structured, is_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use marley_fleet::{SessionEvent, State, reduce};

    fn one_seat() -> FleetSnapshot {
        reduce(
            FleetSnapshot::default(),
            &[SessionEvent::Upsert {
                id: "dev-1/a".into(),
                ts_ms: 1,
                title: "a".into(),
                state: State::Working,
                labels: std::collections::BTreeMap::new(),
                transport: None,
            }],
        )
    }

    #[test]
    fn fleet_snapshot_result_serializes_the_snapshot_never_error() {
        let result = fleet_snapshot_result(&one_seat());
        assert_eq!(result["isError"], false);
        assert_eq!(result["structuredContent"]["seats"][0]["id"], "dev-1/a");
        assert!(result["content"][0]["text"].is_string());
    }

    #[test]
    fn tool_error_is_iserror_with_reason() {
        let error = tool_error("denied");
        assert_eq!(error["isError"], true);
        assert_eq!(error["structuredContent"]["reason"], "denied");
    }

    #[test]
    fn resolve_surface_exact_match_hit_miss_empty() {
        let index = vec![("a".to_string(), 7u64), ("b".to_string(), 9u64)];
        assert_eq!(resolve_surface("b", &index), Some(9));
        assert_eq!(resolve_surface("z", &index), None);
        assert_eq!(resolve_surface("a", &[]), None);
    }

    #[test]
    fn surface_receipt_and_result_reflect_focus_outcome() {
        let accepted = surface_receipt(true, "a");
        assert!(matches!(accepted, Receipt::Accepted { .. }));
        let ok = surface_result(&accepted);
        assert_eq!(ok["isError"], false);
        assert_eq!(ok["structuredContent"]["result"], "accepted");
        assert_eq!(ok["structuredContent"]["value"]["surfaced"], "a");

        let refused = surface_receipt(false, "a");
        assert!(matches!(refused, Receipt::Refused { .. }));
        let no = surface_result(&refused);
        assert_eq!(no["isError"], true);
        assert_eq!(no["structuredContent"]["result"], "refused");
    }
}
