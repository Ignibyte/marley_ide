//! PURE — the `[[mcp.servers]]` settings entry (#371): the single-owner config type (§14) for an MCP
//! server, held as a `Vec<McpServerConfig>` by `marley_app`'s settings schema (the #308
//! `LanguageServerConfig`-in-`marley_lsp` domain-crate rule). `grants()` produces the permission
//! [`GrantTable`] this crate OWNS — closing #370's S1 loop (#370 owns the type, #371's config fills it).
//! Grants are CARRIED, not interpreted here (D7); no live-connection behavior.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::permission::GrantTable;

/// The serde default for `enabled`: a hand-edited entry that OMITS the key is ENABLED. A bare
/// `#[serde(default)]` on a `bool` would give `false` and flip the meaning of an omitted key (D6).
fn default_enabled() -> bool {
    true
}

/// One configured MCP server (`[[mcp.servers]]`): a name, a transport (stdio `command`/`args` OR http
/// `url`), an `enabled` flag, and the tool-class grants a consumer applies. Every non-identity field is
/// `#[serde(default)]` so a hand-edited entry that omits one loads with its default rather than dropping
/// every saved server (the #204 lesson). Vocabulary matches the `.mcp.json` convention (D3); no
/// credential/header field (D4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// The server's name (identity).
    pub name: String,
    /// The stdio launch command (a bare name or a path); mutually exclusive with `url`.
    #[serde(default)]
    pub command: Option<String>,
    /// Extra stdio launch args (defaulted).
    #[serde(default)]
    pub args: Vec<String>,
    /// The http endpoint URL; mutually exclusive with `command`.
    #[serde(default)]
    pub url: Option<String>,
    /// Whether this server is enabled (an omitted key → `true`).
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Allowed READ tool classes — carried opaquely (D7; not yet consumed — #370's read tier is loose).
    #[serde(default)]
    pub allow: Vec<String>,
    /// Allowed WRITE tool classes — mapped to a [`GrantTable`] by [`McpServerConfig::grants`].
    #[serde(default)]
    pub allow_write: Vec<String>,
}

impl Default for McpServerConfig {
    /// Hand-implemented (NOT derived) so `enabled` defaults to `true` — EQUAL to the serde
    /// all-optional-keys-absent decode (D6). A derived `Default` would give `enabled: false`, diverging
    /// from the hand-edit semantics; a test pins the equality.
    fn default() -> Self {
        McpServerConfig {
            name: String::new(),
            command: None,
            args: Vec::new(),
            url: None,
            enabled: default_enabled(),
            allow: Vec::new(),
            allow_write: Vec::new(),
        }
    }
}

/// The launch transport a [`McpServerConfig`] resolves to (D2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpTransport {
    /// A stdio child process.
    Stdio {
        /// The program to launch.
        command: String,
        /// Its launch args.
        args: Vec<String>,
    },
    /// A Streamable-HTTP endpoint.
    Http {
        /// The endpoint URL.
        url: String,
    },
}

/// Why a [`McpServerConfig`] has no valid transport — a PER-ENTRY error, so a sibling entry still
/// resolves (the #87 drop-the-bad-entry posture), never a panic (§14).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpConfigError {
    /// Neither `command` nor `url` was set.
    NoTransport,
    /// BOTH `command` and `url` were set (ambiguous).
    BothTransports,
}

impl fmt::Display for McpConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            McpConfigError::NoTransport => {
                write!(f, "mcp server entry has neither `command` nor `url`")
            }
            McpConfigError::BothTransports => {
                write!(f, "mcp server entry has BOTH `command` and `url`")
            }
        }
    }
}

impl std::error::Error for McpConfigError {}

impl McpServerConfig {
    /// Resolve this entry's transport (D2/REQ-003/004): `command` alone → `Stdio`, `url` alone → `Http`,
    /// neither → `NoTransport`, both → `BothTransports`. Typed, per-entry, never a panic.
    pub fn transport(&self) -> Result<McpTransport, McpConfigError> {
        match (&self.command, &self.url) {
            (Some(command), None) => Ok(McpTransport::Stdio {
                command: command.clone(),
                args: self.args.clone(),
            }),
            (None, Some(url)) => Ok(McpTransport::Http { url: url.clone() }),
            (None, None) => Err(McpConfigError::NoTransport),
            (Some(_), Some(_)) => Err(McpConfigError::BothTransports),
        }
    }

    /// The permission grants this server's `allow_write` classes confer (D7/REQ-007) — the marley_mcp
    /// [`GrantTable`] #370 enforces. `allow` (read classes) is carried but not consumed (read tier is loose).
    pub fn grants(&self) -> GrantTable {
        GrantTable::from_classes(self.allow_write.iter().cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_resolves_all_four_cases() {
        let stdio = McpServerConfig {
            name: "s".into(),
            command: Some("cmd".into()),
            args: vec!["a".into()],
            ..Default::default()
        };
        assert_eq!(
            stdio.transport(),
            Ok(McpTransport::Stdio {
                command: "cmd".into(),
                args: vec!["a".into()]
            })
        );
        let http = McpServerConfig {
            name: "h".into(),
            url: Some("http://x".into()),
            ..Default::default()
        };
        assert_eq!(
            http.transport(),
            Ok(McpTransport::Http {
                url: "http://x".into()
            })
        );
        let neither = McpServerConfig {
            name: "n".into(),
            ..Default::default()
        };
        assert_eq!(neither.transport(), Err(McpConfigError::NoTransport));
        let both = McpServerConfig {
            name: "b".into(),
            command: Some("c".into()),
            url: Some("u".into()),
            ..Default::default()
        };
        assert_eq!(both.transport(), Err(McpConfigError::BothTransports));
    }

    #[test]
    fn grants_maps_allow_write_and_leaves_allow_untouched() {
        let config = McpServerConfig {
            name: "s".into(),
            allow: vec!["read.thing".into()],
            allow_write: vec!["session.write".into(), "editor.write".into()],
            ..Default::default()
        };
        let grants = config.grants();
        assert!(grants.write_classes.contains("session.write"));
        assert!(grants.write_classes.contains("editor.write"));
        assert_eq!(grants.write_classes.len(), 2);
        // `allow` (read classes) is carried, never folded into the write grants (D7).
        assert!(!grants.write_classes.contains("read.thing"));
        assert_eq!(config.allow, vec!["read.thing".to_string()]);
    }

    #[test]
    fn config_error_display_distinguishes_the_two_arms() {
        assert_ne!(
            McpConfigError::NoTransport.to_string(),
            McpConfigError::BothTransports.to_string()
        );
        assert!(McpConfigError::NoTransport.to_string().contains("neither"));
        assert!(McpConfigError::BothTransports.to_string().contains("BOTH"));
    }

    #[test]
    fn default_equals_the_serde_minimal_decode_with_enabled_true() {
        // D6: the HAND-IMPL `Default` (enabled=true) EQUALS a decode of an entry with only `name`. A
        // derived `Default` would give `enabled=false` and this assertion would fail.
        let decoded: McpServerConfig = serde_json::from_str(r#"{"name":"x"}"#).expect("decode");
        assert_eq!(
            decoded,
            McpServerConfig {
                name: "x".into(),
                ..Default::default()
            }
        );
        assert!(decoded.enabled);
    }

    #[test]
    fn serde_round_trips_a_full_entry() {
        let config = McpServerConfig {
            name: "srv".into(),
            command: Some("cmd".into()),
            args: vec!["a".into()],
            url: None,
            enabled: false,
            allow: vec!["r".into()],
            allow_write: vec!["w".into()],
        };
        let json = serde_json::to_string(&config).expect("ser");
        assert_eq!(
            serde_json::from_str::<McpServerConfig>(&json).expect("de"),
            config
        );
    }
}
