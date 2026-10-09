//! Marley's own agent (#683, #687): an agent that explains and configures Marley.
//!
//! `marley.assistant.agent` picks what it runs on:
//! - Claude Code (#683): an entry in the Agent Panel running the registry's `claude-acp` adapter
//!   on the user's own login, with instructions appended to its system prompt and the tools that
//!   edit files or run commands disallowed. The adapter reads both from each session's `_meta`,
//!   which Zed sends from `marley.agent_session_meta` (a hunk in `agent_servers`).
//! - Codex (#687): the same entry running the registry's `codex-acp` in its read-only mode, with
//!   the instructions as Codex's `developer_instructions` in `CODEX_CONFIG`, which the adapter
//!   merges into each thread's config.
//! - Zed's own agent (#687): a `marley` profile with Marley's tools and no file tools. Zed's agent
//!   takes no prompt per profile and drops an MCP server's instructions, so the tools' own
//!   descriptions guide it.
//!
//! Marley's MCP server, which Zed hands every agent, gives each of them the docs and settings
//! tools (#681, #682, #686).
//!
//! While `marley.assistant.enabled` is on, what the agent needs lives in the settings' in-memory
//! defaults, as `context_servers.marley` does (`mcp::offer_to_zeds_agents`): a custom agent named
//! `Marley` whose command is the one Zed resolves for the adapter, so the adapter Zed installed
//! runs it, or the profile. Off, nothing of it is there. Once, when the user's settings do not
//! name the switch, a notification offers it on the first agent found signed in: Claude Code,
//! then Codex, then Zed's default model.
//!
//! `marley: open marley agent in terminal` (#684) runs the same agent as its own interface in a
//! center terminal: `claude` with the instructions as a system-prompt file and the same tools
//! turned off, or `codex` in its read-only sandbox with the instructions as its developer
//! instructions. The palette lists it only while the switch is on and the agent has a terminal
//! interface.
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
use gpui::{
    App, AppContext as _, BorrowAppContext as _, Entity, Global, TaskExt as _, Window, actions,
};
use language_model::LanguageModelRegistry;
use marley_agent::AgentKind;
use project::agent_server_store::{AgentId, AgentServerCommand, AgentServerStore};
use serde_json::{Value, json};
use settings::{
    AgentProfileContent, ContextServerPresetContent, CustomAgentServerSettings,
    MarleyAssistantAgent, SettingsStore,
};
use workspace::notifications::simple_message_notification::MessageNotification;
use workspace::notifications::{NotificationId, show_app_notification};
use workspace::{MultiWorkspace, Workspace};

actions!(
    marley,
    [
        /// Opens Marley's own agent in a terminal: Claude Code or Codex with the Marley agent's
        /// instructions, and no file edits.
        #[derive(Eq)]
        OpenMarleyAgentInTerminal
    ]
);

/// The entry's name in the Agent Panel and its key in the agent servers.
pub(crate) const ENTRY: &str = "Marley";

/// The profile's key in `agent.profiles`, for Zed's own agent.
const PROFILE: &str = "marley";

/// Claude Code's tools the agent may not use: it explains and configures, through Marley's
/// tools, and changes no file and runs no command.
const DISALLOWED_TOOLS: [&str; 5] = ["Bash", "Edit", "Write", "NotebookEdit", "MultiEdit"];

/// Marley's tools the profile turns on for Zed's own agent: the ones that read the docs and the
/// settings, and the three that propose a change the user accepts.
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

/// How long after start the offer waits, so the user's settings and Zed's model providers have
/// loaded.
const OFFER_AFTER: Duration = Duration::from_secs(5);

/// What the agent is told: appended to Claude Code's system prompt, and Codex's developer
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

/// What the defaults hold for the agent, and the adapter's command once resolved.
#[derive(Default)]
struct Assistant {
    /// The agent whose entry or profile is in the defaults; none while the switch is off.
    applied: Option<MarleyAssistantAgent>,
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

/// The offer's notification.
struct AssistantOffer;

/// Follows the switch, and offers the agent once.
pub(crate) fn init(cx: &mut App) {
    cx.set_global(Assistant::default());
    sync(cx);
    cx.observe_global::<SettingsStore>(sync).detach();
    cx.observe_new(|workspace: &mut Workspace, _, cx| {
        workspace.register_action(|workspace, _: &OpenMarleyAgentInTerminal, window, cx| {
            open_in_terminal(workspace, window, cx);
        });
        cx.defer(|cx| {
            filter_palette(has_terminal(wanted(cx)), cx);
            if cx.global::<Assistant>().command.is_none() {
                resolve(cx);
            }
        });
    })
    .detach();
    offer_later(cx);
}

/// The agent the settings ask for: `marley.assistant.agent` while `marley.assistant.enabled` is
/// on.
fn wanted(cx: &App) -> Option<MarleyAssistantAgent> {
    let assistant = cx
        .global::<SettingsStore>()
        .merged_settings()
        .marley
        .as_ref()?
        .assistant
        .as_ref()?;
    assistant
        .enabled
        .unwrap_or(false)
        .then(|| assistant.agent.unwrap_or_default())
}

/// The registry adapter an agent's entry runs; Zed's own agent has none.
const fn adapter(agent: MarleyAssistantAgent) -> Option<&'static str> {
    match agent {
        MarleyAssistantAgent::ClaudeCode => Some("claude-acp"),
        MarleyAssistantAgent::Codex => Some("codex-acp"),
        MarleyAssistantAgent::Zed => None,
    }
}

/// Whether `agent` has an interface of its own to open in a terminal.
fn has_terminal(agent: Option<MarleyAssistantAgent>) -> bool {
    agent.is_some_and(|agent| adapter(agent).is_some())
}

/// Puts the wanted agent's entry or profile in the defaults, and takes out the one there before,
/// when the switch or the agent moved.
fn sync(cx: &mut App) {
    let wanted = wanted(cx);
    filter_palette(has_terminal(wanted), cx);
    let applied = cx.global::<Assistant>().applied;
    if wanted == applied {
        return;
    }
    if let Some(applied) = applied {
        take_out(applied, cx);
    }
    cx.global_mut::<Assistant>().applied = wanted;
    if let Some(agent) = wanted {
        put_in(agent, cx);
    }
}

/// Puts `agent` in the defaults: the profile for Zed's agent; for the others the session meta,
/// the adapter when the user's settings lack it, then the entry once the adapter's command is
/// resolved.
fn put_in(agent: MarleyAssistantAgent, cx: &mut App) {
    let Some(adapter) = adapter(agent) else {
        cx.update_global::<SettingsStore, _>(|store, cx| {
            store.update_default_settings(cx, |defaults| {
                defaults
                    .agent
                    .get_or_insert_default()
                    .profiles
                    .get_or_insert_default()
                    .insert(PROFILE.into(), profile());
            });
        });
        return;
    };
    let user_has_adapter = cx
        .global::<SettingsStore>()
        .raw_user_settings()
        .and_then(|user| user.content.agent_servers.as_ref())
        .is_some_and(|servers| servers.contains_key(adapter));
    cx.global_mut::<Assistant>().added_adapter = !user_has_adapter;
    cx.update_global::<SettingsStore, _>(|store, cx| {
        store.update_default_settings(cx, |defaults| {
            if agent == MarleyAssistantAgent::ClaudeCode {
                defaults
                    .marley
                    .get_or_insert_default()
                    .agent_session_meta
                    .get_or_insert_default()
                    .insert(ENTRY.to_string(), session_meta());
            }
            if !user_has_adapter {
                defaults.agent_servers.get_or_insert_default().insert(
                    adapter.to_string(),
                    CustomAgentServerSettings::Registry {
                        env: HashMap::default(),
                        default_mode: None,
                        default_config_options: HashMap::default(),
                        favorite_config_option_values: HashMap::default(),
                    },
                );
            }
        });
    });
    resolve(cx);
}

/// Takes what `put_in` added for `agent` out of the defaults.
fn take_out(agent: MarleyAssistantAgent, cx: &mut App) {
    let added_adapter = cx.global::<Assistant>().added_adapter;
    cx.update_global::<SettingsStore, _>(|store, cx| {
        store.update_default_settings(cx, |defaults| {
            if let Some(meta) = defaults
                .marley
                .as_mut()
                .and_then(|marley| marley.agent_session_meta.as_mut())
            {
                meta.remove(ENTRY);
            }
            if let Some(servers) = defaults.agent_servers.as_mut() {
                servers.remove(ENTRY);
                if let Some(adapter) = adapter(agent).filter(|_| added_adapter) {
                    servers.remove(adapter);
                }
            }
            if let Some(profiles) = defaults
                .agent
                .as_mut()
                .and_then(|agent| agent.profiles.as_mut())
            {
                profiles.shift_remove(PROFILE);
            }
        });
    });
    let assistant = cx.global_mut::<Assistant>();
    assistant.command = None;
    assistant.added_adapter = false;
    forget_default_profile(cx);
}

/// The `_meta` each Claude Code session of the entry carries: the instructions as an append to
/// its system prompt, the tools it may not use, and no bypass mode.
fn session_meta() -> Value {
    json!({
        "systemPrompt": { "append": INSTRUCTIONS },
        "claudeCode": {
            "options": {
                "disallowedTools": DISALLOWED_TOOLS,
                "allowDangerouslySkipPermissions": false,
            }
        }
    })
}

/// The config `codex-acp` merges into each Codex thread's: the instructions, and the read-only
/// sandbox its read-only mode also asks for.
fn codex_config() -> Value {
    json!({
        "developer_instructions": INSTRUCTIONS,
        "sandbox_mode": "read-only",
    })
}

/// Zed's agent's Marley profile: no built-in tool, and of the context servers' tools only
/// Marley's that read and propose.
fn profile() -> AgentProfileContent {
    let mut context_servers = IndexMap::default();
    context_servers.insert(
        Arc::from(crate::mcp::CONTEXT_SERVER),
        ContextServerPresetContent {
            tools: PROFILE_TOOLS
                .iter()
                .map(|tool| (Arc::from(*tool), true))
                .collect(),
        },
    );
    AgentProfileContent {
        name: ENTRY.into(),
        tools: IndexMap::default(),
        enable_all_context_servers: Some(false),
        context_servers,
        default_model: None,
    }
}

/// Puts a user's `agent.default_profile` that names the Marley profile, which is gone, back to
/// Zed's default, as Zed's own profile delete does: a thread whose profile is missing gets no
/// tools.
fn forget_default_profile(cx: &App) {
    let store = cx.global::<SettingsStore>();
    let names_it = store
        .raw_user_settings()
        .and_then(|user| user.content.agent.as_ref())
        .and_then(|agent| agent.default_profile.as_deref())
        == Some(PROFILE);
    let exists = store
        .merged_settings()
        .agent
        .as_ref()
        .and_then(|agent| agent.profiles.as_ref())
        .is_some_and(|profiles| profiles.contains_key(PROFILE));
    if !names_it || exists {
        return;
    }
    settings::update_settings_file(<dyn Fs>::global(cx), cx, |content, _| {
        if let Some(agent) = content.agent.as_mut()
            && agent.default_profile.as_deref() == Some(PROFILE)
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
/// when it must, and puts the entry in the defaults with it. With no project open yet, a new
/// workspace asks again.
fn resolve(cx: &mut App) {
    let Some(agent) = cx.global::<Assistant>().applied else {
        return;
    };
    let Some(adapter) = adapter(agent) else {
        return;
    };
    if cx.global::<Assistant>().resolving == Some(agent) {
        return;
    }
    if let Some(stand_in) = std::env::var_os("MARLEY_ASSISTANT_ADAPTER") {
        set_entry(
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
            let still_applied = assistant.applied == Some(agent);
            match command {
                // The agent may have changed meanwhile; its own resolution sets its entry.
                Ok(command) if still_applied => set_entry(agent, command, cx),
                Ok(_) => {}
                Err(error) => log::error!("assistant: `{adapter}` could not start: {error:#}"),
            }
        });
    })
    .detach();
}

/// Puts the `Marley` entry in the defaults: the adapter's command, with what tells it to run as
/// the Marley agent on the user's own login.
fn set_entry(agent: MarleyAssistantAgent, command: AgentServerCommand, cx: &mut App) {
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
            env.insert("CODEX_CONFIG".to_string(), codex_config().to_string());
        }
        MarleyAssistantAgent::Zed => {}
    }
    let entry = CustomAgentServerSettings::Custom {
        path: command.path.clone(),
        args: command.args.clone(),
        env,
        default_mode: None,
        default_config_options: HashMap::default(),
        favorite_config_option_values: HashMap::default(),
    };
    cx.global_mut::<Assistant>().command = Some(command);
    cx.update_global::<SettingsStore, _>(|store, cx| {
        store.update_default_settings(cx, |defaults| {
            defaults
                .agent_servers
                .get_or_insert_default()
                .insert(ENTRY.to_string(), entry);
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

/// Offers the agent once, a few seconds after start, when the user's settings do not name the
/// switch and an agent is found: Claude Code signed in, else Codex signed in, else Zed's default
/// model ready.
fn offer_later(cx: &App) {
    cx.spawn(async move |cx| {
        cx.background_executor().timer(OFFER_AFTER).await;
        let programs = cx.update(|cx| {
            // A switch turned off while Marley was closed leaves no profile to name.
            forget_default_profile(cx);
            (!decided(cx)).then(|| {
                (
                    program(AgentKind::Claude, cx),
                    program(AgentKind::Codex, cx),
                )
            })
        });
        let Some((claude, codex)) = programs else {
            return;
        };
        let found = if claude_signed_in(&claude).await {
            Some(MarleyAssistantAgent::ClaudeCode)
        } else if codex_signed_in(&codex).await {
            Some(MarleyAssistantAgent::Codex)
        } else {
            cx.update(|cx| zed_model_ready(cx).then_some(MarleyAssistantAgent::Zed))
        };
        if let Some(agent) = found {
            cx.update(|cx| show_offer(agent, cx));
        }
    })
    .detach();
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

/// Whether the user's settings name `marley.assistant.enabled`, either way.
fn decided(cx: &App) -> bool {
    cx.global::<SettingsStore>()
        .raw_user_settings()
        .and_then(|user| user.content.marley.as_ref())
        .and_then(|marley| marley.assistant.as_ref())
        .is_some_and(|assistant| assistant.enabled.is_some())
}

/// The offer, naming the agent found: Turn On and Not Now each write the choice, so it is asked
/// once.
fn show_offer(agent: MarleyAssistantAgent, cx: &mut App) {
    let message = match agent {
        MarleyAssistantAgent::ClaudeCode => {
            "Marley can help set itself up, through Claude Code: an agent in the Agent Panel that \
             explains Marley and proposes settings changes for you to accept."
        }
        MarleyAssistantAgent::Codex => {
            "Marley can help set itself up, through Codex in its read-only mode: an agent in the \
             Agent Panel that explains Marley and proposes settings changes for you to accept."
        }
        MarleyAssistantAgent::Zed => {
            "Marley can help set itself up, through Zed's agent: a Marley profile, with Marley's \
             tools and no file tools, that explains Marley and proposes settings changes for you \
             to accept."
        }
    };
    show_app_notification(NotificationId::unique::<AssistantOffer>(), cx, move |cx| {
        cx.new(|cx| {
            MessageNotification::new(message, cx)
                .primary_message("Turn On")
                .primary_on_click(move |_, cx| choose(Some(agent), cx))
                .secondary_message("Not Now")
                .secondary_on_click(|_, cx| choose(None, cx))
        })
    });
}

/// Writes the user's answer to the offer into their settings: the switch on with the agent
/// offered, or off.
fn choose(agent: Option<MarleyAssistantAgent>, cx: &App) {
    settings::update_settings_file(<dyn Fs>::global(cx), cx, move |content, _| {
        let assistant = content
            .marley
            .get_or_insert_default()
            .assistant
            .get_or_insert_default();
        assistant.enabled = Some(agent.is_some());
        if agent.is_some() {
            assistant.agent = agent;
        }
    });
}

/// Lists the agent's command in the palette while the switch is on and the agent has a terminal
/// interface, as Rusty's are while Rusty is (#661). Before the palette has its filter there is
/// nothing to set; the next call sets it.
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
    let (kind, line, written) = match wanted(cx) {
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
        Some(MarleyAssistantAgent::Zed) | None => return,
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
