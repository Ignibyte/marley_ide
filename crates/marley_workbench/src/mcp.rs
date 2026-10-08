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

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use futures::StreamExt as _;
use futures::channel::mpsc;
use gpui::{App, AppContext as _, BorrowAppContext as _, Context, Entity, Global};
use marley_fleet::FleetSnapshot;
use marley_mcp::redact::{Redacted, Redactor};
use marley_mcp::{
    AppCall, AppCaller, Caller, GrantTable, Refusal, ToolAnswer, discovery, transport,
};
use marley_terminal::{AnchoredBlock, BlockState, BlockTimes};
use serde_json::{Value, json};
use settings::settings_content::{ContextServerCommand, ContextServerSettingsContent};
use settings::{MarleyTerminalLinks, Settings as _, SettingsStore, SystemOneMode};
use terminal_view::TerminalView;
use terminal_view::terminal_panel::TerminalPanel;
use util::ResultExt as _;
use workspace::item::Item as _;
use workspace::notifications::simple_message_notification::MessageNotification;
use workspace::notifications::{NotificationId, show_app_notification};
use workspace::{MultiWorkspace, Toast, Workspace};

use crate::agent_events::AgentEvents;
use crate::find::{Found, Place};
use crate::{MarleySettings, claude_plugin};

/// How many of the newest blocks `terminal_blocks` lists when the call names no `last`.
const DEFAULT_BLOCKS: usize = 50;

/// The most blocks `terminal_blocks` lists.
const MAX_BLOCKS: usize = 500;

/// The most lines a `terminal_read` page holds, and the most `terminal_find` looks among.
const MAX_READ_LINES: usize = 2_000;

/// The most bytes `terminal_find` looks among, for output whose lines are long.
const MAX_READ_BYTES: usize = 256 * 1024;

/// The most bytes of output a `terminal_read` page holds (#680). The answer carries the page twice,
/// as its text and in its structured result, so a whole answer stays near 30 KB on the wire and
/// about 6,000 tokens, under the 10,000 at which Claude Code warns about a tool's result.
const PAGE_BYTES: usize = 12_000;

/// The most terminal ids a `no_terminal` refusal lists.
const MAX_LISTED_TERMINALS: usize = 20;

/// The most bytes of a block's newest lines `terminal_find` asks about: about 24,000 tokens at
/// four bytes a token (#567).
const MAX_FIND_BYTES: usize = 96_000;

/// The server in the app: why it could not start, until a workspace shows it, and where the
/// app's fleet snapshot and the tools the user turned on go to reach it, while it runs.
struct McpServer {
    failure: Option<String>,
    snapshots: Option<mpsc::UnboundedSender<FleetSnapshot>>,
    enabled: Option<mpsc::UnboundedSender<BTreeSet<String>>>,
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
    // tab, where the user watches each action, are their checks (#492 D2). The editor's is
    // Marley's own `marley-edit`'s, which outside clients are refused (#649). A settings change
    // waits for the user's Apply (#682), as a harness seat does (#692).
    let shared: transport::Shared = Arc::new((
        Mutex::new(transport::ServerData {
            grants: GrantTable::from_classes([
                "browser.write",
                "terminal.write",
                "editor.write",
                "settings.write",
                "harness.write",
            ]),
            ..transport::ServerData::default()
        }),
        Condvar::new(),
    ));
    let published = Arc::clone(&shared);
    let for_clients = Arc::clone(&shared);
    let for_tools = Arc::clone(&shared);
    let (failure, snapshots, enabled) = match transport::spawn(shared, effects, caller) {
        Ok(handle) => {
            write_endpoint(
                data_dir.clone(),
                transport::discovery_json(handle.url(), handle.bearer()),
                cx,
            );
            crate::clients::start(
                data_dir.clone(),
                handle.url().to_string(),
                for_clients,
                Arc::clone(handle.clients()),
                cx,
            );
            offer_to_zeds_agents(data_dir.clone(), cx);
            offer_browser_opener(data_dir.clone(), cx);
            offer_agent_editor(data_dir.clone(), cx);
            crate::agent_reports::start(data_dir.clone(), cx);
            (
                None,
                Some(publisher(published, cx)),
                Some(enabler(for_tools, cx)),
            )
        }
        Err(error) => (
            Some(format!("Marley's MCP server did not start: {error}")),
            None,
            None,
        ),
    };
    if let Some(failure) = &failure {
        log::error!("mcp: {failure}");
    }
    cx.set_global(McpServer {
        failure,
        snapshots,
        enabled,
    });
    cx.observe_global::<AgentEvents>(publish).detach();
    push_enabled(cx);
    cx.observe_global::<SettingsStore>(push_enabled).detach();
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

/// The conditional tools the settings turn on (#567): each while System One is on and its use,
/// of the tool's name, has a mode other than `off`.
fn enabled_tools(cx: &App) -> BTreeSet<String> {
    let settings = &MarleySettings::get_global(cx).system_one;
    marley_mcp::CONDITIONAL_TOOLS
        .iter()
        .copied()
        .filter(|tool| {
            settings.enabled
                && settings
                    .uses
                    .get(*tool)
                    .is_some_and(|mode| *mode != SystemOneMode::Off)
        })
        .map(str::to_string)
        .collect()
}

/// Sends the server the tools the settings turn on (#567), which it lists and calls from then on.
fn push_enabled(cx: &mut App) {
    let Some(enabled) = cx
        .try_global::<McpServer>()
        .and_then(|server| server.enabled.clone())
    else {
        return;
    };
    if let Err(error) = enabled.unbounded_send(enabled_tools(cx)) {
        log::debug!("mcp: the tools turned on came as Marley shut down: {error}");
    }
}

/// A background task that hands the server each set of tools turned on, in order: the server's
/// threads hold the data's lock, so the main thread never waits on it.
fn enabler(shared: transport::Shared, cx: &App) -> mpsc::UnboundedSender<BTreeSet<String>> {
    let (sets, mut incoming) = mpsc::unbounded::<BTreeSet<String>>();
    cx.background_spawn(async move {
        while let Some(enabled) = incoming.next().await {
            transport::set_enabled(&shared, enabled);
        }
    })
    .detach();
    sets
}

/// The context server Zed's own agents know Marley's server by (#501).
pub(crate) const CONTEXT_SERVER: &str = "marley";

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
    write_program_in(data_dir, "marley-mcp-bridge", claude_plugin::BRIDGE)
}

/// The program Marley's local terminals give their programs as `BROWSER` (#561).
const OPENER: &str = include_str!("../bin/marley-open-url");

/// Its file, beside the bridge's.
const OPENER_FILE: &str = "marley-open-url";

/// Gives the local terminals started from now on the opener as `BROWSER` (#561), while
/// `marley.terminal_links` sends some URLs to Browser tabs, and writes it beside the bridge off the
/// main thread. The path is set at once, so the terminals a launch restores carry it: the copy an
/// earlier launch wrote is there until this one's lands, and a missing one sends a program's URL
/// to its next browser.
fn offer_browser_opener(data_dir: PathBuf, cx: &mut App) {
    let opener = data_dir.join("mcp").join(OPENER_FILE);
    give_browser_opener(&opener, cx);
    cx.observe_global::<SettingsStore>(move |cx| give_browser_opener(&opener, cx))
        .detach();
    let written = cx.background_spawn(futures::future::lazy(move |_| {
        write_program_in(&data_dir, OPENER_FILE, OPENER)
    }));
    cx.spawn(async move |_| {
        if let Err(error) = written.await {
            log::error!("mcp: programs in terminals open URLs as before: the opener: {error:#}");
        }
    })
    .detach();
}

/// Writes `marley-edit` beside the opener off the main thread, and publishes its path once written
/// (#649): the terminals Marley opens for agents get it as their editor while
/// `marley.agent_editor_in_tab` is on.
fn offer_agent_editor(data_dir: PathBuf, cx: &App) {
    let written = cx.background_spawn(futures::future::lazy(move |_| {
        write_program_in(
            &data_dir,
            crate::agent_editor::HELPER_FILE,
            crate::agent_editor::HELPER,
        )
    }));
    cx.spawn(async move |cx| match written.await {
        Ok(helper) => cx.update(|cx| crate::agent_editor::set_helper(&helper, cx)),
        Err(error) => {
            log::error!("mcp: agents keep the Rich Input overlay: marley-edit: {error:#}");
        }
    })
    .detach();
}

/// The opener for new terminals under the settings in force: none under `system_browser`, so they
/// keep the `BROWSER` they inherit.
fn give_browser_opener(opener: &Path, cx: &App) {
    let links = MarleySettings::get_global(cx).terminal_links;
    marley_terminal::shell_integration::set_browser_opener(
        (links != MarleyTerminalLinks::SystemBrowser).then(|| opener.to_path_buf()),
    );
}

/// Writes `contents` into `data_dir`'s `mcp` folder as the program `name`; its path.
pub(crate) fn write_program_in(
    data_dir: &Path,
    name: &str,
    contents: &str,
) -> std::io::Result<PathBuf> {
    let dir = data_dir.join("mcp");
    std::fs::create_dir_all(&dir)?;
    let program = dir.join(name);
    std::fs::write(&program, contents)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(program)
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

/// The redactor for what leaves the machine for a model (#565): the built-in rules and the
/// user's patterns, whether or not agents' redaction is on, so the System One layer never sends
/// unmasked text. Before the settings are read it is the built-in rules alone.
pub(crate) fn model_redactor(cx: &App) -> Arc<Redactor> {
    cx.try_global::<AgentRedaction>().map_or_else(
        || Arc::new(Redactor::new(&[]).0),
        |redaction| Arc::clone(&redaction.redactor),
    )
}

/// `text` as an agent may read it.
pub(crate) fn for_agents(text: &str, redactor: Option<&Redactor>) -> Redacted {
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
    // The server refuses an outside client any tool off its grant's list (#524); this is the
    // second wall.
    if let Err(refusal) = marley_mcp::permits(call.principal(), &call.tool) {
        call.answer(Err(refusal));
        return;
    }
    if call.tool.starts_with("browser_") {
        crate::browser_tools::answer(call, cx);
        return;
    }
    if call.tool.starts_with("editor_") {
        crate::agent_editor::answer(call, cx);
        return;
    }
    if call.tool == "ports_list" {
        ports_list(call, cx);
        return;
    }
    if call.tool == "terminal_find" {
        terminal_find(call, cx);
        return;
    }
    if call.tool == "terminal_type" {
        crate::terminal_drive::type_into(call, cx);
        return;
    }
    if call.tool == "terminal_run" {
        crate::terminal_drive::run_at_prompt(call, cx);
        return;
    }
    if call.tool.starts_with("docs_") {
        crate::docs_tools::answer(call, cx);
        return;
    }
    if call.tool == "keymap_change" {
        crate::settings_change::answer_keymap(call, cx);
        return;
    }
    if call.tool == "seat_add" {
        crate::harness_seat::answer_seat_add(call, cx);
        return;
    }
    if call.tool.starts_with("settings_") || call.tool.starts_with("actions_") {
        crate::settings_tools::answer(call, cx);
        return;
    }
    let result = match call.tool.as_str() {
        "terminal_list" => Ok(terminal_list(call.caller(), cx)),
        "terminal_blocks" => terminal_blocks(&call.arguments, call.caller(), cx),
        "terminal_read" => terminal_read(&call.arguments, call.caller(), cx),
        "terminal_screen" => crate::terminal_drive::screen(&call, cx),
        other => Err(Refusal::from(format!(
            "Marley answers no tool named {other}"
        ))),
    };
    call.answer(result);
}

/// `ports_list` (#521): each project's listeners from a scan made now, read off the main thread,
/// by project and port; the command line stays out, since it can carry a token.
fn ports_list(call: AppCall, cx: &App) {
    let names = crate::ports::project_names(cx);
    let scan = crate::ports::list(cx);
    cx.spawn(async move |_| {
        let result = scan
            .await
            .map_err(|error| format!("Marley could not read the listening ports: {error:#}"))
            .map(|by_group| {
                let mut listed: Vec<(String, u16, Value)> = by_group
                    .into_iter()
                    .flat_map(|(key, listeners)| {
                        let project = names.get(&key).cloned().unwrap_or_default();
                        listeners.into_iter().map(move |found| {
                            let listener = found.listener;
                            let port = listener.address.port();
                            let entry = json!({
                                "project": project,
                                "folder": found.folder.display().to_string(),
                                "address": listener.address.ip().to_string(),
                                "port": port,
                                "url": marley_browser::ports::url(listener.address),
                                "pid": listener.pid,
                                "name": listener.name,
                                "cwd": listener.cwd.display().to_string(),
                            });
                            (project.clone(), port, entry)
                        })
                    })
                    .collect();
                listed.sort_by(|left, right| (&left.0, left.1).cmp(&(&right.0, right.1)));
                let ports: Vec<Value> = listed.into_iter().map(|(_, _, entry)| entry).collect();
                ToolAnswer {
                    structured: json!({ "ports": ports }),
                    text: None,
                    image: None,
                }
            });
        call.answer(result);
    })
    .detach();
}

/// `terminal_find` (#567): the line of a block's output `query` names. The words of the query
/// come first, then the System One layer for what they leave open (`crate::find`), off the main
/// thread.
fn terminal_find(call: AppCall, cx: &App) {
    let lines = match FindLines::of(&call.arguments, call.caller(), cx) {
        Ok(lines) => lines,
        Err(error) => {
            call.answer(Err(error));
            return;
        }
    };
    cx.spawn(async move |cx| {
        let FindLines {
            terminal,
            block,
            query,
            lines,
            first,
            cut,
            place,
        } = lines;
        let subject = format!("{terminal}:{block}:{query}");
        let found =
            crate::find::find_items("terminal_find", &subject, &query, &lines, place, cx).await;
        let line = |index: usize| first + index;
        let candidates: Vec<Value> = found
            .candidates
            .iter()
            .filter_map(|(index, probability)| {
                let text = lines.get(*index)?;
                Some(json!({ "line": line(*index), "text": text, "probability": probability }))
            })
            .collect();
        let top = found.top.map(line);
        let shown: Vec<String> = found
            .candidates
            .iter()
            .filter_map(|(index, probability)| {
                let text = lines.get(*index)?;
                Some(probability.map_or_else(
                    || format!("line {}: {text}", line(*index)),
                    |probability| format!("line {}: {text} ({probability:.2})", line(*index)),
                ))
            })
            .collect();
        let named = top.map(|top| format!("line {top}"));
        let text =
            crate::browser_tools::found_text(&found, named.as_deref(), &shown, "terminal_read");
        call.answer::<Refusal>(Ok(ToolAnswer {
            structured: found_lines(&found, terminal, block, &query, top, &candidates, cut),
            text: Some(text),
            image: None,
        }));
    })
    .detach();
}

/// `terminal_find`'s answer as the tool's schema gives it.
fn found_lines(
    found: &Found,
    terminal: u64,
    block: usize,
    query: &str,
    line: Option<usize>,
    candidates: &[Value],
    cut: bool,
) -> Value {
    json!({
        "terminal": terminal,
        "block": block,
        "query": query,
        "source": found.source,
        "sure": found.sure,
        "line": line,
        "candidates": candidates,
        "present": found.present,
        "verify": found.verify,
        "next": (!found.sure).then_some("terminal_read"),
        "cut": cut,
        "note": found.note,
    })
}

/// A block's lines as `terminal_find` looks among them (#567).
struct FindLines {
    terminal: u64,
    block: usize,
    query: String,
    /// The newest lines of the block's output, masked.
    lines: Vec<String>,
    /// The number of the first of them in the block's output, from 1.
    first: usize,
    /// Whether they are less than the block's output.
    cut: bool,
    place: Place,
}

impl FindLines {
    /// The block `arguments` names, in the terminal they name or the caller's own. The output is
    /// masked whole with the rules models get, before any cut, since a cut can part a secret
    /// from what marks it; the model and the answer both see the masked lines. Then the end is
    /// kept as `terminal_read` keeps it, and of that the newest lines up to [`MAX_FIND_BYTES`].
    fn of(arguments: &Value, caller: &Caller, cx: &App) -> Result<Self, Refusal> {
        let (terminal, view) = terminal_of(arguments, caller, cx)?;
        let block = block_argument(arguments)?;
        let query = crate::browser_tools::find_query(arguments)?;
        let terminal_view = view.read(cx);
        let read = terminal_view.terminal().read(cx);
        let found = read
            .blocks()
            .get(block)
            .ok_or_else(|| no_block(terminal, block, read.blocks().len()))?;
        let output = read.block_output(found).ok_or_else(|| output_gone(block))?;
        let masked = model_redactor(cx).redact(&output).text;
        let total = masked.lines().count();
        let (end, _) = tail(&masked);
        let kept: Vec<&str> = end.lines().collect();
        let mut bytes = 0;
        let start = kept
            .iter()
            .rposition(|line| {
                bytes += line.len() + 1;
                bytes > MAX_FIND_BYTES
            })
            .map_or(0, |at| at + 1);
        let lines: Vec<String> = kept
            .get(start..)
            .unwrap_or_default()
            .iter()
            .map(|line| (*line).to_string())
            .collect();
        let first = total.saturating_sub(kept.len()) + start + 1;
        let place = terminal_view.marley_workspace().upgrade().map_or_else(
            || Place {
                project: "no project".to_string(),
                folders: Vec::new(),
                local: true,
            },
            |workspace| {
                let (folders, local) = crate::system_one::project_of(workspace.read(cx), cx);
                let project = crate::system_one::project_name(&folders);
                Place {
                    project,
                    folders,
                    local,
                }
            },
        );
        Ok(Self {
            terminal,
            block,
            query,
            lines,
            cut: first > 1,
            first,
            place,
        })
    }
}

/// Every terminal in Marley's windows, the center panes' and the terminal panel's, with the
/// workspace it belongs to.
pub(crate) fn terminals(cx: &App) -> Vec<(Entity<Workspace>, Entity<TerminalView>)> {
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

/// The terminal whose id (its view's entity id, as the rail's) is `id`; refused with the ids there
/// are when none has it.
fn terminal_with_id(id: u64, cx: &App) -> Result<Entity<TerminalView>, Refusal> {
    let views: Vec<Entity<TerminalView>> =
        terminals(cx).into_iter().map(|(_, view)| view).collect();
    if let Some(view) = views.iter().find(|view| view.entity_id().as_u64() == id) {
        return Ok(view.clone());
    }
    let ids: Vec<String> = views
        .iter()
        .take(MAX_LISTED_TERMINALS)
        .map(|view| view.entity_id().as_u64().to_string())
        .collect();
    let there = if ids.is_empty() {
        "Marley has no terminal open".to_string()
    } else {
        format!("the terminals now: {}", ids.join(", "))
    };
    Err(
        Refusal::new("no_terminal", format!("no terminal has the id {id}"))
            .next("terminal_list lists the terminals, with their titles and projects")
            .next(there),
    )
}

/// The terminal of Marley's a call comes from (#520), with its workspace.
pub(crate) fn caller_terminal(
    caller: &Caller,
    cx: &App,
) -> Option<(Entity<Workspace>, Entity<TerminalView>)> {
    let own = caller.terminal.as_deref()?;
    terminals(cx)
        .into_iter()
        .find(|(_, view)| view.read(cx).terminal().read(cx).marley_terminal_id() == Some(own))
}

/// The terminal a call names in `terminal`, else the caller's own (#520): its id and its view.
pub(crate) fn terminal_of(
    arguments: &Value,
    caller: &Caller,
    cx: &App,
) -> Result<(u64, Entity<TerminalView>), Refusal> {
    if let Some(id) = arguments.get("terminal").and_then(Value::as_u64) {
        return terminal_with_id(id, cx).map(|view| (id, view));
    }
    caller_terminal(caller, cx)
        .map(|(_, view)| (view.entity_id().as_u64(), view))
        .ok_or_else(|| {
            Refusal::new(
                "bad_argument",
                "this call comes from no terminal of Marley's, so it has to name one",
            )
            .next("give `terminal`, a terminal's id from terminal_list")
        })
}

/// The `block` a call names: its index among the terminal's blocks.
fn block_argument(arguments: &Value) -> Result<usize, Refusal> {
    arguments
        .get("block")
        .and_then(Value::as_u64)
        .and_then(|index| usize::try_from(index).ok())
        .ok_or_else(|| {
            Refusal::new("bad_argument", "`block` is missing or not a block's index")
                .next("give `block`, a block's index from terminal_blocks")
        })
}

/// The refusal of block `index` in terminal `terminal`, which has `count` blocks.
fn no_block(terminal: u64, index: usize, count: usize) -> Refusal {
    let there = match count {
        0 => format!("terminal {terminal} has run no command yet"),
        count => format!(
            "terminal_blocks lists terminal {terminal}'s blocks: 0 to {}",
            count - 1
        ),
    };
    Refusal::new(
        "no_block",
        format!("terminal {terminal} has no block {index}"),
    )
    .next(there)
}

/// The refusal of a block whose output has left the terminal's scrollback.
fn output_gone(index: usize) -> Refusal {
    Refusal::new(
        "output_gone",
        format!("block {index}'s output has left the terminal's scrollback"),
    )
    .next("run the command again with terminal_run, or ask the user for the part you need")
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

fn terminal_blocks(arguments: &Value, caller: &Caller, cx: &App) -> Result<ToolAnswer, Refusal> {
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
                anchored.block_host(block.index),
                anchored.times(block.index),
                terminal.block_output_kept(block),
                crate::terminal_drive::run_by_agent(view.entity_id(), block.index, cx),
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

/// One block as `terminal_blocks` lists it. A running block's duration is how long it has run;
/// `agent` says an agent's `terminal_run` typed it (#556).
fn block_entry(
    block: &AnchoredBlock,
    command: &str,
    host: Option<&str>,
    times: Option<BlockTimes>,
    output_kept: bool,
    agent: bool,
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
        "host": host,
        "started_at_ms": started_at_ms,
        "duration_ms": duration_ms,
        "output_kept": output_kept,
        "agent": agent,
    })
}

/// A span in whole milliseconds, when it is one: a clock set back makes an end before its start.
fn milliseconds(span: Result<std::time::Duration, std::time::SystemTimeError>) -> Option<u64> {
    span.ok()
        .and_then(|span| u64::try_from(span.as_millis()).ok())
}

fn terminal_read(arguments: &Value, caller: &Caller, cx: &App) -> Result<ToolAnswer, Refusal> {
    let (id, view) = terminal_of(arguments, caller, cx)?;
    let index = block_argument(arguments)?;
    let before = match arguments.get("before") {
        None | Some(Value::Null) => None,
        Some(before) => Some(
            before
                .as_u64()
                .and_then(|before| usize::try_from(before).ok())
                .ok_or_else(|| {
                    Refusal::new("bad_argument", "`before` is not a line number")
                        .next("pass a page's `previous` as `before`, or leave it out")
                })?,
        ),
    };
    let terminal = view.read(cx).terminal().read(cx);
    let block = terminal
        .blocks()
        .get(index)
        .ok_or_else(|| no_block(id, index, terminal.blocks().len()))?;
    let output = terminal
        .block_output(block)
        .ok_or_else(|| output_gone(index))?;
    let redactor = agent_redactor(cx);
    let command = for_agents(&block.command, redactor.as_deref());
    // Redacted whole before a page is cut: a private key cut at a page's edge would lose the
    // BEGIN line its rule needs.
    let output = for_agents(&output, redactor.as_deref());
    let page = page(&output.text, before)?;
    let mut structured = json!({
        "terminal": id,
        "block": index,
        "command": command.text,
        "running": block.state == BlockState::Running,
        "redacted": command.count + output.count,
    });
    page.fill(&mut structured);
    Ok(ToolAnswer {
        structured,
        text: Some(page.text_block()),
        image: None,
    })
}

/// One page of a block's output (#680): the newest whole lines that fit before a line, numbered
/// from the block's first line, so a block that still runs keeps every earlier page in place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Page {
    text: String,
    first_line: usize,
    last_line: usize,
    total_lines: usize,
    line_cut: bool,
}

impl Page {
    /// The `before` that reads the page ahead of this one, while there is one.
    fn previous(&self) -> Option<usize> {
        (self.first_line > 1).then_some(self.first_line)
    }

    /// Whether the page leaves part of the output out: earlier lines, or the start of a cut line.
    const fn partial(&self) -> bool {
        self.first_line > 1 || self.line_cut
    }

    /// Puts the page into a tool's structured answer.
    pub(crate) fn fill(&self, structured: &mut Value) {
        if let Some(fields) = structured.as_object_mut() {
            fields.insert("output".into(), self.text.clone().into());
            fields.insert("first_line".into(), self.first_line.into());
            fields.insert("last_line".into(), self.last_line.into());
            fields.insert("total_lines".into(), self.total_lines.into());
            fields.insert("previous".into(), self.previous().into());
            fields.insert("line_cut".into(), self.line_cut.into());
            fields.insert("truncated".into(), self.partial().into());
        }
    }

    /// The `after` that reads the page after this one, read forward, while there is one (#681).
    fn next(&self) -> Option<usize> {
        (self.last_line < self.total_lines).then_some(self.last_line)
    }

    /// Puts a page read forward into a tool's structured answer: `docs_read`'s fields (#681).
    pub(crate) fn fill_forward(&self, structured: &mut Value) {
        if let Some(fields) = structured.as_object_mut() {
            fields.insert("text".into(), self.text.clone().into());
            fields.insert("first_line".into(), self.first_line.into());
            fields.insert("last_line".into(), self.last_line.into());
            fields.insert("total_lines".into(), self.total_lines.into());
            fields.insert("next".into(), self.next().into());
            fields.insert("line_cut".into(), self.line_cut.into());
        }
    }

    /// A page read forward as an answer's text, with a first line naming the lines and the
    /// `after` for the rest while the page is not the whole text (#681).
    pub(crate) fn text_block_forward(&self) -> String {
        let whole = self.first_line <= 1 && self.next().is_none() && !self.line_cut;
        if whole {
            return self.text.clone();
        }
        let lines = if self.first_line == self.last_line {
            format!("line {} of {}", self.first_line, self.total_lines)
        } else {
            format!(
                "lines {} to {} of {}",
                self.first_line, self.last_line, self.total_lines
            )
        };
        let cut = if self.line_cut {
            ", the end of the line left out"
        } else {
            ""
        };
        let rest = self.next().map_or_else(String::new, |next| {
            format!("; read on with docs_read after={next}")
        });
        format!("[{lines}{cut}{rest}]\n{}", self.text)
    }

    /// The page as an answer's text: a client that reads only the text learns how to page from
    /// a first line, there while the page leaves part of the output out.
    pub(crate) fn text_block(&self) -> String {
        if !self.partial() {
            return self.text.clone();
        }
        let lines = if self.first_line == self.last_line {
            format!("line {} of {}", self.first_line, self.total_lines)
        } else {
            format!(
                "lines {} to {} of {}",
                self.first_line, self.last_line, self.total_lines
            )
        };
        let cut = if self.line_cut {
            ", the start of the line left out"
        } else {
            ""
        };
        let earlier = self.previous().map_or_else(String::new, |previous| {
            format!("; read earlier lines with terminal_read before={previous}")
        });
        format!("[{lines}{cut}{earlier}]\n{}", self.text)
    }
}

/// The page of `text` that starts just after line `after`, counted from 1, or at its first line
/// without it: whole lines forward while they fit in [`PAGE_BYTES`] and [`MAX_READ_LINES`], or the
/// start of a single longer line (#681, for a document, which reads forward). An `after` at or past
/// the last line is refused.
pub(crate) fn page_from(text: &str, after: Option<usize>) -> Result<Page, Refusal> {
    let lines: Vec<&str> = text.lines().collect();
    let total_lines = lines.len();
    let start = match after {
        None => 0,
        Some(after) if after < total_lines => after,
        Some(after) => {
            return Err(Refusal::new(
                "bad_argument",
                format!("`after` {after} is at or past the end: the text has {total_lines} lines"),
            )
            .next("pass a page's `next` as `after`; a page whose `next` is null was the last"));
        }
    };
    let mut end = start;
    let mut bytes = 0;
    while end < total_lines && end - start < MAX_READ_LINES {
        let Some(line) = lines.get(end) else {
            break;
        };
        let size = line.len() + 1;
        if bytes + size > PAGE_BYTES {
            break;
        }
        bytes += size;
        end += 1;
    }
    if end == start
        && let Some(line) = lines.get(start)
    {
        let cut = (0..=PAGE_BYTES.min(line.len()))
            .rev()
            .find(|&at| line.is_char_boundary(at))
            .unwrap_or(0);
        return Ok(Page {
            text: line.get(..cut).unwrap_or_default().to_string(),
            first_line: start + 1,
            last_line: start + 1,
            total_lines,
            line_cut: cut < line.len(),
        });
    }
    Ok(Page {
        text: lines.get(start..end).unwrap_or_default().join("\n"),
        first_line: start + 1,
        last_line: end,
        total_lines,
        line_cut: false,
    })
}

/// The newest page of `text`.
pub(crate) fn newest_page(text: &str) -> Page {
    let lines: Vec<&str> = text.lines().collect();
    page_ending(&lines, lines.len())
}

/// The page of `text` that ends just before line `before`, counted from 1, or the newest without
/// it; a `before` outside the output is refused.
fn page(text: &str, before: Option<usize>) -> Result<Page, Refusal> {
    let lines: Vec<&str> = text.lines().collect();
    let total = lines.len();
    let end = match before {
        None => total,
        Some(before) if (2..=total + 1).contains(&before) => before - 1,
        Some(before) => {
            return Err(Refusal::new(
                "bad_argument",
                format!(
                    "`before` {before} is outside the output: it has {total} lines, so `before` \
                     is 2 to {}",
                    total + 1
                ),
            )
            .next("pass a page's `previous` as `before`, or leave it out for the newest page"));
        }
    };
    Ok(page_ending(&lines, end))
}

/// The page whose last line is line `end`: whole lines back from it while they fit in
/// [`PAGE_BYTES`] and [`MAX_READ_LINES`], or the end of line `end` alone when it is longer than a
/// page.
fn page_ending(lines: &[&str], end: usize) -> Page {
    let total_lines = lines.len();
    let mut start = end;
    let mut bytes = 0;
    while start > 0 && end - start < MAX_READ_LINES {
        let Some(line) = lines.get(start - 1) else {
            break;
        };
        let size = line.len() + 1;
        if bytes + size > PAGE_BYTES {
            break;
        }
        bytes += size;
        start -= 1;
    }
    if start == end
        && let Some(line) = end.checked_sub(1).and_then(|last| lines.get(last))
    {
        let over = line.len().saturating_sub(PAGE_BYTES);
        let cut = (over..=line.len())
            .find(|&at| line.is_char_boundary(at))
            .unwrap_or(line.len());
        return Page {
            text: line.get(cut..).unwrap_or_default().to_string(),
            first_line: end,
            last_line: end,
            total_lines,
            line_cut: cut > 0,
        };
    }
    Page {
        text: lines.get(start..end).unwrap_or_default().join("\n"),
        first_line: start + 1,
        last_line: end,
        total_lines,
        line_cut: false,
    }
}

/// The end of `text`: at most [`MAX_READ_LINES`] lines and [`MAX_READ_BYTES`] bytes, and whether
/// anything was left out; what `terminal_find` looks among.
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
