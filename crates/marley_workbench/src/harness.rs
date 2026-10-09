//! The harness's sessions (#534): Marley follows rustal-harness's fleet over `rh mcp`, read side
//! only, and shows each session in the rail's Harness section and in a read-only tab.
//!
//! `marley.harness` names the MCP server's command, as a context server's is named; without it
//! Marley starts nothing. With it, Marley starts the command through Zed's MCP client, takes the
//! fleet from `fleet_snapshot` (the harness checks that its snapshot is Marley's own fold of its
//! events), and asks `fleet_events` from the snapshot's cursor every second, folding each event
//! with `marley_fleet::apply`; an expired cursor re-reads the snapshot. Zed's client sees no
//! server exit, so a call that fails, or takes more than five seconds, is the sign: the section
//! says the harness is not running and why, keeps its rows, marked stale, and starts the command
//! again after 1 s, doubling to at most 60 s.
//!
//! With `marley.harness_writes` on (#689), a session's tab answers the question it waits on
//! (`session_answer`), sends it text (`session_send`) and lists the commands that watch it
//! (`session_surface_to_human`), and `marley: open harness session` opens one from a declared
//! profile (`session_open`). The harness Marley runs itself is then followed with `--grant write`;
//! a `marley.harness` command carries its own grant, and a write it lacks comes back refused, with
//! the harness's reason in the tab.
//!
//! With `marley.embedded_harness` on and no `marley.harness`, Marley runs the harness itself
//! (#632): it finds `rh` (`MARLEY_RH`, else the search path), starts `rh --state <data dir>/harness
//! serve` as a child, waits for its ready line, and follows `rh --state <root> mcp` as above. `rh
//! mcp` reads the journal while `serve` is down, so the runtime's own state, from the child, goes
//! in the section's header too; a runtime that ends is started again after a growing wait, and the
//! harness's sessions, in its tmux server, outlive it.
//!
//! Three label conventions on a session say more than its state (#640, the harness's MREQ-005 to
//! MREQ-007): where the state came from, how far the agent is, and how much of its account's
//! quota each window has used. `Signals` reads them; a state the harness's runtime, the agent's
//! protocol or its reports did not declare is drawn weaker and never asks the user anything.

use std::any::TypeId;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::pin::pin;
use std::sync::Arc;
use std::time::Duration;

use agent_ui::{Agent, AgentPanel};
use anyhow::Context as _;
use command_palette_hooks::CommandPaletteFilter;
use context_server::types::requests::CallTool;
use context_server::types::{CallToolParams, CallToolResponse};
use context_server::{ContextServer, ContextServerCommand, ContextServerId};
use editor::Editor;
use futures::future::{Either, select};
use futures::{AsyncBufReadExt as _, StreamExt as _};
use gpui::{
    App, AsyncApp, ClipboardItem, Context, DismissEvent, Entity, EventEmitter, FocusHandle,
    Focusable, Global, SharedString, Subscription, SystemNotification, Task, WeakEntity, actions,
};
use marley_fleet::{FleetSnapshot, SessionEvent, State, apply};
use picker::{Picker, PickerDelegate};
use serde::Deserialize;
use serde_json::{Value, json};
use settings::{Settings as _, SettingsStore};
use ui::{ListItem, ListItemSpacing, prelude::*};
use util::ResultExt as _;
use workspace::item::Item;
use workspace::notifications::DetachAndPromptErr as _;
use workspace::{MultiWorkspace, Workspace};

use crate::{EmbeddedHarness, MarleySettings};

actions!(
    marley,
    [
        /// Opens a harness session from one of its declared profiles, through the harness's
        /// `session_open`.
        #[derive(Eq)]
        OpenHarnessSession
    ]
);

/// How often the fleet is asked for new events.
const POLL: Duration = Duration::from_secs(1);

/// How long one call may take before the harness counts as not answering.
const CALL_TIMEOUT: Duration = Duration::from_secs(5);

/// The longest wait between two starts of the command, in seconds.
const BACKOFF_MAX_S: u64 = 60;

/// How many of a session's last lines its tab shows.
const VIEW_LINES: u32 = 500;

/// How often an open tab reads its session's lines again while the session runs: an actor's
/// output does not change its envelope.
const VIEW_REFRESH: Duration = Duration::from_secs(2);

/// How many characters of a reason the section's header shows.
const REASON_CHARS: usize = 120;

/// How long `rh serve` may take to say it is ready.
const READY_TIMEOUT: Duration = Duration::from_secs(10);

/// The longest path a Unix socket takes, with its terminating NUL left out.
const SOCKET_PATH_MAX: usize = 107;

/// What `rh serve` says when another process already serves the root.
const ROOT_TAKEN: &str = "another runtime owns this state root";

/// The harness's connection, as the section's header says it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Connection {
    Connecting,
    Connected,
    /// Not running, with the reason.
    Down(SharedString),
}

/// The runtime Marley runs itself (#632), as the section's header says it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Runtime {
    Starting,
    Running,
    /// It ended, with the reason.
    Stopped(SharedString),
    /// No `rh` was found, with where Marley looked.
    Missing(SharedString),
}

/// Where the harness Marley follows comes from.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum Source {
    #[default]
    Off,
    /// `marley.harness`'s command.
    Command(ContextServerCommand),
    /// The runtime Marley runs itself, followed with the write grant when `writes` (#689).
    Embedded { writes: bool },
}

/// What Marley follows of the harness: the setting's command, the connection, and the fleet.
#[derive(Default)]
pub(crate) struct Harness {
    source: Source,
    connection: Option<Connection>,
    runtime: Option<Runtime>,
    seats: FleetSnapshot,
    /// Bumped each minute while a session works, or a quota window it shows has a reset ahead, so
    /// its `no update in N m` and its countdown are drawn again.
    minute: u64,
    /// Whether the rail's Harness section folds its rows under its header.
    folded: bool,
    server: Option<Arc<ContextServer>>,
    run: Option<Task<()>>,
    /// The embedded runtime's keeper, which starts `serve` and the run.
    embed: Option<Task<()>>,
    /// Whether the palette lists `OpenHarnessSession` and `NewHarnessSeat`, once its filter was
    /// set.
    palette_shown: Option<bool>,
    /// The `rh` the embedded runtime runs, once found (#691).
    rh: Option<PathBuf>,
    /// The command of the Agent Panel's Manager entry, while it is in the defaults (#694).
    manager_entry: Option<(PathBuf, Vec<String>)>,
    /// How far Marley has read the manager thread, once it has (#688).
    thread_cursor: Option<u64>,
    /// The manager's records posted while its thread was not in front, until it is (#688).
    unread: Vec<ManagerRecord>,
}

/// A record the manager posted to its thread that the user has not seen (#688).
#[derive(Clone, Debug)]
pub(crate) struct ManagerRecord {
    pub(crate) id: String,
    /// `message`, `report` or `confirmation`.
    pub(crate) kind: String,
    /// The first line of its text.
    pub(crate) line: String,
}

impl ManagerRecord {
    /// The record's kind as a word the user reads: Report, Message, Confirmation.
    pub(crate) fn kind_word(&self) -> String {
        let mut characters = self.kind.chars();
        characters.next().map_or_else(String::new, |first| {
            first.to_uppercase().chain(characters).collect()
        })
    }
}

impl Global for Harness {}

impl Harness {
    /// The connection, once Marley follows a harness.
    pub(crate) fn connection(cx: &App) -> Option<Connection> {
        cx.try_global::<Self>()?.connection.clone()
    }

    /// The state of the runtime Marley runs itself, while it runs one (#632).
    pub(crate) fn runtime(cx: &App) -> Option<Runtime> {
        cx.try_global::<Self>()?.runtime.clone()
    }

    /// Whether the rail's Harness section folds its rows.
    pub(crate) fn folded(cx: &App) -> bool {
        cx.try_global::<Self>()
            .is_some_and(|harness| harness.folded)
    }

    /// Folds the rail's Harness section, or unfolds it.
    pub(crate) fn toggle_folded(cx: &mut App) {
        let harness = cx.global_mut::<Self>();
        harness.folded = !harness.folded;
    }

    /// The harness's MCP server, while Marley is connected to it.
    fn server(cx: &App) -> Option<Arc<ContextServer>> {
        cx.try_global::<Self>()?.server.clone()
    }

    /// The folder of the declared profiles, while Marley runs the harness itself and knows its
    /// root.
    fn profiles_dir(cx: &App) -> Option<PathBuf> {
        let harness = cx.try_global::<Self>()?;
        matches!(harness.source, Source::Embedded { .. })
            .then(|| paths::data_dir().join("harness").join("profiles"))
    }

    /// The program and the arguments that run a harness command on the harness Marley follows
    /// (#691): a `marley.harness` command's up to its `mcp`, so one over SSH stays over SSH, or
    /// the embedded runtime's `rh --state <root>`.
    pub(crate) fn seat_command(cx: &App) -> Option<(PathBuf, Vec<String>)> {
        let harness = cx.try_global::<Self>()?;
        match &harness.source {
            Source::Off => None,
            Source::Command(command) => {
                let mcp = command.args.iter().position(|argument| argument == "mcp")?;
                Some((command.path.clone(), command.args[..mcp].to_vec()))
            }
            Source::Embedded { .. } => {
                let root = paths::data_dir().join("harness");
                Some((
                    harness.rh.clone()?,
                    vec!["--state".to_string(), root.display().to_string()],
                ))
            }
        }
    }

    /// The manager's records the user has not seen (#688).
    pub(crate) fn unread(cx: &App) -> &[ManagerRecord] {
        cx.try_global::<Self>()
            .map_or(&[], |harness| harness.unread.as_slice())
    }

    /// The harness's sessions, in the order the harness first published them.
    pub(crate) fn seats(cx: &App) -> &FleetSnapshot {
        static NONE: std::sync::LazyLock<FleetSnapshot> =
            std::sync::LazyLock::new(FleetSnapshot::default);
        cx.try_global::<Self>()
            .map_or(&NONE, |harness| &harness.seats)
    }
}

/// Follows `marley.harness`; [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    cx.set_global(Harness::default());
    follow_setting(cx);
    cx.observe_global::<SettingsStore>(follow_setting).detach();
    cx.observe_new(|workspace: &mut Workspace, _, cx| {
        workspace.register_action(|workspace, _: &OpenHarnessSession, window, cx| {
            open_session_picker(workspace, window, cx);
        });
        workspace.register_action(
            |workspace, _: &crate::harness_seat::NewHarnessSeat, window, cx| {
                crate::harness_seat::open_form(workspace, window, cx);
            },
        );
        cx.defer(filter_palette);
    })
    .detach();
}

/// Whether `marley.harness_writes` is on.
fn harness_writes(cx: &App) -> bool {
    cx.global::<SettingsStore>()
        .merged_settings()
        .marley
        .as_ref()
        .and_then(|marley| marley.harness_writes)
        .unwrap_or(false)
}

/// Whether the harness's write verbs are on: `marley.harness_writes`, with a harness to follow.
pub(crate) fn writes_on(cx: &App) -> bool {
    harness_writes(cx)
        && cx
            .try_global::<Harness>()
            .is_some_and(|harness| harness.source != Source::Off)
}

/// The Agent Panel's entry for the root's manager thread, and its key in the agent servers (#694).
pub(crate) const MANAGER_ENTRY: &str = "Manager";

/// Whether the followed harness has a manager: a session labelled `role: manager`, which the
/// harness moves with the designation.
fn has_manager(cx: &App) -> bool {
    Harness::seats(cx).seats().iter().any(|seat| {
        seat.labels
            .get("role")
            .is_some_and(|role| role == "manager")
    })
}

/// Whether the user is looking at the manager thread: the active window's workspace shows the
/// Agent Panel with the Manager agent selected (#688).
fn manager_in_front(cx: &App) -> bool {
    let Some(multi_workspace) = cx
        .active_window()
        .and_then(|window| window.downcast::<MultiWorkspace>())
        .and_then(|window| window.read(cx).ok())
    else {
        return false;
    };
    let workspace = multi_workspace.workspace();
    let manager = Agent::Custom {
        id: project::AgentId::new(MANAGER_ENTRY),
    };
    AgentPanel::is_visible(workspace, cx)
        && workspace
            .read(cx)
            .panel::<AgentPanel>(cx)
            .is_some_and(|panel| panel.read(cx).selected_agent(cx) == manager)
}

/// A page of `thread_read` (#688).
#[derive(Deserialize)]
struct ThreadPage {
    events: Vec<ThreadEntry>,
    next_cursor: u64,
}

#[derive(Deserialize)]
struct ThreadEntry {
    kind: String,
    change: Value,
}

/// The manager thread's records after `after`, read to the end of what the harness keeps, and the
/// cursor reached: the manager's records only.
async fn read_thread(
    server: &ContextServer,
    after: u64,
    cx: &AsyncApp,
) -> Result<(Vec<ManagerRecord>, u64), Failure> {
    let mut cursor = after;
    let mut records = Vec::new();
    // A page holds 128 changes and the thread keeps 8,192 records, so this many pages reach its
    // end.
    for _ in 0..=THREAD_PAGES {
        let value = call(server, "thread_read", json!({ "after": cursor }), cx).await?;
        let page = serde_json::from_value::<ThreadPage>(value).map_err(|error| {
            Failure::Failed(anyhow::anyhow!("thread_read did not parse: {error}"))
        })?;
        cursor = page.next_cursor.max(cursor);
        if page.events.is_empty() {
            break;
        }
        records.extend(page.events.iter().filter_map(|entry| {
            let change = &entry.change;
            let text = |name: &str| change.get(name).and_then(Value::as_str).unwrap_or_default();
            (entry.kind == "thread_record" && text("author") == "manager").then(|| ManagerRecord {
                id: text("id").to_string(),
                kind: text("kind").to_string(),
                line: text("text").lines().next().unwrap_or_default().to_string(),
            })
        }));
    }
    Ok((records, cursor))
}

/// How many pages of the thread one read goes through at most.
const THREAD_PAGES: usize = 64;

/// Follows the manager thread while the fleet has a manager and the write verbs are on: the first
/// read goes to the end and raises nothing, and each later one hands the manager's new records to
/// `manager_posted`. A thread that cannot be read is logged and read again on the next pass.
async fn follow_thread(server: &ContextServer, cx: &AsyncApp) {
    let cursor = cx.update(|cx| {
        (has_manager(cx) && writes_on(cx)).then(|| cx.global::<Harness>().thread_cursor)
    });
    let Some(cursor) = cursor else {
        return;
    };
    match read_thread(server, cursor.unwrap_or(0), cx).await {
        Ok((records, reached)) => cx.update(|cx| {
            cx.global_mut::<Harness>().thread_cursor = Some(reached);
            if cursor.is_some() {
                manager_posted(records, cx);
            }
        }),
        Err(Failure::Resync) => {
            // Past what the harness keeps: read to the end again, quietly.
            cx.update(|cx| cx.global_mut::<Harness>().thread_cursor = None);
        }
        Err(Failure::Failed(error)) => log::debug!("harness: the manager thread: {error:#}"),
    }
    cx.update(|cx| {
        if !cx.global::<Harness>().unread.is_empty() && manager_in_front(cx) {
            cx.global_mut::<Harness>().unread.clear();
        }
    });
}

/// Raises the manager's new records while its thread is not in front: a desktop notice for each,
/// and a mark the rail's inbox lists until the thread is in front (#688).
fn manager_posted(records: Vec<ManagerRecord>, cx: &mut App) {
    if records.is_empty() || manager_in_front(cx) {
        return;
    }
    for record in &records {
        log::info!(
            "harness: the manager posted a {}: {}",
            record.kind,
            record.line
        );
        cx.show_system_notification(SystemNotification {
            tag: SharedString::from(format!("marley-manager-{}", record.id)),
            title: SharedString::from(format!("Manager: {}", record.kind_word())),
            body: SharedString::from(record.line.clone()),
            actions: Vec::new(),
        });
    }
    cx.global_mut::<Harness>().unread.extend(records);
}

/// Puts a `Manager` agent server in the settings' in-memory defaults while the write verbs are on
/// and the followed harness has a manager, a session labelled `role: manager`, which the harness
/// moves with the designation; takes it out otherwise (#694). It runs the harness's `rh acp`
/// through the command Marley follows the harness with, as `seat_command` gives it, so a root over
/// SSH is reached over SSH; the thread it shows is the person's side of the manager's.
fn sync_manager_entry(cx: &mut App) {
    let command = (writes_on(cx) && has_manager(cx))
        .then(|| Harness::seat_command(cx))
        .flatten()
        .map(|(program, mut args)| {
            args.push("acp".to_string());
            (program, args)
        });
    if cx.global::<Harness>().manager_entry == command {
        return;
    }
    cx.global_mut::<Harness>()
        .manager_entry
        .clone_from(&command);
    cx.update_global::<SettingsStore, _>(|store, cx| {
        store.update_default_settings(cx, |defaults| {
            let servers = defaults.agent_servers.get_or_insert_default();
            match command {
                Some((path, args)) => {
                    servers.insert(
                        MANAGER_ENTRY.to_string(),
                        settings::CustomAgentServerSettings::Custom {
                            path,
                            args,
                            env: collections::HashMap::default(),
                            default_mode: None,
                            default_config_options: collections::HashMap::default(),
                            favorite_config_option_values: collections::HashMap::default(),
                        },
                    );
                }
                None => {
                    servers.remove(MANAGER_ENTRY);
                }
            }
        });
    });
}

/// Lists `OpenHarnessSession` in the palette while the write verbs are on. Before the palette has
/// its filter there is nothing to set; the next call sets it.
fn filter_palette(cx: &mut App) {
    let on = writes_on(cx);
    if cx.global::<Harness>().palette_shown == Some(on)
        || CommandPaletteFilter::try_global(cx).is_none()
    {
        return;
    }
    cx.global_mut::<Harness>().palette_shown = Some(on);
    let command = [
        TypeId::of::<OpenHarnessSession>(),
        TypeId::of::<crate::harness_seat::NewHarnessSeat>(),
    ];
    CommandPaletteFilter::update_global(cx, |filter, _| {
        if on {
            filter.show_action_types(&command);
        } else {
            filter.hide_action_types(&command);
        }
    });
}

/// Starts following the harness the settings name, or the one Marley runs itself, or stops, when
/// the settings changed: `marley.harness` first, then `marley.embedded_harness`.
fn follow_setting(cx: &mut App) {
    let settings = MarleySettings::get_global(cx);
    let source = match (&settings.harness, settings.embedded_harness) {
        (Some(command), _) => Source::Command(command.clone()),
        (None, EmbeddedHarness::Run) => Source::Embedded {
            writes: harness_writes(cx),
        },
        (None, EmbeddedHarness::Off) => Source::Off,
    };
    if cx.global::<Harness>().source == source {
        filter_palette(cx);
        sync_manager_entry(cx);
        return;
    }
    let (run, embed) = match &source {
        Source::Off => (None, None),
        Source::Command(command) => {
            let command = command.clone();
            let run = cx.spawn(async move |cx| follow(command, cx).await);
            (Some(run), None)
        }
        Source::Embedded { .. } => (None, Some(cx.spawn(async move |cx| embed(cx).await))),
    };
    let harness = cx.global_mut::<Harness>();
    harness.connection = matches!(source, Source::Command(_)).then_some(Connection::Connecting);
    harness.runtime = matches!(source, Source::Embedded { .. }).then_some(Runtime::Starting);
    harness.source = source;
    harness.seats = FleetSnapshot::default();
    harness.server = None;
    // The old run and runtime go with their tasks: `serve`'s child dies with its handle, and the
    // harness's sessions live on in its tmux server.
    harness.run = run;
    harness.embed = embed;
    filter_palette(cx);
    sync_manager_entry(cx);
}

/// Keeps the runtime Marley runs itself: finds `rh`, serves the root, follows it once it is
/// ready, and serves it again after a growing wait whenever `serve` ends (#632).
async fn embed(cx: &AsyncApp) {
    let rh = match find_rh(cx).await {
        Ok(rh) => rh,
        Err(reason) => {
            set_runtime(Runtime::Missing(reason.into()), cx);
            return;
        }
    };
    cx.update(|cx| cx.global_mut::<Harness>().rh = Some(rh.clone()));
    let root = paths::data_dir().join("harness");
    let socket = root.join("runtime.sock");
    if socket.as_os_str().len() > SOCKET_PATH_MAX {
        let reason = format!(
            "{} is too long a path for the harness's socket",
            socket.display()
        );
        set_runtime(Runtime::Stopped(reason.into()), cx);
        return;
    }
    let mut failures = 0_u32;
    loop {
        set_runtime(Runtime::Starting, cx);
        let ended = serve(&rh, &root, &mut failures, cx).await;
        let wait_s = 1_u64
            .checked_shl(failures)
            .unwrap_or(BACKOFF_MAX_S)
            .min(BACKOFF_MAX_S);
        failures = failures.saturating_add(1);
        match ended {
            Served::Elsewhere => {
                // Another process serves the root: follow it, and look again in a while.
                set_runtime(Runtime::Running, cx);
                follow_embedded(&rh, &root, cx);
                cx.background_executor()
                    .timer(Duration::from_secs(BACKOFF_MAX_S))
                    .await;
                continue;
            }
            Served::Ended(reason) => {
                log::info!("harness: the runtime stopped: {reason}");
                set_runtime(Runtime::Stopped(clip(&reason).into()), cx);
            }
        }
        cx.background_executor()
            .timer(Duration::from_secs(wait_s))
            .await;
    }
}

/// How a `serve` came to an end.
enum Served {
    /// It ran, or failed to, and ended, with why.
    Ended(String),
    /// The root's lock was taken: another process serves it.
    Elsewhere,
}

/// Runs `rh --state <root> serve` until it ends: once it says it is ready, the runtime runs and
/// Marley follows it; a run that got ready resets `failures`.
async fn serve(rh: &Path, root: &Path, failures: &mut u32, cx: &AsyncApp) -> Served {
    let arguments = [OsStr::new("--state"), root.as_os_str(), OsStr::new("serve")];
    let mut child = match crate::process::follow_with_errors(rh, &arguments) {
        Ok(child) => child,
        Err(error) => return Served::Ended(format!("`rh serve` did not start: {error}")),
    };
    // The last line it writes to stderr says why it ended.
    let errors = child.stderr.take().map(|stderr| {
        cx.background_spawn(async move {
            let mut lines = futures::io::BufReader::new(stderr).lines();
            let mut last = String::new();
            while let Some(Ok(line)) = lines.next().await {
                if !line.trim().is_empty() {
                    last = line;
                }
            }
            last
        })
    });
    let Some(stdout) = child.stdout.take() else {
        return Served::Ended("`rh serve` gave no output".to_string());
    };
    let mut lines = futures::io::BufReader::new(stdout).lines();
    // Some(ready) when the output said whether it got ready, None when the wait ran out.
    let ready = {
        let waiting = async {
            while let Some(line) = lines.next().await {
                if line.is_ok_and(|line| is_ready(&line)) {
                    return true;
                }
            }
            false
        };
        match select(
            pin!(waiting),
            pin!(cx.background_executor().timer(READY_TIMEOUT)),
        )
        .await
        {
            Either::Left((ready, _)) => Some(ready),
            Either::Right(_) => None,
        }
    };
    match ready {
        Some(true) => {
            *failures = 0;
            set_runtime(Runtime::Running, cx);
            follow_embedded(rh, root, cx);
            // It runs until its output ends.
            while lines.next().await.is_some() {}
        }
        Some(false) => {}
        None => {
            if let Err(error) = child.kill() {
                log::warn!("harness: stopping a `rh serve` that never got ready: {error}");
            }
        }
    }
    let status = child.status().await;
    let said = match errors {
        Some(errors) => errors.await,
        None => String::new(),
    };
    if said.contains(ROOT_TAKEN) {
        return Served::Elsewhere;
    }
    let exit = match status {
        Ok(status) => status.code().map_or_else(
            || "it was stopped by a signal".to_string(),
            |code| format!("exit {code}"),
        ),
        Err(error) => format!("its end was not read: {error}"),
    };
    Served::Ended(if said.is_empty() {
        exit
    } else {
        format!("{exit}: {said}")
    })
}

/// Whether `line` is `rh serve`'s ready line, `{"ready": true, …}`.
fn is_ready(line: &str) -> bool {
    serde_json::from_str::<Value>(line)
        .ok()
        .and_then(|value| value.get("ready")?.as_bool())
        .unwrap_or(false)
}

/// Starts following the embedded runtime's `rh mcp`, with `--grant write` while
/// `marley.harness_writes` is on, unless Marley follows it already.
fn follow_embedded(rh: &Path, root: &Path, cx: &AsyncApp) {
    let mut args = vec![
        "--state".to_string(),
        root.display().to_string(),
        "mcp".to_string(),
    ];
    if cx.update(|cx| harness_writes(cx)) {
        args.extend(["--grant".to_string(), "write".to_string()]);
    }
    let command = ContextServerCommand {
        path: rh.to_path_buf(),
        args,
        env: None,
        timeout: None,
    };
    cx.update(|cx| {
        if cx.global::<Harness>().run.is_some() {
            return;
        }
        let run = cx.spawn(async move |cx| follow(command, cx).await);
        let harness = cx.global_mut::<Harness>();
        harness.connection = Some(Connection::Connecting);
        harness.run = Some(run);
    });
}

/// Finds `rh`: `MARLEY_RH`, else the search path; or says where Marley looked.
async fn find_rh(cx: &AsyncApp) -> Result<PathBuf, String> {
    let named = std::env::var_os("MARLEY_RH").map(PathBuf::from);
    // A stat and a walk of the search path: off the main thread.
    cx.background_spawn(futures::future::lazy(move |_| match named {
        Some(path) if path.is_file() => Ok(path),
        Some(path) => Err(format!(
            "no rh at MARLEY_RH: {} is not there",
            path.display()
        )),
        None => {
            which::which("rh").map_err(|_| "no rh on the PATH, and MARLEY_RH is unset".to_string())
        }
    }))
    .await
}

/// Sets the embedded runtime's state.
fn set_runtime(runtime: Runtime, cx: &AsyncApp) {
    cx.update(|cx| {
        // Taken mutably only on a change, so the rail redraws only then.
        if cx.global::<Harness>().runtime.as_ref() != Some(&runtime) {
            cx.global_mut::<Harness>().runtime = Some(runtime);
        }
    });
}

/// `reason`'s first line, cut to what a header shows.
fn clip(reason: &str) -> String {
    reason
        .lines()
        .next()
        .unwrap_or_default()
        .chars()
        .take(REASON_CHARS)
        .collect()
}

/// Keeps a connection to the harness's MCP server: connects, follows the fleet until a call
/// fails, and connects again after a growing wait.
async fn follow(command: ContextServerCommand, cx: &AsyncApp) {
    let mut failures = 0_u32;
    loop {
        let error = match connected(&command, &mut failures, cx).await {
            Ok(never) => match never {},
            Err(error) => error,
        };
        // The root cause says the most in the few words a header has.
        let reason = error.root_cause().to_string();
        let reason: String = reason
            .lines()
            .next()
            .unwrap_or_default()
            .chars()
            .take(REASON_CHARS)
            .collect();
        log::info!("harness: not running: {error:#}");
        cx.update(|cx| {
            let harness = cx.global_mut::<Harness>();
            harness.server = None;
            harness.connection = Some(Connection::Down(reason.into()));
        });
        let wait_s = 1_u64
            .checked_shl(failures)
            .unwrap_or(BACKOFF_MAX_S)
            .min(BACKOFF_MAX_S);
        failures = failures.saturating_add(1);
        cx.background_executor()
            .timer(Duration::from_secs(wait_s))
            .await;
        cx.update(|cx| {
            cx.global_mut::<Harness>().connection = Some(Connection::Connecting);
        });
    }
}

/// One connection: starts the server, takes the fleet, and folds its events each second. Returns
/// only the failure that ends it.
async fn connected(
    command: &ContextServerCommand,
    failures: &mut u32,
    cx: &AsyncApp,
) -> anyhow::Result<std::convert::Infallible> {
    let server = Arc::new(ContextServer::stdio(
        ContextServerId(Arc::from("marley-harness")),
        command.clone(),
        None,
        None,
    ));
    let executor = cx.background_executor().clone();
    match select(pin!(server.start(cx)), pin!(executor.timer(CALL_TIMEOUT))).await {
        Either::Left((started, _)) => started.context("the harness's MCP server did not start")?,
        Either::Right(_) => anyhow::bail!("the harness's MCP server did not answer"),
    }
    let mut cursor = seed(&server, cx).await?;
    *failures = 0;
    cx.update(|cx| {
        let harness = cx.global_mut::<Harness>();
        harness.server = Some(Arc::clone(&server));
        harness.connection = Some(Connection::Connected);
    });
    let mut minute = now_minute();
    loop {
        executor.timer(POLL).await;
        loop {
            let page = match call(&server, "fleet_events", json!({ "after": cursor }), cx).await {
                Ok(page) => serde_json::from_value::<EventPage>(page)
                    .context("fleet_events's answer did not parse")?,
                Err(Failure::Resync) => {
                    cursor = seed(&server, cx).await?;
                    break;
                }
                Err(Failure::Failed(error)) => return Err(error),
            };
            cursor = page.next_cursor;
            if page.events.is_empty() {
                break;
            }
            cx.update(|cx| {
                let harness = cx.global_mut::<Harness>();
                for entry in &page.events {
                    apply(&mut harness.seats, &entry.event);
                }
                sync_manager_entry(cx);
            });
        }
        follow_thread(&server, cx).await;
        let now = now_minute();
        if now != minute {
            minute = now;
            cx.update(|cx| {
                let now_ms = now_ms();
                let counting = Harness::seats(cx).seats().iter().any(|seat| {
                    seat.state == State::Working || Signals::of(&seat.labels).resets_after(now_ms)
                });
                if counting {
                    cx.global_mut::<Harness>().minute = now;
                }
            });
        }
    }
}

/// Takes the fleet from `fleet_snapshot` and gives the cursor it holds at.
async fn seed(server: &ContextServer, cx: &AsyncApp) -> anyhow::Result<u64> {
    let value = match call(server, "fleet_snapshot", json!({}), cx).await {
        Ok(value) => value,
        Err(Failure::Resync) => anyhow::bail!("fleet_snapshot asked for a resync"),
        Err(Failure::Failed(error)) => return Err(error),
    };
    let cursor = value
        .get("cursor")
        .and_then(Value::as_u64)
        .context("fleet_snapshot gave no cursor")?;
    let seats: FleetSnapshot =
        serde_json::from_value(value).context("fleet_snapshot's seats did not parse")?;
    cx.update(|cx| {
        cx.global_mut::<Harness>().seats = seats;
        sync_manager_entry(cx);
    });
    Ok(cursor)
}

/// A page of `fleet_events`.
#[derive(Deserialize)]
struct EventPage {
    events: Vec<EventEntry>,
    next_cursor: u64,
}

#[derive(Deserialize)]
struct EventEntry {
    event: SessionEvent,
}

/// Why a call came back without an answer.
enum Failure {
    /// The cursor fell behind what the harness keeps.
    Resync,
    Failed(anyhow::Error),
}

/// Calls one of the harness's read tools and gives its structured content, within
/// [`CALL_TIMEOUT`].
async fn call(
    server: &ContextServer,
    tool: &str,
    arguments: Value,
    cx: &AsyncApp,
) -> Result<Value, Failure> {
    let response = request(server, tool, arguments, cx)
        .await
        .map_err(Failure::Failed)?;
    answer(tool, response)
}

/// Calls one of the harness's write verbs and reads its receipt: the accepted `value`, or the
/// refusal's reason. A refused receipt comes with `isError` set, a missing grant's included.
async fn call_write(
    server: &ContextServer,
    tool: &str,
    arguments: Value,
    cx: &AsyncApp,
) -> Result<Value, SharedString> {
    let response = request(server, tool, arguments, cx)
        .await
        .map_err(|error| SharedString::from(format!("{error:#}")))?;
    let text = response.text_contents();
    let receipt = response
        .structured_content
        .or_else(|| serde_json::from_str::<Value>(&text).ok());
    let Some(receipt) = receipt else {
        let said = text.lines().next().unwrap_or("no receipt").to_string();
        return Err(said.into());
    };
    if receipt.get("result").and_then(Value::as_str) == Some("accepted") {
        return Ok(receipt.get("value").cloned().unwrap_or(Value::Null));
    }
    let reason = receipt
        .get("reason")
        .and_then(Value::as_str)
        .unwrap_or("refused")
        .to_string();
    Err(reason.into())
}

/// Sends one tool call, within [`CALL_TIMEOUT`].
async fn request(
    server: &ContextServer,
    tool: &str,
    arguments: Value,
    cx: &AsyncApp,
) -> anyhow::Result<CallToolResponse> {
    let executor = cx.background_executor().clone();
    let asking = async {
        let protocol = server
            .client()
            .context("the harness's MCP server is not running")?;
        protocol
            .request::<CallTool>(CallToolParams {
                name: tool.to_string(),
                arguments: Some(arguments),
                meta: None,
            })
            .await
            .with_context(|| format!("{tool} failed"))
    };
    match select(pin!(asking), pin!(executor.timer(CALL_TIMEOUT))).await {
        Either::Left((response, _)) => response,
        Either::Right(_) => Err(anyhow::anyhow!(
            "the harness did not answer {tool} within {} s",
            CALL_TIMEOUT.as_secs()
        )),
    }
}

/// A tool's structured content, or why there is none.
fn answer(tool: &str, response: CallToolResponse) -> Result<Value, Failure> {
    let text = response.text_contents();
    if response.is_error == Some(true) {
        if text.starts_with("resync_required") {
            return Err(Failure::Resync);
        }
        return Err(Failure::Failed(anyhow::anyhow!(
            "{tool} failed: {}",
            text.lines().next().unwrap_or_default()
        )));
    }
    response.structured_content.map_or_else(
        || {
            serde_json::from_str(&text)
                .with_context(|| format!("{tool} gave no JSON"))
                .map_err(Failure::Failed)
        },
        Ok,
    )
}

/// A session's question as the rail shows it: without the `[wait <key>, generation <id>]` the
/// harness adds to an actor's prompt to route its answer, which leaves no room for the options.
pub(crate) fn shown_prompt(prompt: &str) -> &str {
    match prompt.rfind(" [wait ") {
        Some(start) if prompt.ends_with(']') => &prompt[..start],
        _ => prompt,
    }
}

/// The label naming where a session's state came from (MREQ-005).
const STATE_SOURCE: &str = "state.source";

/// The label holding how far the agent is, from 0 to 100 (MREQ-006).
const PROGRESS_PERCENT: &str = "progress.percent";

/// The label holding what the agent is doing, one line (MREQ-006).
const PROGRESS_ACTIVITY: &str = "progress.activity";

/// What starts a quota label: `quota.KIND.percent_used` and `quota.KIND.resets_at_ms`
/// (MREQ-007).
const QUOTA: &str = "quota.";

/// How a quota window's percent used ends its label.
const PERCENT_USED: &str = ".percent_used";

/// How a quota window's reset ends its label, in milliseconds since the epoch.
const RESETS_AT_MS: &str = ".resets_at_ms";

/// The label naming the account the quota belongs to: a digest, never an email address.
const QUOTA_ACCOUNT: &str = "quota.account";

/// Where a harness session's state came from, from its `state.source` label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum StateSource {
    /// The client's own protocol, such as Codex's App Server.
    Protocol,
    /// The agent's own reports.
    Reported,
    /// The harness's runtime, while a session it restarted starts (the harness's D173).
    Runtime,
    /// Read off the agent's screen.
    Detected,
    /// A word Marley does not know.
    Other(String),
}

impl StateSource {
    fn from_label(word: &str) -> Self {
        match word {
            "protocol" => Self::Protocol,
            "reported" => Self::Reported,
            "runtime" => Self::Runtime,
            "detected" => Self::Detected,
            other => Self::Other(other.to_string()),
        }
    }

    /// Whether Marley acts on a state from here: a detected state, and one from a source Marley
    /// does not know, is shown and never asks the user anything.
    pub(crate) const fn declared(&self) -> bool {
        matches!(self, Self::Protocol | Self::Reported | Self::Runtime)
    }

    /// The source as the harness names it.
    pub(crate) fn word(&self) -> &str {
        match self {
            Self::Protocol => "protocol",
            Self::Reported => "reported",
            Self::Runtime => "runtime",
            Self::Detected => "detected",
            Self::Other(word) => word,
        }
    }
}

/// How far a session's agent is, from its `progress.` labels; either part may be missing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Progress {
    /// From 0 to 100.
    pub(crate) percent: Option<u8>,
    /// The activity's first line.
    pub(crate) activity: Option<String>,
}

/// One window of an account's quota, from its `quota.KIND.` labels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct QuotaWindow {
    /// The window's kind as the harness sends it, such as `five_hour`.
    pub(crate) kind: String,
    /// How much of the window is used, from 0 to 100.
    pub(crate) percent_used: u8,
    /// When the window resets, in milliseconds since the epoch.
    pub(crate) resets_at_ms: Option<u64>,
}

/// What a harness session's labels say beside its state (#640). A value that does not parse is
/// left out, never guessed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Signals {
    /// Where the state came from; none for a harness that sends no source.
    pub(crate) source: Option<StateSource>,
    pub(crate) progress: Option<Progress>,
    /// Most used first, then by kind.
    pub(crate) quota: Vec<QuotaWindow>,
    /// The quota's account, unless it reads as an email address.
    pub(crate) account: Option<String>,
}

impl Signals {
    pub(crate) fn of(labels: &BTreeMap<String, String>) -> Self {
        let source = labels
            .get(STATE_SOURCE)
            .map(|word| word.trim())
            .filter(|word| !word.is_empty())
            .map(StateSource::from_label);
        let percent = labels
            .get(PROGRESS_PERCENT)
            .and_then(|text| percent_of(text));
        let activity = labels
            .get(PROGRESS_ACTIVITY)
            .and_then(|text| text.lines().next())
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string);
        let progress =
            (percent.is_some() || activity.is_some()).then_some(Progress { percent, activity });
        let mut quota: Vec<QuotaWindow> = labels
            .iter()
            .filter_map(|(key, value)| {
                let kind = key.strip_prefix(QUOTA)?.strip_suffix(PERCENT_USED)?;
                if kind.is_empty() {
                    return None;
                }
                let resets_at_ms = labels
                    .get(&format!("{QUOTA}{kind}{RESETS_AT_MS}"))
                    .and_then(|text| text.trim().parse().ok());
                Some(QuotaWindow {
                    kind: kind.to_string(),
                    percent_used: percent_of(value)?,
                    resets_at_ms,
                })
            })
            .collect();
        quota.sort_by(|left, right| {
            right
                .percent_used
                .cmp(&left.percent_used)
                .then_with(|| left.kind.cmp(&right.kind))
        });
        let account = labels
            .get(QUOTA_ACCOUNT)
            .map(|name| name.trim())
            .filter(|name| !name.is_empty() && !name.contains('@'))
            .map(str::to_string);
        Self {
            source,
            progress,
            quota,
            account,
        }
    }

    /// Whether Marley acts on the session's state; a harness that sends no source is declared.
    pub(crate) fn declared(&self) -> bool {
        self.source.as_ref().is_none_or(StateSource::declared)
    }

    /// Whether the labels say anything beside the state.
    pub(crate) const fn is_empty(&self) -> bool {
        self.source.is_none()
            && self.progress.is_none()
            && self.quota.is_empty()
            && self.account.is_none()
    }

    /// Whether a quota window resets after `now_ms`, so its countdown is drawn again.
    fn resets_after(&self, now_ms: u64) -> bool {
        self.quota
            .iter()
            .any(|window| window.resets_at_ms.is_some_and(|reset| reset > now_ms))
    }
}

/// A percent label's value: a finite number from 0 to 100, rounded to a whole one.
fn percent_of(text: &str) -> Option<u8> {
    let value: f64 = text.trim().parse().ok()?;
    if !value.is_finite() || !(0.0..=100.0).contains(&value) {
        return None;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // rounded, from 0 to 100
    Some(value.round() as u8)
}

/// The minutes since the epoch.
fn now_minute() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs() / 60)
}

/// The milliseconds since the epoch.
pub(crate) fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| {
            u64::try_from(since.as_millis()).unwrap_or(u64::MAX)
        })
}

/// Shows the tab of the session `id` in the window's Home group, as the rail's row does (#676): a
/// session belongs to no project.
pub(crate) fn open_in_home(
    asked_from: WeakEntity<Workspace>,
    id: String,
    window: &Window,
    cx: &mut App,
) {
    crate::groups::in_group(
        crate::groups::GroupKind::Home,
        asked_from,
        window,
        cx,
        move |workspace, window, cx| open(workspace, &id, window, cx),
    );
}

/// Shows the tab of the session `id` in `workspace`, or brings forward the one open there.
pub(crate) fn open(
    workspace: &mut Workspace,
    id: &str,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let open = workspace
        .items_of_type::<HarnessView>(cx)
        .find(|view| view.read(cx).id == id);
    if let Some(open) = open {
        workspace.activate_item(&open, true, true, window, cx);
        return;
    }
    let id = id.to_string();
    let handle = workspace.weak_handle();
    let view = cx.new(|cx| HarnessView::new(id, handle, window, cx));
    workspace.add_item_to_center(Box::new(view), window, cx);
}

/// A harness session's output: its last lines as the harness renders them. While the write verbs
/// are on, the question it waits on with a button per option, a line to send it, and the commands
/// that watch it (#689).
pub(crate) struct HarnessView {
    id: String,
    /// The workspace the tab opened in, where a view's terminal opens (#690).
    workspace: WeakEntity<Workspace>,
    title: SharedString,
    lines: Vec<SharedString>,
    error: Option<SharedString>,
    /// The envelope's time and state when the lines were last asked for.
    seen: Option<(u64, State)>,
    focus_handle: FocusHandle,
    reading: Option<Task<()>>,
    /// The text to send the session.
    send_editor: Entity<Editor>,
    /// What the last write came to, and its color.
    status: Option<(SharedString, Color)>,
    /// The commands that watch the session, once asked for: each view's kind and command line.
    views: Vec<(SharedString, SharedString)>,
    writing: Option<Task<()>>,
    _harness: Subscription,
    _refresh: Task<()>,
}

impl std::fmt::Debug for HarnessView {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HarnessView")
            .field("id", &self.id)
            .field("lines", &self.lines.len())
            .finish_non_exhaustive()
    }
}

impl HarnessView {
    fn new(
        id: String,
        workspace: WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let title = Harness::seats(cx).get(&id).map_or_else(
            || SharedString::from(id.clone()),
            |seat| seat.title.clone().into(),
        );
        let harness = cx.observe_global_in::<Harness>(window, |view, _, cx| view.seat_changed(cx));
        let refresh = cx.spawn(async move |view, cx| {
            loop {
                cx.background_executor().timer(VIEW_REFRESH).await;
                let Ok(()) = view.update(cx, |view, cx| {
                    let running = Harness::seats(cx)
                        .get(&view.id)
                        .is_some_and(|seat| seat.state != State::Done);
                    if running {
                        view.ask_lines(cx);
                    }
                }) else {
                    break;
                };
            }
        });
        let send_editor = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("Text to send the session", window, cx);
            editor
        });
        let mut view = Self {
            id,
            workspace,
            title,
            lines: Vec::new(),
            error: None,
            seen: None,
            focus_handle: cx.focus_handle(),
            reading: None,
            send_editor,
            status: None,
            views: Vec::new(),
            writing: None,
            _harness: harness,
            _refresh: refresh,
        };
        view.seat_changed(cx);
        view
    }

    /// Reads the lines again when the session's envelope changed.
    fn seat_changed(&mut self, cx: &mut Context<Self>) {
        let Some(seat) = Harness::seats(cx).get(&self.id) else {
            return;
        };
        let now = (seat.last_event_ms, seat.state);
        let title = SharedString::from(seat.title.clone());
        if self.title != title {
            self.title = title;
            cx.notify();
        }
        if self.seen != Some(now) {
            self.seen = Some(now);
            self.ask_lines(cx);
        }
    }

    /// Asks the harness for the session's last lines, unless a read is under way.
    fn ask_lines(&mut self, cx: &Context<Self>) {
        if self.reading.is_some() {
            return;
        }
        let Some(server) = cx
            .try_global::<Harness>()
            .and_then(|harness| harness.server.clone())
        else {
            return;
        };
        let arguments = json!({
            "id": self.id,
            "range": { "mode": "tail", "lines": VIEW_LINES },
        });
        self.reading = Some(cx.spawn(async move |view, cx| {
            let read = call(&server, "session_read", arguments, cx).await;
            view.update(cx, |view, cx| {
                view.reading = None;
                match read.map_err(|failure| match failure {
                    Failure::Resync => anyhow::anyhow!("session_read asked for a resync"),
                    Failure::Failed(error) => error,
                }) {
                    Ok(value) => match read_lines(&value) {
                        Ok(lines) => {
                            view.lines = lines;
                            view.error = None;
                        }
                        Err(error) => view.error = Some(error.to_string().into()),
                    },
                    Err(error) => view.error = Some(format!("{error:#}").into()),
                }
                cx.notify();
            })
            .log_err();
        }));
    }
}

impl HarnessView {
    /// Answers the question the session waits on with `choice`, naming the question by its full
    /// prompt, so an answer meant for it never lands on a later one.
    fn answer(&mut self, choice: String, window: &Window, cx: &mut Context<Self>) {
        let Some(question) = Harness::seats(cx)
            .get(&self.id)
            .and_then(|seat| seat.question.clone())
        else {
            return;
        };
        let arguments = json!({ "id": self.id, "choice": choice, "prompt": question.prompt });
        self.write(
            "session_answer",
            arguments,
            window,
            cx,
            move |view, _, _, _| {
                view.status = Some((format!("Answered {choice}").into(), Color::Success));
            },
        );
    }

    /// Sends the editor's text to the session as a new delivery, and empties the editor once the
    /// harness took it.
    fn send(&mut self, window: &Window, cx: &mut Context<Self>) {
        let text = self.send_editor.read(cx).text(cx);
        if text.trim().is_empty() {
            return;
        }
        let delivery = uuid::Uuid::new_v4().to_string();
        let arguments = json!({ "id": self.id, "text": text, "delivery": delivery });
        self.write(
            "session_send",
            arguments,
            window,
            cx,
            |view, value, window, cx| {
                let state = value.get("state").and_then(Value::as_str).unwrap_or("sent");
                let detail = value.get("detail").and_then(Value::as_str).unwrap_or("");
                let said = if detail.is_empty() {
                    format!("Sent: {state}")
                } else {
                    format!("Sent: {state} ({detail})")
                };
                view.status = Some((said.into(), Color::Success));
                view.send_editor
                    .update(cx, |editor, cx| editor.set_text("", window, cx));
            },
        );
    }

    /// Asks the harness for the commands that watch the session, and lists them.
    fn surface(&mut self, window: &Window, cx: &mut Context<Self>) {
        let arguments = json!({ "id": self.id });
        self.write(
            "session_surface_to_human",
            arguments,
            window,
            cx,
            |view, value, _, _| {
                view.views = value
                    .get("views")
                    .and_then(Value::as_array)
                    .map(|views| views.iter().filter_map(view_line).collect())
                    .unwrap_or_default();
                view.status = if view.views.is_empty() {
                    Some((
                        SharedString::new_static("The harness has no view of this session"),
                        Color::Muted,
                    ))
                } else {
                    None
                };
            },
        );
    }

    /// Starts a view's command line in a new terminal of the tab's workspace, typed into its shell
    /// as #684 types the Marley agent's, so the shell is left when the view ends (#690).
    fn open_view(&self, line: &SharedString, window: &Window, cx: &mut Context<Self>) {
        let Some(workspace) = self.workspace.upgrade() else {
            return;
        };
        let input = format!("{line}\n").into_bytes();
        workspace
            .update(cx, |workspace, cx| {
                crate::agents::start_in_terminal(
                    workspace,
                    None,
                    None,
                    Some(input),
                    None,
                    window,
                    cx,
                )
            })
            .detach_and_log_err(cx);
    }

    /// Calls the write verb `tool`, unless a write is under way, and hands an accepted value to
    /// `accepted`; a refusal shows with the harness's reason.
    fn write(
        &mut self,
        tool: &'static str,
        arguments: Value,
        window: &Window,
        cx: &mut Context<Self>,
        accepted: impl FnOnce(&mut Self, Value, &mut Window, &mut Context<Self>) + 'static,
    ) {
        if self.writing.is_some() {
            return;
        }
        let Some(server) = Harness::server(cx) else {
            self.status = Some((
                SharedString::new_static("The harness is not running"),
                Color::Error,
            ));
            cx.notify();
            return;
        };
        self.writing = Some(cx.spawn_in(window, async move |view, cx| {
            let receipt = call_write(&server, tool, arguments, cx).await;
            view.update_in(cx, |view, window, cx| {
                view.writing = None;
                match receipt {
                    Ok(value) => accepted(view, value, window, cx),
                    Err(reason) => {
                        view.status = Some((format!("Refused: {reason}").into(), Color::Error));
                    }
                }
                cx.notify();
            })
            .log_err();
        }));
    }

    /// The question, the line to send and the views, while the write verbs are on.
    fn render_controls(&self, cx: &Context<Self>) -> AnyElement {
        let colors = cx.theme().colors();
        let question = Harness::seats(cx)
            .get(&self.id)
            .and_then(|seat| seat.question.clone());
        v_flex()
            .flex_none()
            .gap_1()
            .p_2()
            .border_b_1()
            .border_color(colors.border_variant)
            .when_some(question, |this, question| {
                this.child(
                    h_flex()
                        .gap_2()
                        .flex_wrap()
                        .child(Label::new(shown_prompt(&question.prompt).to_string()))
                        .children(question.options.into_iter().enumerate().map(
                            |(index, option)| {
                                let choice = option.clone();
                                Button::new(("marley-harness-option", index), option)
                                    .style(ButtonStyle::Filled)
                                    .on_click(cx.listener(move |view, _, window, cx| {
                                        view.answer(choice.clone(), window, cx);
                                    }))
                            },
                        )),
                )
            })
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .px_2()
                            .py_0p5()
                            .rounded_md()
                            .border_1()
                            .border_color(colors.border)
                            .bg(colors.editor_background)
                            .child(self.send_editor.clone()),
                    )
                    .child(
                        Button::new("marley-harness-send", "Send")
                            .on_click(cx.listener(|view, _, window, cx| view.send(window, cx))),
                    )
                    .child(
                        Button::new("marley-harness-views", "Views")
                            .on_click(cx.listener(|view, _, window, cx| view.surface(window, cx))),
                    ),
            )
            .when_some(self.status.clone(), |this, (status, color)| {
                this.child(Label::new(status).color(color).size(LabelSize::Small))
            })
            .children(self.views.iter().enumerate().map(|(index, (kind, line))| {
                let copied = line.to_string();
                let opened = line.clone();
                // Open and Copy come before the line, which is cut at the tab's edge: a view's
                // command carries the root's path and a workspace id, longer than most tabs are
                // wide.
                h_flex()
                    .gap_2()
                    .child(
                        Label::new(kind.clone())
                            .color(Color::Muted)
                            .size(LabelSize::Small),
                    )
                    .child(
                        Button::new(("marley-harness-open", index), "Open")
                            .label_size(LabelSize::Small)
                            .on_click(cx.listener(move |view, _, window, cx| {
                                view.open_view(&opened, window, cx);
                            })),
                    )
                    .child(
                        Button::new(("marley-harness-copy", index), "Copy")
                            .label_size(LabelSize::Small)
                            .on_click(move |_, _, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(copied.clone()));
                            }),
                    )
                    .child(
                        div().flex_1().min_w_0().overflow_hidden().child(
                            Label::new(line.clone())
                                .buffer_font(cx)
                                .size(LabelSize::Small)
                                .single_line()
                                .truncate(),
                        ),
                    )
            }))
            .into_any_element()
    }
}

/// A view of `session_surface_to_human` as its kind and its command line, each argument quoted
/// where a shell would split it.
fn view_line(view: &Value) -> Option<(SharedString, SharedString)> {
    let kind = view.get("kind")?.as_str()?.to_string();
    let line = view
        .get("argv")?
        .as_array()?
        .iter()
        .filter_map(Value::as_str)
        .map(|argument| {
            if argument.is_empty() || argument.contains(|c: char| c.is_whitespace() || c == '\'') {
                format!("'{}'", argument.replace('\'', "'\\''"))
            } else {
                argument.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    Some((kind.into(), line.into()))
}

/// The lines of a `session_read` receipt, or what it refused.
fn read_lines(value: &Value) -> anyhow::Result<Vec<SharedString>> {
    let result = value
        .get("result")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if result != "accepted" {
        let reason = value
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or(result);
        anyhow::bail!("session_read was refused: {reason}");
    }
    let lines = value
        .pointer("/value/lines")
        .and_then(Value::as_array)
        .context("session_read gave no lines")?;
    Ok(lines
        .iter()
        .map(|line| SharedString::from(line.as_str().unwrap_or_default().to_string()))
        .collect())
}

impl Render for HarnessView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let background = cx.theme().colors().editor_background;
        let controls = if writes_on(cx) {
            Some(self.render_controls(cx))
        } else {
            None
        };
        v_flex()
            .id("marley-harness-view")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(background)
            .on_action(cx.listener(|view, _: &menu::Confirm, window, cx| view.send(window, cx)))
            .children(controls)
            .child(
                v_flex()
                    .id("marley-harness-lines")
                    .flex_1()
                    .overflow_y_scroll()
                    .p_2()
                    .when_some(self.error.clone(), |this, error| {
                        this.child(Label::new(error).color(Color::Error))
                    })
                    .children(
                        self.lines
                            .iter()
                            .map(|line| Label::new(line.clone()).buffer_font(cx).single_line()),
                    ),
            )
    }
}

impl Focusable for HarnessView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<()> for HarnessView {}

impl Item for HarnessView {
    type Event = ();

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        self.title.clone()
    }
}

/// `marley: open harness session`: a picker of the declared profiles, and the typed name.
fn open_session_picker(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    if !writes_on(cx) {
        return;
    }
    let delegate = OpenSessionDelegate {
        workspace: workspace.weak_handle(),
        profiles: Vec::new(),
        matches: Vec::new(),
        selected_index: 0,
    };
    let profiles_dir = Harness::profiles_dir(cx);
    workspace.toggle_modal(window, cx, |window, cx| {
        let picker = Picker::uniform_list(delegate, window, cx);
        if let Some(dir) = profiles_dir {
            let listed = cx
                .background_executor()
                .spawn(futures::future::lazy(move |_| profile_names(&dir)));
            cx.spawn_in(window, async move |picker, cx| {
                let profiles = listed.await;
                picker
                    .update_in(cx, |picker, window, cx| {
                        picker.delegate.profiles = profiles;
                        picker.refresh(window, cx);
                    })
                    .log_err();
            })
            .detach();
        }
        picker
    });
}

/// The names of the profiles declared in `dir`: each `NAME.json`, sorted.
fn profile_names(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            (path.extension()? == "json")
                .then(|| path.file_stem()?.to_str().map(str::to_string))
                .flatten()
        })
        .collect();
    names.sort();
    names
}

/// The profiles the picker offers, and the name typed.
struct OpenSessionDelegate {
    workspace: WeakEntity<Workspace>,
    profiles: Vec<String>,
    /// The profiles holding the query, then the query itself when it names none of them.
    matches: Vec<String>,
    selected_index: usize,
}

impl PickerDelegate for OpenSessionDelegate {
    type ListItem = ListItem;

    fn name() -> &'static str {
        "marley open harness session"
    }

    fn match_count(&self) -> usize {
        self.matches.len()
    }

    fn selected_index(&self) -> usize {
        self.selected_index
    }

    fn set_selected_index(&mut self, index: usize, _: &mut Window, _: &mut Context<Picker<Self>>) {
        self.selected_index = index;
    }

    fn placeholder_text(&self, _: &mut Window, _: &mut App) -> Arc<str> {
        "A profile to open a harness session from…".into()
    }

    fn update_matches(
        &mut self,
        query: String,
        _: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Task<()> {
        let query = query.trim().to_string();
        let lowered = query.to_lowercase();
        self.matches = self
            .profiles
            .iter()
            .filter(|profile| profile.to_lowercase().contains(&lowered))
            .cloned()
            .collect();
        if !query.is_empty() && !self.matches.contains(&query) {
            self.matches.push(query);
        }
        self.selected_index = 0;
        cx.notify();
        Task::ready(())
    }

    fn confirm(&mut self, _: bool, window: &mut Window, cx: &mut Context<Picker<Self>>) {
        let Some(profile) = self.matches.get(self.selected_index).cloned() else {
            return;
        };
        let workspace = self.workspace.clone();
        let server = Harness::server(cx);
        cx.spawn_in(window, async move |_, cx| {
            let server = server.context("the harness is not running")?;
            let request = uuid::Uuid::new_v4().to_string();
            let arguments = json!({ "profile": profile, "request": request });
            let value = call_write(&server, "session_open", arguments, cx)
                .await
                .map_err(|reason| anyhow::anyhow!("{reason}"))?;
            let id = value
                .get("id")
                .and_then(Value::as_str)
                .context("session_open gave no id")?
                .to_string();
            workspace.update_in(cx, |workspace, window, cx| {
                open_in_home(workspace.weak_handle(), id, window, cx);
            })
        })
        .detach_and_prompt_err("Could not open the session", window, cx, |_, _, _| None);
        self.dismissed(window, cx);
    }

    fn dismissed(&mut self, _: &mut Window, cx: &mut Context<Picker<Self>>) {
        cx.emit(DismissEvent);
    }

    fn render_match(
        &self,
        index: usize,
        selected: bool,
        _: &mut Window,
        _: &mut Context<Picker<Self>>,
    ) -> Option<Self::ListItem> {
        let profile = self.matches.get(index)?;
        Some(
            ListItem::new(index)
                .inset(true)
                .spacing(ListItemSpacing::Sparse)
                .toggle_state(selected)
                .child(Label::new(profile.clone())),
        )
    }
}
