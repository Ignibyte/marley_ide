//! Marley as Claude Code's IDE (#653).
//!
//! With `marley.claude_code_ide` on, and Claude Code's IDE link on for the installed version
//! (#648's table, or allowed), each local project gets an IDE server of its own
//! ([`marley_mcp::ide_transport`]), a lock file in Claude Code's `ide` folder, and its port in its
//! new terminals' environment ([`marley_terminal::ide`]), so a `claude` started there links to
//! that project's Marley. The link carries the last file editor's selection, the diagnostics of
//! Zed's language servers, the project's folders and its open editors, and send selection's
//! mention. Each client's own version decides whether it gets the selection and the mention.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use editor::{Editor, EditorEvent, ToPoint as _};
use futures::StreamExt as _;
use futures::channel::mpsc;
use gpui::{
    App, AppContext as _, Context, Entity, EntityId, Global, Subscription, Task, WeakEntity,
};
use language::{Anchor, Buffer, PointUtf16};
use lsp::{DiagnosticSeverity, NumberOrString};
use marley_agent::AgentKind;
use marley_agent::versions::{
    self, CLAUDE_IDE_CONNECTION, CLAUDE_IDE_MENTION, CLAUDE_IDE_SELECTION, Found, Integration,
};
use marley_mcp::ide::{self, Diagnostic, EditorSelection, FileDiagnostics, Parts, Place, Severity};
use marley_mcp::ide_transport::{self, ConnectionId, IdeCall, IdeMessage, IdeServer};
use project::{Project, ProjectPath};
use settings::{Settings as _, SettingsStore};
use workspace::Workspace;
use workspace::item::ItemHandle;

use crate::MarleySettings;
use crate::claude_plugin::ClaudePlugin;

/// The rows of #648's table the link rests on.
pub(crate) const ROWS: [&Integration; 3] = [
    &CLAUDE_IDE_CONNECTION,
    &CLAUDE_IDE_SELECTION,
    &CLAUDE_IDE_MENTION,
];

/// How long a selection rests before it is sent.
const SETTLE: Duration = Duration::from_millis(100);

/// The most files one `getDiagnostics` without a file answers.
const MAX_DIAGNOSTIC_FILES: usize = 200;

/// `marley.claude_code_ide`: whether Marley serves Claude Code's IDE link.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ClaudeCodeIde {
    /// No server, no lock file, no variables: the default.
    #[default]
    Off,
    /// A server for each local project.
    On,
}

impl ClaudeCodeIde {
    /// The switch as the settings hold it: off unless set.
    pub(crate) fn from_content(marley: Option<&settings::MarleySettingsContent>) -> Self {
        match marley.and_then(|marley| marley.claude_code_ide) {
            Some(true) => Self::On,
            Some(false) | None => Self::Off,
        }
    }
}

/// What the clients' threads judge a version by: the allowed rows and the installed version.
#[derive(Default)]
struct JudgeState {
    allowed: BTreeSet<String>,
    installed: Option<Found>,
}

/// What a client's thread hands the main thread.
enum Incoming {
    Connected {
        connection: ConnectionId,
        version: Option<String>,
    },
    Identified {
        connection: ConnectionId,
        chain: Vec<(u32, u64)>,
    },
    Closed {
        connection: ConnectionId,
    },
    Call(IdeCall),
}

/// One client of a project's server.
#[derive(Default)]
struct Client {
    version: Option<String>,
    /// The terminal view its Claude Code runs in, once it named its process.
    terminal: Option<EntityId>,
}

/// A project Marley serves the link for.
struct Served {
    server: IdeServer,
    lock: PathBuf,
    folders: Vec<String>,
    workspace: WeakEntity<Workspace>,
    project: WeakEntity<Project>,
    clients: HashMap<ConnectionId, Client>,
    /// The last file editor that was the workspace's active item, and its events.
    editor: Option<(WeakEntity<Editor>, Subscription)>,
    latest: Option<EditorSelection>,
    _project_events: Subscription,
}

/// The link's state: the local projects' workspaces, and the projects served.
struct ClaudeIde {
    /// Each local workspace and its project's entity id.
    workspaces: HashMap<EntityId, (WeakEntity<Workspace>, EntityId)>,
    served: HashMap<EntityId, Served>,
    /// The wait before each served project's next selection is sent.
    settling: HashMap<EntityId, Task<()>>,
    judge: Arc<RwLock<JudgeState>>,
    incoming: mpsc::UnboundedSender<(EntityId, Incoming)>,
}

impl Global for ClaudeIde {}

/// Follows the switch, the version verdicts and the workspaces; [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    let (incoming, mut received) = mpsc::unbounded::<(EntityId, Incoming)>();
    cx.set_global(ClaudeIde {
        workspaces: HashMap::new(),
        served: HashMap::new(),
        settling: HashMap::new(),
        judge: Arc::default(),
        incoming,
    });
    cx.spawn(async move |cx| {
        while let Some((project, incoming)) = received.next().await {
            cx.update(|cx| take(project, incoming, cx));
        }
    })
    .detach();
    cx.observe_global::<SettingsStore>(reconcile).detach();
    crate::agent_versions::observe(cx, reconcile).detach();
    cx.on_app_quit(|cx| {
        let served = std::mem::take(&mut cx.global_mut::<ClaudeIde>().served);
        let mut locks = Vec::new();
        for (project, served) in served {
            marley_terminal::ide::set_port(project.as_u64(), None);
            locks.push(served.lock.clone());
            drop(served);
        }
        cx.background_spawn(futures::future::lazy(move |_| {
            for lock in &locks {
                remove_lock(lock);
            }
        }))
    })
    .detach();
    cx.observe_new(
        |workspace: &mut Workspace, _, cx: &mut Context<Workspace>| {
            let project = workspace.project().clone();
            if !project.read(cx).is_local() {
                return;
            }
            let project_id = project.entity_id();
            let workspace_id = cx.entity_id();
            let weak = cx.weak_entity();
            let _previous = cx
                .global_mut::<ClaudeIde>()
                .workspaces
                .insert(workspace_id, (weak, project_id));
            cx.subscribe_self(move |workspace, event: &workspace::Event, cx| {
                if matches!(event, workspace::Event::ActiveItemChanged)
                    && let Some(editor) = file_editor(workspace.active_item(cx), cx)
                {
                    follow_editor(project_id, &editor, cx);
                }
            })
            .detach();
            cx.on_release(move |_, cx| {
                let _gone = cx
                    .global_mut::<ClaudeIde>()
                    .workspaces
                    .remove(&workspace_id);
                cx.defer(reconcile);
            })
            .detach();
            // The workspace is being built, so the reconcile that reads it waits for the update
            // to end; it still runs before a restored terminal builds its environment.
            cx.defer(reconcile);
        },
    )
    .detach();
}

/// Starts a server for each local project while the link should run, and stops every other.
fn reconcile(cx: &mut App) {
    let wanted = MarleySettings::get_global(cx).claude_code_ide == ClaudeCodeIde::On
        && crate::agent_versions::is_on(&CLAUDE_IDE_CONNECTION, cx);
    {
        let allowed = MarleySettings::get_global(cx)
            .allow_untested_versions
            .clone();
        let installed = crate::agent_versions::found(AgentKind::Claude, cx);
        let ide = cx.global::<ClaudeIde>();
        let mut judge = ide
            .judge
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        judge.allowed = allowed;
        judge.installed = installed;
    }
    let ide = cx.global::<ClaudeIde>();
    let live: HashMap<EntityId, Entity<Workspace>> = ide
        .workspaces
        .values()
        .filter_map(|(workspace, project)| Some((*project, workspace.upgrade()?)))
        .collect();
    let stopping: Vec<EntityId> = ide
        .served
        .keys()
        .filter(|project| !wanted || !live.contains_key(project))
        .copied()
        .collect();
    let starting: Vec<(EntityId, Entity<Workspace>)> = if wanted {
        live.into_iter()
            .filter(|(project, _)| !ide.served.contains_key(project))
            .collect()
    } else {
        Vec::new()
    };
    for project in stopping {
        stop(project, cx);
    }
    for (project, workspace) in starting {
        start(project, &workspace, cx);
    }
}

/// Claude Code's `ide` folder: `CLAUDE_CONFIG_DIR`'s, else `~/.claude`'s.
fn ide_folder(cx: &App) -> Option<PathBuf> {
    cx.try_global::<ClaudePlugin>()
        .map(|plugin| plugin.config_dir.join("ide"))
}

/// The project's visible folders, as the lock file names them.
fn folders_of(project: &Entity<Project>, cx: &App) -> Vec<String> {
    project
        .read(cx)
        .visible_worktrees(cx)
        .map(|worktree| worktree.read(cx).abs_path().to_string_lossy().into_owned())
        .collect()
}

fn start(project_id: EntityId, workspace: &Entity<Workspace>, cx: &mut App) {
    let Some(folder) = ide_folder(cx) else {
        return;
    };
    let project = workspace.read(cx).project().clone();
    let judge_state = Arc::clone(&cx.global::<ClaudeIde>().judge);
    let judge: ide_transport::Judge = Arc::new(move |version| {
        parts_for(
            version,
            &judge_state
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
    });
    let sender = cx.global::<ClaudeIde>().incoming.clone();
    let handler: ide_transport::IdeHandler = Arc::new(move |message| {
        let incoming = match message {
            IdeMessage::Connected {
                connection,
                version,
            } => Incoming::Connected {
                connection,
                version,
            },
            IdeMessage::Identified { connection, pid } => Incoming::Identified {
                connection,
                chain: crate::agent_reports::process_chain(pid),
            },
            IdeMessage::Closed { connection } => Incoming::Closed { connection },
            IdeMessage::Call(call) => Incoming::Call(call),
        };
        if sender.unbounded_send((project_id, incoming)).is_err() {
            log::debug!("claude ide: a client's message came as Marley shut down");
        }
    });
    let server = match ide_transport::spawn_ide(judge, handler) {
        Ok(server) => server,
        Err(error) => {
            log::warn!("claude ide: no server for a project: {error}");
            return;
        }
    };
    let port = server.port();
    marley_terminal::ide::set_port(project_id.as_u64(), Some(port));
    let folders = folders_of(&project, cx);
    let lock = folder.join(ide::lock_name(port));
    write_lock(
        folder,
        port,
        ide::lock_json(std::process::id(), &folders, server.token()),
        cx,
    );
    log::info!(
        "claude ide: serving {} on port {port}",
        folders.first().map_or("a project", String::as_str)
    );
    let project_events = cx.subscribe(&project, move |project, event: &project::Event, cx| {
        if matches!(
            event,
            project::Event::WorktreeAdded(_) | project::Event::WorktreeRemoved(_)
        ) {
            refolder(project_id, &project, cx);
        }
    });
    let _previous = cx.global_mut::<ClaudeIde>().served.insert(
        project_id,
        Served {
            server,
            lock,
            folders,
            workspace: workspace.downgrade(),
            project: project.downgrade(),
            clients: HashMap::new(),
            editor: None,
            latest: None,
            _project_events: project_events,
        },
    );
    let active = workspace.read(cx).active_item(cx);
    if let Some(editor) = file_editor(active, cx) {
        follow_editor(project_id, &editor, cx);
    }
}

fn stop(project: EntityId, cx: &mut App) {
    let ide = cx.global_mut::<ClaudeIde>();
    let _settling = ide.settling.remove(&project);
    let Some(served) = ide.served.remove(&project) else {
        return;
    };
    marley_terminal::ide::set_port(project.as_u64(), None);
    let lock = served.lock.clone();
    log::info!("claude ide: stopped serving port {}", served.server.port());
    drop(served);
    cx.background_spawn(futures::future::lazy(move |_| remove_lock(&lock)))
        .detach();
}

/// Writes the lock file again when the project's folders change.
fn refolder(project_id: EntityId, project: &Entity<Project>, cx: &mut App) {
    let folders = folders_of(project, cx);
    let Some(folder) = ide_folder(cx) else {
        return;
    };
    let Some(served) = cx.global_mut::<ClaudeIde>().served.get_mut(&project_id) else {
        return;
    };
    if served.folders == folders {
        return;
    }
    let json = ide::lock_json(std::process::id(), &folders, served.server.token());
    let port = served.server.port();
    served.folders = folders;
    write_lock(folder, port, json, cx);
}

/// Removes the lock files a Marley left behind, then writes `json` as the lock file for `port`,
/// through a temporary name, so Claude Code never reads half of it. Off the main thread.
fn write_lock(folder: PathBuf, port: u16, json: String, cx: &App) {
    cx.background_spawn(futures::future::lazy(move |_| {
        if let Err(error) = write_lock_in(&folder, port, &json) {
            log::warn!(
                "claude ide: writing the lock file in {}: {error}",
                folder.display()
            );
        }
    }))
    .detach();
}

fn write_lock_in(folder: &Path, port: u16, json: &str) -> std::io::Result<()> {
    if !folder.exists() {
        use std::os::unix::fs::DirBuilderExt as _;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(folder)?;
    }
    sweep_in(folder);
    let name = ide::lock_name(port);
    let temporary = format!(".{name}.tmp");
    marley_mcp::discovery::write_endpoint_file_in(folder, &temporary, json)?;
    std::fs::rename(folder.join(&temporary), folder.join(&name))
}

/// Removes each lock file in `folder` that names Marley and a process that has ended.
fn sweep_in(folder: &Path) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    for path in entries.flatten().map(|entry| entry.path()) {
        if path.extension().is_none_or(|extension| extension != "lock") {
            continue;
        }
        let stale = std::fs::read_to_string(&path)
            .is_ok_and(|json| ide::is_stale_marley_lock(&json, process_alive));
        if stale {
            log::info!(
                "claude ide: removing {}, left by a Marley that ended",
                path.display()
            );
            remove_lock(&path);
        }
    }
}

/// Whether process `pid` runs; true where `/proc` cannot say, so nothing is removed on a guess.
fn process_alive(pid: u32) -> bool {
    !Path::new("/proc/self").exists() || Path::new(&format!("/proc/{pid}")).exists()
}

fn remove_lock(lock: &Path) {
    if let Err(error) = std::fs::remove_file(lock)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        log::warn!("claude ide: {} stays: {error}", lock.display());
    }
}

/// What a client of `version` may be sent; one that named none is judged by the installed
/// version.
fn parts_for(version: Option<&str>, state: &JudgeState) -> Parts {
    let found = version.map_or_else(
        || state.installed.clone(),
        |version| {
            Some(match versions::parse_version(version) {
                Ok(version) => Found::Version(version),
                Err(why) => Found::Unreadable(why),
            })
        },
    );
    let on = |row: &Integration| {
        versions::verdict(row, found.as_ref(), state.allowed.contains(row.id)).is_on()
    };
    Parts {
        selection: on(&CLAUDE_IDE_SELECTION),
        mention: on(&CLAUDE_IDE_MENTION),
    }
}

fn client_parts(client: &Client, cx: &App) -> Parts {
    let ide = cx.global::<ClaudeIde>();
    parts_for(
        client.version.as_deref(),
        &ide.judge
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
    )
}

/// Takes one client's message on the main thread.
fn take(project: EntityId, incoming: Incoming, cx: &mut App) {
    match incoming {
        Incoming::Connected {
            connection,
            version,
        } => {
            let ide = cx.global_mut::<ClaudeIde>();
            let Some(served) = ide.served.get_mut(&project) else {
                return;
            };
            served
                .clients
                .entry(connection)
                .or_default()
                .version
                .clone_from(&version);
            let parts = parts_for(
                version.as_deref(),
                &ide.judge
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            );
            log::info!(
                "claude ide: client {} on port {}: selection {}, mention {}",
                version.as_deref().unwrap_or("of no stated version"),
                served.server.port(),
                on_off(parts.selection),
                on_off(parts.mention)
            );
        }
        Incoming::Identified { connection, chain } => {
            let terminal = crate::agent_reports::terminal_on_chain(&chain, cx)
                .map(|(_, view)| view.entity_id());
            if let Some(client) = cx
                .global_mut::<ClaudeIde>()
                .served
                .get_mut(&project)
                .and_then(|served| served.clients.get_mut(&connection))
            {
                client.terminal = terminal;
            }
            send_selection(project, Some(connection), cx);
        }
        Incoming::Closed { connection } => {
            if let Some(served) = cx.global_mut::<ClaudeIde>().served.get_mut(&project) {
                let _gone = served.clients.remove(&connection);
            }
        }
        Incoming::Call(call) => answer(project, call, cx),
    }
}

const fn on_off(on: bool) -> &'static str {
    if on { "on" } else { "off" }
}

/// `item` as a file editor: an editor of one file on this machine.
fn file_editor(item: Option<Box<dyn ItemHandle>>, cx: &App) -> Option<Entity<Editor>> {
    let editor = item?.act_as::<Editor>(cx)?;
    let buffer = editor.read(cx).buffer().read(cx).as_singleton()?;
    buffer
        .read(cx)
        .file()?
        .as_local()
        .is_some()
        .then_some(editor)
}

/// Makes `editor` the served project's last file editor and sends its selection once it rests.
fn follow_editor(project: EntityId, editor: &Entity<Editor>, cx: &mut App) {
    let same = cx.global::<ClaudeIde>().served.get(&project).map(|served| {
        served
            .editor
            .as_ref()
            .is_some_and(|(followed, _)| followed.entity_id() == editor.entity_id())
    });
    match same {
        None => return,
        Some(true) => {}
        Some(false) => {
            let events = cx.subscribe(editor, move |_, event: &EditorEvent, cx| {
                if matches!(event, EditorEvent::SelectionsChanged { .. }) {
                    settle_then_send(project, cx);
                }
            });
            if let Some(served) = cx.global_mut::<ClaudeIde>().served.get_mut(&project) {
                served.editor = Some((editor.downgrade(), events));
            }
        }
    }
    settle_then_send(project, cx);
}

/// Sends the selection after [`SETTLE`], a newer change starting the wait again.
fn settle_then_send(project: EntityId, cx: &mut App) {
    if !cx.global::<ClaudeIde>().served.contains_key(&project) {
        return;
    }
    let sending = cx.spawn(async move |cx| {
        cx.background_executor().timer(SETTLE).await;
        cx.update(|cx| send_selection(project, None, cx));
    });
    let _replaced = cx
        .global_mut::<ClaudeIde>()
        .settling
        .insert(project, sending);
}

/// The selection of the project's last file editor, if it is still open.
fn current_selection(project: EntityId, cx: &App) -> Option<EditorSelection> {
    let editor = cx
        .global::<ClaudeIde>()
        .served
        .get(&project)?
        .editor
        .as_ref()?
        .0
        .upgrade()?;
    editor_selection(&editor, cx)
}

/// `editor`'s newest selection, as Claude Code reads it.
fn editor_selection(editor: &Entity<Editor>, cx: &App) -> Option<EditorSelection> {
    let editor = editor.read(cx);
    let snapshot = editor.buffer().read(cx).snapshot(cx);
    let selection = editor.selections.newest_anchor();
    let range = selection.start.to_point(&snapshot)..selection.end.to_point(&snapshot);
    let (buffer, range) = snapshot.range_to_buffer_range(range).or_else(|| {
        let (buffer, point) =
            snapshot.point_to_buffer_point(selection.head().to_point(&snapshot))?;
        Some((buffer, point..point))
    })?;
    let path = buffer.file()?.as_local()?.abs_path(cx);
    let text: String = buffer.text_for_range(range.clone()).collect();
    let place = |point: PointUtf16| Place {
        line: point.row,
        character: point.column,
    };
    Some(EditorSelection {
        path: path.to_string_lossy().into_owned(),
        text,
        start: place(buffer.point_to_point_utf16(range.start)),
        end: place(buffer.point_to_point_utf16(range.end)),
    })
}

/// Sends the last file editor's selection to the project's clients that take it, unless it is the
/// one sent last, or to `only`, which has not had it.
fn send_selection(project: EntityId, only: Option<ConnectionId>, cx: &mut App) {
    let Some(selection) = current_selection(project, cx) else {
        return;
    };
    let message = ide::selection_changed(&selection);
    let ide = cx.global::<ClaudeIde>();
    let Some(served) = ide.served.get(&project) else {
        return;
    };
    // The workspace says its active item changed far more often than the item does.
    if only.is_none() && served.latest.as_ref() == Some(&selection) {
        return;
    }
    for (connection, client) in &served.clients {
        if only.is_some_and(|only| only != *connection) || !client_parts(client, cx).selection {
            continue;
        }
        if !served.server.notify(*connection, message.clone()) {
            log::debug!("claude ide: client {connection} left before its selection");
        }
    }
    if let Some(served) = cx.global_mut::<ClaudeIde>().served.get_mut(&project) {
        served.latest = Some(selection);
    }
}

/// Mentions `path`'s `lines`, from 1, to the Claude Code running in the terminal `view` over its
/// link; false when no link of that terminal takes mentions, so the caller types the reference.
pub(crate) fn mention(view: EntityId, path: &Path, lines: Option<(u32, u32)>, cx: &App) -> bool {
    let Some(ide) = cx.try_global::<ClaudeIde>() else {
        return false;
    };
    let message = ide::at_mentioned(
        &path.to_string_lossy(),
        lines.map(|(first, last)| (first.saturating_sub(1), last.saturating_sub(1))),
    );
    ide.served.values().any(|served| {
        served.clients.iter().any(|(connection, client)| {
            client.terminal == Some(view)
                && client_parts(client, cx).mention
                && served.server.notify(*connection, message.clone())
        })
    })
}

/// Answers a client's tool call.
fn answer(project: EntityId, call: IdeCall, cx: &mut App) {
    let Some(served) = cx.global::<ClaudeIde>().served.get(&project) else {
        call.answer(Err("Marley stopped serving this project".to_string()));
        return;
    };
    match call.tool.clone() {
        ide::Tool::WorkspaceFolders => {
            let answer = ide::folders_answer(&served.folders);
            call.answer(Ok(answer));
        }
        ide::Tool::CurrentSelection => {
            let selection = current_selection(project, cx);
            call.answer(Ok(ide::selection_answer(selection.as_ref())));
        }
        ide::Tool::LatestSelection => {
            let selection = served
                .latest
                .clone()
                .or_else(|| current_selection(project, cx));
            call.answer(Ok(ide::selection_answer(selection.as_ref())));
        }
        ide::Tool::OpenEditors => {
            let editors = open_editors(served, cx);
            call.answer(Ok(ide::open_editors_answer(&editors)));
        }
        ide::Tool::Diagnostics { uri } => match served.project.upgrade() {
            Some(project) => diagnostics(&project, uri.as_deref(), call, cx),
            None => call.answer(Err("the project closed".to_string())),
        },
    }
}

fn open_editors(served: &Served, cx: &App) -> Vec<ide::OpenEditor> {
    let Some(workspace) = served.workspace.upgrade() else {
        return Vec::new();
    };
    let active = served.editor.as_ref().map(|(editor, _)| editor.entity_id());
    workspace
        .read(cx)
        .items_of_type::<Editor>(cx)
        .filter_map(|editor| {
            let buffer = editor.read(cx).buffer().read(cx).as_singleton()?;
            let buffer = buffer.read(cx);
            let path = buffer.file()?.as_local()?.abs_path(cx);
            Some(ide::OpenEditor {
                path: path.to_string_lossy().into_owned(),
                active: active == Some(editor.entity_id()),
                dirty: buffer.is_dirty(),
            })
        })
        .collect()
}

/// Answers `getDiagnostics`: the file `uri` names, or every file of the project with errors or
/// warnings, each opened and read for its diagnostics' primary entries.
fn diagnostics(project: &Entity<Project>, uri: Option<&str>, call: IdeCall, cx: &mut App) {
    let paths: Vec<ProjectPath> = if let Some(uri) = uri {
        let Some(path) = ide::path_of_uri(uri) else {
            call.answer(Err(format!("{uri} names no file on this machine")));
            return;
        };
        let Some(project_path) = project.read(cx).find_project_path(&path, cx) else {
            call.answer(Err(format!("{path} is not in this project")));
            return;
        };
        vec![project_path]
    } else {
        let mut paths: Vec<ProjectPath> = Vec::new();
        for (path, _, summary) in project.read(cx).diagnostic_summaries(false, cx) {
            if summary.error_count + summary.warning_count > 0 && !paths.contains(&path) {
                paths.push(path);
            }
        }
        paths.truncate(MAX_DIAGNOSTIC_FILES);
        paths
    };
    let opening: Vec<Task<anyhow::Result<Entity<Buffer>>>> = paths
        .into_iter()
        .map(|path| project.update(cx, |project, cx| project.open_buffer(path, cx)))
        .collect();
    let one_file = uri.is_some();
    cx.spawn(async move |cx| {
        let mut files = Vec::new();
        for open in opening {
            let buffer = match open.await {
                Ok(buffer) => buffer,
                Err(error) => {
                    log::debug!("claude ide: opening a file for its diagnostics: {error}");
                    continue;
                }
            };
            let file = cx.update(|cx| file_diagnostics(&buffer, cx));
            if let Some(file) = file.filter(|file| one_file || !file.diagnostics.is_empty()) {
                files.push(file);
            }
        }
        call.answer(Ok(ide::diagnostics_answer(&files)));
    })
    .detach();
}

/// `buffer`'s diagnostics, one per group, its primary entry.
fn file_diagnostics(buffer: &Entity<Buffer>, cx: &App) -> Option<FileDiagnostics> {
    let buffer = buffer.read(cx);
    let path = buffer.file()?.as_local()?.abs_path(cx);
    let snapshot = buffer.snapshot();
    let place = |anchor: &Anchor| {
        let point: PointUtf16 = snapshot.summary_for_anchor(anchor);
        Place {
            line: point.row,
            character: point.column,
        }
    };
    let diagnostics = snapshot
        .diagnostic_groups(None)
        .into_iter()
        .filter_map(|(_, group)| {
            let entry = group.entries.get(group.primary_ix)?;
            let severity = match entry.diagnostic.severity {
                DiagnosticSeverity::ERROR => Severity::Error,
                DiagnosticSeverity::WARNING => Severity::Warning,
                DiagnosticSeverity::INFORMATION => Severity::Information,
                _ => Severity::Hint,
            };
            Some(Diagnostic {
                message: entry.diagnostic.message.as_str().to_string(),
                severity,
                start: place(&entry.range.start),
                end: place(&entry.range.end),
                source: entry.diagnostic.source.clone(),
                code: entry.diagnostic.code.as_ref().map(|code| match code {
                    NumberOrString::Number(number) => number.to_string(),
                    NumberOrString::String(text) => text.clone(),
                }),
            })
        })
        .collect();
    Some(FileDiagnostics {
        path: path.to_string_lossy().into_owned(),
        diagnostics,
    })
}
