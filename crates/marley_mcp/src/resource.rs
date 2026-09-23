//! The fleet as a subscribable MCP RESOURCE (D3) — the standard resources contract, not a bespoke
//! channel. One resource (a custom `fleet://` URI, RFC3986-legal per the spec — the shape the retired
//! forge sidecar's feed used, #411; orchestration-shell §6); on a snapshot change the server pushes
//! `notifications/resources/updated`
//! and the client re-reads. Because the resource is a SNAPSHOT, a missed notification is self-healing
//! (re-read = current), which is why L1 skips resumability. PURE.

use crate::jsonrpc;
use marley_fleet::FleetSnapshot;
use serde_json::{Value, json};

/// The single L1 fleet resource URI (D3).
pub const FLEET_RESOURCE_URI: &str = "fleet://snapshot";

/// The `resources/list` result: the one subscribable fleet resource.
#[must_use]
pub fn resources_list() -> Value {
    json!({
        "resources": [
            {
                "uri": FLEET_RESOURCE_URI,
                "name": "Fleet snapshot",
                "description": "The current fleet snapshot; subscribe for change notifications.",
                "mimeType": "application/json",
            }
        ]
    })
}

/// The `resources/read` result for `uri` (D3): the fleet resource serves the SAME serialization as
/// the `fleet.snapshot` tool.
///
/// # Errors
///
/// An unknown uri → `Err((RESOURCE_NOT_FOUND, msg))`.
pub fn resource_read(snapshot: &FleetSnapshot, uri: &str) -> Result<Value, (i64, String)> {
    if uri != FLEET_RESOURCE_URI {
        return Err((
            jsonrpc::RESOURCE_NOT_FOUND,
            format!("unknown resource: {uri}"),
        ));
    }
    // `to_string` on the plain snapshot is infallible; `unwrap_or_default` is the never-taken fallback.
    let serialized = serde_json::to_string(snapshot).unwrap_or_default();
    Ok(json!({
        "contents": [
            { "uri": FLEET_RESOURCE_URI, "mimeType": "application/json", "text": serialized }
        ]
    }))
}

/// Build the `notifications/resources/updated` message for `uri` (D3) — the standing-stream push a
/// subscribed client gets on a snapshot change. A notification carries no `id`.
#[must_use]
pub fn resource_updated_notification(uri: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "method": "notifications/resources/updated",
        "params": { "uri": uri }
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resources_list_advertises_the_fleet_resource() {
        let list = resources_list();
        assert_eq!(list["resources"][0]["uri"], FLEET_RESOURCE_URI);
    }

    #[test]
    fn resource_read_serves_known_uri_and_errors_unknown() {
        let snapshot = FleetSnapshot::default();
        let ok = resource_read(&snapshot, FLEET_RESOURCE_URI).expect("known uri");
        assert_eq!(ok["contents"][0]["uri"], FLEET_RESOURCE_URI);
        assert!(ok["contents"][0]["text"].is_string());
        assert_eq!(
            resource_read(&snapshot, "bogus://x")
                .expect_err("unknown")
                .0,
            jsonrpc::RESOURCE_NOT_FOUND
        );
    }

    #[test]
    fn resource_updated_notification_has_the_uri_and_no_id() {
        let value: Value =
            serde_json::from_str(&resource_updated_notification(FLEET_RESOURCE_URI)).expect("json");
        assert_eq!(value["method"], "notifications/resources/updated");
        assert_eq!(value["params"]["uri"], FLEET_RESOURCE_URI);
        assert!(value.get("id").is_none());
    }
}
