// Marley: the `marley` settings block. The content types live here because the settings derive
// macros only resolve inside this crate; `crates/marley_workbench` reads them.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings_macros::{MergeFrom, with_fallible_options};

/// Settings for Marley, the fork of Zed this build is.
#[with_fallible_options]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct MarleySettingsContent {
    /// Which layout the windows use.
    ///
    /// Default: "marley"
    pub layout: Option<MarleyLayout>,
    /// Whether Marley hides secrets (keys, tokens, passwords) in what its MCP tools give agents
    /// from terminals and the browser's console.
    ///
    /// Default: true
    pub redact_secrets_for_agents: Option<bool>,
    /// Regular expressions whose matches Marley also hides from agents, beside its own rules.
    ///
    /// Default: []
    pub redaction_patterns: Option<Vec<String>>,
    /// Minutes a working Claude Code may go without a hook event before its rail row says
    /// `no update in N m` instead of `working`; 0 turns this off.
    ///
    /// Default: 30
    pub no_update_after_minutes: Option<u64>,
    /// Where Marley pushes a line to the phone when Claude Code needs input, finishes or fails
    /// (#535): an ntfy server on this machine. Unset, Marley pushes nothing.
    pub push: Option<MarleyPushSettingsContent>,
}

/// The ntfy server and topic agent events are pushed to.
#[with_fallible_options]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct MarleyPushSettingsContent {
    /// The ntfy server, on this machine: `http://127.0.0.1:<port>`, `[::1]` or `localhost`.
    pub url: Option<String>,
    /// The topic on the server.
    pub topic: Option<String>,
    /// A file holding an ntfy access token, readable by its owner alone.
    pub token_file: Option<String>,
}

/// A window layout.
// Marley: `VariantArray` and `VariantNames` give the Settings window's Marley page its dropdown (#515).
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum MarleyLayout {
    /// Zed's own layout: the Threads Sidebar and Zed's panel defaults.
    Zed,
    /// The Marley layout: a rail of projects with their terminals, and terminals in the center.
    #[default]
    Marley,
}
