//! Marley's own agent (#683, #687) and Rusty's (#696): Agent Panel entries, one that explains and
//! configures Marley, one that reaches the user's Rusty.
//!
//! `marley.assistant.agent` picks what both run on:
//! - `auto` (#696), the default: the first found at start, Claude Code signed in, then Codex signed
//!   in, then Zed's default model with its provider signed in. With none found, neither entry is
//!   there.
//! - Claude Code (#683): an entry running the registry's `claude-acp` adapter on the user's own
//!   login, with instructions appended to its system prompt and the tools that edit files or run
//!   commands disallowed. The adapter reads both from each session's `_meta`, which Zed sends from
//!   `marley.agent_session_meta` (a hunk in `agent_servers`).
//! - Codex (#687): the same entry running the registry's `codex-acp` in its read-only mode, with
//!   the instructions as Codex's `developer_instructions` in `CODEX_CONFIG`, which the adapter
//!   merges into each thread's config.
//! - Zed's own agent (#687): a profile with no file tools. Zed's agent takes no prompt per profile
//!   and drops an MCP server's instructions, so the tools' own descriptions guide it.
//!
//! The Marley entry is there while `marley.assistant.enabled` is on, the default since #696. It
//! uses Marley's MCP server, which Zed hands every agent, for the docs and settings tools (#681,
//! #682, #686). Rusty's entry is there while Rusty is on and Marley has reached it. Its sessions
//! carry Rusty's server themselves, as `_meta.claudeCode.options.mcpServers` or Codex's
//! `mcp_servers`, so Rusty's tools reach that entry alone. Zed's agent reaches tools only through
//! a context server, so while its Rusty profile is wanted Marley offers it the `rusty` server, as
//! `marley.rusty.agent_tools` does.
//!
//! What the entries need lives in the settings' in-memory defaults, as `context_servers.marley`
//! does (`mcp::offer_to_zeds_agents`): a custom agent server whose command is the one Zed resolves
//! for the adapter, so the adapter Zed installed runs it, or the profile. Off, nothing of it is
//! there.
//!
//! `marley: open marley agent in terminal` (#684) runs the Marley agent as its own interface in a
//! center terminal: `claude` with the instructions as a system-prompt file and the same tools
//! turned off, or `codex` in its read-only sandbox with the instructions as its developer
//! instructions. The palette lists it only while the Marley entry is there on an agent with a
//! terminal interface.
//!
//! `MARLEY_ASSISTANT_ADAPTER`, when set, names an ACP program to run in the adapter's place, as
//! `MARLEY_CODEX` names a stand-in Codex: a scenario's scripted agent, which reads the `_meta`
//! the sessions carry and the environment it was started with.

use std::any::TypeId;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use agent_settings::AgentProfileId;
use collections::{HashMap, IndexMap};
use command_palette_hooks::CommandPaletteFilter;
use fs::Fs;
use gpui::{App, BorrowAppContext as _, Entity, Global, TaskExt as _, Window, actions};
use language_model::LanguageModelRegistry;
use marley_agent::AgentKind;
use project::agent_server_store::{AgentId, AgentServerCommand, AgentServerStore};
use serde_json::{Map, Value, json};
use settings::{
    AgentProfileContent, ContextServerPresetContent, CustomAgentServerSettings,
    MarleyAssistantAgent, SettingsStore,
};
use workspace::{MultiWorkspace, Workspace};

use crate::rusty::RustyServer;

actions!(
    marley,
    [
        /// Opens Marley's own agent in a terminal: Claude Code or Codex with the Marley agent's
        /// instructions, and no file edits.
        #[derive(Eq)]
        OpenMarleyAgentInTerminal
    ]
);

/// Claude Code's tools the agents may not use: they work through Marley's or Rusty's tools, and
/// change no file and run no command.
const DISALLOWED_TOOLS: [&str; 5] = ["Bash", "Edit", "Write", "NotebookEdit", "MultiEdit"];

/// Marley's tools the Marley profile turns on for Zed's own agent: the ones that read the docs and
/// the settings, and the three that propose a change the user accepts.
const PROFILE_TOOLS: [&str; 8] = [
    "docs_search",
    "docs_read",
    "settings_schema",
    "settings_read",
    "settings_change",
    "keymap_change",
    "seat_add",
    "actions_list",
];

/// How long `auto` waits before it looks: the user's settings have loaded by then, and Zed's own
/// tests, whose clock stands still, start no status check.
const DETECT_AFTER: Duration = Duration::from_secs(1);

/// How long after start, or after Claude Code and Codex were found signed out, Zed's model is
/// checked, so Zed's model providers have signed in.
const ZED_MODEL_AFTER: Duration = Duration::from_secs(5);

/// What the Marley agent is told: appended to Claude Code's system prompt, and Codex's developer
/// instructions.
const INSTRUCTIONS: &str = "You are Marley's own agent, inside Marley, a code editor built on \
Zed with a block terminal, a Browser tab and agent tools. Your job is to explain Marley and Zed to \
the user and to set Marley up the way they want.

- To answer how something works, use docs_search, then docs_read: they read this Marley's own \
docs and guide, so they match the version that runs. Quote the docs rather than guessing.
- To say what a setting does or is set to, use settings_schema and settings_read. Name the key \
path, the value now and where it is set.
- To change a setting, use settings_change, and to bind a key, keymap_change. The user sees the \
change and accepts or declines it; say what you proposed and why, and never claim a change was \
made before it answers applied.
- To set up a seat on the harness Marley follows, such as a manager working in a folder, use \
seat_add. The user accepts it first; it then starts and shows in the rail.
- To find which key runs a command, use actions_list.

You do not edit files, run commands or change code here. When the user asks for that, say so and \
suggest they ask an agent in a terminal.";

/// What Rusty's entry is told, in the same places.
const RUSTY_INSTRUCTIONS: &str = "You are Rusty, the user's personal assistant, inside Marley, a \
code editor built on Zed. Rusty keeps the user's to-do lists, notes, long-term memories, their \
brain (a markdown wiki) and their skills, and the rusty server's tools are how you reach them.

- Look things up before you answer: search the brain and the memories, and read a page rather \
than guessing.
- Add or change a task, a note, a memory or a page when the user asks, and say what you changed.
- Questions about Marley itself and its settings are the Marley agent's: suggest the user ask it \
in the Agent Panel.

You do not edit files, run commands or change code here. When the user asks for that, say so and \
suggest they ask an agent in a terminal.";

/// The Agent Panel entries Marley adds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Entry {
    /// Marley's own agent (#683).
    Marley,
    /// Rusty's (#696).
    Rusty,
}

impl Entry {
    const ALL: [Self; 2] = [Self::Marley, Self::Rusty];

    /// The entry's name in the Agent Panel, and its key in the agent servers and the session meta.
    const fn name(self) -> &'static str {
        match self {
            Self::Marley => "Marley",
            Self::Rusty => "Rusty",
        }
    }

    /// Its profile's key in `agent.profiles`, for Zed's own agent.
    const fn profile(self) -> &'static str {
        match self {
            Self::Marley => "marley",
            Self::Rusty => "rusty",
        }
    }

    const fn instructions(self) -> &'static str {
        match self {
            Self::Marley => INSTRUCTIONS,
            Self::Rusty => RUSTY_INSTRUCTIONS,
        }
    }
}

/// The entries, as the defaults hold them or as the settings want them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Entries {
    /// The agent they run on; none while neither is there.
    agent: Option<MarleyAssistantAgent>,
    /// Whether the Marley entry is there.
    marley: bool,
    /// Where Rusty's entry reaches Rusty, while it is there.
    rusty: Option<RustyServer>,
}

impl Entries {
    /// What `entry` runs on, and Rusty's server for Rusty's; none while it is not there.
    fn of(&self, entry: Entry) -> Option<(MarleyAssistantAgent, Option<&RustyServer>)> {
        let agent = self.agent?;
        match entry {
            Entry::Marley => self.marley.then_some((agent, None)),
            Entry::Rusty => self.rusty.as_ref().map(|server| (agent, Some(server))),
        }
    }

    /// Whether the Marley entry is there on an agent with a terminal interface of its own.
    fn marley_in_terminal(&self) -> bool {
        self.marley && self.agent.and_then(adapter).is_some()
    }

    /// Whether Rusty's entry is Zed's agent's `rusty` profile, which needs Rusty's context server.
    fn rusty_profile(&self) -> bool {
        self.agent == Some(MarleyAssistantAgent::Zed) && self.rusty.is_some()
    }
}

/// Where `auto`'s look for an agent stands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Detected {
    /// It has not looked, or the setting moved off `auto` since.
    #[default]
    NotYet,
    Looking,
    /// It looked: the agent it found, or none.
    Found(Option<MarleyAssistantAgent>),
}

/// What the defaults hold for the entries, what `auto` found, and the adapter's command once
/// resolved.
#[derive(Default)]
struct Assistant {
    applied: Entries,
    detected: Detected,
    /// The adapter's command, resolved once a project's agent server store has it.
    command: Option<AgentServerCommand>,
    /// Whether Marley put the adapter in the defaults, the user's settings lacking it.
    added_adapter: bool,
    /// The agent whose adapter is being resolved.
    resolving: Option<MarleyAssistantAgent>,
    /// Whether the palette lists the agent's command, once its filter was set.
    palette_shown: Option<bool>,
}

impl Global for Assistant {}

/// Follows `marley.assistant` and Rusty's connection.
pub(crate) fn init(cx: &mut App) {
    cx.set_global(Assistant::default());
    sync(cx);
    cx.observe_global::<SettingsStore>(sync).detach();
    cx.observe_global::<crate::rusty::Rusty>(sync).detach();
    cx.observe_new(|workspace: &mut Workspace, _, cx| {
        workspace.register_action(|workspace, _: &OpenMarleyAgentInTerminal, window, cx| {
            open_in_terminal(workspace, window, cx);
        });
        cx.defer(|cx| {
            filter_palette(cx.global::<Assistant>().applied.marley_in_terminal(), cx);
            if cx.global::<Assistant>().command.is_none() {
                resolve(cx);
            }
        });
    })
    .detach();
    cx.spawn(async move |cx| {
        cx.background_executor().timer(ZED_MODEL_AFTER).await;
        // A Marley profile gone while Marley was closed leaves a user's default profile naming
        // it; `auto`, still looking, does this when it is done.
        cx.update(|cx| {
            if cx.global::<Assistant>().detected != Detected::Looking {
                forget_default_profile(Entry::Marley, cx);
            }
        });
    })
    .detach();
}

/// Whether Zed's agent's `rusty` profile is in the defaults, so Rusty's context server must be
/// offered to Zed's agents.
pub(crate) fn wants_rusty_profile(cx: &App) -> bool {
    cx.try_global::<Assistant>()
        .is_some_and(|assistant| assistant.applied.rusty_profile())
}

/// `marley.assistant` as the settings merge it: whether the Marley entry is on, and the agent.
fn setting(cx: &App) -> (bool, MarleyAssistantAgent) {
    let assistant = cx
        .global::<SettingsStore>()
        .merged_settings()
        .marley
        .as_ref()
        .and_then(|marley| marley.assistant.as_ref());
    (
        assistant
            .and_then(|assistant| assistant.enabled)
            .unwrap_or(false),
        assistant
            .and_then(|assistant| assistant.agent)
            .unwrap_or_default(),
    )
}

/// The entries the settings and Rusty's connection ask for.
fn wanted(cx: &App) -> Entries {
    let (enabled, agent) = setting(cx);
    let rusty = crate::rusty::agent_server(cx);
    let agent = match agent {
        MarleyAssistantAgent::Auto => match cx.global::<Assistant>().detected {
            Detected::Found(found) => found,
            Detected::NotYet | Detected::Looking => None,
        },
        agent => Some(agent),
    };
    match agent {
        Some(agent) if enabled || rusty.is_some() => Entries {
            agent: Some(agent),
            marley: enabled,
            rusty,
        },
        _ => Entries::default(),
    }
}

/// The registry adapter an agent's entries run; Zed's own agent has none.
const fn adapter(agent: MarleyAssistantAgent) -> Option<&'static str> {
    match agent {
        MarleyAssistantAgent::ClaudeCode => Some("claude-acp"),
        MarleyAssistantAgent::Codex => Some("codex-acp"),
        MarleyAssistantAgent::Zed | MarleyAssistantAgent::Auto => None,
    }
}

/// Puts the wanted entries in the defaults and takes out the ones no longer wanted, each only
/// when it changed, so an entry a thread runs on stays while the other moves.
fn sync(cx: &mut App) {
    detect(cx);
    let wanted = wanted(cx);
    filter_palette(wanted.marley_in_terminal(), cx);
    let applied = cx.global::<Assistant>().applied.clone();
    if wanted == applied {
        return;
    }
    for entry in Entry::ALL {
        if applied.of(entry).is_some() && applied.of(entry) != wanted.of(entry) {
            take_out(entry, cx);
        }
    }
    if applied.agent != wanted.agent
        && let Some(agent) = applied.agent
    {
        drop_adapter(agent, cx);
    }
    cx.global_mut::<Assistant>().applied = wanted.clone();
    if applied.agent != wanted.agent
        && let Some(agent) = wanted.agent
    {
        add_adapter(agent, cx);
    }
    for entry in Entry::ALL {
        if let Some((agent, rusty)) = wanted.of(entry)
            && applied.of(entry) != wanted.of(entry)
        {
            put_in(entry, agent, rusty, cx);
        }
    }
    if applied.rusty_profile() != wanted.rusty_profile() {
        crate::rusty::offer_again(cx);
    }
}

/// Looks for the agent `auto` stands for, once, while `auto` is set and an entry could want it.
/// The setting moved off `auto` forgets what was found, so `auto` chosen again looks again.
fn detect(cx: &mut App) {
    let (enabled, agent) = setting(cx);
    if agent != MarleyAssistantAgent::Auto {
        cx.global_mut::<Assistant>().detected = Detected::NotYet;
        return;
    }
    let wants_an_agent = enabled || crate::rusty::is_on(cx);
    if !wants_an_agent || cx.global::<Assistant>().detected != Detected::NotYet {
        return;
    }
    cx.global_mut::<Assistant>().detected = Detected::Looking;
    let claude = program(AgentKind::Claude, cx);
    let codex = program(AgentKind::Codex, cx);
    cx.spawn(async move |cx| {
        cx.background_executor().timer(DETECT_AFTER).await;
        let found = if claude_signed_in(&claude).await {
            Some(MarleyAssistantAgent::ClaudeCode)
        } else if codex_signed_in(&codex).await {
            Some(MarleyAssistantAgent::Codex)
        } else {
            cx.background_executor().timer(ZED_MODEL_AFTER).await;
            cx.update(|cx| zed_model_ready(cx).then_some(MarleyAssistantAgent::Zed))
        };
        cx.update(|cx| {
            match found {
                Some(agent) => log::info!("assistant: auto chose {}", label(agent)),
                None => log::info!(
                    "assistant: auto found no agent: Claude Code and Codex are not signed in, \
                     and Zed's agent has no default model set up"
                ),
            }
            // The setting moved off `auto` while it looked: what it found answers nothing.
            let assistant = cx.global_mut::<Assistant>();
            if assistant.detected != Detected::Looking {
                return;
            }
            assistant.detected = Detected::Found(found);
            sync(cx);
            forget_default_profile(Entry::Marley, cx);
        });
    })
    .detach();
}

/// An agent's name, for the log.
const fn label(agent: MarleyAssistantAgent) -> &'static str {
    match agent {
        MarleyAssistantAgent::Auto => "auto",
        MarleyAssistantAgent::ClaudeCode => "Claude Code",
        MarleyAssistantAgent::Codex => "Codex",
        MarleyAssistantAgent::Zed => "Zed's agent",
    }
}

/// Puts `agent`'s adapter in the defaults when the user's settings lack it.
fn add_adapter(agent: MarleyAssistantAgent, cx: &mut App) {
    let Some(adapter) = adapter(agent) else {
        return;
    };
    let user_has_adapter = cx
        .global::<SettingsStore>()
        .raw_user_settings()
        .and_then(|user| user.content.agent_servers.as_ref())
        .is_some_and(|servers| servers.contains_key(adapter));
    cx.global_mut::<Assistant>().added_adapter = !user_has_adapter;
    if user_has_adapter {
        return;
    }
    cx.update_global::<SettingsStore, _>(|store, cx| {
        store.update_default_settings(cx, |defaults| {
            defaults.agent_servers.get_or_insert_default().insert(
                adapter.to_string(),
                CustomAgentServerSettings::Registry {
                    env: HashMap::default(),
                    default_mode: None,
                    default_config_options: HashMap::default(),
                    favorite_config_option_values: HashMap::default(),
                },
            );
        });
    });
}

/// Takes the adapter `add_adapter` added out of the defaults, and forgets its command.
fn drop_adapter(agent: MarleyAssistantAgent, cx: &mut App) {
    let assistant = cx.global_mut::<Assistant>();
    let added_adapter = std::mem::take(&mut assistant.added_adapter);
    assistant.command = None;
    let Some(adapter) = adapter(agent).filter(|_| added_adapter) else {
        return;
    };
    cx.update_global::<SettingsStore, _>(|store, cx| {
        store.update_default_settings(cx, |defaults| {
            if let Some(servers) = defaults.agent_servers.as_mut() {
                servers.remove(adapter);
            }
        });
    });
}

/// Puts `entry` in the defaults on `agent`: the profile for Zed's agent; for the others the
/// session meta, then the entry with the adapter's command, resolved first when it must be.
fn put_in(entry: Entry, agent: MarleyAssistantAgent, rusty: Option<&RustyServer>, cx: &mut App) {
    if adapter(agent).is_none() {
        let profile = profile(entry);
        cx.update_global::<SettingsStore, _>(|store, cx| {
            store.update_default_settings(cx, |defaults| {
                defaults
                    .agent
                    .get_or_insert_default()
                    .profiles
                    .get_or_insert_default()
                    .insert(entry.profile().into(), profile);
            });
        });
        return;
    }
    if agent == MarleyAssistantAgent::ClaudeCode {
        let meta = session_meta(entry, rusty);
        cx.update_global::<SettingsStore, _>(|store, cx| {
            store.update_default_settings(cx, |defaults| {
                defaults
                    .marley
                    .get_or_insert_default()
                    .agent_session_meta
                    .get_or_insert_default()
                    .insert(entry.name().to_string(), meta);
            });
        });
    }
    match cx.global::<Assistant>().command.clone() {
        Some(command) => set_entry(entry, agent, &command, rusty, cx),
        None => resolve(cx),
    }
}

/// Takes what `put_in` added for `entry` out of the defaults.
fn take_out(entry: Entry, cx: &mut App) {
    cx.update_global::<SettingsStore, _>(|store, cx| {
        store.update_default_settings(cx, |defaults| {
            if let Some(meta) = defaults
                .marley
                .as_mut()
                .and_then(|marley| marley.agent_session_meta.as_mut())
            {
                meta.remove(entry.name());
            }
            if let Some(servers) = defaults.agent_servers.as_mut() {
                servers.remove(entry.name());
            }
            if let Some(profiles) = defaults
                .agent
                .as_mut()
                .and_then(|agent| agent.profiles.as_mut())
            {
                profiles.shift_remove(entry.profile());
            }
        });
    });
    forget_default_profile(entry, cx);
}

/// The `_meta` each Claude Code session of `entry` carries: its instructions as an append to the
/// system prompt, the tools it may not use, and no bypass mode. Rusty's also names Rusty's server,
/// which the adapter adds to the servers Zed passes, and keeps Marley's tools out.
fn session_meta(entry: Entry, rusty: Option<&RustyServer>) -> Value {
    let mut disallowed: Vec<String> = DISALLOWED_TOOLS.iter().map(ToString::to_string).collect();
    let mut options = Map::new();
    if let Some(server) = rusty {
        disallowed.push(format!("mcp__{}", crate::mcp::CONTEXT_SERVER));
        let mut servers = Map::new();
        servers.insert(
            crate::rusty::CONTEXT_SERVER.to_string(),
            match server {
                RustyServer::Stdio(path) => json!({
                    "type": "stdio",
                    "command": path.display().to_string(),
                    "args": [],
                }),
                RustyServer::Http(url) => json!({ "type": "http", "url": url }),
            },
        );
        options.insert("mcpServers".to_string(), Value::Object(servers));
    }
    options.insert("disallowedTools".to_string(), json!(disallowed));
    options.insert(
        "allowDangerouslySkipPermissions".to_string(),
        Value::Bool(false),
    );
    json!({
        "systemPrompt": { "append": entry.instructions() },
        "claudeCode": { "options": options },
    })
}

/// The config `codex-acp` merges into each Codex thread of `entry`: its instructions, the
/// read-only sandbox its read-only mode also asks for, and for Rusty's, Rusty's server.
fn codex_config(entry: Entry, rusty: Option<&RustyServer>) -> Value {
    let mut config = Map::new();
    config.insert(
        "developer_instructions".to_string(),
        Value::String(entry.instructions().to_string()),
    );
    config.insert(
        "sandbox_mode".to_string(),
        Value::String("read-only".to_string()),
    );
    if let Some(server) = rusty {
        let mut servers = Map::new();
        servers.insert(
            crate::rusty::CONTEXT_SERVER.to_string(),
            match server {
                RustyServer::Stdio(path) => {
                    json!({ "command": path.display().to_string(), "args": [] })
                }
                RustyServer::Http(url) => json!({ "url": url }),
            },
        );
        config.insert("mcp_servers".to_string(), Value::Object(servers));
    }
    Value::Object(config)
}

/// Zed's agent's profile for `entry`, with no built-in tool. Marley's turns on Marley's tools that
/// read and propose. Rusty's turns on the context servers' tools, Rusty's among them, since a
/// profile names a server's tools one by one and Rusty's are its own to change; Zed asks before
/// each call.
fn profile(entry: Entry) -> AgentProfileContent {
    let mut context_servers = IndexMap::default();
    if entry == Entry::Marley {
        context_servers.insert(
            Arc::from(crate::mcp::CONTEXT_SERVER),
            ContextServerPresetContent {
                tools: PROFILE_TOOLS
                    .iter()
                    .map(|tool| (Arc::from(*tool), true))
                    .collect(),
            },
        );
    }
    AgentProfileContent {
        name: entry.name().into(),
        tools: IndexMap::default(),
        enable_all_context_servers: Some(entry == Entry::Rusty),
        context_servers,
        default_model: None,
    }
}

/// Puts a user's `agent.default_profile` that names `entry`'s profile, which is gone, back to
/// Zed's default, as Zed's own profile delete does: a thread whose profile is missing gets no
/// tools.
fn forget_default_profile(entry: Entry, cx: &App) {
    let store = cx.global::<SettingsStore>();
    let names_it = store
        .raw_user_settings()
        .and_then(|user| user.content.agent.as_ref())
        .and_then(|agent| agent.default_profile.as_deref())
        == Some(entry.profile());
    let exists = store
        .merged_settings()
        .agent
        .as_ref()
        .and_then(|agent| agent.profiles.as_ref())
        .is_some_and(|profiles| profiles.contains_key(entry.profile()));
    if !names_it || exists {
        return;
    }
    settings::update_settings_file(<dyn Fs>::global(cx), cx, move |content, _| {
        if let Some(agent) = content.agent.as_mut()
            && agent.default_profile.as_deref() == Some(entry.profile())
        {
            agent.default_profile = Some(AgentProfileId::default().0);
        }
    });
}

/// The first open project's agent server store.
fn agent_server_store(cx: &App) -> Option<Entity<AgentServerStore>> {
    cx.windows()
        .into_iter()
        .filter_map(|window| window.downcast::<MultiWorkspace>()?.read(cx).ok())
        .flat_map(|multi_workspace| multi_workspace.workspaces().cloned().collect::<Vec<_>>())
        .map(|workspace| {
            workspace
                .read(cx)
                .project()
                .read(cx)
                .agent_server_store()
                .clone()
        })
        .next()
}

/// Asks a project's agent server store for the applied agent's adapter command, installing it
/// when it must, and puts the wanted entries in the defaults with it. With no project open yet, a
/// new workspace asks again.
fn resolve(cx: &mut App) {
    let Some(agent) = cx.global::<Assistant>().applied.agent else {
        return;
    };
    let Some(adapter) = adapter(agent) else {
        return;
    };
    if cx.global::<Assistant>().resolving == Some(agent) {
        return;
    }
    if let Some(stand_in) = std::env::var_os("MARLEY_ASSISTANT_ADAPTER") {
        set_entries(
            agent,
            AgentServerCommand {
                path: PathBuf::from(stand_in),
                args: Vec::new(),
                env: None,
            },
            cx,
        );
        return;
    }
    let Some(store) = agent_server_store(cx) else {
        return;
    };
    let task = store.update(cx, |store, cx| {
        store
            .get_external_agent(&AgentId::new(adapter))
            .map(|adapter| adapter.get_command(Vec::new(), HashMap::default(), &mut cx.to_async()))
    });
    let Some(task) = task else {
        log::info!("assistant: `{adapter}` is not registered yet; a new workspace asks again");
        return;
    };
    cx.global_mut::<Assistant>().resolving = Some(agent);
    cx.spawn(async move |cx| {
        let command = task.await;
        cx.update(|cx| {
            let assistant = cx.global_mut::<Assistant>();
            if assistant.resolving == Some(agent) {
                assistant.resolving = None;
            }
            let still_applied = assistant.applied.agent == Some(agent);
            match command {
                // The agent may have changed meanwhile; its own resolution sets its entries.
                Ok(command) if still_applied => set_entries(agent, command, cx),
                Ok(_) => {}
                Err(error) => log::error!("assistant: `{adapter}` could not start: {error:#}"),
            }
        });
    })
    .detach();
}

/// Keeps `command` as the adapter's, and puts each applied entry in the defaults with it.
fn set_entries(agent: MarleyAssistantAgent, command: AgentServerCommand, cx: &mut App) {
    let applied = cx.global::<Assistant>().applied.clone();
    for entry in Entry::ALL {
        if let Some((_, rusty)) = applied.of(entry) {
            set_entry(entry, agent, &command, rusty, cx);
        }
    }
    cx.global_mut::<Assistant>().command = Some(command);
}

/// Puts `entry` in the agent servers of the defaults: the adapter's command, with what tells it to
/// run as that entry on the user's own login.
fn set_entry(
    entry: Entry,
    agent: MarleyAssistantAgent,
    command: &AgentServerCommand,
    rusty: Option<&RustyServer>,
    cx: &mut App,
) {
    let mut env = command.env.clone().unwrap_or_default();
    match agent {
        MarleyAssistantAgent::ClaudeCode => {
            env.insert(
                "CLAUDE_CODE_EXECUTABLE".to_string(),
                program(AgentKind::Claude, cx).display().to_string(),
            );
            env.insert("ANTHROPIC_API_KEY".to_string(), String::new());
        }
        MarleyAssistantAgent::Codex => {
            env.insert("INITIAL_AGENT_MODE".to_string(), "read-only".to_string());
            env.insert(
                "CODEX_CONFIG".to_string(),
                codex_config(entry, rusty).to_string(),
            );
        }
        MarleyAssistantAgent::Zed | MarleyAssistantAgent::Auto => {}
    }
    let server = CustomAgentServerSettings::Custom {
        path: command.path.clone(),
        args: command.args.clone(),
        env,
        default_mode: None,
        default_config_options: HashMap::default(),
        favorite_config_option_values: HashMap::default(),
    };
    cx.update_global::<SettingsStore, _>(|store, cx| {
        store.update_default_settings(cx, |defaults| {
            defaults
                .agent_servers
                .get_or_insert_default()
                .insert(entry.name().to_string(), server);
        });
    });
}

/// The `claude` or `codex` Marley launches: the one the last version check found, else
/// `MARLEY_CLAUDE` or `MARLEY_CODEX`, else the search path's.
fn program(kind: AgentKind, cx: &App) -> PathBuf {
    crate::agent_versions::program(kind, cx)
        .or_else(|| std::env::var_os(crate::agent_versions::variable(kind)).map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(kind.program()))
}

/// Whether `claude auth status` says Claude Code is signed in. It prints no token.
async fn claude_signed_in(claude: &Path) -> bool {
    match crate::process::output(claude, ["auth", "status"], None, &[]).await {
        Ok(output) => serde_json::from_slice::<Value>(&output.stdout)
            .ok()
            .and_then(|status| status.get("loggedIn").and_then(Value::as_bool))
            .unwrap_or(false),
        Err(error) => {
            log::debug!("assistant: `claude auth status` did not run: {error}");
            false
        }
    }
}

/// Whether `codex login status` says Codex is signed in: it exits with success only then, and
/// what it prints is not read.
async fn codex_signed_in(codex: &Path) -> bool {
    match crate::process::output(codex, ["login", "status"], None, &[]).await {
        Ok(output) => output.status.success(),
        Err(error) => {
            log::debug!("assistant: `codex login status` did not run: {error}");
            false
        }
    }
}

/// Whether Zed's agent has a default model whose provider is signed in.
fn zed_model_ready(cx: &App) -> bool {
    let registry = LanguageModelRegistry::read_global(cx);
    registry
        .default_model()
        .and_then(|model| registry.provider(&model.provider_id))
        .is_some_and(|provider| provider.is_authenticated(cx))
}

/// Lists the agent's command in the palette while the Marley entry is there on an agent with a
/// terminal interface, as Rusty's are while Rusty is (#661). Before the palette has its filter
/// there is nothing to set; the next call sets it.
fn filter_palette(on: bool, cx: &mut App) {
    if cx.global::<Assistant>().palette_shown == Some(on)
        || CommandPaletteFilter::try_global(cx).is_none()
    {
        return;
    }
    cx.global_mut::<Assistant>().palette_shown = Some(on);
    let command = [TypeId::of::<OpenMarleyAgentInTerminal>()];
    CommandPaletteFilter::update_global(cx, |filter, _| {
        if on {
            filter.show_action_types(&command);
        } else {
            filter.hide_action_types(&command);
        }
    });
}

/// `marley: open marley agent in terminal`: starts the agent in a center terminal of
/// `workspace`. Claude Code reads the instructions from a file, written first, with the tools
/// turned off; Codex takes them as a config value, in its read-only sandbox.
fn open_in_terminal(workspace: &Workspace, window: &Window, cx: &gpui::Context<Workspace>) {
    let marley = cx
        .global::<Assistant>()
        .applied
        .of(Entry::Marley)
        .map(|(agent, _)| agent);
    let (kind, line, written) = match marley {
        Some(MarleyAssistantAgent::ClaudeCode) => {
            let file = paths::data_dir().join("assistant").join("instructions.md");
            let line = format!(
                "{} --append-system-prompt-file {} --disallowedTools {}\n",
                quoted(&program(AgentKind::Claude, cx).display().to_string()),
                quoted(&file.display().to_string()),
                DISALLOWED_TOOLS.join(" ")
            );
            let written = cx
                .background_executor()
                .spawn(futures::future::lazy(move |_| write_instructions(&file)));
            (AgentKind::Claude, line, Some(written))
        }
        Some(MarleyAssistantAgent::Codex) => {
            // A JSON string is also a TOML basic string, which `-c` reads its value as.
            let instructions = Value::String(INSTRUCTIONS.to_string());
            let line = format!(
                "{} --sandbox read-only -c {}\n",
                quoted(&program(AgentKind::Codex, cx).display().to_string()),
                quoted(&format!("developer_instructions={instructions}"))
            );
            (AgentKind::Codex, line, None)
        }
        Some(MarleyAssistantAgent::Zed | MarleyAssistantAgent::Auto) | None => return,
    };
    let directory = terminal_view::default_working_directory(workspace, cx);
    cx.spawn_in(window, async move |workspace, cx| {
        if let Some(written) = written {
            written.await?;
        }
        workspace
            .update_in(cx, |workspace, window, cx| {
                crate::agents::start_in_terminal(
                    workspace,
                    directory,
                    Some(kind),
                    Some(line.into_bytes()),
                    None,
                    window,
                    cx,
                )
            })?
            .await?;
        anyhow::Ok(())
    })
    .detach_and_log_err(cx);
}

/// Writes the instructions file, its folder made where missing.
fn write_instructions(file: &Path) -> anyhow::Result<()> {
    if let Some(folder) = file.parent() {
        std::fs::create_dir_all(folder)?;
    }
    std::fs::write(file, INSTRUCTIONS)?;
    Ok(())
}

/// `text` as one word of a shell's command line.
fn quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}
