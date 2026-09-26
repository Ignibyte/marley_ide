//! Marley's MCP server in the app (prong 2 C0, #491).
//!
//! `marley_mcp`'s loopback server starts once, at startup, from `zed`'s `main`; its endpoint is
//! written where agents find it, and the app answers the tools that are its own, the terminal
//! family first.
//!
//! The server runs on threads of its own and answers what it can by itself. A call whose answer is
//! the app's arrives here as an [`AppCall`], through a channel a foreground task reads, and is
//! answered on the main thread. `mcp-endpoint.json` in Marley's data directory (mode 0600) holds
//! the URL and the bearer as an MCP client's server entry, and goes when Marley quits; the Claude
//! Code plugin's bridge (`claude_plugin/marley/bin/marley-mcp-bridge`) reads it. Zed's own agents
//! reach the server through the same bridge, which Marley registers as the context server
//! `marley` among Zed's default settings (#501). The server's fleet snapshot, which
//! `fleet_snapshot` and the `fleet://snapshot` resource serve, is the app's Claude Code sessions,
//! handed over at each change (#547).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use futures::StreamExt as _;
use futures::channel::mpsc;
use gpui::{App, AppContext as _, BorrowAppContext as _, Context, Entity, Global};
use marley_fleet::FleetSnapshot;
use marley_mcp::redact::{Redacted, Redactor};
use marley_mcp::{AppCall, AppCaller, Caller, GrantTable, ToolAnswer, discovery, transport};
use marley_terminal::{AnchoredBlock, BlockState, BlockTimes};
use serde_json::{Value, json};
use settings::settings_content::{ContextServerCommand, ContextServerSettingsContent};
use settings::{Settings as _, SettingsStore};
use terminal_view::TerminalView;
use terminal_view::terminal_panel::TerminalPanel;
use util::ResultExt as _;
use workspace::item::Item as _;
use workspace::notifications::simple_message_notification::MessageNotification;
use workspace::notifications::{NotificationId, show_app_notification};
use workspace::{MultiWorkspace, Toast, Workspace};

use crate::agent_events::AgentEvents;
use crate::{MarleySettings, claude_plugin};

/// How many of the newest blocks `terminal_blocks` lists when the call names no `last`.
const DEFAULT_BLOCKS: usize = 50;

/// The most blocks `terminal_blocks` lists.
const MAX_BLOCKS: usize = 500;

/// The most lines `terminal_read` gives; the end is kept.
const MAX_READ_LINES: usize = 2_000;

/// The most bytes `terminal_read` gives, for output whose lines are long.
const MAX_READ_BYTES: usize = 256 * 1024;

/// The server in the app: why it could not start, until a workspace shows it, and where the
/// app's fleet snapshot goes to reach it, while it runs.
struct McpServer {
    failure: Option<String>,
    snapshots: Option<mpsc::UnboundedSender<FleetSnapshot>>,
}

impl Global for McpServer {}

/// Starts Marley's MCP server, once per process.
///
/// It writes the endpoint file into the data directory, removes it when Marley quits, and answers
/// the tool calls that are the app's. A server that cannot start is logged and shown once, in the
/// first workspace; Marley runs on without it.
pub fn start(cx: &mut App) {
    if cx.has_global::<McpServer>() {
        return;
    }
    let (calls, mut incoming) = mpsc::unbounded::<AppCall>();
    let caller: AppCaller = Arc::new(move |call| {
        if let Err(error) = calls.unbounded_send(call) {
            // The call drops with the error, and its connection answers that Marley takes no calls.
            log::debug!("mcp: a tool call came as Marley shut down: {error}");
        }
    });
    // None of the tools the server lists asks for an effect.
    let (effects, _) = std::sync::mpsc::channel();
    refresh_redaction(cx);
    cx.observe_global::<SettingsStore>(refresh_redaction)
        .detach();
    let data_dir = paths::data_dir().clone();
    // The browser's write tools are granted: the client's approval of each call and the Browser
    // tab, where the user watches each action, are their checks (#492 D2).
    let shared: transport::Shared = Arc::new((
        Mutex::new(transport::ServerData {
            grants: GrantTable::from_classes(["browser.write"]),
            ..transport::ServerData::default()
        }),
        Condvar::new(),
    ));
    let published = Arc::clone(&shared);
    let (failure, snapshots) = match transport::spawn(shared, effects, caller) {
        Ok(handle) => {
            write_endpoint(
                data_dir.clone(),
                transport::discovery_json(handle.url(), handle.bearer()),
                cx,
            );
            offer_to_zeds_agents(data_dir.clone(), cx);
            (None, Some(publisher(published, cx)))
        }
        Err(error) => (
            Some(format!("Marley's MCP server did not start: {error}")),
            None,
        ),
    };
    if let Some(failure) = &failure {
        log::error!("mcp: {failure}");
    }
    cx.set_global(McpServer { failure, snapshots });
    cx.observe_global::<AgentEvents>(publish).detach();
    cx.on_app_quit(move |cx| {
        let data_dir = data_dir.clone();
        cx.background_spawn(futures::future::lazy(move |_| {
            discovery::remove_discovery_file_in(&data_dir).log_err();
        }))
    })
    .detach();
    cx.observe_new(|_: &mut Workspace, _, cx: &mut Context<Workspace>| {
        show_failure(cx);
    })
    .detach();
    cx.spawn(async move |cx| {
        while let Some(call) = incoming.next().await {
            cx.update(|cx| answer(call, cx));
        }
    })
    .detach();
}

/// Sends the server the app's Claude Code sessions (#547), for `fleet_snapshot` and the
/// `fleet://snapshot` resource.
fn publish(cx: &mut App) {
    let Some(snapshots) = cx
        .try_global::<McpServer>()
        .and_then(|server| server.snapshots.clone())
    else {
        return;
    };
    let snapshot = cx
        .try_global::<AgentEvents>()
        .map(|events| events.snapshot().clone())
        .unwrap_or_default();
    if let Err(error) = snapshots.unbounded_send(snapshot) {
        log::debug!("mcp: a fleet snapshot came as Marley shut down: {error}");
    }
}

/// A background task that hands the server each snapshot sent to it, in order, and wakes the
/// streams of the clients that follow `fleet://snapshot`. The server's threads hold the data's
/// lock, so the main thread never waits on it.
fn publisher(shared: transport::Shared, cx: &App) -> mpsc::UnboundedSender<FleetSnapshot> {
    let (snapshots, mut incoming) = mpsc::unbounded::<FleetSnapshot>();
    cx.background_spawn(async move {
        while let Some(snapshot) = incoming.next().await {
            {
                let (data, _) = &*shared;
                data.lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .snapshot = snapshot;
            }
            transport::signal_change(&shared);
        }
    })
    .detach();
    snapshots
}

/// The context server Zed's own agents know Marley's server by (#501).
const CONTEXT_SERVER: &str = "marley";

/// Registers Marley's server as the context server `marley` among Zed's default settings (#501),
/// so the Zed Agent lists its tools and each external agent's new session is handed it: a stdio
/// server running the bridge, which reads the endpoint file, so no bearer goes into a setting
/// and a restart of Marley is followed. The bridge is written off the main thread first; a user's
/// own `context_servers.marley` wins, as settings do.
fn offer_to_zeds_agents(data_dir: PathBuf, cx: &App) {
    let written = cx.background_spawn(futures::future::lazy(move |_| {
        let bridge = write_bridge_in(&data_dir)?;
        anyhow::Ok((bridge, data_dir.join("mcp-endpoint.json")))
    }));
    cx.spawn(async move |cx| match written.await {
        Ok((bridge, endpoint)) => cx.update(|cx| {
            log::info!(
                "mcp: Zed's own agents reach Marley's tools through the context server \
                 {CONTEXT_SERVER}, which runs {}",
                bridge.display()
            );
            let command = ContextServerCommand {
                path: bridge,
                args: Vec::new(),
                // A Marley started from a Marley terminal would hand its parent's identity to its
                // own agents' bridge, so both are blank there (#520).
                env: Some(
                    [
                        (
                            "MARLEY_MCP_ENDPOINT".to_string(),
                            endpoint.to_string_lossy().into_owned(),
                        ),
                        (
                            marley_terminal::identity::TERMINAL_ID_VARIABLE.to_string(),
                            String::new(),
                        ),
                        (
                            marley_terminal::identity::PROJECT_VARIABLE.to_string(),
                            String::new(),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                ),
                timeout: None,
            };
            cx.update_global::<SettingsStore, _>(|store, cx| {
                store.update_default_settings(cx, |defaults| {
                    defaults.project.context_servers.insert(
                        CONTEXT_SERVER.into(),
                        ContextServerSettingsContent::Stdio {
                            enabled: true,
                            remote: false,
                            command,
                        },
                    );
                });
            });
        }),
        Err(error) => {
            log::error!("mcp: Zed's own agents get no Marley tools: the bridge: {error:#}");
        }
    })
    .detach();
}

/// Writes the bridge into `data_dir` as a program; its path.
fn write_bridge_in(data_dir: &Path) -> std::io::Result<PathBuf> {
    let dir = data_dir.join("mcp");
    std::fs::create_dir_all(&dir)?;
    let bridge = dir.join("marley-mcp-bridge");
    std::fs::write(&bridge, claude_plugin::BRIDGE)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&bridge, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(bridge)
}

/// Writes the endpoint file off the main thread; a failure is shown as the server's would be.
fn write_endpoint(data_dir: PathBuf, json: String, cx: &App) {
    let written = cx.background_spawn(futures::future::lazy(move |_| {
        discovery::write_discovery_file_in(&data_dir, &json)
    }));
    cx.spawn(async move |cx| {
        if let Err(error) = written.await {
            let failure = format!(
                "Marley's MCP server started, but its endpoint file could not be written, so no \
                 agent can find it: {error}"
            );
            log::error!("mcp: {failure}");
            cx.update(|cx| cx.global_mut::<McpServer>().failure = Some(failure));
        }
    })
    .detach();
}

/// Shows why the server did not start, in the workspace just made, the first time one is.
fn show_failure(cx: &mut Context<Workspace>) {
    let Some(failure) = cx.global_mut::<McpServer>().failure.take() else {
        return;
    };
    let workspace = cx.entity();
    cx.defer(move |cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.show_toast(
                Toast::new(NotificationId::unique::<McpServer>(), failure),
                cx,
            );
        });
    });
}

/// The redactor what the tools give agents runs through (#516), and the settings it was built
/// from, so a settings change that leaves them alone keeps it.
struct AgentRedaction {
    enabled: bool,
    patterns: Vec<String>,
    redactor: Arc<Redactor>,
}

impl Global for AgentRedaction {}

/// Rebuilds the redactor when `marley.redact_secrets_for_agents` or `marley.redaction_patterns`
/// changed; a pattern that is not a regular expression is left out and named in a notification.
fn refresh_redaction(cx: &mut App) {
    let settings = MarleySettings::get_global(cx);
    let (enabled, patterns) = (settings.redact_secrets, settings.redaction_patterns.clone());
    if cx
        .try_global::<AgentRedaction>()
        .is_some_and(|current| current.enabled == enabled && current.patterns == patterns)
    {
        return;
    }
    let (redactor, errors) = Redactor::new(&patterns);
    cx.set_global(AgentRedaction {
        enabled,
        patterns,
        redactor: Arc::new(redactor),
    });
    if errors.is_empty() {
        return;
    }
    let message = format!(
        "Marley left out redaction patterns that are not regular expressions: {}",
        errors.join("; ")
    );
    log::error!("mcp: {message}");
    show_app_notification(NotificationId::unique::<AgentRedaction>(), cx, move |cx| {
        cx.new(|cx| MessageNotification::new(message.clone(), cx))
    });
}

/// The redactor for what agents read, or `None` when the user turned redaction off. Before the
/// settings are read it is the built-in rules alone, so nothing leaves unredacted by accident.
pub(crate) fn agent_redactor(cx: &App) -> Option<Arc<Redactor>> {
    match cx.try_global::<AgentRedaction>() {
        Some(redaction) if redaction.enabled => Some(Arc::clone(&redaction.redactor)),
        Some(_) => None,
        None => Some(Arc::new(Redactor::new(&[]).0)),
    }
}

/// `text` as an agent may read it.
fn for_agents(text: &str, redactor: Option<&Redactor>) -> Redacted {
    redactor.map_or_else(
        || Redacted {
            text: text.to_string(),
            count: 0,
        },
        |redactor| redactor.redact(text),
    )
}

/// Answers `call` from the app's state; a browser call answers from its own task.
fn answer(call: AppCall, cx: &mut App) {
    if call.tool.starts_with("browser_") {
        crate::browser_tools::answer(call, cx);
        return;
    }
    let result = match call.tool.as_str() {
        "terminal_list" => Ok(terminal_list(call.caller(), cx)),
        "terminal_blocks" => terminal_blocks(&call.arguments, call.caller(), cx),
        "terminal_read" => terminal_read(&call.arguments, call.caller(), cx),
        other => Err(format!("Marley answers no tool named {other}")),
    };
    call.answer(result);
}

/// Every terminal in Marley's windows, the center panes' and the terminal panel's, with the
/// workspace it belongs to.
fn terminals(cx: &App) -> Vec<(Entity<Workspace>, Entity<TerminalView>)> {
    let workspaces: Vec<Entity<Workspace>> = cx
        .windows()
        .into_iter()
        .filter_map(|window| window.downcast::<MultiWorkspace>()?.read(cx).ok())
        .flat_map(|multi_workspace| multi_workspace.workspaces().cloned().collect::<Vec<_>>())
        .collect();
    workspaces
        .into_iter()
        .flat_map(|workspace| {
            let read = workspace.read(cx);
            let panel_views = read
                .panel::<TerminalPanel>(cx)
                .map(|panel| {
                    panel
                        .read(cx)
                        .panes()
                        .into_iter()
                        .flat_map(|pane| pane.read(cx).items_of_type::<TerminalView>())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            read.items_of_type::<TerminalView>(cx)
                .chain(panel_views)
                .map(|view| (workspace.clone(), view))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The terminal whose id (its view's entity id, as the rail's) is `id`.
fn terminal_with_id(id: u64, cx: &App) -> Result<Entity<TerminalView>, String> {
    terminals(cx)
        .into_iter()
        .map(|(_, view)| view)
        .find(|view| view.entity_id().as_u64() == id)
        .ok_or_else(|| format!("no terminal has the id {id}; terminal_list names them"))
}

/// The terminal a call names in `terminal`, else the caller's own (#520): its id and its view.
fn terminal_of(
    arguments: &Value,
    caller: &Caller,
    cx: &App,
) -> Result<(u64, Entity<TerminalView>), String> {
    if let Some(id) = arguments.get("terminal").and_then(Value::as_u64) {
        return terminal_with_id(id, cx).map(|view| (id, view));
    }
    let own = caller.terminal.as_deref().and_then(|own| {
        terminals(cx)
            .into_iter()
            .map(|(_, view)| view)
            .find(|view| view.read(cx).terminal().read(cx).marley_terminal_id() == Some(own))
    });
    own.map(|view| (view.entity_id().as_u64(), view))
        .ok_or_else(|| {
            "give `terminal`, a terminal's id from terminal_list: this call comes from no \
             terminal of Marley's"
                .to_string()
        })
}

fn terminal_list(caller: &Caller, cx: &App) -> ToolAnswer {
    let terminals: Vec<Value> = terminals(cx)
        .iter()
        .map(|(workspace, view)| {
            let terminal = view.read(cx).terminal().read(cx);
            let blocks = terminal.blocks();
            let running = blocks
                .last()
                .filter(|block| block.state == BlockState::Running)
                .map(|block| block.command.clone());
            let cwd = terminal
                .working_directory()
                .map(|path| path.to_string_lossy().into_owned());
            let project = workspace
                .read(cx)
                .project()
                .read(cx)
                .visible_worktrees(cx)
                .next()
                .map(|worktree| worktree.read(cx).root_name_str().to_string());
            let terminal_id = terminal.marley_terminal_id();
            json!({
                "id": view.entity_id().as_u64(),
                "terminal_id": terminal_id,
                // Marley: the caller's own terminal (#520).
                "self": terminal_id.is_some() && terminal_id == caller.terminal.as_deref(),
                "title": view.read(cx).tab_content_text(0, cx).to_string(),
                "project": project,
                "cwd": cwd,
                "running": running,
                "blocks": blocks.len(),
            })
        })
        .collect();
    ToolAnswer {
        structured: json!({ "terminals": terminals }),
        text: None,
        image: None,
    }
}

fn terminal_blocks(arguments: &Value, caller: &Caller, cx: &App) -> Result<ToolAnswer, String> {
    let (id, view) = terminal_of(arguments, caller, cx)?;
    let last = arguments
        .get("last")
        .and_then(Value::as_u64)
        .map_or(DEFAULT_BLOCKS, |last| {
            usize::try_from(last).map_or(MAX_BLOCKS, |last| last.clamp(1, MAX_BLOCKS))
        });
    let terminal = view.read(cx).terminal().read(cx);
    let anchored = terminal.marley_anchored();
    let blocks = anchored.blocks();
    let now = SystemTime::now();
    let redactor = agent_redactor(cx);
    let mut redacted = 0;
    let listed: Vec<Value> = blocks
        .iter()
        .skip(blocks.len().saturating_sub(last))
        .map(|block| {
            let command = for_agents(&block.command, redactor.as_deref());
            redacted += command.count;
            block_entry(
                block,
                &command.text,
                anchored.times(block.index),
                terminal.block_output_kept(block),
                now,
            )
        })
        .collect();
    Ok(ToolAnswer {
        structured: json!({
            "terminal": id,
            "total": blocks.len(),
            "blocks": listed,
            "redacted": redacted,
        }),
        text: None,
        image: None,
    })
}

/// One block as `terminal_blocks` lists it. A running block's duration is how long it has run.
fn block_entry(
    block: &AnchoredBlock,
    command: &str,
    times: Option<BlockTimes>,
    output_kept: bool,
    now: SystemTime,
) -> Value {
    let started_at_ms =
        times.and_then(|times| milliseconds(times.started.duration_since(UNIX_EPOCH)));
    let duration_ms = times.and_then(|times| {
        milliseconds(times.finished.unwrap_or(now).duration_since(times.started))
    });
    json!({
        "index": block.index,
        "command": command,
        "verified": block.command_verified,
        "running": block.state == BlockState::Running,
        "exit_code": block.exit_code.0,
        "cwd": block.prompt.pwd,
        "started_at_ms": started_at_ms,
        "duration_ms": duration_ms,
        "output_kept": output_kept,
    })
}

/// A span in whole milliseconds, when it is one: a clock set back makes an end before its start.
fn milliseconds(span: Result<std::time::Duration, std::time::SystemTimeError>) -> Option<u64> {
    span.ok()
        .and_then(|span| u64::try_from(span.as_millis()).ok())
}

fn terminal_read(arguments: &Value, caller: &Caller, cx: &App) -> Result<ToolAnswer, String> {
    let (id, view) = terminal_of(arguments, caller, cx)?;
    let index = arguments
        .get("block")
        .and_then(Value::as_u64)
        .and_then(|index| usize::try_from(index).ok())
        .ok_or_else(|| "give `block`, a block's index from terminal_blocks".to_string())?;
    let terminal = view.read(cx).terminal().read(cx);
    let block = terminal
        .blocks()
        .get(index)
        .ok_or_else(|| format!("terminal {id} has no block {index}"))?;
    let output = terminal
        .block_output(block)
        .ok_or_else(|| format!("block {index}'s output has left the terminal's scrollback"))?;
    let redactor = agent_redactor(cx);
    let command = for_agents(&block.command, redactor.as_deref());
    // Redacted whole before the tail is cut: a private key cut at the tail would lose the
    // BEGIN line its rule needs.
    let output = for_agents(&output, redactor.as_deref());
    let (text, truncated) = tail(&output.text);
    Ok(ToolAnswer {
        structured: json!({
            "terminal": id,
            "block": index,
            "command": command.text,
            "running": block.state == BlockState::Running,
            "output": text,
            "truncated": truncated,
            "redacted": command.count + output.count,
        }),
        text: Some(text),
        image: None,
    })
}

/// The end of `text`: at most [`MAX_READ_LINES`] lines and [`MAX_READ_BYTES`] bytes, and whether
/// anything was left out.
fn tail(text: &str) -> (String, bool) {
    let lines: Vec<&str> = text.lines().collect();
    let skipped = lines.len().saturating_sub(MAX_READ_LINES);
    let kept = lines
        .iter()
        .skip(skipped)
        .copied()
        .collect::<Vec<_>>()
        .join("\n");
    let over = kept.len().saturating_sub(MAX_READ_BYTES);
    let cut = (over..=kept.len())
        .find(|&at| kept.is_char_boundary(at))
        .unwrap_or(kept.len());
    let truncated = skipped > 0 || cut > 0;
    (kept.get(cut..).unwrap_or_default().to_string(), truncated)
}
