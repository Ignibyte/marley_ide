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
//! Code plugin's bridge (`claude_plugin/marley/bin/marley-mcp-bridge`) reads it.

use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use futures::StreamExt as _;
use futures::channel::mpsc;
use gpui::{App, AppContext as _, Context, Entity, Global};
use marley_mcp::{AppCall, AppCaller, GrantTable, ToolAnswer, discovery, transport};
use marley_terminal::{AnchoredBlock, BlockState, BlockTimes};
use serde_json::{Value, json};
use terminal_view::TerminalView;
use terminal_view::terminal_panel::TerminalPanel;
use util::ResultExt as _;
use workspace::item::Item as _;
use workspace::notifications::NotificationId;
use workspace::{MultiWorkspace, Toast, Workspace};

/// How many of the newest blocks `terminal_blocks` lists when the call names no `last`.
const DEFAULT_BLOCKS: usize = 50;

/// The most blocks `terminal_blocks` lists.
const MAX_BLOCKS: usize = 500;

/// The most lines `terminal_read` gives; the end is kept.
const MAX_READ_LINES: usize = 2_000;

/// The most bytes `terminal_read` gives, for output whose lines are long.
const MAX_READ_BYTES: usize = 256 * 1024;

/// The server in the app: why it could not start, until a workspace shows it.
struct McpServer {
    failure: Option<String>,
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
    let failure = match transport::spawn(shared, effects, caller) {
        Ok(handle) => {
            write_endpoint(
                data_dir.clone(),
                transport::discovery_json(handle.url(), handle.bearer()),
                cx,
            );
            None
        }
        Err(error) => Some(format!("Marley's MCP server did not start: {error}")),
    };
    if let Some(failure) = &failure {
        log::error!("mcp: {failure}");
    }
    cx.set_global(McpServer { failure });
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

/// Answers `call` from the app's state; a browser call answers from its own task.
fn answer(call: AppCall, cx: &mut App) {
    if call.tool.starts_with("browser_") {
        crate::browser_tools::answer(call, cx);
        return;
    }
    let result = match call.tool.as_str() {
        "terminal_list" => Ok(terminal_list(cx)),
        "terminal_blocks" => terminal_blocks(&call.arguments, cx),
        "terminal_read" => terminal_read(&call.arguments, cx),
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

/// The `terminal` argument: a terminal's id.
fn terminal_argument(arguments: &Value) -> Result<u64, String> {
    arguments
        .get("terminal")
        .and_then(Value::as_u64)
        .ok_or_else(|| "give `terminal`, a terminal's id from terminal_list".to_string())
}

fn terminal_list(cx: &App) -> ToolAnswer {
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
            json!({
                "id": view.entity_id().as_u64(),
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

fn terminal_blocks(arguments: &Value, cx: &App) -> Result<ToolAnswer, String> {
    let id = terminal_argument(arguments)?;
    let last = arguments
        .get("last")
        .and_then(Value::as_u64)
        .map_or(DEFAULT_BLOCKS, |last| {
            usize::try_from(last).map_or(MAX_BLOCKS, |last| last.clamp(1, MAX_BLOCKS))
        });
    let view = terminal_with_id(id, cx)?;
    let terminal = view.read(cx).terminal().read(cx);
    let anchored = terminal.marley_anchored();
    let blocks = anchored.blocks();
    let now = SystemTime::now();
    let listed: Vec<Value> = blocks
        .iter()
        .skip(blocks.len().saturating_sub(last))
        .map(|block| {
            block_entry(
                block,
                anchored.times(block.index),
                terminal.block_output_kept(block),
                now,
            )
        })
        .collect();
    Ok(ToolAnswer {
        structured: json!({ "terminal": id, "total": blocks.len(), "blocks": listed }),
        text: None,
        image: None,
    })
}

/// One block as `terminal_blocks` lists it. A running block's duration is how long it has run.
fn block_entry(
    block: &AnchoredBlock,
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
        "command": block.command,
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

fn terminal_read(arguments: &Value, cx: &App) -> Result<ToolAnswer, String> {
    let id = terminal_argument(arguments)?;
    let index = arguments
        .get("block")
        .and_then(Value::as_u64)
        .and_then(|index| usize::try_from(index).ok())
        .ok_or_else(|| "give `block`, a block's index from terminal_blocks".to_string())?;
    let view = terminal_with_id(id, cx)?;
    let terminal = view.read(cx).terminal().read(cx);
    let block = terminal
        .blocks()
        .get(index)
        .ok_or_else(|| format!("terminal {id} has no block {index}"))?;
    let output = terminal
        .block_output(block)
        .ok_or_else(|| format!("block {index}'s output has left the terminal's scrollback"))?;
    let (output, truncated) = tail(&output);
    Ok(ToolAnswer {
        structured: json!({
            "terminal": id,
            "block": index,
            "command": block.command,
            "running": block.state == BlockState::Running,
            "output": output,
            "truncated": truncated,
        }),
        text: Some(output),
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
