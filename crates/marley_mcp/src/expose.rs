//! PURE — the `[mcp.expose]` singleton settings table (#374): the tool-class grants an operator confers
//! on Marley's OWN loopback expose server (#370). Distinct from `[[mcp.servers]]` (config.rs — the
//! client/entry array): Marley IS the server, so there is no transport to resolve, only a grant table —
//! which is why it is a singleton `[mcp.expose]` table, not a `[[mcp.servers]]` row (a transport-less "us"
//! row would resolve to `McpConfigError::NoTransport`, D1). `grants()` builds the `marley_mcp` [`GrantTable`]
//! #370 enforces, closing #371's S3 loop: operator config now reaches the LIVE permission check.

use serde::{Deserialize, Serialize};

use crate::permission::GrantTable;

/// The `[mcp.expose]` config — the grants for Marley's own expose server.
///
/// Both fields `#[serde(default)]`, `derive(Default)` == the all-keys-absent decode (D2 — no
/// `enabled`-true trap like `McpServerConfig`; still pinned by a test). MINIMAL by design: no
/// `enabled` (serving is verb-gated), no port/bind (loopback + OS-assigned port fixed by #370).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExposeConfig {
    /// Allowed READ tool classes — carried opaquely (D3; #370's read tier is loose, so not consumed).
    #[serde(default)]
    pub allow: Vec<String>,
    /// Allowed WRITE tool classes — mapped to the [`GrantTable`] the server enforces (e.g. `"session.write"`).
    #[serde(default)]
    pub allow_write: Vec<String>,
}

impl ExposeConfig {
    /// The permission grants this expose config confers — `GrantTable::from_classes(allow_write)`, the
    /// byte-identical semantics of the shipped `McpServerConfig::grants()`. `allow` (read classes) is
    /// carried but not consumed (the read tier is loose). An ABSENT `[mcp.expose]` decodes to
    /// `ExposeConfig::default()`, whose `grants()` equals `GrantTable::default()` — so an unconfigured
    /// Marley serves exactly as the pre-#374 hardcoded default (deny-by-default), byte-for-byte (D5).
    #[must_use]
    pub fn grants(&self) -> GrantTable {
        GrantTable::from_classes(self.allow_write.iter().cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Handled, Outgoing, RequestCtx, Subscriptions, handle_message};
    use marley_fleet::FleetSnapshot;

    /// The JSON body of the first outbound message.
    fn response_body(handled: &Handled) -> serde_json::Value {
        let (Outgoing::Response(text) | Outgoing::Notification(text)) =
            handled.outgoing.first().expect("a response")
        else {
            panic!("a deferred call carries no body");
        };
        serde_json::from_str(text).expect("json")
    }

    // REQ-001 — an ExposeConfig's grants gate the REAL dispatch path end to end: an `allow_write` class lets
    // its write tool through, an unlisted class is denied with the typed `isError` refusal (deny-by-default).
    #[test]
    fn expose_grants_gate_the_write_verb_end_to_end() {
        let snapshot = FleetSnapshot::default();
        let index = [("dev-1/a".to_string(), 42u64)];
        let call = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"session_surface_to_human","arguments":{"id":"dev-1/a"}}}"#;

        // Granted: allow_write = ["session.write"] → the permission check passes → the surface is accepted.
        let granted = ExposeConfig {
            allow: Vec::new(),
            allow_write: vec!["session.write".into()],
        };
        let grants = granted.grants();
        let ctx = RequestCtx {
            snapshot: &snapshot,
            grants: &grants,
            surface_index: &index,
            principal: &crate::Principal::Marley,
            enabled: &std::collections::BTreeSet::new(),
        };
        let mut subs = Subscriptions::default();
        let body = response_body(&handle_message(&ctx, &mut subs, call));
        assert_eq!(
            body["result"]["isError"], false,
            "an allow_write grant lets the write verb through"
        );

        // Denied: only an UNRELATED class granted → the write verb is refused (isError, not a protocol error).
        let unrelated = ExposeConfig {
            allow: Vec::new(),
            allow_write: vec!["editor.write".into()],
        };
        let grants = unrelated.grants();
        let ctx = RequestCtx {
            snapshot: &snapshot,
            grants: &grants,
            surface_index: &index,
            principal: &crate::Principal::Marley,
            enabled: &std::collections::BTreeSet::new(),
        };
        let mut subs = Subscriptions::default();
        let body = response_body(&handle_message(&ctx, &mut subs, call));
        assert!(
            body.get("error").is_none(),
            "a permission refusal is a tool error, not a protocol error"
        );
        assert_eq!(
            body["result"]["isError"], true,
            "an unlisted class stays denied (deny-by-default preserved)"
        );
    }

    // REQ-001/D3 — grants() maps every allow_write class into write_classes + carries `allow` untouched.
    #[test]
    fn grants_maps_allow_write_and_carries_allow() {
        let config = ExposeConfig {
            allow: vec!["read.thing".into()],
            allow_write: vec!["session.write".into(), "editor.write".into()],
        };
        let grants = config.grants();
        assert!(grants.write_classes.contains("session.write"));
        assert!(grants.write_classes.contains("editor.write"));
        assert_eq!(grants.write_classes.len(), 2);
        // `allow` (read classes) is carried, never folded into the write grants (D3).
        assert!(!grants.write_classes.contains("read.thing"));
        assert_eq!(config.allow, vec!["read.thing".to_string()]);
    }

    // REQ-002 (D5) — an absent/default ExposeConfig grants exactly today's hardcoded default, byte-for-byte.
    #[test]
    fn default_grants_equal_the_shipped_default_table() {
        assert_eq!(ExposeConfig::default().grants(), GrantTable::default());
    }

    // REQ-006 (D2) — the derived Default equals the all-keys-absent decode; a full struct round-trips.
    #[test]
    fn default_equals_empty_decode_and_round_trips() {
        let decoded: ExposeConfig = serde_json::from_str("{}").expect("decode empty");
        assert_eq!(decoded, ExposeConfig::default());
        assert!(decoded.allow.is_empty() && decoded.allow_write.is_empty());
        let full = ExposeConfig {
            allow: vec!["r".into()],
            allow_write: vec!["w".into()],
        };
        let json = serde_json::to_string(&full).expect("ser");
        assert_eq!(
            serde_json::from_str::<ExposeConfig>(&json).expect("de"),
            full
        );
    }
}
