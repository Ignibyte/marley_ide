//! Marley's own agent (#683): an entry in the Agent Panel that explains and configures Marley.
//!
//! It is Claude Code, through the registry's `claude-acp` adapter on the user's own login, with
//! instructions appended to its system prompt and the tools that edit files or run commands
//! disallowed; Marley's MCP server, which Zed hands every external agent, gives it the docs and
//! settings tools (#681, #682). The adapter reads the instructions and the limits from each
//! session's `_meta`, which Zed sends from `marley.agent_session_meta` (a hunk in
//! `agent_servers`).
//!
//! While `marley.assistant.enabled` is on, the entry lives in the settings' in-memory defaults,
//! as `context_servers.marley` does (`mcp::offer_to_zeds_agents`): a custom agent named `Marley`
//! whose command is the one Zed resolves for `claude-acp`, so the adapter Zed installed runs it.
//! Off, nothing of it is there. Once, when the user's settings do not name the switch and
//! `claude auth status` says Claude Code is signed in, a notification offers it.
//!
//! `marley: open marley agent in terminal` (#684) runs the same agent as Claude Code's own
//! interface in a center terminal: `claude` with the instructions as a system-prompt file and the
//! same tools turned off, an ordinary agent terminal otherwise. The palette lists it only while the
//! switch is on.
//!
//! `MARLEY_ASSISTANT_ADAPTER`, when set, names an ACP program to run in the adapter's place, as
//! `MARLEY_CODEX` names a stand-in Codex: a scenario's scripted agent, which reads the `_meta`
//! the sessions carry.

use std::any::TypeId;
use std::path::{Path, PathBuf};
use std::time::Duration;

use collections::HashMap;
use command_palette_hooks::CommandPaletteFilter;
use fs::Fs;
use gpui::{
    App, AppContext as _, BorrowAppContext as _, Entity, Global, TaskExt as _, Window, actions,
};
use project::agent_server_store::{AgentId, AgentServerCommand, AgentServerStore};
use serde_json::{Value, json};
use settings::{CustomAgentServerSettings, SettingsStore};
use workspace::notifications::simple_message_notification::MessageNotification;
use workspace::notifications::{NotificationId, show_app_notification};
use workspace::{MultiWorkspace, Workspace};

actions!(
    marley,
    [
        /// Opens Marley's own agent as Claude Code in a terminal: the Marley agent's
        /// instructions, and no file edits or commands.
        #[derive(Eq)]
        OpenMarleyAgentInTerminal
    ]
);

/// The entry's name in the Agent Panel and its key in the agent servers.
pub(crate) const ENTRY: &str = "Marley";

/// The registry adapter the entry runs.
const ADAPTER: &str = "claude-acp";

/// Claude Code's tools the agent may not use: it explains and configures, through Marley's
/// tools, and changes no file and runs no command.
const DISALLOWED_TOOLS: [&str; 5] = ["Bash", "Edit", "Write", "NotebookEdit", "MultiEdit"];

/// How long after start the offer waits, so the user's settings have loaded.
const OFFER_AFTER: Duration = Duration::from_secs(5);

/// What the agent is told, appended to Claude Code's own system prompt.
const INSTRUCTIONS: &str = "You are Marley's own agent, inside Marley, a code editor built on \
Zed with a block terminal, a Browser tab and agent tools. Your job is to explain Marley and Zed to \
the user and to set Marley up the way they want.

- To answer how something works, use docs_search, then docs_read: they read this Marley's own \
docs and guide, so they match the version that runs. Quote the docs rather than guessing.
- To say what a setting does or is set to, use settings_schema and settings_read. Name the key \
path, the value now and where it is set.
- To change a setting, use settings_change. The user sees the change and accepts or declines \
it; say what you proposed and why, and never claim a change was made before it answers applied.
- To find which key runs a command, use actions_list.

You do not edit files, run commands or change code: those tools are turned off here. When the \
user asks for that, say so and suggest they ask an agent in a terminal.";

/// Whether the entry is in the defaults, and the adapter's command once resolved.
#[derive(Default)]
struct Assistant {
    /// Whether the switch was on when the defaults were last set.
    enabled: bool,
    /// The adapter's command, resolved once a project's agent server store has it.
    command: Option<AgentServerCommand>,
    /// Whether Marley put `claude-acp` in the defaults, the user's settings lacking it.
    added_adapter: bool,
    /// Whether a resolution is under way.
    resolving: bool,
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
            filter_palette(enabled(cx), cx);
            if enabled(cx) && cx.global::<Assistant>().command.is_none() {
                resolve(cx);
            }
        });
    })
    .detach();
    offer_later(cx);
}

/// Whether `marley.assistant.enabled` is on.
fn enabled(cx: &App) -> bool {
    cx.global::<SettingsStore>()
        .merged_settings()
        .marley
        .as_ref()
        .and_then(|marley| marley.assistant.as_ref())
        .and_then(|assistant| assistant.enabled)
        .unwrap_or(false)
}

/// Puts the entry in the defaults or takes it out when the switch moved.
fn sync(cx: &mut App) {
    let on = enabled(cx);
    filter_palette(on, cx);
    if on == cx.global::<Assistant>().enabled {
        return;
    }
    cx.global_mut::<Assistant>().enabled = on;
    if on {
        let user_has_adapter = cx
            .global::<SettingsStore>()
            .raw_user_settings()
            .and_then(|user| user.content.agent_servers.as_ref())
            .is_some_and(|servers| servers.contains_key(ADAPTER));
        cx.global_mut::<Assistant>().added_adapter = !user_has_adapter;
        cx.update_global::<SettingsStore, _>(|store, cx| {
            store.update_default_settings(cx, |defaults| {
                defaults
                    .marley
                    .get_or_insert_default()
                    .agent_session_meta
                    .get_or_insert_default()
                    .insert(ENTRY.to_string(), session_meta());
                if !user_has_adapter {
                    defaults.agent_servers.get_or_insert_default().insert(
                        ADAPTER.to_string(),
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
    } else {
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
                    if added_adapter {
                        servers.remove(ADAPTER);
                    }
                }
            });
        });
        let assistant = cx.global_mut::<Assistant>();
        assistant.command = None;
        assistant.added_adapter = false;
    }
}

/// The `_meta` each session of the entry carries: the instructions as an append to Claude Code's
/// system prompt, the tools it may not use, and no bypass mode.
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

/// Asks a project's agent server store for the adapter's command, installing it when it must,
/// and puts the entry in the defaults with it. With no project open yet, a new workspace asks
/// again.
fn resolve(cx: &mut App) {
    if cx.global::<Assistant>().resolving {
        return;
    }
    if let Some(stand_in) = std::env::var_os("MARLEY_ASSISTANT_ADAPTER") {
        set_entry(
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
            .get_external_agent(&AgentId::new(ADAPTER))
            .map(|adapter| adapter.get_command(Vec::new(), HashMap::default(), &mut cx.to_async()))
    });
    let Some(task) = task else {
        log::info!("assistant: `{ADAPTER}` is not registered yet; a new workspace asks again");
        return;
    };
    cx.global_mut::<Assistant>().resolving = true;
    cx.spawn(async move |cx| {
        let command = task.await;
        cx.update(|cx| {
            cx.global_mut::<Assistant>().resolving = false;
            match command {
                Ok(command) if enabled(cx) => set_entry(command, cx),
                Ok(_) => {}
                Err(error) => log::error!("assistant: `{ADAPTER}` could not start: {error:#}"),
            }
        });
    })
    .detach();
}

/// Puts the `Marley` entry in the defaults: the adapter's command, run with the `claude` Marley
/// launches and no API key, so Claude Code uses the user's own login.
fn set_entry(command: AgentServerCommand, cx: &mut App) {
    let mut env = command.env.clone().unwrap_or_default();
    env.insert(
        "CLAUDE_CODE_EXECUTABLE".to_string(),
        claude_program(cx).display().to_string(),
    );
    env.insert("ANTHROPIC_API_KEY".to_string(), String::new());
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

/// The `claude` Marley launches: the one the last version check found, else `MARLEY_CLAUDE`,
/// else the search path's.
fn claude_program(cx: &App) -> PathBuf {
    crate::agent_versions::program(marley_agent::AgentKind::Claude, cx)
        .or_else(|| std::env::var_os("MARLEY_CLAUDE").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("claude"))
}

/// Offers the agent once, a few seconds after start, when the user's settings do not name the
/// switch and Claude Code says it is signed in.
fn offer_later(cx: &App) {
    cx.spawn(async move |cx| {
        cx.background_executor().timer(OFFER_AFTER).await;
        let program = cx.update(|cx| (!decided(cx)).then(|| claude_program(cx)));
        let Some(program) = program else {
            return;
        };
        let signed_in = match crate::process::output(&program, ["auth", "status"], None, &[]).await
        {
            Ok(output) => serde_json::from_slice::<Value>(&output.stdout)
                .ok()
                .and_then(|status| status.get("loggedIn").and_then(Value::as_bool))
                .unwrap_or(false),
            Err(error) => {
                log::debug!("assistant: `claude auth status` did not run: {error}");
                false
            }
        };
        if signed_in {
            cx.update(show_offer);
        }
    })
    .detach();
}

/// Whether the user's settings name `marley.assistant.enabled`, either way.
fn decided(cx: &App) -> bool {
    cx.global::<SettingsStore>()
        .raw_user_settings()
        .and_then(|user| user.content.marley.as_ref())
        .and_then(|marley| marley.assistant.as_ref())
        .is_some_and(|assistant| assistant.enabled.is_some())
}

/// The offer: Turn On and Not Now each write the choice, so it is asked once.
fn show_offer(cx: &mut App) {
    show_app_notification(NotificationId::unique::<AssistantOffer>(), cx, |cx| {
        cx.new(|cx| {
            MessageNotification::new(
                "Marley can help set itself up, through Claude Code: an agent in the Agent \
                 Panel that explains Marley and proposes settings changes for you to accept.",
                cx,
            )
            .primary_message("Turn On")
            .primary_on_click(|_, cx| choose(true, cx))
            .secondary_message("Not Now")
            .secondary_on_click(|_, cx| choose(false, cx))
        })
    });
}

/// Writes the user's answer to the offer into their settings.
fn choose(enabled: bool, cx: &App) {
    settings::update_settings_file(<dyn Fs>::global(cx), cx, move |content, _| {
        content
            .marley
            .get_or_insert_default()
            .assistant
            .get_or_insert_default()
            .enabled = Some(enabled);
    });
}

/// Lists the agent's command in the palette while the switch is on, as Rusty's are while Rusty is
/// (#661). Before the palette has its filter there is nothing to set; the next call sets it.
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

/// `marley: open marley agent in terminal`: writes the instructions where Claude Code reads them,
/// then starts it in a center terminal of `workspace` with them and the tools turned off.
fn open_in_terminal(workspace: &Workspace, window: &Window, cx: &gpui::Context<Workspace>) {
    if !enabled(cx) {
        return;
    }
    let file = paths::data_dir().join("assistant").join("instructions.md");
    let claude = claude_program(cx);
    let written = cx.background_executor().spawn(futures::future::lazy({
        let file = file.clone();
        move |_| write_instructions(&file)
    }));
    let directory = terminal_view::default_working_directory(workspace, cx);
    cx.spawn_in(window, async move |workspace, cx| {
        written.await?;
        let line = format!(
            "{} --append-system-prompt-file {} --disallowedTools {}\n",
            quoted(&claude.display().to_string()),
            quoted(&file.display().to_string()),
            DISALLOWED_TOOLS.join(" ")
        );
        workspace
            .update_in(cx, |workspace, window, cx| {
                crate::agents::start_in_terminal(
                    workspace,
                    directory,
                    Some(marley_agent::AgentKind::Claude),
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
