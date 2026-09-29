// Marley: the `marley` settings block. The content types live here because the settings derive
// macros only resolve inside this crate; `crates/marley_workbench` reads them.
use std::collections::BTreeMap;

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
    /// Seconds a working Claude Code may be quiet before the stall kind's first check (#569);
    /// the next come at twice, four and eight times it, and 0 turns the quiet checks off.
    ///
    /// Default: 60
    pub stall_check_after_seconds: Option<u64>,
    /// Whether closing a terminal, a window or Marley asks first while an agent in it is working
    /// (#550).
    ///
    /// Default: true
    pub ask_before_ending_a_working_agent: Option<bool>,
    /// Seconds a working agent's terminal closed from its tab is kept, running, for Undo or
    /// `ctrl-shift-t`; 0 ends it at once.
    ///
    /// Default: 60
    pub undo_close_seconds: Option<u64>,
    /// Where Marley pushes a line to the phone when Claude Code needs input, finishes or fails
    /// (#535): an ntfy server on this machine. Unset, Marley pushes nothing.
    pub push: Option<MarleyPushSettingsContent>,
    /// Where a URL Ctrl+clicked in a terminal opens (#503). Shift+Ctrl+click opens it in the
    /// other place.
    ///
    /// Default: "local_in_browser_tab"
    pub terminal_links: Option<MarleyTerminalLinks>,
    /// Whether a block's command is pinned over the terminal's top row while the view is scrolled
    /// back into the block's output; a click on it scrolls to the block's start (#529).
    ///
    /// Default: true
    pub sticky_command_header: Option<bool>,
    /// How many seconds a command runs before its end shows a desktop notification, when its
    /// terminal is not the one in front; 0 turns this off (#551).
    ///
    /// Default: 30
    pub long_command_seconds: Option<u64>,
    /// Which agents' consequential clicks in the Browser tab wait for Allow or Refuse while the
    /// click consequence's use is on (#571).
    ///
    /// Default: "agents_without_prompts"
    pub browser_click_pause_agents: Option<MarleyClickPauseAgents>,
    /// When an agent's `terminal_type` asks the user before it types into a running program
    /// (#525).
    ///
    /// Default: "ask_first_write"
    pub agent_terminal_writes: Option<MarleyAgentTerminalWrites>,
    /// Regular expressions for the commands an agent's `terminal_run` types at a shell's prompt
    /// without asking, each matched against the whole command and every part of a pipeline or a
    /// list (#556). Warp's defaults.
    pub agent_command_allowlist: Option<Vec<String>>,
    /// Regular expressions for the commands an agent's `terminal_run` always asks about, whatever
    /// the allowlist says (#556). Warp's defaults.
    pub agent_command_denylist: Option<Vec<String>>,
    /// Whether an agent's `terminal_run` asks before a command neither list matches (#556).
    ///
    /// Default: "run"
    pub agent_commands_outside_lists: Option<MarleyAgentCommandsOutsideLists>,
    /// Whether Marley starts Claude Code with its own permission prompts or with
    /// `--dangerously-skip-permissions` (#532). An entry of `agent_permissions_by_project` wins
    /// for its project.
    ///
    /// Default: "ask"
    pub claude_code_permissions: Option<ClaudeCodePermissions>,
    /// Whether Marley starts Codex with its own approvals and sandbox or with full access,
    /// `--sandbox danger-full-access --ask-for-approval never` (#532). An entry of
    /// `agent_permissions_by_project` wins for its project.
    ///
    /// Default: "ask"
    pub codex_permissions: Option<CodexPermissions>,
    /// What Marley starts Claude Code and Codex with in a project, by the project's folder (`~/`
    /// for your home folder), over the two defaults. An entry applies to a local project whose main
    /// folder is its folder or inside it, and the longest folder wins. Only your own settings set
    /// this: a project's `.zed/settings.json` cannot.
    ///
    /// Default: {}
    pub agent_permissions_by_project: Option<BTreeMap<String, AgentPermissionsContent>>,
    /// Who answers Claude Code's question whether to trust a folder when it asks it in a worktree
    /// New Agent in Worktree made (#587): you, from a notification, or Marley, when Zed trusts
    /// the worktree's folder. Claude Code asks for a repository it has not trusted yet.
    ///
    /// Default: "ask"
    pub claude_code_worktree_trust: Option<ClaudeCodeWorktreeTrust>,
    /// The System One layer (#565): typed questions to a model about states Marley builds from
    /// what it knows. Off until it is turned on.
    pub system_one: Option<SystemOneSettingsContent>,
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

/// The System One layer's settings (#565). The key is never a setting: Marley reads it from
/// `MARLEY_SYSTEM_ONE_KEY`, else from the system keyring.
#[with_fallible_options]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct SystemOneSettingsContent {
    /// Whether Marley asks at all. Off, it makes no request, reads no key and writes no file.
    ///
    /// Default: false
    pub enabled: Option<bool>,
    /// Who answers.
    ///
    /// Default: "typesafe"
    pub provider: Option<SystemOneProvider>,
    /// The `compatible` provider's URL: `https`, or `http` on this machine.
    pub endpoint: Option<String>,
    /// The model asked, pinned to a version.
    ///
    /// Default: "jev-1.13.0"
    pub model: Option<String>,
    /// Folders whose projects may send their state: facts and text.
    ///
    /// Default: []
    pub projects: Option<Vec<String>>,
    /// Folders whose projects send only the facts Marley computes, and none of their text.
    ///
    /// Default: []
    pub metadata_only_projects: Option<Vec<String>>,
    /// The most the layer spends in a day, in cents; once it is spent, calls wait for the next day.
    ///
    /// Default: 50
    pub daily_budget_cents: Option<u64>,
    /// The `compatible` provider's price, in cents per million input tokens.
    ///
    /// Default: 0
    pub price_cents_per_million_tokens: Option<f32>,
    /// Each use's mode, by the use's name.
    ///
    /// Default: {"check": "act", "stop_kind": "off", "browser_find": "off", "terminal_find": "off",
    /// "stall_kind": "off", "click_consequence": "off", "inbox": "off", "question_route": "off"}
    pub uses: Option<BTreeMap<String, SystemOneMode>>,
}

/// Who answers the System One layer's questions (#565).
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
pub enum SystemOneProvider {
    /// TypeSafe's API.
    #[default]
    Typesafe,
    /// Another server that speaks the same request, at `endpoint`.
    Compatible,
    /// Each use's own rules, with no request.
    Rules,
    /// Answers recorded in `system_one/replay.jsonl` under Marley's data directory.
    Replay,
}

/// How a use of the System One layer acts on what it reads (#565).
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
pub enum SystemOneMode {
    /// The use asks nothing.
    #[default]
    Off,
    /// The use asks and logs; what it read shows only in the Decisions view.
    Shadow,
    /// The use shows what it read, for you to confirm.
    Suggest,
    /// The use acts on what it read, as far as its own rules allow.
    Act,
}

/// Where a URL clicked in a terminal opens (#503).
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
pub enum MarleyTerminalLinks {
    /// A local URL (`localhost`, a loopback address) in a Browser tab of the terminal's project,
    /// and any other in the system browser.
    #[default]
    LocalInBrowserTab,
    /// Every http and https URL in a Browser tab of the terminal's project.
    AllInBrowserTab,
    /// Every URL in the system browser, as Zed opens them.
    SystemBrowser,
}

/// Which agents' consequential clicks in the Browser tab wait for Allow or Refuse (#571).
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
pub enum MarleyClickPauseAgents {
    /// Agents that run without a permission prompt of their own, and callers Marley cannot name.
    #[default]
    AgentsWithoutPrompts,
    /// Every agent.
    AllAgents,
}

/// When an agent's `terminal_type` asks the user before it types into a running program (#525).
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
pub enum MarleyAgentTerminalWrites {
    /// The first write to each program; later writes to it go in without asking.
    #[default]
    AskFirstWrite,
    /// Every write.
    AskEveryWrite,
    /// Never: the agent's writes go in as it sends them.
    NeverAsk,
}

/// Whether an agent's `terminal_run` asks before a command outside the allowlist and the denylist
/// (#556).
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
pub enum MarleyAgentCommandsOutsideLists {
    /// It runs: the agent's own permission prompt already asked.
    #[default]
    Run,
    /// It waits for Run or Refuse under the terminal.
    Ask,
}

/// What Marley starts Claude Code and Codex with in one project (#532).
#[with_fallible_options]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct AgentPermissionsContent {
    /// Claude Code's, over `claude_code_permissions`.
    pub claude_code: Option<ClaudeCodePermissions>,
    /// Codex's, over `codex_permissions`.
    pub codex: Option<CodexPermissions>,
}

/// What Marley starts Claude Code with (#532).
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
pub enum ClaudeCodePermissions {
    /// Its own permission prompts.
    #[default]
    Ask,
    /// `--dangerously-skip-permissions`: it asks for no permission.
    Bypass,
}

/// What Marley starts Codex with (#532).
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
pub enum CodexPermissions {
    /// Its own approvals and sandbox.
    #[default]
    Ask,
    /// `--sandbox danger-full-access --ask-for-approval never`: no sandbox and no approvals.
    FullAccess,
}

/// Who answers Claude Code's question whether to trust a folder, when it asks it in a worktree
/// New Agent in Worktree made (#587).
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
pub enum ClaudeCodeWorktreeTrust {
    /// You: Marley shows the question, and Trust Folder answers it.
    #[default]
    Ask,
    /// Marley, with the trust option, when Zed trusts the worktree's folder; else you.
    FollowZed,
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
