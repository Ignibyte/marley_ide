//! The rail: the Marley layout's sidebar. Each project group, and under it the terminals in that
//! group's center panes, with the agent CLIs running in them, and the group's agent threads. It
//! implements Zed's `workspace::Sidebar`, so the `MultiWorkspace` keeps the resize handle, open
//! state, persistence and the toggle actions; the rows and the one selected row come from
//! `marley_rail`.

use std::borrow::Cow;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::hash::{Hash as _, Hasher as _};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use acp_thread::{AcpThread, AcpThreadEvent, SelectedPermissionOutcome};
use agent_client_protocol::schema::v1 as acp;
use agent_ui::thread_metadata_store::{ThreadId, ThreadMetadata, ThreadMetadataStore};
use agent_ui::thread_worktree_archive::{self, RootPlan};
use agent_ui::threads_archive_view::fuzzy_match_positions;
use agent_ui::{Agent, AgentPanel, AgentPanelEvent, AgentThreadSource, ConversationView};
use anyhow::Context as _;
use editor::{Editor, EditorEvent};
use fs::Fs;
use git_ui::branch_diff::BranchDiff;
use gpui::{
    Anchor, AnyElement, AnyView, App, ClickEvent, ClipboardItem, Context, DismissEvent, Div,
    ElementId, Entity, EntityId, EventEmitter, FocusHandle, Focusable, Hsla, Image, MouseButton,
    MouseDownEvent, Pixels, Point, PromptLevel, Render, RenderImage, Stateful, Subscription, Task,
    WeakEntity, Window, anchored, deferred, img, px,
};
use marley_agent::risk::{self, Action, Chip, ChipKind, ChipSource, ToolClass};
use marley_agent::route::{self, Route, RouteMark, RouteSource};
use marley_agent::{AgentKind, WAITING_AFTER, claude_events, codex_events};
use marley_browser::consequence::Class;
use marley_browser::ports::{Service, Stopped};
use marley_mcp::redact::Redactor;
use marley_rail::{
    BrowserRow, BrowserSnapshot, CommandSnapshot, DriftSnapshot, Focus, InboxEntry, InboxKind,
    PortContainer, PortRow, PortService, PortSnapshot, ProjectRow, ProjectSnapshot, RailSnapshot,
    Reporting, Row, RunningError, Selection, SwitcherRow, TerminalAgent, TerminalRow,
    TerminalSnapshot, ThreadRow, ThreadSnapshot, ThreadStatus, TurnSnapshot, WorktreeRow,
    WorktreeSnapshot,
};
use marley_system_one::reading::{Reading, Signal};
use marley_system_one::{INBOX_RISK, QUESTION_ROUTE};
use menu::{
    Cancel, Confirm, SelectChild, SelectFirst, SelectLast, SelectNext, SelectParent, SelectPrevious,
};
use project::git_store::{GitStoreEvent, Repository, RepositoryEvent};
use project::{
    AgentId, AgentServerStore, AgentServersUpdated, Project, ProjectGroupKey, UpdatedEntriesSet,
    WorktreeId,
};
use recent_projects::sidebar_recent_projects::SidebarRecentProjects;
use settings::{Settings as _, SystemOneMode};
use terminal::Terminal;
use terminal_view::{RenameTerminal, TerminalView, terminal_panel::TerminalPanel};
use ui::{
    AgentThreadStatus, CommonAnimationExt as _, ContextMenu, ContextMenuEntry, DiffStat,
    Disclosure, Divider, HighlightedLabel, Icon, IconButton, IconName, IconSize, Indicator,
    KeyBinding, Label, LabelSize, PopoverMenu, PopoverMenuHandle, ThreadItem, Tooltip, prelude::*,
    right_click_menu, utils::platform_title_bar_height,
};
use util::ResultExt as _;
use util::path_list::PathList;
use util::shell::ShellKind;
use uuid::Uuid;
use workspace::{
    MultiWorkspace, MultiWorkspaceEvent, OpenMode, ProjectGroup, RemovalIntent, SaveIntent,
    Sidebar, SidebarEvent, SidebarSide, Toast, Workspace, WorkspaceId,
    item::{Item as _, ItemEvent},
    notifications::{DetachAndPromptErr as _, NotificationId},
};
use zed_actions::agents_sidebar::FocusSidebarFilter;

use crate::agent_events::{self, AgentEvents};
use crate::agents::{self, AgentIcon};
use crate::browser::{BrowserEvent, BrowserHub, BrowserView};
use crate::github::{self, PullRequest, PullRequestState};
use crate::groups;
use crate::harness::{Connection, Harness, QuotaWindow, Runtime, Signals, StateSource};
use crate::ports::{self, ContainerStop, Ports, ProjectListener, ServiceAction, Stop};
use crate::rusty::{self, brain::BrainView};
use crate::system_one::{self, Asking};
use crate::turns::Turns;
use crate::worktree_git::{self, BranchEnd, Drift, MergeOwner};
use crate::{MarleySettings, ToggleBrainView, browser, launch, worktree_agents};

#[path = "rail_order.rs"]
mod order;
#[path = "rail_switcher.rs"]
mod switcher;

use order::{DraggedRailHeader, DraggedRailRow, Hold, SavedOrder};
use switcher::{RailSwitcher, SwitcherEntry, SwitcherEvent};

const DEFAULT_WIDTH: Pixels = px(260.);
const MIN_WIDTH: Pixels = px(180.);
const MAX_WIDTH: Pixels = px(600.);

/// How long a repository's drift run waits after it is scheduled (#560): an agent commits often.
const DRIFT_DEBOUNCE: Duration = Duration::from_secs(1);

/// How long a project's pull request state is kept before `gh` is asked again, while the window
/// is active (#531).
const PULL_REQUEST_EVERY: Duration = Duration::from_mins(2);

/// How long a worktree's Review waits for the window to open its workspace (#511).
const REVIEW_WAIT: Duration = Duration::from_secs(30);

/// Reads the command a terminal's foreground process runs. Production asks the PTY; a test's
/// display-only terminal has no process, so tests hand in their own.
type ForegroundCommand = fn(&Entity<Terminal>, &App) -> Option<String>;

/// Zed's own sidebar and whether it was open, kept by the rail that replaced it.
pub type KeptSidebar = (Entity<sidebar::Sidebar>, bool);

/// The Marley layout's sidebar: each project group, with the terminals in its center panes and
/// its agent threads under it.
pub struct Rail {
    multi_workspace: WeakEntity<MultiWorkspace>,
    focus_handle: FocusHandle,
    width: Pixels,
    /// Whether `width` came from the user, which is when it is saved.
    width_set_by_user: bool,
    /// Whether the user closed the rail, which the window's saved state keeps.
    closed: bool,
    /// Whether this rail keeps the port scan running, which it does while it shows (#521).
    watching_ports: bool,
    /// The row the keyboard is on while the rail holds focus.
    cursor: Option<Selection>,
    /// The filter's field, under the header.
    filter_editor: Entity<Editor>,
    /// When each terminal and thread last took the window's focus, as a count of the changes of
    /// the window's row: the switcher lists the highest first.
    shown_at: HashMap<Selection, u64>,
    shown_count: u64,
    /// The terminal or thread row that held the window's focus at the last rebuild.
    window_row: Option<Selection>,
    /// The switcher, while it is open.
    switcher: Option<OpenSwitcher>,
    foreground_command: ForegroundCommand,
    /// When each terminal last wrote output, on the executor's clock.
    terminal_output: HashMap<EntityId, Instant>,
    /// Per agent terminal: a refresh due once its output has been quiet for `WAITING_AFTER`,
    /// replaced on each output.
    quiet_timers: HashMap<EntityId, Task<()>>,
    /// While a Claude Code seat works, a refresh due when a row's `no update in N m` next changes
    /// (#547).
    minute_timer: Option<Task<()>>,
    /// When the rail first saw each inbox entry, by its key, on the executor's clock (#508).
    inbox_seen: HashMap<String, Instant>,
    /// While the inbox shows, a refresh every half minute, so its ages move.
    inbox_timer: Option<Task<()>>,
    /// What the inbox's risk use keeps of each tool entry, by its key (#568).
    risk: HashMap<String, Risk>,
    /// What the question route keeps of each tool entry, by its key (#570).
    routes: HashMap<String, RouteState>,
    /// The terminals whose turns are listed under their rows, by the rail's terminal id (#509).
    turns_open: HashSet<u64>,
    /// What each listed worktree's branch would meet merging its base, by its folder (#560).
    drift: HashMap<String, DriftState>,
    /// Per repository, by its main checkout: the drift's run, pending or going.
    drift_runs: HashMap<PathBuf, Task<()>>,
    /// The repositories whose git refused `merge-tree --write-tree`, logged once.
    drift_unsupported: HashSet<PathBuf>,
    /// Each project's changed lines and pull request, by its folder (#531).
    project_git: HashMap<PathBuf, ProjectGitState>,
    /// Per project folder: its read, pending or going.
    project_git_runs: HashMap<PathBuf, Task<()>>,
    /// When a tracked file last changed, until a read takes it in.
    git_edited: Option<Instant>,
    /// The folders whose `gh` failed, logged once.
    github_failed: HashSet<PathBuf>,
    /// A worktree's Review waiting for the window to open its workspace (#511).
    pending_review: Option<PendingReview>,
    /// The window as the rail last read it. `render` draws from this alone, so another entity's
    /// notify does not redraw the window, and an event that changes nothing shown does not either:
    /// workspaces and terminal views report every chunk of terminal output.
    snapshot: Snapshot,
    /// Zed's sidebar, kept while the rail stands in for it, so the switch back loses neither its
    /// state nor the work it has running.
    zed_sidebar: Option<KeptSidebar>,
    /// Zed's sidebar state restored into a window that opened in the Marley layout. The rail
    /// reads its own fields from it and writes it back with every other field kept.
    zed_sidebar_state: Option<String>,
    add_project_menu: PopoverMenuHandle<SidebarRecentProjects>,
    workspace_subscriptions: HashMap<EntityId, Subscription>,
    /// Per project: its folders, which decide its row, the row's name and its terminals'
    /// subtitles.
    project_subscriptions: HashMap<EntityId, [Subscription; 2]>,
    terminal_subscriptions: HashMap<EntityId, [Subscription; 2]>,
    /// Per Browser tab: its item events (#504).
    browser_subscriptions: HashMap<EntityId, Subscription>,
    /// The browser's hub, once something made it: its pages' titles, icons and counts (#504).
    hub_subscription: Option<Subscription>,
    /// Per Agent Panel: its events, and focus entering and leaving it.
    panel_subscriptions: HashMap<EntityId, [Subscription; 3]>,
    /// Per live thread: the events that can change its row.
    thread_subscriptions: HashMap<EntityId, Subscription>,
    /// Per project: its agent servers, which name the New Agent Thread entries.
    agent_server_subscriptions: HashMap<EntityId, Subscription>,
    /// The thread metadata store, once it exists: it notifies and emits nothing else.
    thread_store_subscription: Option<Subscription>,
    /// Each listed thread's status at the last rebuild, by key, so a run's end is seen.
    thread_statuses: HashMap<String, ThreadStatus>,
    /// The threads whose attention dot is lit.
    noted_threads: HashSet<String>,
    _multi_workspace_subscriptions: [Subscription; 2],
    /// Claude Code's hook events, which move its terminals' rows (#519), its turns (#509), and the
    /// terminals' unread marks (#538); the projectless groups (#600); the harness (#534).
    _agent_events: [Subscription; 6],
    /// The menu a right-click on the rail's empty space opens, where it opened (#600).
    empty_menu: Option<(Entity<ContextMenu>, Point<Pixels>, Subscription)>,
    /// The order the window saved: its projectless groups', which orders the groups a restart
    /// brings back (#601), and the one the user left by dragging (#602).
    saved_order: SavedOrder,
    /// Each project's own icon, by its group's first folder (#564).
    project_icons: HashMap<PathBuf, ProjectIcon>,
    /// The order the rail showed when the pointer came over it, held until it leaves (#542) and
    /// while a drag from it is in flight (#602).
    hold: Hold,
    /// The rows' ports, from the scan an open rail keeps running (#521), and the settings that
    /// can show the rail again.
    _ports: [Subscription; 2],
    /// Focus leaving the rail, and focus given to it while the Brain view shows.
    _focus: [Subscription; 2],
    _filter_edits: Subscription,
    brain: BrainSide,
}

/// The rail's Brain view, and the connection to Rusty it follows (#644).
struct BrainSide {
    /// The view the user chose; Brain shows only while Rusty is connected.
    view: RailView,
    /// Made when it first shows, and dropped when Rusty turns off.
    entity: Option<Entity<BrainView>>,
    /// Whether Rusty was connected at its last change, which the header and the body follow.
    connected: bool,
    _rusty: Subscription,
}

impl BrainSide {
    fn new(window: &Window, cx: &mut Context<Rail>) -> Self {
        Self {
            view: RailView::default(),
            entity: None,
            connected: rusty::is_connected(cx),
            _rusty: cx.observe_global_in::<rusty::Rusty>(window, Rail::rusty_changed),
        }
    }
}

/// What the rail shows under its header (#644).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum RailView {
    /// The projects, their rows, the filter and the inbox.
    #[default]
    Projects,
    /// Rusty's vault: the [`BrainView`].
    Brain,
}

impl std::fmt::Debug for Rail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rail")
            .field("width", &self.width)
            .field("rows", &self.snapshot.rail)
            .finish_non_exhaustive()
    }
}

/// The open switcher, what had focus before it, and its events.
struct OpenSwitcher {
    view: Entity<RailSwitcher>,
    return_focus: Option<FocusHandle>,
    _events: Subscription,
}

/// The entities behind a project row, held weakly: a closed tab must not outlive its terminal.
struct GroupEntry {
    key: ProjectGroupKey,
    workspace: WeakEntity<Workspace>,
    /// The project's checkout, when its folder is a repository's main checkout (#531).
    source: Option<GitSource>,
    /// Its branch's changed lines and pull request, as the row shows them (#531).
    git: Option<ProjectGit>,
    /// The projectless group it is, when it is one (#600); its `key` is then empty.
    group: Option<Uuid>,
    /// That group's workspace id, which the window's saved state keeps (#601).
    database_id: Option<WorkspaceId>,
    /// Whether the window holds no workspace of the group (#606); `workspace` is then an invalid
    /// handle, and a click on the header opens the project.
    closed: bool,
}

/// What a header's right-click menu acts on: a project's, or a projectless group's (#600).
#[derive(Clone)]
struct HeaderMenu {
    rail: WeakEntity<Rail>,
    key: ProjectGroupKey,
    workspace: WeakEntity<Workspace>,
    /// The projectless group and its name, when the header is one.
    group: Option<(Uuid, String)>,
    /// Whether the project is the first and the last, for Move Project Up and Down.
    at: (bool, bool),
    /// Whether the project is closed (#606), which has no browser to clear.
    closed: bool,
    /// The project's archived threads, newest first (#616).
    archived: Vec<ArchivedThread>,
}

/// A project's archived thread, as its header's menu lists it (#616).
#[derive(Clone)]
struct ArchivedThread {
    thread_id: ThreadId,
    agent: Agent,
    work_dirs: PathList,
    title: SharedString,
    archived_at: String,
}

/// A thread to open in its project's Agent Panel.
struct ThreadToOpen {
    thread_id: ThreadId,
    agent: Agent,
    work_dirs: PathList,
    title: Option<SharedString>,
}

/// Opens the thread, focused, in the workspace's Agent Panel. A workspace the rail has just opened
/// (#617) may not have its panel yet: the rail loads one then, and adds it unless Zed's own start
/// of the workspace added one meanwhile.
fn load_thread(
    workspace: &mut Workspace,
    thread: ThreadToOpen,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    if let Some(panel) = workspace.panel::<AgentPanel>(cx) {
        show_thread(workspace, &panel, thread, window, cx);
        return;
    }
    cx.spawn_in(window, async move |workspace, cx| {
        let loaded = AgentPanel::load(workspace.clone(), cx.clone()).await?;
        workspace.update_in(cx, |workspace, window, cx| {
            let panel = workspace.panel::<AgentPanel>(cx).unwrap_or_else(|| {
                workspace.add_panel(loaded.clone(), window, cx);
                loaded
            });
            show_thread(workspace, &panel, thread, window, cx);
        })
    })
    .detach_and_log_err(cx);
}

/// Loads the thread into `panel`, which unarchives an archived one (#616), and focuses the panel.
fn show_thread(
    workspace: &mut Workspace,
    panel: &Entity<AgentPanel>,
    thread: ThreadToOpen,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let ThreadToOpen {
        thread_id,
        agent,
        work_dirs,
        title,
    } = thread;
    panel.update(cx, |panel, cx| {
        panel.load_agent_thread(
            agent,
            thread_id,
            Some(work_dirs),
            title,
            true,
            AgentThreadSource::Sidebar,
            window,
            cx,
        );
    });
    workspace.focus_panel::<AgentPanel>(window, cx);
}

/// How many of a project's archived threads its menu lists.
const ARCHIVED_LISTED: usize = 20;

/// The project group `key`'s archived threads, newest first, as the rail's own threads are
/// matched: by its main folders and its host (#616).
fn archived_threads(key: &ProjectGroupKey, cx: &App) -> Vec<ArchivedThread> {
    let Some(store) = ThreadMetadataStore::try_global(cx) else {
        return Vec::new();
    };
    let host = key.host();
    let mut found: Vec<&ThreadMetadata> = store
        .read(cx)
        .archived_entries()
        .filter(|thread| {
            thread.main_worktree_paths() == key.path_list()
                && thread.matches_remote_connection(host.as_ref())
        })
        .collect();
    found.sort_by_key(|thread| Reverse(thread.updated_at));
    found
        .into_iter()
        .take(ARCHIVED_LISTED)
        .map(|thread| ArchivedThread {
            thread_id: thread.thread_id,
            agent: Agent::from(thread.agent_id.clone()),
            work_dirs: thread.folder_paths().clone(),
            title: thread
                .title
                .clone()
                .unwrap_or_else(|| SharedString::new_static("Untitled thread")),
            archived_at: thread
                .updated_at
                .with_timezone(&chrono::Local)
                .format("%b %-d, %H:%M")
                .to_string(),
        })
        .collect()
}

impl HeaderMenu {
    fn build(&self, window: &mut Window, cx: &mut App) -> Entity<ContextMenu> {
        match &self.group {
            Some((group, name)) => Rail::group_context_menu(
                self.rail.clone(),
                *group,
                name.clone(),
                self.workspace.clone(),
                window,
                cx,
            ),
            None => Rail::project_context_menu(self.clone(), window, cx),
        }
    }
}

/// An element id for a closed group's header (#606), which has no workspace to take one from.
fn closed_id(key: &ProjectGroupKey) -> EntityId {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut hasher);
    EntityId::from(hasher.finish())
}

/// The toasts of a closed project that did not open (#606).
struct ClosedProject;

/// A header's icon: a projectless group's (#600), or the project's own (#564).
fn header_icon(
    index: usize,
    icon: Option<Arc<RenderImage>>,
    projectless: bool,
) -> Option<AnyElement> {
    if projectless {
        return Some(
            Icon::new(IconName::ListTree)
                .size(IconSize::Small)
                .color(Color::Muted)
                .into_any_element(),
        );
    }
    icon.map(|icon| project_icon(index, icon).into_any_element())
}

/// The repository's own icon on its project's header, at 16 px (#564).
fn project_icon(index: usize, icon: Arc<RenderImage>) -> Div {
    div()
        .debug_selector(move || format!("marley-rail-project-icon-{index}"))
        .flex_none()
        .child(img(icon).size_4().rounded_sm())
}

/// A project's own icon on its header (#564).
#[derive(Default)]
struct ProjectIcon {
    image: Option<Arc<RenderImage>>,
    /// The file it came from, which a change to searches again.
    chosen: Option<PathBuf>,
    searched_at: Option<Instant>,
    search: Option<Task<()>>,
    /// Whether another search was asked for while this one ran.
    again: bool,
}

/// How often a project's icon is searched for at most (#564).
const ICON_SEARCH_EVERY: Duration = Duration::from_secs(1);

/// A project's checkout, which its row's counts and pull request are read for (#531).
#[derive(Clone)]
struct GitSource {
    root: PathBuf,
    branch: Option<String>,
    head: Option<String>,
    repository: WeakEntity<Repository>,
}

/// A project's branch against its base, and its pull request (#531).
#[derive(Clone, Debug, PartialEq, Eq)]
struct ProjectGit {
    /// The base, and the lines added and removed since the branch left it.
    counts: Option<(String, u32, u32)>,
    pull_request: Option<PullRequest>,
}

/// What the rail keeps of a project's git reads, by its folder (#531).
struct ProjectGitState {
    /// The branch and `HEAD` it was read for.
    branch: Option<String>,
    head: Option<String>,
    shown: ProjectGit,
    /// When `gh` was last asked, on the executor's clock.
    asked_at: Option<Instant>,
}

/// The entities behind a terminal row, held weakly.
struct TerminalEntry {
    workspace: WeakEntity<Workspace>,
    view: WeakEntity<TerminalView>,
}

/// The entities behind a Browser tab's row (#504), held weakly.
struct BrowserEntry {
    workspace: WeakEntity<Workspace>,
    view: WeakEntity<BrowserView>,
}

/// What a thread row opens, and the icon it draws.
struct ThreadEntry {
    workspace: WeakEntity<Workspace>,
    thread_id: ThreadId,
    agent: Agent,
    work_dirs: PathList,
    title: Option<SharedString>,
    icon: AgentIcon,
    /// The agent's name, for the row's second line.
    agent_name: SharedString,
    /// The project's key while the window holds no workspace of it (#617): `workspace` is then an
    /// invalid handle, and opening the thread opens the project first.
    closed: Option<ProjectGroupKey>,
}

/// The window, read once: the pure snapshot, plus the entities the handlers act on.
#[derive(Default)]
struct Snapshot {
    rail: RailSnapshot,
    groups: Vec<GroupEntry>,
    terminals: HashMap<u64, TerminalEntry>,
    /// The Browser tabs' rows' entities, and the icons their pages have (#504).
    browsers: HashMap<u64, BrowserEntry>,
    favicons: HashMap<u64, Arc<Image>>,
    /// The terminal views an agent CLI is running in.
    agent_terminals: HashSet<EntityId>,
    threads: HashMap<String, ThreadEntry>,
    /// The thread the displayed workspace's visible Agent Panel shows, which is seen by now.
    shown_thread: Option<String>,
    /// What each inbox entry acts on, by its key (#508).
    inbox: HashMap<String, InboxTarget>,
    /// Beside each tool entry of the inbox, what its risk use logs or asks, while the use is on
    /// (#568).
    inbox_risk: HashMap<String, RiskAsking>,
    /// Beside each tool entry of the inbox, what the question route logs or asks, while its use
    /// is on (#570).
    inbox_route: HashMap<String, RouteAsking>,
    /// The entities behind each worktree's row, by its folder (#510).
    worktrees: HashMap<String, WorktreeEntry>,
    /// Each terminal's and Browser tab's place in the order the user leaves by dragging (#602).
    places: HashMap<Selection, String>,
}

/// The entities behind a worktree's row (#510), held weakly.
struct WorktreeEntry {
    /// Its workspace, while the window has it open.
    member: Option<WeakEntity<Workspace>>,
    /// Its project's workspace, which opens it when it is not open.
    project: WeakEntity<Workspace>,
    path: PathBuf,
    name: String,
    /// Its branch as git names it, and its `HEAD`, which its drift is read for (#560).
    branch: Option<String>,
    commit: Option<String>,
    /// Its repository's main checkout, where the drift's git runs, and the repository the rail
    /// reads the branches' tips and the trust from.
    main: Option<PathBuf>,
    repository: Option<WeakEntity<Repository>>,
}

/// What the rail knows of a worktree's drift (#560), once its repository's run has read it.
#[derive(Clone)]
struct DriftState {
    /// The base the run read: #510's record, else the default branch; none for no chip.
    base: Option<String>,
    /// Whether the base is #510's record, which Merge needs (#511).
    recorded: bool,
    /// Who merges the repository's worktree branches, as the run found it (#511).
    owner: MergeOwner,
    /// The tips the drift was read for, the worktree's then the base's; none when the base has
    /// no tip.
    tips: Option<(String, String)>,
    drift: Option<DriftSnapshot>,
}

/// A worktree's Review waiting for the window to open its workspace (#511).
struct PendingReview {
    /// The worktree's folder.
    path: String,
    /// The base the diff compares with.
    base: String,
    /// When the wait ends, on the executor's clock.
    until: Instant,
}

/// What a removal of a worktree works on (#589).
struct RemoveTarget {
    /// The worktree's folder.
    folder: PathBuf,
    /// Its repository's main checkout, where its branch is ended.
    main: Option<PathBuf>,
    /// Its branch.
    branch: Option<String>,
    /// Its own workspace, when the window has it open.
    member: Option<Entity<Workspace>>,
    /// Zed's plan for removing it.
    plan: RootPlan,
}

/// What a worktree row's menu offers for removing it (#589).
enum RemoveLine {
    /// Remove.
    Remove,
    /// A line that is not a choice: why there is no Remove.
    Note(String),
}

/// What a worktree row's menu offers for merging its branch (#511).
enum MergeLine {
    /// Merge, with its entry's words.
    Merge(String),
    /// A line that is not a choice: why there is no Merge.
    Note(String),
}

/// A merge's toast (#511).
struct WorktreeMerge;

/// What the question route logs or asks about one tool entry (#570).
struct RouteAsking {
    /// The entry's ask, which a row or a reading belongs to.
    ask: String,
    /// The state: with the rules' verdict when they marked the entry, and none when they left it
    /// open, which is asked.
    asking: Asking,
}

/// The question route for one tool entry (#570).
struct RouteState {
    /// The ask it belongs to.
    ask: String,
    /// When the entry was first seen.
    seen: Instant,
    /// The row that logged it, a call or a `rules` row, for its outcome.
    row: Option<String>,
    /// The model's reading, once it came: the route, and its confidence when there was an answer.
    reading: Option<(Route, Option<f64>)>,
    /// Whether the user answered it from the inbox, or had its terminal or its thread in front
    /// with the focus while it waited.
    owner_seen: bool,
    /// The ask in flight.
    _task: Option<Task<()>>,
}

/// What the inbox's risk use logs or asks about one tool entry (#568).
struct RiskAsking {
    /// The entry's ask, which a row or a reading belongs to.
    ask: String,
    /// The entry's tool, which sets its level when nothing marks it.
    tool: ToolClass,
    /// The state: with the verdict of Marley's rules when they marked the entry.
    asking: Asking,
}

/// The inbox's risk use for one tool entry (#568).
struct Risk {
    /// The ask it belongs to.
    ask: String,
    /// When the entry was first seen.
    seen: Instant,
    /// The row that logged it, a call or a `rules` row, for its outcome.
    row: Option<String>,
    /// The model's reading, once it came.
    reading: Option<RiskReading>,
    /// How the inbox answered the entry: `allowed` or `denied`.
    answered: Option<&'static str>,
    /// The ask in flight.
    _task: Option<Task<()>>,
}

/// A model's reading of a tool entry (#568): the chips it adds, each with its probability, and
/// its urgency as a level.
#[derive(Default)]
struct RiskReading {
    chips: Vec<(ChipKind, f64)>,
    urgency: Option<u8>,
}

/// What one workspace's inbox entries are read against (#568), and which uses read them: the
/// risk use (#568) and the question route (#570).
struct RiskScope {
    folders: Vec<PathBuf>,
    local: bool,
    redactor: Arc<Redactor>,
    marking: bool,
    routing: bool,
}

/// What a tool entry waits to do, as the risk rules read it (#568).
struct Waiting {
    tool: ToolClass,
    /// The tool's name, a fact of the state.
    tool_name: String,
    /// What it acts on: a command, a path or a question.
    line: String,
    paths: Vec<PathBuf>,
    /// Where it runs, absolute or against the project's first folder.
    cwd: Option<PathBuf>,
    /// Who waits, in words, a fact of the state.
    agent: String,
    /// A question's options (#570).
    options: Vec<String>,
    /// The user's prompt the wait belongs to, when the seat has it (#570).
    prompt: Option<String>,
}

/// What an inbox entry acts on (#508).
#[derive(Clone)]
enum InboxTarget {
    /// An Agent Panel thread's tool call, and the allow-once and deny-once options that answer it
    /// in place, when the prompt offers both and the panel does not gate its Allow.
    Thread {
        thread_key: String,
        conversation: WeakEntity<ConversationView>,
        session: acp::SessionId,
        tool_call: acp::ToolCallId,
        answers: Option<[(acp::PermissionOptionId, acp::PermissionOptionKind); 2]>,
        icon: AgentIcon,
    },
    /// A terminal's Claude Code, by the rail's terminal id.
    Terminal(u64),
    /// A Browser tab's paused click, by its page (#571).
    Click(String),
    /// A harness session, by its id (#534).
    Harness(String),
}

impl Rail {
    /// The globals whose changes redraw rows: agents' events and turns, a terminal's unread mark
    /// (#538), a running command's failure (#572), the projectless groups (#600) and the
    /// harness's sessions (#534).
    fn observe_marks(window: &Window, cx: &mut Context<Self>) -> [Subscription; 6] {
        [
            cx.observe_global_in::<Harness>(window, Self::refresh),
            cx.observe_global_in::<AgentEvents>(window, Self::refresh),
            cx.observe_global_in::<Turns>(window, Self::refresh),
            cx.observe_global_in::<crate::notifications::Attention>(window, Self::refresh),
            cx.observe_global_in::<crate::running_errors::ErrorMarks>(window, Self::refresh),
            cx.observe_global_in::<groups::Groups>(window, Self::groups_changed),
        ]
    }

    /// A rail for `multi_workspace`, keeping Zed's sidebar when it replaces one.
    pub fn new(
        multi_workspace: &Entity<MultiWorkspace>,
        zed_sidebar: Option<KeptSidebar>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let subscriptions = [
            cx.subscribe_in(
                multi_workspace,
                window,
                |rail, _, _: &MultiWorkspaceEvent, window, cx| rail.refresh(window, cx),
            ),
            // Re-keying a project group notifies without an event, and so do opening and
            // closing the sidebar.
            cx.observe_in(
                multi_workspace,
                window,
                |rail, multi_workspace, window, cx| {
                    rail.note_open(&multi_workspace, cx);
                    rail.watch_ports_while_shown(&multi_workspace, cx);
                    rail.refresh(window, cx);
                },
            ),
        ];
        // Built over Zed's sidebar, the rail starts at its width: one width for both layouts.
        let saved = zed_sidebar
            .as_ref()
            .and_then(|(sidebar, _)| sidebar.read(cx).serialized_state(cx))
            .map(|blob| read_rail_state(&blob))
            .unwrap_or_default();
        // The `MultiWorkspace` may be mid-update while its sidebar is built, so the first read
        // waits for the end of this effect cycle.
        cx.defer_in(window, Self::refresh);
        let focus_handle = cx.focus_handle();
        let focus = Self::follow_focus(&focus_handle, window, cx);
        let (filter_editor, filter_edits) = Self::filter_field(window, cx);
        cx.on_release(|rail, cx| {
            if rail.watching_ports {
                ports::unwatch(cx);
            }
        })
        .detach();
        let ports_scanned = cx.observe_global_in::<Ports>(window, Self::refresh);
        // Turning AI back on shows an open rail again without a word from the `MultiWorkspace`,
        // and the stop kind's mode changes what an idle agent's row says (#566).
        let settings_changed =
            cx.observe_global_in::<settings::SettingsStore>(window, |rail, window, cx| {
                if let Some(multi_workspace) = rail.multi_workspace.upgrade() {
                    rail.watch_ports_while_shown(&multi_workspace, cx);
                }
                rail.refresh(window, cx);
            });
        Self {
            multi_workspace: multi_workspace.downgrade(),
            focus_handle,
            width: saved
                .width
                .map_or(DEFAULT_WIDTH, |width| px(width).clamp(MIN_WIDTH, MAX_WIDTH)),
            width_set_by_user: saved.width.is_some(),
            closed: false,
            watching_ports: false,
            cursor: None,
            filter_editor,
            shown_at: HashMap::default(),
            shown_count: 0,
            window_row: None,
            switcher: None,
            foreground_command: |terminal, cx| terminal.read(cx).foreground_process_command_name(),
            terminal_output: HashMap::default(),
            quiet_timers: HashMap::default(),
            minute_timer: None,
            inbox_seen: HashMap::default(),
            inbox_timer: None,
            risk: HashMap::default(),
            routes: HashMap::default(),
            turns_open: HashSet::default(),
            drift: HashMap::default(),
            drift_runs: HashMap::default(),
            drift_unsupported: HashSet::default(),
            project_git: HashMap::default(),
            project_git_runs: HashMap::default(),
            git_edited: None,
            github_failed: HashSet::default(),
            pending_review: None,
            snapshot: Snapshot::default(),
            zed_sidebar,
            zed_sidebar_state: None,
            add_project_menu: PopoverMenuHandle::default(),
            workspace_subscriptions: HashMap::default(),
            project_subscriptions: HashMap::default(),
            terminal_subscriptions: HashMap::default(),
            panel_subscriptions: HashMap::default(),
            thread_subscriptions: HashMap::default(),
            agent_server_subscriptions: HashMap::default(),
            browser_subscriptions: HashMap::default(),
            hub_subscription: None,
            thread_store_subscription: None,
            thread_statuses: HashMap::default(),
            noted_threads: HashSet::default(),
            _multi_workspace_subscriptions: subscriptions,
            _agent_events: Self::observe_marks(window, cx),
            empty_menu: None,
            saved_order: SavedOrder::default(),
            project_icons: HashMap::default(),
            hold: Hold::default(),
            _ports: [ports_scanned, settings_changed],
            _focus: focus,
            _filter_edits: filter_edits,
            brain: BrainSide::new(window, cx),
        }
    }

    /// Follows Rusty's connection: the header's switch and the Brain view show only while it is
    /// up, and the Brain view goes with Rusty turned off. Most changes of Rusty's state are reads
    /// of its settings, which change nothing here (L-572).
    fn rusty_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let connected = rusty::is_connected(cx);
        let brain_focused = self.brain_shown() && self.contains_brain_focus(window, cx);
        if !rusty::is_on(cx) && self.brain.entity.take().is_some() {
            cx.notify();
        }
        if connected == self.brain.connected {
            return;
        }
        self.brain.connected = connected;
        if brain_focused && !connected {
            window.focus(&self.focus_handle, cx);
        }
        cx.notify();
    }

    /// The Brain view, while it shows.
    fn shown_brain(&self) -> Option<&Entity<BrainView>> {
        self.brain
            .entity
            .as_ref()
            .filter(|_| self.brain.view == RailView::Brain && self.brain.connected)
    }

    fn brain_shown(&self) -> bool {
        self.shown_brain().is_some()
    }

    fn contains_brain_focus(&self, window: &Window, cx: &App) -> bool {
        self.brain
            .entity
            .as_ref()
            .is_some_and(|brain| brain.focus_handle(cx).contains_focused(window, cx))
    }

    /// Shows `view` under the header and focuses it; the Brain view is made the first time.
    fn show_view(&mut self, view: RailView, window: &mut Window, cx: &mut Context<Self>) {
        self.brain.view = view;
        if view == RailView::Brain && self.brain.entity.is_none() {
            let multi_workspace = self.multi_workspace.clone();
            self.brain.entity = Some(cx.new(|cx| BrainView::new(multi_workspace, window, cx)));
        }
        match self.shown_brain() {
            Some(brain) => window.focus(&brain.focus_handle(cx), cx),
            None => window.focus(&self.focus_handle, cx),
        }
        cx.notify();
    }

    /// Why `marley::ToggleBrainView` cannot show the Brain view now, if it can't.
    pub(crate) fn brain_refusal(&self, cx: &App) -> Option<SharedString> {
        if self.brain_shown() {
            None
        } else {
            rusty::unavailable(cx)
        }
    }

    /// `marley::ToggleBrainView`: the other view, focused.
    pub(crate) fn toggle_brain_view(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let next = if self.brain_shown() {
            RailView::Projects
        } else {
            RailView::Brain
        };
        self.show_view(next, window, cx);
    }

    /// The keyboard's row is the selection only while the rail holds focus. While the Brain view
    /// shows, focus given to the rail (Zed's focus-the-sidebar actions, a click on the header)
    /// goes into it: Zed keeps the rail's own handle as the window's sidebar focus.
    fn follow_focus(
        focus_handle: &FocusHandle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> [Subscription; 2] {
        [
            cx.on_focus_out(focus_handle, window, |rail, _, window, cx| {
                rail.cursor = None;
                rail.refresh(window, cx);
            }),
            cx.on_focus(focus_handle, window, |rail, window, cx| {
                if let Some(brain) = rail.shown_brain() {
                    let brain = brain.focus_handle(cx);
                    window.focus(&brain, cx);
                }
            }),
        ]
    }

    /// The filter's field under the header, and its edits, which refilter the rows.
    fn filter_field(window: &mut Window, cx: &mut Context<Self>) -> (Entity<Editor>, Subscription) {
        let filter_editor = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("Filter…", window, cx);
            editor
        });
        let filter_edits = cx.subscribe_in(
            &filter_editor,
            window,
            |rail, _, event: &EditorEvent, window, cx| {
                if matches!(event, EditorEvent::BufferEdited) {
                    rail.filter_edited(window, cx);
                }
            },
        );
        (filter_editor, filter_edits)
    }

    /// Hands Zed's sidebar back for the switch to the Zed layout: the one the rail kept, and the
    /// state to restore into a fresh one when it kept none, with the rail's width in it.
    pub(crate) fn take_zed_sidebar(&mut self) -> (Option<KeptSidebar>, Option<String>) {
        let state = write_rail_state(self.zed_sidebar_state.take().as_deref(), self.rail_state());
        (self.zed_sidebar.take(), Some(state))
    }

    /// Notes whether the user has closed the rail. Nothing is noted while the sidebar cannot be
    /// shown at all.
    fn note_open(&mut self, multi_workspace: &Entity<MultiWorkspace>, cx: &App) {
        let multi_workspace = multi_workspace.read(cx);
        if multi_workspace.multi_workspace_enabled(cx) {
            self.closed = !multi_workspace.sidebar_open();
        }
    }

    /// Keeps the port scan running while this rail is the window's sidebar and open, and lets it
    /// stop while it is closed.
    fn watch_ports_while_shown(
        &mut self,
        multi_workspace: &Entity<MultiWorkspace>,
        cx: &mut Context<Self>,
    ) {
        let rail = cx.entity_id();
        let shown = {
            let multi_workspace = multi_workspace.read(cx);
            multi_workspace.multi_workspace_enabled(cx)
                && multi_workspace.sidebar_open()
                && multi_workspace
                    .sidebar()
                    .is_some_and(|sidebar| sidebar.to_any().entity_id() == rail)
        };
        if shown == self.watching_ports {
            return;
        }
        self.watching_ports = shown;
        if shown {
            ports::watch(cx);
        } else {
            ports::unwatch(cx);
        }
    }

    /// A group made, renamed, folded, adopted or removed: the rows follow, and the window's saved
    /// state keeps its groups (#601).
    fn groups_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.refresh(window, cx);
        self.multi_workspace
            .update(cx, MultiWorkspace::serialize)
            .log_err();
    }

    /// The window's projectless groups' workspace ids, in the rail's order.
    fn group_ids(&self) -> Vec<WorkspaceId> {
        self.snapshot
            .groups
            .iter()
            .filter_map(|group| group.database_id)
            .collect()
    }

    fn rail_state(&self) -> RailState {
        RailState {
            width: self.width_set_by_user.then(|| f32::from(self.width)),
            closed: self.closed,
        }
    }

    /// Follows every workspace, terminal, Agent Panel and live thread, rereads the window, and
    /// redraws only when what the rail shows has changed.
    fn refresh(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(multi_workspace) = self.multi_workspace.upgrade() {
            groups::adopt(&multi_workspace, cx);
        }
        self.sync_subscriptions(window, cx);
        let filter = self.filter_editor.read(cx).text(cx);
        let mut snapshot = self
            .multi_workspace
            .upgrade()
            .map(|multi_workspace| {
                build_snapshot(
                    &multi_workspace,
                    self.foreground_command,
                    &self.terminal_output,
                    &filter,
                    &self.saved_order,
                    window,
                    cx,
                )
            })
            .unwrap_or_default();
        order::arrange(&mut snapshot, &self.saved_order);
        self.note_turns_open(&mut snapshot);
        self.note_ended_runs(&mut snapshot);
        self.note_inbox(&mut snapshot, window, cx);
        self.note_window_row(&snapshot.rail);
        self.note_claude_code(&snapshot.rail, window, cx);
        self.note_drift(&mut snapshot);
        self.note_project_git(&mut snapshot);
        // The rail's order, and the order it holds while the pointer is over it (#542).
        snapshot.rail.order = MarleySettings::get_global(cx).rail_order;
        snapshot.rail.held.clone_from(&self.hold.order);
        if self.focus_handle.contains_focused(window, cx) {
            snapshot.rail.focus.cursor.clone_from(&self.cursor);
        }
        if snapshot.rail != self.snapshot.rail {
            cx.notify();
        }
        self.snapshot = snapshot;
        self.follow_icons(cx);
        self.follow_risk(window, cx);
        self.follow_route(window, cx);
        self.follow_drift(window, cx);
        self.follow_project_git(window, cx);
        self.take_review(window, cx);
    }

    /// Searches each local project's folder for its icon the first time its group shows, and
    /// forgets the groups that went (#564).
    fn follow_icons(&mut self, cx: &Context<Self>) {
        let roots: Vec<PathBuf> =
            self.snapshot
                .groups
                .iter()
                .filter(|group| {
                    // A closed local project keeps its icon, dimmed (#606).
                    (group.closed && group.key.host().is_none())
                        || group.workspace.upgrade().is_some_and(|workspace| {
                            workspace.read(cx).project().read(cx).is_local()
                        })
                })
                .filter_map(|group| group.key.path_list().paths().first().cloned())
                .collect();
        self.project_icons.retain(|root, _| roots.contains(root));
        for root in roots {
            if !self.project_icons.contains_key(&root) {
                self.search_icon(root, cx);
            }
        }
    }

    /// Reads `root`'s icon off the main thread; a search asked for while one runs, or within a
    /// second of the last, waits for its turn.
    fn search_icon(&mut self, root: PathBuf, cx: &Context<Self>) {
        let now = cx.background_executor().now();
        let icon = self.project_icons.entry(root.clone()).or_default();
        if icon.search.is_some() {
            icon.again = true;
            return;
        }
        let wait = icon
            .searched_at
            .map(|at| ICON_SEARCH_EVERY.saturating_sub(now.saturating_duration_since(at)))
            .unwrap_or_default();
        icon.search = Some(cx.spawn(async move |rail, cx| {
            if !wait.is_zero() {
                cx.background_executor().timer(wait).await;
            }
            let searched = root.clone();
            let found = cx
                .background_spawn(futures::future::lazy(move |_| {
                    crate::project_icons::icon_in(&searched)
                }))
                .await;
            rail.update(cx, |rail, cx| {
                let now = cx.background_executor().now();
                let Some(icon) = rail.project_icons.get_mut(&root) else {
                    return;
                };
                let (chosen, image) = found.unzip();
                icon.chosen = chosen;
                icon.image = image;
                icon.searched_at = Some(now);
                icon.search = None;
                if std::mem::take(&mut icon.again) {
                    rail.search_icon(root, cx);
                }
                cx.notify();
            })
            .log_err();
        }));
    }

    /// A project's files changed: its icon is searched again when a changed file can be one, or
    /// is the one it shows (#564).
    fn note_icon_files(
        &mut self,
        project: &Entity<Project>,
        worktree: WorktreeId,
        changes: &UpdatedEntriesSet,
        cx: &Context<Self>,
    ) {
        let Some(root) = project
            .read(cx)
            .worktree_for_id(worktree, cx)
            .map(|worktree| worktree.read(cx).abs_path().to_path_buf())
        else {
            return;
        };
        let Some(icon) = self.project_icons.get(&root) else {
            return;
        };
        let chosen = icon
            .chosen
            .as_ref()
            .and_then(|chosen| chosen.strip_prefix(&root).ok())
            .map(Path::to_path_buf);
        let touched = changes.iter().any(|(path, _, _)| {
            let relative = path.as_unix_str();
            crate::project_icons::is_candidate(relative)
                || chosen
                    .as_deref()
                    .is_some_and(|chosen| chosen == Path::new(relative))
        });
        if touched {
            self.search_icon(root, cx);
        }
    }

    /// Puts each project's kept changed lines and pull request on its row (#531).
    fn note_project_git(&self, snapshot: &mut Snapshot) {
        for group in &mut snapshot.groups {
            group.git = group
                .source
                .as_ref()
                .and_then(|source| self.project_git.get(&source.root))
                .map(|state| state.shown.clone());
        }
    }

    /// Forgets the projects no longer listed, and while the rail shows, schedules a read for each
    /// project Zed trusts whose branch or `HEAD` moved, whose tracked files changed, or whose pull
    /// request was last asked about [`PULL_REQUEST_EVERY`] ago while the window is active (#531).
    fn follow_project_git(&mut self, window: &Window, cx: &Context<Self>) {
        let sources: Vec<GitSource> = self
            .snapshot
            .groups
            .iter()
            .filter_map(|group| group.source.clone())
            .collect();
        self.project_git
            .retain(|root, _| sources.iter().any(|source| source.root == *root));
        if !self.watching_ports {
            return;
        }
        let edited = self.git_edited.take();
        let edits = edited.is_some();
        let now = cx.background_executor().now();
        let active = window.is_window_active();
        for source in sources {
            if self.project_git_runs.contains_key(&source.root) {
                // The run in flight may have read before the edit.
                self.git_edited = self.git_edited.or(edited);
                continue;
            }
            let trusted = source
                .repository
                .upgrade()
                .is_some_and(|repository| repository.read(cx).is_trusted());
            if !trusted {
                continue;
            }
            let state = self.project_git.get(&source.root);
            let moved = state
                .is_none_or(|state| state.branch != source.branch || state.head != source.head);
            let ask = moved
                || (active
                    && state
                        .and_then(|state| state.asked_at)
                        .is_none_or(|at| now.saturating_duration_since(at) >= PULL_REQUEST_EVERY));
            if moved || ask || edits {
                let root = source.root.clone();
                let run = Self::project_git_run(source, ask, window, cx);
                self.project_git_runs.insert(root, run);
            }
        }
    }

    /// A project's read (#531). A second after it is scheduled: when `ask`, its pull request from
    /// `gh` (GitHub remotes only), else the one kept; then its lines changed against the pull
    /// request's base, else the repository's default branch; the rail keeps both and redraws.
    fn project_git_run(
        source: GitSource,
        ask: bool,
        window: &Window,
        cx: &Context<Self>,
    ) -> Task<()> {
        let debounce = cx.background_executor().timer(DRIFT_DEBOUNCE);
        cx.spawn_in(window, async move |rail, cx| {
            debounce.await;
            let asked = rail.update(cx, |rail, cx| {
                let repository = source
                    .repository
                    .upgrade()
                    .filter(|repository| repository.read(cx).is_trusted())?;
                let github = ask.then(|| github_repository(&repository, cx)).flatten();
                let kept = rail
                    .project_git
                    .get(&source.root)
                    .and_then(|state| state.shown.pull_request.clone());
                let default =
                    repository.update(cx, |repository, _| repository.default_branch(true));
                Some((github, kept, default))
            });
            let Ok(Some((github, kept, default))) = asked else {
                rail.update(cx, |rail, _| rail.project_git_runs.remove(&source.root))
                    .log_err();
                return;
            };
            let default = default
                .await
                .ok()
                .and_then(util::ResultExt::log_err)
                .flatten()
                .map(|default| default.to_string());
            let (asked, counts) = cx
                .background_spawn({
                    let (root, branch) = (source.root.clone(), source.branch.clone());
                    let github = github.filter(|_| ask);
                    async move { read_project_git(&root, branch, github, kept, default).await }
                })
                .await;
            rail.update_in(cx, |rail, window, cx| {
                rail.project_git_runs.remove(&source.root);
                let previous = rail.project_git.get(&source.root);
                // A read due to ask counts as asked even when there was nothing to ask (no GitHub
                // remote, no branch), or the next refresh would schedule another at once.
                let asked_at = if ask {
                    Some(cx.background_executor().now())
                } else {
                    previous.and_then(|state| state.asked_at)
                };
                let mut pull_request = previous.and_then(|state| state.shown.pull_request.clone());
                match asked {
                    Some(Ok(found)) => {
                        pull_request = found;
                    }
                    Some(Err(error)) => {
                        pull_request = None;
                        if rail.github_failed.insert(source.root.clone()) {
                            log::warn!(
                                "no pull request state for {}: {error:#}",
                                source.root.display()
                            );
                        }
                    }
                    None => {}
                }
                let counts = counts
                    .map_err(|error| {
                        log::warn!("no changed lines for {}: {error:#}", source.root.display());
                    })
                    .ok()
                    .flatten();
                rail.project_git.insert(
                    source.root.clone(),
                    ProjectGitState {
                        branch: source.branch.clone(),
                        head: source.head.clone(),
                        shown: ProjectGit {
                            counts,
                            pull_request,
                        },
                        asked_at,
                    },
                );
                rail.refresh(window, cx);
                cx.notify();
            })
            .log_err();
        })
    }

    /// Puts each worktree's kept drift on its row (#560). A drift read for tips that have moved
    /// stays until its repository's next run lands, so the chip lags a commit rather than
    /// blinking at each one.
    fn note_drift(&self, snapshot: &mut Snapshot) {
        for worktree in snapshot
            .rail
            .projects
            .iter_mut()
            .flat_map(|project| project.worktrees.iter_mut())
        {
            worktree.drift = self
                .drift
                .get(&worktree.path)
                .and_then(|state| state.drift.clone());
        }
    }

    /// Forgets the worktrees no longer listed, and while the rail shows (F-521), schedules a run
    /// for each repository Zed trusts with a worktree whose drift is unread or whose tips have
    /// moved (#560). A repository's run in flight reads the tips when it starts; one that moved
    /// during it is found at the refresh its end makes.
    fn follow_drift(&mut self, window: &Window, cx: &Context<Self>) {
        let listed = &self.snapshot.worktrees;
        self.drift.retain(|path, _| listed.contains_key(path));
        if !self.watching_ports {
            return;
        }
        let mut due: Vec<PathBuf> = Vec::new();
        for (path, entry) in &self.snapshot.worktrees {
            let (Some(main), Some(repository)) = (&entry.main, &entry.repository) else {
                continue;
            };
            if self.drift_runs.contains_key(main) || due.contains(main) {
                continue;
            }
            let Some(repository) = repository.upgrade() else {
                continue;
            };
            let repository = repository.read(cx);
            let moved = self.drift.get(path).is_none_or(|state| {
                !same_tips(
                    state.tips.as_ref(),
                    drift_tips(
                        entry.commit.as_deref(),
                        state.base.as_deref(),
                        &repository.branch_list,
                    ),
                )
            });
            if moved && repository.is_trusted() {
                due.push(main.clone());
            }
        }
        for main in due {
            let run = Self::drift_run(main.clone(), window, cx);
            self.drift_runs.insert(main, run);
        }
    }

    /// The drift run of the repository whose main checkout is `main` (#560). A second after it
    /// is scheduled, it reads each listed worktree's base and runs a summary for each whose base
    /// or tips have moved, off the main thread; the rail keeps each result with the tips it was
    /// read for, and refreshes. A repository gone, or no longer trusted, runs nothing.
    fn drift_run(main: PathBuf, window: &Window, cx: &Context<Self>) -> Task<()> {
        let debounce = cx.background_executor().timer(DRIFT_DEBOUNCE);
        cx.spawn_in(window, async move |rail, cx| {
            debounce.await;
            let asked = rail.update(cx, |rail, cx| {
                let repository = rail
                    .snapshot
                    .worktrees
                    .values()
                    .filter(|entry| entry.main.as_ref() == Some(&main))
                    .find_map(|entry| entry.repository.as_ref()?.upgrade())
                    .filter(|repository| repository.read(cx).is_trusted());
                let Some(repository) = repository else {
                    // The run ends here, and its handle with it.
                    rail.drift_runs.remove(&main);
                    return None;
                };
                let reads: Vec<DriftRead> = rail
                    .snapshot
                    .worktrees
                    .iter()
                    .filter(|(_, entry)| entry.main.as_ref() == Some(&main))
                    .map(|(path, entry)| DriftRead {
                        path: path.clone(),
                        branch: entry.branch.clone(),
                        commit: entry.commit.clone(),
                        kept: rail.drift.get(path).cloned(),
                    })
                    .collect();
                let branches = Arc::clone(&repository.read(cx).branch_list);
                let default =
                    repository.update(cx, |repository, _| repository.default_branch(false));
                Some((reads, branches, default, <dyn Fs>::global(cx)))
            });
            let Ok(Some((reads, branches, default, fs))) = asked else {
                return;
            };
            let default = default
                .await
                .ok()
                .and_then(util::ResultExt::log_err)
                .flatten()
                .map(|default| default.to_string());
            let states = cx
                .background_spawn({
                    let main = main.clone();
                    async move { read_drifts(&main, reads, &branches, default, fs.as_ref()).await }
                })
                .await;
            rail.update_in(cx, |rail, window, cx| {
                rail.drift_runs.remove(&main);
                let unsupported = states.iter().any(|(_, state)| {
                    state
                        .drift
                        .as_ref()
                        .is_some_and(|drift| drift.conflicts.is_none())
                });
                if unsupported && rail.drift_unsupported.insert(main.clone()) {
                    log::warn!(
                        "git in {} refuses merge-tree --write-tree, which came in git 2.38: its \
                         worktree rows show how far each branch is behind, without its conflicts",
                        main.display()
                    );
                }
                rail.drift.extend(states);
                rail.refresh(window, cx);
            })
            .log_err();
        })
    }

    /// Logs each new tool entry of the inbox once for the question route (#570), a `rules` row
    /// when Marley's rules marked it and a call when they left it open, whose reading lands with a
    /// refresh; notes who had each entry's place in front with the focus; and logs who answered
    /// each entry with a row once it leaves: `owner`, or `agent`.
    fn follow_route(&mut self, window: &Window, cx: &mut Context<Self>) {
        if system_one::use_mode(QUESTION_ROUTE.name, cx) == SystemOneMode::Off {
            self.routes.clear();
            return;
        }
        let now = cx.background_executor().now();
        let focus = &self.snapshot.rail.focus;
        if window.is_window_active() {
            for (key, state) in &mut self.routes {
                let watched = match self.snapshot.inbox.get(key) {
                    Some(InboxTarget::Terminal(id)) => {
                        focus.terminal == Some(*id) && focus.terminal_focused
                    }
                    Some(InboxTarget::Thread { thread_key, .. }) => {
                        focus.thread.as_deref() == Some(thread_key.as_str())
                    }
                    Some(InboxTarget::Click(_) | InboxTarget::Harness(_)) | None => false,
                };
                state.owner_seen |= watched;
            }
        }
        let gone: Vec<String> = self
            .routes
            .keys()
            .filter(|key| !self.snapshot.inbox_route.contains_key(*key))
            .cloned()
            .collect();
        for key in gone {
            let Some(RouteState {
                row: Some(row),
                seen,
                owner_seen,
                ..
            }) = self.routes.remove(&key)
            else {
                continue;
            };
            let who = if owner_seen { "owner" } else { "agent" };
            let waited = now.saturating_duration_since(seen).as_secs();
            system_one::outcome(&row, format!("{who} after {waited} s"), cx);
        }
        for (key, item) in &self.snapshot.inbox_route {
            if self
                .routes
                .get(key)
                .is_some_and(|state| state.ask == item.ask)
            {
                continue;
            }
            let (row, task) = if item.asking.verdict.is_some() {
                let row = system_one::record(QUESTION_ROUTE, &item.asking, cx)
                    .row
                    .map(|row| row.id);
                (row, None)
            } else {
                let asked = system_one::ask(QUESTION_ROUTE, &item.asking, cx);
                let (key, ask) = (key.clone(), item.ask.clone());
                let task = cx.spawn_in(window, async move |rail, cx| {
                    let asked = asked.await;
                    rail.update_in(cx, |rail, window, cx| {
                        if let Some(state) = rail.routes.get_mut(&key)
                            && state.ask == ask
                        {
                            state.row = asked.row.as_ref().map(|row| row.id.clone());
                            state.reading = route_reading(&asked.reading);
                            rail.refresh(window, cx);
                        }
                    })
                    .log_err();
                });
                (None, Some(task))
            };
            let state = RouteState {
                ask: item.ask.clone(),
                seen: self.inbox_seen.get(key).copied().unwrap_or(now),
                row,
                reading: None,
                owner_seen: false,
                _task: task,
            };
            self.routes.insert(key.clone(), state);
        }
    }

    /// Logs each new tool entry of the inbox once (#568): as a `rules` row when Marley's rules
    /// marked it, and as a call to the System One layer when they found nothing, whose reading
    /// lands with a refresh. Logs how each entry with a row was cleared once it leaves.
    fn follow_risk(&mut self, window: &Window, cx: &mut Context<Self>) {
        if system_one::use_mode(INBOX_RISK.name, cx) == SystemOneMode::Off {
            self.risk.clear();
            return;
        }
        let now = cx.background_executor().now();
        let gone: Vec<String> = self
            .risk
            .keys()
            .filter(|key| !self.snapshot.inbox_risk.contains_key(*key))
            .cloned()
            .collect();
        for key in gone {
            let Some(Risk {
                row: Some(row),
                seen,
                answered,
                ..
            }) = self.risk.remove(&key)
            else {
                continue;
            };
            let how = answered.map_or_else(
                || "cleared elsewhere".to_string(),
                |how| format!("{how} from the inbox"),
            );
            let waited = now.saturating_duration_since(seen).as_secs();
            system_one::outcome(&row, format!("{how} after {waited} s"), cx);
        }
        for (key, item) in &self.snapshot.inbox_risk {
            if self.risk.get(key).is_some_and(|risk| risk.ask == item.ask) {
                continue;
            }
            let (row, task) = if item.asking.verdict.is_some() {
                let row = system_one::record(INBOX_RISK, &item.asking, cx)
                    .row
                    .map(|row| row.id);
                (row, None)
            } else {
                let asked = system_one::ask(INBOX_RISK, &item.asking, cx);
                let (key, ask) = (key.clone(), item.ask.clone());
                let task = cx.spawn_in(window, async move |rail, cx| {
                    let asked = asked.await;
                    rail.update_in(cx, |rail, window, cx| {
                        if let Some(risk) = rail.risk.get_mut(&key)
                            && risk.ask == ask
                        {
                            risk.row = asked.row.as_ref().map(|row| row.id.clone());
                            risk.reading = Some(risk_reading(&asked.reading));
                            rail.refresh(window, cx);
                        }
                    })
                    .log_err();
                });
                (None, Some(task))
            };
            let risk = Risk {
                ask: item.ask.clone(),
                seen: self.inbox_seen.get(key).copied().unwrap_or(now),
                row,
                reading: None,
                answered: None,
                _task: task,
            };
            self.risk.insert(key.clone(), risk);
        }
    }

    /// Orders the inbox by when the rail first saw each entry, oldest first, and says how long each
    /// has waited (#508). The ages move by a refresh every half minute while any entry waits.
    /// While the risk use is on, a reading's chips join its entry's in `suggest` and `act`, the
    /// reading may raise the level in `act`, and the entries go by level first (#568).
    fn note_inbox(&mut self, snapshot: &mut Snapshot, window: &Window, cx: &Context<Self>) {
        let now = cx.background_executor().now();
        let keys: HashSet<&str> = snapshot
            .rail
            .inbox
            .iter()
            .map(|entry| entry.key.as_str())
            .collect();
        self.inbox_seen.retain(|key, _| keys.contains(key.as_str()));
        for entry in &mut snapshot.rail.inbox {
            let seen = *self.inbox_seen.entry(entry.key.clone()).or_insert(now);
            entry.waited = marley_rail::waited_words(now.saturating_duration_since(seen).as_secs());
        }
        let mode = system_one::use_mode(INBOX_RISK.name, cx);
        snapshot.rail.inbox_suggests = mode == SystemOneMode::Suggest;
        if matches!(mode, SystemOneMode::Suggest | SystemOneMode::Act) {
            for entry in &mut snapshot.rail.inbox {
                let Some(reading) = self
                    .risk
                    .get(&entry.key)
                    .filter(|risk| risk.ask == entry.ask)
                    .and_then(|risk| risk.reading.as_ref())
                else {
                    continue;
                };
                let added: Vec<Chip> = reading
                    .chips
                    .iter()
                    .filter(|(kind, _)| !entry.chips.iter().any(|chip| chip.kind == *kind))
                    .map(|(kind, _)| Chip {
                        kind: *kind,
                        source: ChipSource::Model,
                    })
                    .collect();
                entry.chips.extend(added);
                if mode == SystemOneMode::Act {
                    let tool = snapshot
                        .inbox_risk
                        .get(&entry.key)
                        .map_or(ToolClass::Other, |item| item.tool);
                    entry.level = risk::level(tool, &entry.chips)
                        .max(reading.urgency.unwrap_or_default())
                        .max(entry.level);
                }
            }
        }
        let routing = system_one::use_mode(QUESTION_ROUTE.name, cx);
        snapshot.rail.route_suggests = routing == SystemOneMode::Suggest;
        if matches!(routing, SystemOneMode::Suggest | SystemOneMode::Act) {
            for entry in &mut snapshot.rail.inbox {
                if entry.route.is_some() {
                    continue;
                }
                entry.route = self
                    .routes
                    .get(&entry.key)
                    .filter(|state| state.ask == entry.ask)
                    .and_then(|state| state.reading)
                    .map(|(route, _)| RouteMark {
                        route,
                        source: RouteSource::Reading,
                    });
            }
        }
        let seen = &self.inbox_seen;
        if routing == SystemOneMode::Act {
            // An entry the route has not marked yet waits for a person, as `unclear` does (D5).
            snapshot.rail.inbox.sort_by_key(|entry| {
                let rank = entry
                    .route
                    .map_or(Route::Unclear.rank(), |mark| mark.route.rank());
                (Reverse(entry.level), rank, seen.get(&entry.key).copied())
            });
        } else if mode == SystemOneMode::Off {
            snapshot
                .rail
                .inbox
                .sort_by_key(|entry| seen.get(&entry.key).copied());
        } else {
            snapshot
                .rail
                .inbox
                .sort_by_key(|entry| (Reverse(entry.level), seen.get(&entry.key).copied()));
        }
        if snapshot.rail.inbox.is_empty() {
            self.inbox_timer = None;
        } else if self.inbox_timer.is_none() {
            self.inbox_timer = Some(cx.spawn_in(window, async move |rail, cx| {
                cx.background_executor()
                    .timer(Duration::from_secs(30))
                    .await;
                rail.update_in(cx, |rail, window, cx| {
                    rail.inbox_timer = None;
                    rail.refresh(window, cx);
                })
                .log_err();
            }));
        }
    }

    /// Answers the Agent Panel prompt the inbox entry `key` lists, allowing it once or denying it
    /// once, through the panel's own path (#508). Out of the rail's update, since the thread's
    /// view updates its thread.
    fn answer_thread(&self, key: &str, allow: bool, window: &Window, cx: &mut Context<Self>) {
        let Some(InboxTarget::Thread {
            conversation,
            session,
            tool_call,
            answers: Some([allowing, denying]),
            ..
        }) = self.snapshot.inbox.get(key).cloned()
        else {
            return;
        };
        let (option, kind) = if allow { allowing } else { denying };
        cx.defer_in(window, move |_, window, cx| {
            let Some(conversation) = conversation.upgrade() else {
                return;
            };
            let Some(view) = conversation.read(cx).thread_view(&session) else {
                return;
            };
            view.update(cx, |view, cx| {
                view.authorize_tool_call(
                    session,
                    tool_call,
                    SelectedPermissionOutcome::new(option, kind),
                    window,
                    cx,
                );
            });
        });
    }

    /// Shows what the inbox entry `key` waits in: its thread, its terminal, or its Browser tab
    /// with the focus on the paused click's card (#508).
    fn open_inbox_entry(&self, key: &str, window: &mut Window, cx: &mut Context<Self>) {
        match self.snapshot.inbox.get(key).cloned() {
            Some(InboxTarget::Thread { thread_key, .. }) => {
                self.open_thread(&thread_key, window, cx).log_err();
            }
            Some(InboxTarget::Terminal(id)) => {
                if let Some(terminal) = self.snapshot.terminals.get(&id) {
                    self.activate_terminal(&terminal.workspace, &terminal.view, window, cx)
                        .log_err();
                }
            }
            Some(InboxTarget::Click(target)) => {
                cx.defer_in(window, move |_, window, cx| {
                    browser::show_paused(&target, window, cx);
                });
            }
            Some(InboxTarget::Harness(id)) => self.open_harness(&id, window, cx),
            None => {}
        }
    }

    /// Shows the harness session `id`'s tab in the displayed workspace (#534).
    fn open_harness(&self, id: &str, window: &Window, cx: &mut Context<Self>) {
        let Some(shown) = self
            .multi_workspace
            .upgrade()
            .map(|multi_workspace| multi_workspace.read(cx).workspace().clone())
        else {
            return;
        };
        let id = id.to_string();
        cx.defer_in(window, move |_, window, cx| {
            shown.update(cx, |workspace, cx| {
                crate::harness::open(workspace, &id, window, cx);
            });
        });
    }

    /// The inbox (#508): the agents that wait on the user, oldest first, over the projects,
    /// while any waits. An entry that answers in place carries its two buttons under it.
    fn render_inbox(&self, cx: &Context<Self>) -> Option<AnyElement> {
        let entries = &self.snapshot.rail.inbox;
        if entries.is_empty() {
            return None;
        }
        Some(
            v_flex()
                .id("marley-rail-inbox")
                .debug_selector(|| "marley-rail-inbox".into())
                .px_2()
                .pt_1p5()
                .gap_0p5()
                .child(
                    h_flex()
                        .px_2()
                        .gap_1()
                        .child(
                            Label::new("Needs you")
                                .size(LabelSize::Small)
                                .color(Color::Warning),
                        )
                        .child(
                            Label::new(entries.len().to_string())
                                .size(LabelSize::Small)
                                .color(Color::Muted),
                        ),
                )
                .children(
                    entries
                        .iter()
                        .map(|entry| self.render_inbox_entry(entry, cx)),
                )
                .child(div().pt_1p5().child(Divider::horizontal()))
                .into_any_element(),
        )
    }

    /// One inbox entry: the agent and its project, what it asks and how long it has waited; a
    /// click shows where it waits, and its buttons, when it has them, answer it there.
    fn render_inbox_entry(&self, entry: &InboxEntry, cx: &Context<Self>) -> AnyElement {
        let key = entry.key.clone();
        let icon = match (entry.kind, self.snapshot.inbox.get(&entry.key)) {
            (InboxKind::Thread, Some(InboxTarget::Thread { icon, .. })) => match icon {
                AgentIcon::Named(icon) => Icon::new(*icon),
                AgentIcon::Svg(path) => Icon::from_external_svg(path.clone()),
            },
            // The entry names the seat's agent (#650).
            (InboxKind::Terminal, _) => Icon::new(
                AgentKind::ALL
                    .into_iter()
                    .find(|kind| kind.display_name() == entry.agent)
                    .map_or(IconName::AiClaude, agents::cli_icon),
            ),
            (InboxKind::Harness, _) => Icon::new(IconName::Server),
            _ => Icon::new(IconName::ToolWeb),
        };
        let open_key = key.clone();
        let card = row_card(
            SharedString::from(format!("marley-rail-inbox-{key}")),
            format!("marley-rail-inbox-icon-{key}"),
            false,
            icon.size(IconSize::Small)
                .color(Color::Muted)
                .into_any_element(),
            Label::new(format!("{} · {}", entry.agent, entry.project))
                .size(LabelSize::Small)
                .truncate()
                .into_any_element(),
            vec![entry.ask.clone()],
            cx,
        )
        .child(
            Label::new(entry.waited.clone())
                .size(LabelSize::XSmall)
                .color(Color::Muted),
        )
        .on_click(cx.listener(move |rail, _, window, cx| {
            rail.open_inbox_entry(&open_key, window, cx);
        }));
        let mut chips = self.render_chips(entry, cx);
        chips.extend(self.render_route(entry));
        let buttons = entry.answers.then(|| {
            let (allow_key, deny_key) = (key.clone(), key.clone());
            let refuse = if entry.kind == InboxKind::Click {
                "Refuse"
            } else {
                "Deny"
            };
            h_flex()
                .justify_end()
                .gap_1()
                .pr_1p5()
                .child(
                    Button::new(
                        SharedString::from(format!("marley-rail-inbox-deny-{key}")),
                        refuse,
                    )
                    .label_size(LabelSize::Small)
                    .on_click(cx.listener(move |rail, _, window, cx| {
                        cx.stop_propagation();
                        rail.answer_inbox(&deny_key, false, window, cx);
                    })),
                )
                .child(
                    Button::new(
                        SharedString::from(format!("marley-rail-inbox-allow-{key}")),
                        "Allow",
                    )
                    .style(ButtonStyle::Filled)
                    .label_size(LabelSize::Small)
                    .on_click(cx.listener(move |rail, _, window, cx| {
                        cx.stop_propagation();
                        rail.answer_inbox(&allow_key, true, window, cx);
                    })),
                )
        });
        let selector = format!("marley-rail-inbox-entry-{key}");
        let line = (!chips.is_empty() || buttons.is_some()).then(|| {
            h_flex()
                .pl_2()
                .gap_1()
                .justify_between()
                .child(h_flex().flex_wrap().gap_1().children(chips))
                .children(buttons)
        });
        v_flex()
            .debug_selector(move || selector)
            .gap_0p5()
            .child(card)
            .children(line)
            .into_any_element()
    }

    /// An inbox entry's chips (#568): Marley's plain, a reading's with a question mark in
    /// `suggest` and a dashed border in `act`, with its probability in a tooltip.
    fn render_chips(&self, entry: &InboxEntry, cx: &Context<Self>) -> Vec<AnyElement> {
        let suggests = self.snapshot.rail.inbox_suggests;
        let reading = self
            .risk
            .get(&entry.key)
            .and_then(|risk| risk.reading.as_ref());
        entry
            .chips
            .iter()
            .enumerate()
            .map(|(index, chip)| {
                let model = chip.source == ChipSource::Model;
                let words = if model && suggests {
                    format!("{}?", chip.kind.words())
                } else {
                    chip.kind.words().to_string()
                };
                let color = if chip.kind.level() == Some(risk::TOP_LEVEL) {
                    Color::Error
                } else {
                    Color::Warning
                };
                let probability = reading.and_then(|reading| {
                    reading
                        .chips
                        .iter()
                        .find(|(kind, _)| *kind == chip.kind)
                        .map(|(_, probability)| *probability)
                });
                div()
                    .id(SharedString::from(format!(
                        "marley-rail-inbox-chip-{}-{index}",
                        entry.key
                    )))
                    .px_1()
                    .rounded_sm()
                    .border_1()
                    .border_color(cx.theme().colors().border)
                    .when(model && !suggests, Styled::border_dashed)
                    .child(Label::new(words).size(LabelSize::XSmall).color(color))
                    .when_some(probability.filter(|_| model), |this, probability| {
                        this.tooltip(Tooltip::text(format!(
                            "The model reads it at {probability:.2}"
                        )))
                    })
                    .into_any_element()
            })
            .collect()
    }

    /// An inbox entry's route (#570): who should answer it, with a question mark while a reading's
    /// is a suggestion, and the rule or the reading in a tooltip.
    fn render_route(&self, entry: &InboxEntry) -> Option<AnyElement> {
        let mark = entry.route?;
        let (words, tooltip) = match mark.source {
            RouteSource::Rule(rule) => (
                mark.route.words().to_string(),
                format!("Marley's rule: {rule}"),
            ),
            RouteSource::Reading => {
                let confidence = self
                    .routes
                    .get(&entry.key)
                    .and_then(|state| state.reading)
                    .and_then(|(_, confidence)| confidence);
                let suffix = if self.snapshot.rail.route_suggests {
                    "?"
                } else {
                    ""
                };
                (
                    format!("{}{suffix}", mark.route.words()),
                    confidence.map_or_else(
                        || "System One could not tell".to_string(),
                        |confidence| {
                            format!("System One: {} ({confidence:.2})", mark.route.words())
                        },
                    ),
                )
            }
        };
        Some(
            div()
                .id(SharedString::from(format!(
                    "marley-rail-inbox-route-{}",
                    entry.key
                )))
                .px_1()
                .child(
                    Label::new(words)
                        .size(LabelSize::XSmall)
                        .color(Color::Accent),
                )
                .tooltip(Tooltip::text(tooltip))
                .into_any_element(),
        )
    }

    /// Allow or Deny, or for a paused click Allow or Refuse, on the inbox entry `key` (#508).
    fn answer_inbox(&mut self, key: &str, allow: bool, window: &Window, cx: &mut Context<Self>) {
        // For the outcome of the risk use's row (#568) and the route's (#570); a held click has
        // neither.
        if let Some(risk) = self.risk.get_mut(key) {
            risk.answered = Some(if allow { "allowed" } else { "denied" });
        }
        if let Some(state) = self.routes.get_mut(key) {
            state.owner_seen = true;
        }
        match self.snapshot.inbox.get(key) {
            Some(InboxTarget::Thread { .. }) => self.answer_thread(key, allow, window, cx),
            Some(InboxTarget::Click(target)) => {
                let target = target.clone();
                if let Some(hub) = BrowserHub::try_global(cx) {
                    hub.update(cx, |hub, cx| hub.answer_pause(&target, allow, cx));
                }
            }
            Some(InboxTarget::Terminal(_) | InboxTarget::Harness(_)) | None => {}
        }
    }

    /// Ends the seat of each terminal that no longer runs the seat's agent (#547, #650): Claude Code
    /// left without a `SessionEnd`, or was killed, or Codex left the foreground. The refresh that
    /// ending the seats causes finds none to end.
    /// While a seat works, arms the refresh due when a row's `no update in N m` next changes; a
    /// timer that comes early, after a newer event, only arms the next one.
    fn note_claude_code(&mut self, rail: &RailSnapshot, window: &Window, cx: &mut Context<Self>) {
        let gone: Vec<u64> = rail
            .projects
            .iter()
            .flat_map(|project| &project.terminals)
            .filter(|terminal| {
                let seat_agent = cx
                    .try_global::<AgentEvents>()
                    .and_then(|events| events.seat(EntityId::from(terminal.id)))
                    .and_then(agent_events::seat_agent);
                seat_agent.is_some_and(|seat_agent| {
                    terminal.agent.is_none_or(|agent| agent.kind != seat_agent)
                })
            })
            .map(|terminal| terminal.id)
            .collect();
        agent_events::end(&gone, cx);
        let next = cx.try_global::<AgentEvents>().and_then(|events| {
            events.next_quiet_change(agent_events::now_ms(), no_update_after_ms(cx))
        });
        match next {
            None => self.minute_timer = None,
            Some(delay) if self.minute_timer.is_none() => {
                // A little past the moment, so the refresh sees the row changed.
                let due = cx
                    .background_executor()
                    .timer(delay + Duration::from_millis(100));
                self.minute_timer = Some(cx.spawn_in(window, async move |rail, cx| {
                    due.await;
                    rail.update_in(cx, |rail, window, cx| {
                        rail.minute_timer = None;
                        rail.refresh(window, cx);
                    })
                    .log_err();
                }));
            }
            Some(_) => {}
        }
    }

    /// Notes a change of the terminal or thread row that holds the window's focus, for the
    /// switcher's order, and forgets the rows that are gone. The switcher's own focus is no row,
    /// so opening it notes nothing.
    fn note_window_row(&mut self, rail: &RailSnapshot) {
        let row = marley_rail::window_row(rail);
        if row != self.window_row {
            if let Some(row) = &row {
                self.shown_count += 1;
                self.shown_at.insert(row.clone(), self.shown_count);
            }
            self.window_row = row;
        }
        let listed: HashSet<Selection> = marley_rail::switcher_rows(rail, |_| None)
            .iter()
            .map(SwitcherRow::selection)
            .collect();
        self.shown_at.retain(|row, _| listed.contains(row));
    }

    /// Lights the dot of each thread whose run ended since the last rebuild while it was not
    /// shown, and keeps it lit until the thread is shown.
    fn note_ended_runs(&mut self, snapshot: &mut Snapshot) {
        let shown = snapshot.shown_thread.as_deref();
        let mut statuses = HashMap::default();
        let mut noted = HashSet::default();
        for thread in snapshot
            .rail
            .projects
            .iter_mut()
            .flat_map(|project| project.threads.iter_mut())
        {
            thread.attention = marley_rail::thread_attention(
                self.thread_statuses.get(&thread.key).copied(),
                thread.status,
                shown == Some(thread.key.as_str()),
                self.noted_threads.contains(&thread.key),
            );
            if thread.attention {
                noted.insert(thread.key.clone());
            }
            statuses.insert(thread.key.clone(), thread.status);
        }
        self.thread_statuses = statuses;
        self.noted_threads = noted;
    }

    /// Notes a terminal's output, and for an agent terminal re-arms the refresh that reads it as
    /// waiting once the output stops.
    fn note_output(&mut self, view: EntityId, window: &mut Window, cx: &mut Context<Self>) {
        self.terminal_output
            .insert(view, cx.background_executor().now());
        self.refresh(window, cx);
        if self.snapshot.agent_terminals.contains(&view) {
            let quiet = cx.background_executor().timer(WAITING_AFTER);
            let timer = cx.spawn_in(window, async move |rail, cx| {
                quiet.await;
                rail.update_in(cx, Self::refresh).log_err();
            });
            self.quiet_timers.insert(view, timer);
        }
    }

    fn sync_subscriptions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Watched {
            workspaces,
            projects,
            views,
            browsers,
            panels,
            threads,
            agent_servers,
        } = self
            .multi_workspace
            .upgrade()
            .map(|multi_workspace| Watched::in_window(&multi_workspace, cx))
            .unwrap_or_default();
        let open: HashSet<EntityId> = views.iter().map(Entity::entity_id).collect();
        self.terminal_output.retain(|view, _| open.contains(view));
        self.quiet_timers.retain(|view, _| open.contains(view));
        // Rebuilt from what exists now: a restore replaces panes without reporting removals, and
        // whatever is left in the old maps is dropped, ending those subscriptions.
        self.workspace_subscriptions = resubscribe(
            &mut self.workspace_subscriptions,
            &workspaces,
            |workspace| {
                cx.subscribe_in(
                    workspace,
                    window,
                    |rail, _, _: &workspace::Event, window, cx| rail.refresh(window, cx),
                )
            },
        );
        self.project_subscriptions =
            resubscribe(&mut self.project_subscriptions, &projects, |project| {
                Self::follow_folders(project, window, cx)
            });
        self.terminal_subscriptions =
            resubscribe(&mut self.terminal_subscriptions, &views, |view| {
                [
                    cx.subscribe_in(
                        view,
                        window,
                        |rail, view, event: &terminal::Event, window, cx| {
                            if *event == terminal::Event::Wakeup {
                                rail.note_output(view.entity_id(), window, cx);
                            } else {
                                rail.refresh(window, cx);
                            }
                        },
                    ),
                    cx.subscribe_in(view, window, |rail, _, _: &ItemEvent, window, cx| {
                        rail.refresh(window, cx);
                    }),
                ]
            });
        self.follow_browsers(&browsers, window, cx);
        self.panel_subscriptions = resubscribe(&mut self.panel_subscriptions, &panels, |panel| {
            // The panel's own focus handle wraps whatever view it shows, so focus inside a
            // thread counts as focus in the panel.
            let focus_handle = panel.focus_handle(cx);
            [
                cx.subscribe_in(panel, window, |rail, _, _: &AgentPanelEvent, window, cx| {
                    rail.refresh(window, cx);
                }),
                cx.on_focus_in(&focus_handle, window, Self::refresh),
                cx.on_focus_out(&focus_handle, window, |rail, _, window, cx| {
                    rail.refresh(window, cx);
                }),
            ]
        });
        self.thread_subscriptions =
            resubscribe(&mut self.thread_subscriptions, &threads, |thread| {
                cx.subscribe_in(
                    thread,
                    window,
                    |rail, _, event: &AcpThreadEvent, window, cx| {
                        if changes_the_row(event) {
                            rail.refresh(window, cx);
                        }
                    },
                )
            });
        self.agent_server_subscriptions = resubscribe(
            &mut self.agent_server_subscriptions,
            &agent_servers,
            |store| {
                cx.subscribe_in(
                    store,
                    window,
                    |rail, _, _: &AgentServersUpdated, window, cx| {
                        rail.refresh(window, cx);
                    },
                )
            },
        );
        if self.thread_store_subscription.is_none()
            && let Some(store) = ThreadMetadataStore::try_global(cx)
        {
            self.thread_store_subscription =
                Some(cx.observe_in(&store, window, |rail, _, window, cx| {
                    rail.refresh(window, cx);
                }));
        }
    }

    /// Rebuilds the rail after each change to `project`'s folders, and to its repositories'
    /// worktrees and branches, which make the worktree rows (#510).
    fn follow_folders(
        project: &Entity<Project>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> [Subscription; 2] {
        let git_store = project.read(cx).git_store().clone();
        [
            cx.subscribe_in(
                project,
                window,
                |rail, project, event: &project::Event, window, cx| {
                    // The project reports a folder before the `MultiWorkspace` rekeys the
                    // project's group. A rebuild in between would find the project in no group and
                    // forget its rows' recency and attention dots.
                    if changes_the_folders(event) {
                        cx.defer_in(window, Self::refresh);
                    } else if let project::Event::WorktreeUpdatedEntries(worktree, changes) = event
                    {
                        rail.note_icon_files(project, *worktree, changes, cx);
                    }
                },
            ),
            cx.subscribe_in(
                &git_store,
                window,
                |rail, _, event: &GitStoreEvent, window, cx| {
                    if changes_the_worktrees(event) {
                        cx.defer_in(window, Self::refresh);
                    } else if matches!(
                        event,
                        GitStoreEvent::RepositoryUpdated(_, RepositoryEvent::StatusesChanged, _)
                    ) {
                        // A save changes the counts but no row, so no rebuild (#531).
                        rail.git_edited = Some(cx.background_executor().now());
                        cx.defer_in(window, |rail, window, cx| {
                            rail.follow_project_git(window, cx);
                        });
                    }
                },
            ),
        ]
    }

    /// Shows `workspace` in the window. The rows hold their entities weakly, so the project may
    /// have closed since the rail last drew.
    fn activate_workspace(
        &self,
        workspace: &WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<Entity<Workspace>> {
        let workspace = workspace.upgrade().context("the project was closed")?;
        self.multi_workspace
            .update(cx, |multi_workspace, cx| {
                multi_workspace.activate(workspace.clone(), None, window, cx);
            })
            .map(|()| workspace)
    }

    fn activate_terminal(
        &self,
        workspace: &WeakEntity<Workspace>,
        view: &WeakEntity<TerminalView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let view = view.upgrade().context("the terminal was closed")?;
        let workspace = self.activate_workspace(workspace, window, cx)?;
        workspace.update(cx, |workspace, cx| {
            workspace.activate_item(&view, true, true, window, cx)
        });
        view.update(cx, TerminalView::clear_bell);
        Ok(())
    }

    /// Follows each Browser tab's item events, and the hub once something made it: its pages'
    /// titles, icons and counts (#504). The hub is read without being made, which would start
    /// the browser.
    fn follow_browsers(
        &mut self,
        browsers: &[Entity<BrowserView>],
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        self.browser_subscriptions =
            resubscribe(&mut self.browser_subscriptions, browsers, |view| {
                cx.subscribe_in(view, window, |rail, _, _: &ItemEvent, window, cx| {
                    rail.refresh(window, cx);
                })
            });
        if self.hub_subscription.is_none()
            && let Some(hub) = BrowserHub::try_global(cx)
        {
            self.hub_subscription =
                Some(
                    cx.subscribe_in(&hub, window, |rail, _, event: &BrowserEvent, window, cx| {
                        if matches!(
                            event,
                            BrowserEvent::PageInfoChanged { .. }
                                | BrowserEvent::PageStatusChanged { .. }
                                | BrowserEvent::PageOpened { .. }
                                | BrowserEvent::PageClosed { .. }
                        ) {
                            rail.refresh(window, cx);
                        }
                    }),
                );
        }
    }

    /// Shows a Browser tab's project and brings the tab forward with the focus (#504).
    fn activate_browser(
        &self,
        workspace: &WeakEntity<Workspace>,
        view: &WeakEntity<BrowserView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let view = view.upgrade().context("the Browser tab was closed")?;
        let workspace = self.activate_workspace(workspace, window, cx)?;
        workspace.update(cx, |workspace, cx| {
            workspace.activate_item(&view, true, true, window, cx)
        });
        Ok(())
    }

    /// Closes a Browser tab through its pane, as its tab's close does, which closes its page
    /// (#504).
    fn close_browser(
        workspace: &WeakEntity<Workspace>,
        view: &WeakEntity<BrowserView>,
        window: &mut Window,
        cx: &mut App,
    ) -> anyhow::Result<()> {
        let view = view.upgrade().context("the Browser tab was closed")?;
        let workspace = workspace.upgrade().context("the project was closed")?;
        let pane = workspace
            .read(cx)
            .pane_for(&view)
            .context("the Browser tab is in no pane")?;
        let closing = pane.update(cx, |pane, cx| {
            pane.close_item_by_id(view.entity_id(), SaveIntent::Close, window, cx)
        });
        closing.detach_and_prompt_err("Could not close the Browser tab", window, cx, |_, _, _| {
            None
        });
        Ok(())
    }

    /// Shows a terminal and starts Zed's rename on its tab, as the tab's own Rename does: the name
    /// is edited in the tab and kept on Enter.
    fn rename_terminal(
        &self,
        workspace: &WeakEntity<Workspace>,
        view: &WeakEntity<TerminalView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        self.activate_terminal(workspace, view, window, cx)?;
        let view = view.upgrade().context("the terminal was closed")?;
        view.update(cx, |view, cx| {
            view.rename_terminal(&RenameTerminal, window, cx);
        });
        Ok(())
    }

    /// Closes a terminal through its pane, as its tab's close does: Zed asks first while a task
    /// runs in it.
    fn close_terminal(
        workspace: &WeakEntity<Workspace>,
        view: &WeakEntity<TerminalView>,
        window: &mut Window,
        cx: &mut App,
    ) -> anyhow::Result<()> {
        let view = view.upgrade().context("the terminal was closed")?;
        let workspace = workspace.upgrade().context("the project was closed")?;
        let pane = workspace
            .read(cx)
            .pane_for(&view)
            .context("the terminal is in no pane")?;
        let closing = pane.update(cx, |pane, cx| {
            pane.close_item_by_id(view.entity_id(), SaveIntent::Close, window, cx)
        });
        closing.detach_and_prompt_err("Could not close the terminal", window, cx, |_, _, _| None);
        Ok(())
    }

    /// Opens a terminal in `workspace`'s center, where Zed's own New Terminal would start one: the
    /// workspace's own project directory (a linked worktree's, not its main repository's).
    fn new_terminal(
        &self,
        workspace: &WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let workspace = self.activate_workspace(workspace, window, cx)?;
        let factory = agents::launcher(cx).terminal_factory;
        workspace.update(cx, |workspace, cx| {
            let directory = terminal_view::default_working_directory(workspace, cx);
            TerminalPanel::add_center_terminal(workspace, window, cx, move |project, cx| {
                factory(project, directory, collections::HashMap::default(), cx)
            })
            .detach_and_prompt_err(
                "Could not open a terminal",
                window,
                cx,
                |_, _, _| None,
            );
        });
        Ok(())
    }

    /// Shows `workspace` and opens a blank page in a new Browser tab there, the focus in its
    /// address bar (#500).
    fn new_browser_tab(
        &self,
        workspace: &WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let workspace = self.activate_workspace(workspace, window, cx)?;
        workspace.update(cx, |workspace, cx| browser::new_tab(workspace, window, cx));
        Ok(())
    }

    /// The project's launch configs (#527), under a Launch header in the file's order, or one
    /// disabled entry naming the file and why it could not be read; nothing without the file.
    fn launch_entries(
        menu: ContextMenu,
        rail: &WeakEntity<Self>,
        workspace: &WeakEntity<Workspace>,
        cx: &App,
    ) -> ContextMenu {
        let Some(root) = workspace.upgrade().and_then(|workspace| {
            workspace
                .read(cx)
                .project()
                .read(cx)
                .visible_worktrees(cx)
                .next()
                .map(|worktree| worktree.read(cx).abs_path().to_path_buf())
        }) else {
            return menu;
        };
        match launch::configs(&root, cx) {
            None => menu,
            Some(Ok(configs)) if configs.is_empty() => menu,
            Some(Err(why)) => {
                let why: String = why.chars().take(90).collect();
                menu.separator().header("Launch").item(
                    ContextMenuEntry::new(format!("{}: {why}", launch::LAUNCH_FILE)).disabled(true),
                )
            }
            Some(Ok(configs)) => {
                configs
                    .into_iter()
                    .fold(menu.separator().header("Launch"), |menu, config| {
                        let (rail, workspace, root) =
                            (rail.clone(), workspace.clone(), root.clone());
                        let name = config.name.clone();
                        menu.entry(config.name, None, move |window, cx| {
                            rail.update(cx, |rail, cx| {
                                rail.launch(&workspace, root.clone(), name.clone(), window, cx)
                            })
                            .flatten()
                            .log_err();
                        })
                    })
            }
        }
    }

    /// Shows `workspace` and runs its launch config `name` there (#527).
    fn launch(
        &self,
        workspace: &WeakEntity<Workspace>,
        root: PathBuf,
        name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let workspace = self.activate_workspace(workspace, window, cx)?;
        launch::run(workspace.downgrade(), root, name, window, cx);
        Ok(())
    }

    /// Shows `workspace` and starts `kind` in a new center terminal there.
    fn new_agent(
        &self,
        workspace: &WeakEntity<Workspace>,
        kind: AgentKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let workspace = self.activate_workspace(workspace, window, cx)?;
        workspace.update(cx, |workspace, cx| {
            agents::start_cli(workspace, kind, window, cx);
        });
        Ok(())
    }

    /// Shows a thread: displays its workspace and opens the thread, focused, in the workspace's
    /// Agent Panel, which sits on the right in the Marley layout.
    fn open_thread(
        &self,
        key: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let thread = self
            .snapshot
            .threads
            .get(key)
            .context("the thread is no longer listed")?;
        let opening = ThreadToOpen {
            thread_id: thread.thread_id,
            agent: thread.agent.clone(),
            work_dirs: thread.work_dirs.clone(),
            title: thread.title.clone(),
        };
        self.open_thread_with(
            &thread.workspace.clone(),
            thread.closed.clone().as_ref(),
            opening,
            window,
            cx,
        )
    }

    /// Shows `workspace` and opens the thread in its Agent Panel, which unarchives an archived
    /// one (#616). A closed project's thread opens its own folders under the project's key first,
    /// as Zed's Threads Sidebar opens one, so a thread that ran in a linked worktree opens there
    /// (#617).
    fn open_thread_with(
        &self,
        workspace: &WeakEntity<Workspace>,
        closed: Option<&ProjectGroupKey>,
        thread: ThreadToOpen,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let closed = closed.map(|key| {
            let paths = if thread.work_dirs.paths().is_empty() {
                key.path_list().clone()
            } else {
                thread.work_dirs.clone()
            };
            (key, paths)
        });
        self.in_workspace(
            workspace,
            closed,
            move |workspace, window, cx| load_thread(workspace, thread, window, cx),
            window,
            cx,
        )
    }

    /// Runs `then` in a row's workspace, shown: at once, or for a row of a closed project (#617)
    /// once its folders `paths` have opened.
    fn in_workspace(
        &self,
        workspace: &WeakEntity<Workspace>,
        closed: Option<(&ProjectGroupKey, PathList)>,
        then: impl FnOnce(&mut Workspace, &mut Window, &mut Context<Workspace>) + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let Some((key, paths)) = closed else {
            let workspace = self.activate_workspace(workspace, window, cx)?;
            workspace.update(cx, |workspace, cx| then(workspace, window, cx));
            return Ok(());
        };
        let opening = self.open_closed(key, paths, window, cx);
        cx.spawn_in(window, async move |_, cx| {
            if let Some(workspace) = opening.await {
                workspace.update_in(cx, then).log_err();
            }
        })
        .detach();
        Ok(())
    }

    /// Deletes a thread once the user confirms (#616). Every Agent Panel of the window lets it go
    /// first, since a conversation it keeps would save the thread again; then its record goes, the
    /// worktrees its archive kept are cleaned up, and the agent deletes its session where its list
    /// of sessions can.
    fn delete_thread(
        &self,
        thread_id: ThreadId,
        title: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let answer = window.prompt(
            PromptLevel::Warning,
            &format!("Delete the thread “{title}”?"),
            Some("Its messages are deleted, not archived."),
            &["Delete Thread", "Cancel"],
            cx,
        );
        let multi_workspace = self.multi_workspace.clone();
        cx.spawn_in(window, async move |_, cx| {
            if answer.await.ok() != Some(0) {
                return anyhow::Ok(());
            }
            let found = cx.update(|_, cx| {
                ThreadMetadataStore::try_global(cx)
                    .and_then(|store| store.read(cx).entry(thread_id).cloned())
            })?;
            let connections = multi_workspace.update_in(cx, |multi_workspace, window, cx| {
                let panels: Vec<Entity<AgentPanel>> = multi_workspace
                    .workspaces()
                    .filter_map(|workspace| workspace.read(cx).panel::<AgentPanel>(cx))
                    .collect();
                let mut connections = None;
                for panel in panels {
                    panel.update(cx, |panel, cx| {
                        connections.get_or_insert_with(|| panel.connection_store().clone());
                        // Without a draft in its place: a delete should not start a thread.
                        panel.remove_thread_without_activating_draft(thread_id, window, cx);
                    });
                }
                connections
            })?;
            cx.update(|_, cx| {
                if let Some(store) = ThreadMetadataStore::try_global(cx) {
                    store.update(cx, |store, cx| store.delete(thread_id, cx));
                }
            })?;
            thread_worktree_archive::cleanup_thread_archived_worktrees(thread_id, cx).await;
            let (Some(found), Some(connections)) = (found, connections) else {
                return anyhow::Ok(());
            };
            let Some(session) = found.session_id else {
                return anyhow::Ok(());
            };
            let agent = Agent::from(found.agent_id);
            let connecting = cx.update(|_, cx| {
                let fs = <dyn Fs>::global(cx);
                connections.update(cx, |connections, cx| {
                    connections
                        .request_connection(
                            agent.clone(),
                            agent.server(fs, agent::ThreadStore::global(cx)),
                            cx,
                        )
                        .read(cx)
                        .wait_for_connection()
                })
            })?;
            let connected = connecting.await?;
            let deleting = cx.update(|_, cx| {
                connected
                    .connection
                    .session_list(cx)
                    .filter(|sessions| sessions.supports_delete())
                    .map(|sessions| sessions.delete_session(&session, cx))
            })?;
            if let Some(deleting) = deleting {
                deleting.await?;
            }
            anyhow::Ok(())
        })
        .detach_and_log_err(cx);
    }

    /// Shows `workspace` and starts a thread of `agent` in its Agent Panel.
    fn new_agent_thread(
        &self,
        workspace: &WeakEntity<Workspace>,
        agent: &AgentId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let workspace = self.activate_workspace(workspace, window, cx)?;
        workspace.update(cx, |workspace, cx| {
            agents::start_thread(workspace, agent, window, cx)
        })
    }

    /// Puts the keyboard's row where `to` says, from the rail as it stands.
    fn move_cursor(
        &mut self,
        to: impl FnOnce(&RailSnapshot) -> Selection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.cursor = Some(to(&self.snapshot.rail));
        self.refresh(window, cx);
    }

    /// Puts the keyboard's row on `row` with the focus in the rail, so Enter opens it (#604).
    fn mark_row(&mut self, row: Selection, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
        self.cursor = Some(row);
        self.refresh(window, cx);
    }

    fn select_next(&mut self, _: &SelectNext, window: &mut Window, cx: &mut Context<Self>) {
        self.move_cursor(|rail| marley_rail::step(rail, true), window, cx);
    }

    fn select_previous(&mut self, _: &SelectPrevious, window: &mut Window, cx: &mut Context<Self>) {
        self.move_cursor(|rail| marley_rail::step(rail, false), window, cx);
    }

    fn select_first(&mut self, _: &SelectFirst, window: &mut Window, cx: &mut Context<Self>) {
        self.move_cursor(marley_rail::first_row, window, cx);
    }

    fn select_last(&mut self, _: &SelectLast, window: &mut Window, cx: &mut Context<Self>) {
        self.move_cursor(marley_rail::last_row, window, cx);
    }

    /// Left: folds an open project, or climbs from a row to its project's header.
    fn select_parent(&mut self, _: &SelectParent, window: &mut Window, cx: &mut Context<Self>) {
        let selected = marley_rail::selection(&self.snapshot.rail);
        match selected {
            Selection::Project(index) => self.fold(index, true, window, cx),
            row => self.move_cursor(|rail| marley_rail::parent(rail, &row), window, cx),
        }
    }

    /// Right: unfolds a folded project.
    fn select_child(&mut self, _: &SelectChild, window: &mut Window, cx: &mut Context<Self>) {
        if let Selection::Project(index) = marley_rail::selection(&self.snapshot.rail) {
            self.fold(index, false, window, cx);
        }
    }

    /// Folds or unfolds the project at `index`, when it is not that way already and no filter
    /// decides what shows.
    fn fold(&mut self, index: usize, fold: bool, window: &mut Window, cx: &mut Context<Self>) {
        let expanded = self
            .snapshot
            .rail
            .projects
            .get(index)
            .is_some_and(|project| project.expanded);
        if let Some(group) = self.snapshot.groups.get(index)
            && expanded == fold
            && !self.snapshot.rail.filtering
        {
            if let Some(id) = group.group {
                groups::toggle_expanded(id, cx);
            } else {
                let key = group.key.clone();
                self.toggle_expanded(&key, window, cx);
            }
        }
    }

    /// `ctrl-f`: the filter takes focus, as Zed's action gives its own sidebar's.
    fn focus_filter(
        &mut self,
        _: &FocusSidebarFilter,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match self.shown_brain() {
            Some(brain) => {
                let search = brain.read(cx).search_focus(cx);
                window.focus(&search, cx);
            }
            None => window.focus(&self.filter_editor.focus_handle(cx), cx),
        }
    }

    /// Escape, as Zed's Threads Sidebar has it: it clears the filter, and from an empty filter
    /// goes back to the rows. With neither to do it passes on.
    fn cancel(&mut self, _: &Cancel, window: &mut Window, cx: &mut Context<Self>) {
        // The Brain view's own Escape passed on; the hidden filter stays as it is.
        if self.brain_shown() {
            cx.propagate();
        } else if self.snapshot.rail.filtering {
            self.clear_filter(window, cx);
        } else if self.filter_editor.focus_handle(cx).is_focused(window) {
            window.focus(&self.focus_handle, cx);
        } else {
            cx.propagate();
        }
    }

    fn clear_filter(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.filter_editor
            .update(cx, |editor, cx| editor.set_text("", window, cx));
    }

    /// Each edit rereads the rail and puts the keyboard on the first row that matched, as Zed's
    /// Threads Sidebar selects its first match.
    fn filter_edited(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.refresh(window, cx);
        if self.snapshot.rail.filtering {
            self.move_cursor(marley_rail::first_match, window, cx);
        }
    }

    /// Enter: what a click on the keyboard's row does.
    fn confirm(&mut self, _: &Confirm, window: &mut Window, cx: &mut Context<Self>) {
        let selection = marley_rail::selection(&self.snapshot.rail);
        self.open_row(selection, window, cx).log_err();
    }

    /// Opens a row as a click on it does.
    fn open_row(
        &self,
        selection: Selection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        match selection {
            Selection::None => Ok(()),
            Selection::Project(index) => match self.snapshot.groups.get(index) {
                Some(group) if group.closed => {
                    self.open_closed_project(&group.key, window, cx);
                    Ok(())
                }
                Some(group) => {
                    let workspace = group.workspace.clone();
                    self.activate_workspace(&workspace, window, cx).map(drop)
                }
                None => Ok(()),
            },
            Selection::Terminal(id) => {
                let terminal = self.snapshot.terminals.get(&id);
                let terminal =
                    terminal.map(|terminal| (terminal.workspace.clone(), terminal.view.clone()));
                terminal.map_or(Ok(()), |(workspace, view)| {
                    self.activate_terminal(&workspace, &view, window, cx)
                })
            }
            Selection::Browser(id) => {
                let browser = self.snapshot.browsers.get(&id);
                let browser =
                    browser.map(|browser| (browser.workspace.clone(), browser.view.clone()));
                browser.map_or(Ok(()), |(workspace, view)| {
                    self.activate_browser(&workspace, &view, window, cx)
                })
            }
            Selection::Thread(key) => self.open_thread(&key, window, cx),
            Selection::Port(port, pid) => {
                let found =
                    self.snapshot
                        .rail
                        .projects
                        .iter()
                        .enumerate()
                        .find_map(|(index, project)| {
                            let shown = project
                                .ports
                                .iter()
                                .find(|shown| shown.port == port && shown.pid == pid)?;
                            let group = self.snapshot.groups.get(index)?;
                            let closed = group.closed.then(|| group.key.clone());
                            Some((group.workspace.clone(), closed, shown.url.clone()))
                        });
                found.map_or(Ok(()), |(workspace, closed, url)| {
                    self.open_port(&workspace, closed.as_ref(), url, window, cx)
                })
            }
            Selection::Worktree(path) => self.open_worktree(&path, window, cx),
        }
    }

    /// Shows a worktree's workspace, or opens the worktree in the window as Zed's worktree
    /// picker does, when the window has it not open (#510).
    fn open_worktree(
        &self,
        path: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let entry = self
            .snapshot
            .worktrees
            .get(path)
            .context("the worktree is gone")?;
        if let Some(member) = &entry.member {
            return self.activate_workspace(member, window, cx).map(drop);
        }
        let workspace = entry.project.upgrade().context("the project was closed")?;
        let action = zed_actions::SwitchWorktree {
            path: entry.path.clone(),
            display_name: entry.name.clone(),
        };
        workspace.update(cx, |workspace, cx| {
            git_ui_core::worktree_service::handle_switch_worktree(
                workspace, &action, window, None, cx,
            );
        });
        Ok(())
    }

    /// What a worktree row's menu offers for merging its branch (#511), from what the rail kept:
    /// Merge only for a branch that records its base, a branch, in a repository Zed trusts and the
    /// Rustal workflow does not merge; else a line that says why not.
    fn merge_line(&self, path: &str, cx: &App) -> MergeLine {
        let Some(entry) = self.snapshot.worktrees.get(path) else {
            return MergeLine::Note("The worktree is gone".to_string());
        };
        if entry.branch.is_none() {
            return MergeLine::Note("On no branch: nothing to merge".to_string());
        }
        let trusted = entry
            .repository
            .as_ref()
            .and_then(WeakEntity::upgrade)
            .is_some_and(|repository| repository.read(cx).is_trusted());
        if !trusted {
            return MergeLine::Note("Merge waits until Zed trusts the repository".to_string());
        }
        let Some(state) = self.drift.get(path) else {
            return MergeLine::Note("Reading the branch…".to_string());
        };
        let Some(base) = state.base.as_deref().filter(|_| state.recorded) else {
            return MergeLine::Note("No base recorded".to_string());
        };
        if worktree_git::is_commit(base) {
            return MergeLine::Note(format!(
                "Started from commit {}: no branch to merge into",
                short_commit(base)
            ));
        }
        let ahead = state.drift.as_ref().map(|drift| drift.ahead);
        match (state.owner, ahead) {
            (MergeOwner::Workflow, Some(0)) => MergeLine::Note(format!(
                "Nothing ahead of {base}: the Rustal workflow merges here"
            )),
            (MergeOwner::Workflow, Some(ahead)) => MergeLine::Note(format!(
                "{} ahead of {base}: the Rustal workflow merges here",
                commits(ahead)
            )),
            (MergeOwner::Workflow, None) => {
                MergeLine::Note("The Rustal workflow merges here".to_string())
            }
            (MergeOwner::Marley, Some(0)) => {
                MergeLine::Note(format!("Nothing to merge into {base}"))
            }
            (MergeOwner::Marley, Some(ahead)) => {
                MergeLine::Merge(format!("Merge {} into {base}…", commits(ahead)))
            }
            // The drift could not be read; Merge's checks say what stands in the way.
            (MergeOwner::Marley, None) => MergeLine::Merge(format!("Merge into {base}…")),
        }
    }

    /// What a worktree row's menu offers for removing it (#589): Remove, except in a repository
    /// the Rustal workflow merges while the branch has commits its base lacks, since the workflow
    /// may be running in it.
    fn remove_line(&self, path: &str) -> RemoveLine {
        let Some(state) = self.drift.get(path) else {
            return RemoveLine::Remove;
        };
        match (state.owner, state.drift.as_ref().map(|drift| drift.ahead)) {
            (MergeOwner::Workflow, Some(0)) | (MergeOwner::Marley, _) => RemoveLine::Remove,
            (MergeOwner::Workflow, _) => RemoveLine::Note(
                "Remove waits until the Rustal workflow merges the branch".to_string(),
            ),
        }
    }

    /// Removes a worktree (#589): Zed's plan for it while it is open, what git has not committed,
    /// Zed's prompt, the worktree's own workspace removed (its terminals with it, Zed asking about
    /// unsaved files), Zed's `remove_root`, which checks Zed made it and deletes the folder, and
    /// then the branch when its commits are in a target. The outcome shows in a toast, a refusal
    /// or a failure in a prompt.
    fn remove_worktree(&self, path: &str, window: &Window, cx: &Context<Self>) {
        let name = self
            .snapshot
            .worktrees
            .get(path)
            .map_or_else(|| path.to_string(), |entry| entry.name.clone());
        let path = path.to_string();
        let failure = format!("Could not remove {name}");
        cx.spawn_in(window, async move |rail, cx| {
            let target = rail.read_with(cx, |rail, cx| rail.remove_target(&path, cx))??;
            let RemoveTarget {
                folder,
                main,
                branch,
                member,
                plan,
            } = target;
            let changes = cx
                .background_spawn({
                    let folder = folder.clone();
                    async move { worktree_git::uncommitted(&folder).await }
                })
                .await?;
            let (question, confirm) = if changes == 0 {
                (format!("Remove {name}?"), "Remove")
            } else {
                (
                    format!(
                        "Remove {name} and its {changes} uncommitted {}?",
                        if changes == 1 { "change" } else { "changes" }
                    ),
                    "Remove Anyway",
                )
            };
            let detail = format!(
                "Marley closes its terminals and deletes {}. Its branch goes too once its commits \
                 are in its base; otherwise it stays.",
                folder.display()
            );
            let answer = cx.update(|window, cx| {
                window.prompt(
                    PromptLevel::Warning,
                    &question,
                    Some(&detail),
                    &[confirm, "Cancel"],
                    cx,
                )
            })?;
            if !matches!(answer.await, Ok(0)) {
                return Ok(());
            }
            if let Some(member) = &member
                && !worktree_agents::tear_down(member, &name, cx).await?
            {
                return Ok(());
            }
            if let Some(member) = member {
                // Outside the rail's own update: the window's sidebar is the rail.
                let multi_workspace = rail.read_with(cx, |rail, _| rail.multi_workspace.clone())?;
                let removing = multi_workspace.update_in(cx, |multi_workspace, window, cx| {
                    multi_workspace.remove([member], RemovalIntent::KeepProject, window, cx)
                })?;
                if !removing.await? {
                    return Ok(());
                }
            }
            thread_worktree_archive::remove_root(plan, cx).await?;
            let end = match (main, branch) {
                (Some(main), Some(branch)) => {
                    let end = cx
                        .background_spawn(async move {
                            let base = worktree_git::recorded_base(&main, &branch)
                                .await
                                .log_err()
                                .flatten();
                            let end =
                                worktree_git::end_branch(&main, &branch, base.as_deref()).await;
                            (branch, end)
                        })
                        .await;
                    Some(end)
                }
                _ => None,
            };
            let message = match end {
                Some((branch, Ok(BranchEnd::Deleted))) => {
                    format!("Removed {name} and its branch {branch}")
                }
                Some((branch, Ok(BranchEnd::Kept(why)))) => {
                    format!("Removed {name}. Kept its branch {branch}: {why}")
                }
                Some((branch, Err(error))) => {
                    format!("Removed {name}. Kept its branch {branch}: {error:#}")
                }
                None => format!("Removed {name}"),
            };
            rail.update(cx, |rail, cx| rail.show_toast(message, cx))?;
            anyhow::Ok(())
        })
        .detach_and_prompt_err(&failure, window, cx, |_, _, _| None);
    }

    /// What a removal of the worktree at `path` works on (#589): its folder, main checkout, branch
    /// and own workspace, and Zed's plan, which exists only for a worktree Zed made that an open
    /// project holds.
    fn remove_target(&self, path: &str, cx: &App) -> anyhow::Result<RemoveTarget> {
        let entry = self
            .snapshot
            .worktrees
            .get(path)
            .context("the worktree is gone")?;
        let multi_workspace = self
            .multi_workspace
            .upgrade()
            .context("the window is closing")?;
        let workspaces: Vec<Entity<Workspace>> =
            multi_workspace.read(cx).workspaces().cloned().collect();
        let plan = thread_worktree_archive::build_root_plan(&entry.path, None, &workspaces, cx)
            .context(
                "Marley removes a worktree Zed made while a project has it open; open it first, \
                 or remove it with git worktree remove",
            )?;
        Ok(RemoveTarget {
            folder: entry.path.clone(),
            main: entry.main.clone(),
            branch: entry.branch.clone(),
            member: entry.member.as_ref().and_then(WeakEntity::upgrade),
            plan,
        })
    }

    /// Opens Zed's branch diff of a worktree in its own workspace (#511), against the base its
    /// branch recorded, else its repository's default branch, as `git: diff branch` does. A
    /// worktree the window has not open opens first, as its row's click opens it.
    fn review_worktree(&self, path: &str, window: &Window, cx: &Context<Self>) {
        let entry = self.snapshot.worktrees.get(path);
        let recorded = match entry
            .and_then(|entry| Some((entry.main.clone()?, entry.branch.clone()?)))
        {
            Some((main, branch)) => cx
                .background_spawn(async move { worktree_git::recorded_base(&main, &branch).await }),
            None => Task::ready(Ok(None)),
        };
        let repository = entry.and_then(|entry| entry.repository.clone());
        let path = path.to_string();
        cx.spawn_in(window, async move |rail, cx| {
            let base = if let Some(base) = recorded.await? {
                base
            } else {
                let repository = repository
                    .as_ref()
                    .and_then(WeakEntity::upgrade)
                    .context("the worktree's repository is gone")?;
                let default =
                    repository.update(cx, |repository, _| repository.default_branch(true));
                default
                    .await??
                    .context(
                        "its branch records no base, and the repository has no default branch",
                    )?
                    .to_string()
            };
            rail.update_in(cx, |rail, window, cx| {
                rail.start_review(path, base, window, cx)
            })?
        })
        .detach_and_prompt_err("Could not review the worktree", window, cx, |_, _, _| None);
    }

    /// Keeps a worktree's Review until its workspace is in the window (#511), opening the
    /// worktree when the window has it not open, with no first terminal, so the diff is its item;
    /// and deploys it at once when it is open.
    fn start_review(
        &mut self,
        path: String,
        base: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let open = self
            .snapshot
            .worktrees
            .get(&path)
            .context("the worktree is gone")?
            .member
            .is_some();
        if !open {
            worktree_agents::skip_seed(PathBuf::from(&path), cx);
            self.open_worktree(&path, window, cx)?;
        }
        let until = cx.background_executor().now() + REVIEW_WAIT;
        self.pending_review = Some(PendingReview { path, base, until });
        self.take_review(window, cx);
        Ok(())
    }

    /// Deploys the pending Review once its worktree's workspace is in the window (#511): Zed's
    /// branch diff there, with the worktree's own repository. A wait past `REVIEW_WAIT` is
    /// dropped.
    fn take_review(&mut self, window: &Window, cx: &mut Context<Self>) {
        let Some(pending) = &self.pending_review else {
            return;
        };
        if cx.background_executor().now() > pending.until {
            worktree_agents::drop_seed_skip(Path::new(&pending.path), cx);
            self.pending_review = None;
            return;
        }
        let member = self
            .snapshot
            .worktrees
            .get(&pending.path)
            .and_then(|entry| entry.member.as_ref())
            .and_then(WeakEntity::upgrade);
        let Some(member) = member else {
            return;
        };
        // The worktree's repository shows up a moment after its workspace does.
        let Some(repository) = member_git(&member, cx).and_then(|git| git.repository.upgrade())
        else {
            return;
        };
        let Some(pending) = self.pending_review.take() else {
            return;
        };
        worktree_agents::drop_seed_skip(Path::new(&pending.path), cx);
        let member = member.downgrade();
        // Outside the rebuild, which may run while another entity reports its events.
        cx.defer_in(window, move |rail, window, cx| {
            let shown = rail
                .activate_workspace(&member, window, cx)
                .map(|workspace| {
                    workspace.update(cx, |workspace, cx| {
                        let project = workspace.project().clone();
                        BranchDiff::deploy_branch_diff_with_base_ref(
                            workspace,
                            project,
                            repository,
                            pending.base.into(),
                            None,
                            window,
                            cx,
                        );
                    });
                });
            Task::ready(shown).detach_and_prompt_err(
                "Could not review the worktree",
                window,
                cx,
                |_, _, _| None,
            );
        });
    }

    /// Merges a worktree's branch into its base in the main checkout (#511): [`Self::merge_target`]'s
    /// checks, [`worktree_git::ready_to_merge`] in the background, Zed's prompt, both again, and
    /// the merge. A merge shows in a toast, a refusal or a failure in a prompt.
    fn merge_worktree(&self, path: &str, window: &Window, cx: &Context<Self>) {
        let name = self
            .snapshot
            .worktrees
            .get(path)
            .map_or_else(|| path.to_string(), |entry| entry.name.clone());
        let path = path.to_string();
        let fs = <dyn Fs>::global(cx);
        cx.spawn_in(window, async move |rail, cx| {
            let (main, worktree, branch) =
                rail.read_with(cx, |rail, cx| rail.merge_target(&path, cx))??;
            let (base, ahead) = cx
                .background_spawn({
                    let (main, worktree, branch, fs) =
                        (main.clone(), worktree.clone(), branch.clone(), Arc::clone(&fs));
                    async move {
                        worktree_git::ready_to_merge(&main, &worktree, &branch, fs.as_ref()).await
                    }
                })
                .await?;
            let question = format!("Merge {} into {base}?", commits(ahead));
            let detail = format!(
                "Marley merges {branch} into {base} with a merge commit in {}. Nothing is pushed.",
                main.display()
            );
            let answer = cx.update(|window, cx| {
                window.prompt(
                    PromptLevel::Info,
                    &question,
                    Some(&detail),
                    &["Merge", "Cancel"],
                    cx,
                )
            })?;
            if !matches!(answer.await, Ok(0)) {
                return Ok(());
            }
            rail.read_with(cx, |rail, cx| rail.merge_target(&path, cx))??;
            let merged = cx
                .background_spawn(async move {
                    let (base_now, ahead_now) =
                        worktree_git::ready_to_merge(&main, &worktree, &branch, fs.as_ref()).await?;
                    anyhow::ensure!(
                        base_now == base && ahead_now == ahead,
                        "{branch} or {base} moved while Marley asked, so nothing was merged; choose \
                         Merge again to see what it merges"
                    );
                    let commit = worktree_git::merge(&main, &branch, &base).await?;
                    anyhow::Ok(format!(
                        "Merged {} of {branch} into {base}: {commit}. Nothing was pushed.",
                        commits(ahead)
                    ))
                })
                .await?;
            rail.update(cx, |rail, cx| rail.show_toast(merged, cx))?;
            anyhow::Ok(())
        })
        .detach_and_prompt_err(&format!("Could not merge {name}"), window, cx, |_, _, _| None);
    }

    /// What a merge of a worktree's branch runs on (#511): its main checkout, its folder and its
    /// branch, once Zed trusts the repository, whose hooks a merge runs, and no project in the
    /// window holds an unsaved change to a file of the main checkout, which the merge rewrites.
    fn merge_target(&self, path: &str, cx: &App) -> anyhow::Result<(PathBuf, PathBuf, String)> {
        let entry = self
            .snapshot
            .worktrees
            .get(path)
            .context("the worktree is gone")?;
        let branch = entry
            .branch
            .clone()
            .context("the worktree is on no branch, so there is nothing to merge")?;
        let main = entry
            .main
            .clone()
            .context("the worktree's repository has no main checkout")?;
        let trusted = entry
            .repository
            .as_ref()
            .and_then(WeakEntity::upgrade)
            .is_some_and(|repository| repository.read(cx).is_trusted());
        anyhow::ensure!(
            trusted,
            "Zed does not trust the repository yet, and a merge runs its hooks"
        );
        let unsaved = self.unsaved_in(&main, cx);
        anyhow::ensure!(
            unsaved.is_empty(),
            "the main checkout has unsaved changes in {}; save or discard them first",
            unsaved.join(", ")
        );
        Ok((main, entry.path.clone(), branch))
    }

    /// The files of the main checkout `main` with unsaved changes in the window's projects, as
    /// paths inside it; the files of its linked worktrees the rail lists are theirs.
    fn unsaved_in(&self, main: &Path, cx: &App) -> Vec<String> {
        let linked: Vec<&Path> = self
            .snapshot
            .worktrees
            .values()
            .filter(|entry| entry.main.as_deref() == Some(main))
            .map(|entry| entry.path.as_path())
            .collect();
        let Some(multi_workspace) = self.multi_workspace.upgrade() else {
            return Vec::new();
        };
        let mut files = Vec::new();
        for workspace in multi_workspace.read(cx).workspaces() {
            let project = workspace.read(cx).project().read(cx);
            for project_path in project.dirty_buffers(cx) {
                let Some(absolute) = project.absolute_path(&project_path, cx) else {
                    continue;
                };
                if linked.iter().any(|path| absolute.starts_with(path)) {
                    continue;
                }
                if let Ok(inside) = absolute.strip_prefix(main) {
                    let file = inside.display().to_string();
                    if !files.contains(&file) {
                        files.push(file);
                    }
                }
            }
        }
        files
    }

    /// Shows `message` in a toast in the displayed workspace (#511).
    fn show_toast(&self, message: String, cx: &mut App) {
        let Some(multi_workspace) = self.multi_workspace.upgrade() else {
            return;
        };
        let workspace = multi_workspace.read(cx).workspace().clone();
        workspace.update(cx, |workspace, cx| {
            workspace.show_toast(
                Toast::new(NotificationId::unique::<WorktreeMerge>(), message),
                cx,
            );
        });
    }

    /// Shows `workspace` and opens `url` there in a Browser tab of its project, or brings forward
    /// the tab already on it (#521); a closed project's opens first (#617).
    fn open_port(
        &self,
        workspace: &WeakEntity<Workspace>,
        closed: Option<&ProjectGroupKey>,
        url: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let closed = closed.map(|key| (key, key.path_list().clone()));
        self.in_workspace(
            workspace,
            closed,
            move |workspace, window, cx| {
                browser::open_url_tab(workspace, url, window, cx);
            },
            window,
            cx,
        )
    }

    /// Stops the server on `port` (#521) once a scan made now finds process `pid` listening there
    /// still; why not, when it could not, shows as a toast in the window's workspace.
    fn stop_port(&self, port: u16, pid: u32, window: &Window, cx: &Context<Self>) {
        let stop = ports::stop(port, pid, cx);
        let multi_workspace = self.multi_workspace.clone();
        cx.spawn_in(window, async move |_, cx| {
            let toast = match stop.await {
                Ok(Stop::Signalled(Stopped::Sent | Stopped::Gone) | Stop::Unit(_)) => return,
                Ok(Stop::Signalled(Stopped::NotListening)) => Toast::new(
                    NotificationId::unique::<Ports>(),
                    format!(
                        "Process {pid} no longer listens on port {port}, so Marley left it alone."
                    ),
                ),
                Ok(Stop::Refused { service, reason }) => refused_toast(&service, &reason),
                Err(error) => Toast::new(
                    NotificationId::unique::<Ports>(),
                    format!("Could not stop the server on port {port}: {error:#}"),
                ),
            };
            let shown = multi_workspace
                .read_with(cx, |multi_workspace, _| multi_workspace.workspace().clone());
            if let Ok(workspace) = shown {
                workspace.update(cx, |workspace, cx| workspace.show_toast(toast, cx));
            }
        })
        .detach();
    }

    /// Stops the container that publishes `port` (#614), or says why it could not.
    fn stop_container_port(&self, port: u16, window: &Window, cx: &Context<Self>) {
        let Some(container) = ports::container_at(port, cx) else {
            return;
        };
        let stop = ports::stop_container(&container, port, cx);
        let multi_workspace = self.multi_workspace.clone();
        cx.spawn_in(window, async move |_, cx| {
            let ContainerStop::Refused { command, reason } = stop.await else {
                return;
            };
            let toast = container_refused_toast(command, &reason);
            let shown = multi_workspace
                .read_with(cx, |multi_workspace, _| multi_workspace.workspace().clone());
            if let Ok(workspace) = shown {
                workspace.update(cx, |workspace, cx| workspace.show_toast(toast, cx));
            }
        })
        .detach();
    }

    /// Restarts the service that listens on `port` in the project `group` (#615), keeping its row
    /// while its port is quiet, or says why it could not.
    fn restart_port(
        &self,
        group: &ProjectGroupKey,
        port: u16,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let Some(service) = ports::begin_restart(group, port, cx) else {
            return;
        };
        let unit = service.unit.clone();
        let restart = ports::restart_service(service, cx);
        let multi_workspace = self.multi_workspace.clone();
        cx.spawn_in(window, async move |_, cx| {
            let toast = match restart.await {
                Ok(ServiceAction::Done) => return,
                Ok(ServiceAction::Refused { command, reason }) => {
                    let message = format!(
                        "Could not restart the service {unit}: {reason}. To restart it yourself, \
                         run: {command}"
                    );
                    Toast::new(NotificationId::unique::<Ports>(), message).on_click(
                        "Copy Command",
                        move |_, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(command.clone()));
                        },
                    )
                }
                Err(error) => Toast::new(
                    NotificationId::unique::<Ports>(),
                    format!("Could not restart the service {unit}: {error:#}"),
                ),
            };
            let shown = multi_workspace
                .read_with(cx, |multi_workspace, _| multi_workspace.workspace().clone());
            if let Ok(workspace) = shown {
                workspace.update(cx, |workspace, cx| workspace.show_toast(toast, cx));
            }
        })
        .detach();
    }

    /// Stops a service itself, for a row whose process is gone (#615).
    fn stop_service_port(&self, service: PortService, window: &Window, cx: &Context<Self>) {
        let service = Service {
            unit: service.unit,
            user: service.user,
        };
        let stop = ports::stop_service(service, cx);
        let multi_workspace = self.multi_workspace.clone();
        cx.spawn_in(window, async move |_, cx| {
            let toast = match stop.await {
                Ok(Stop::Refused { service, reason }) => refused_toast(&service, &reason),
                Ok(_) => return,
                Err(error) => Toast::new(
                    NotificationId::unique::<Ports>(),
                    format!("Could not stop the service: {error:#}"),
                ),
            };
            let shown = multi_workspace
                .read_with(cx, |multi_workspace, _| multi_workspace.workspace().clone());
            if let Ok(workspace) = shown {
                workspace.update(cx, |workspace, cx| workspace.show_toast(toast, cx));
            }
        })
        .detach();
    }

    /// Opens the service's journal, followed, in a new terminal of the row's project (#615), shown;
    /// a closed project's opens first (#617).
    fn show_logs(
        &self,
        workspace: &WeakEntity<Workspace>,
        closed: Option<&ProjectGroupKey>,
        service: &PortService,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let unit = ShellKind::Posix
            .try_quote(&service.unit)
            .map_or_else(|| service.unit.clone(), Cow::into_owned);
        let command = if service.user {
            format!("journalctl --user -u {unit} -f")
        } else {
            format!("journalctl -u {unit} -f")
        };
        let closed = closed.map(|key| (key, key.path_list().clone()));
        self.in_workspace(
            workspace,
            closed,
            move |workspace, window, cx| {
                // The project's own folder, as a terminal opened from its `+` would be.
                let folder = workspace
                    .project()
                    .read(cx)
                    .visible_worktrees(cx)
                    .next()
                    .map(|worktree| worktree.read(cx).abs_path().to_path_buf());
                agents::start_in_terminal(
                    workspace,
                    folder,
                    None,
                    Some(marley_agent::send_payload(&command)),
                    None,
                    window,
                    cx,
                )
                .detach_and_log_err(cx);
            },
            window,
            cx,
        )
    }

    /// The switcher's rows, from the window as the rail last read it.
    fn switcher_entries(&self) -> Vec<SwitcherEntry> {
        let rail = &self.snapshot.rail;
        let project_name = |index: usize| {
            rail.projects
                .get(index)
                .map(|project| SharedString::from(project.name.clone()))
        };
        marley_rail::switcher_rows(rail, |row| self.shown_at.get(row).copied())
            .into_iter()
            .filter_map(|row| match row {
                SwitcherRow::Terminal(row) => {
                    let project = project_name(row.project)?;
                    Some(SwitcherEntry::Terminal { row, project })
                }
                SwitcherRow::Thread(row) => {
                    let project = project_name(row.project)?;
                    let icon = self.snapshot.threads.get(&row.key)?.icon.clone();
                    Some(SwitcherEntry::Thread { row, project, icon })
                }
            })
            .collect()
    }

    /// Closes the switcher and opens what it chose, or after Escape gives focus back.
    fn switcher_ended(
        &mut self,
        event: &SwitcherEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let return_focus = self.switcher.take().and_then(|open| open.return_focus);
        self.multi_workspace
            .update(cx, |multi_workspace, cx| {
                multi_workspace.set_sidebar_overlay(None, cx);
            })
            .log_err();
        match event {
            SwitcherEvent::Confirmed(selection) => {
                self.open_row(selection.clone(), window, cx).log_err();
            }
            SwitcherEvent::Cancelled { restore_focus } => {
                if let Some(focus) = return_focus.filter(|_| *restore_focus) {
                    window.focus(&focus, cx);
                }
            }
        }
    }

    /// Removes a project from the window, as Zed's own sidebar does: its workspaces close, asking
    /// first about unsaved work, and its browser stops (#507).
    fn remove_project(&self, key: &ProjectGroupKey, window: &mut Window, cx: &mut Context<Self>) {
        let removed = self.multi_workspace.update(cx, |multi_workspace, cx| {
            multi_workspace.remove_project_group(key, window, cx)
        });
        if let Some(task) = removed.log_err() {
            task.detach_and_log_err(cx);
        }
    }

    /// Opens a project the window lists but holds no workspace of (#606).
    fn open_closed_project(
        &self,
        key: &ProjectGroupKey,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_closed(key, key.path_list().clone(), window, cx)
            .detach();
    }

    /// Opens the folders `paths` of a project the window lists but holds no workspace of (#606),
    /// as Zed's Threads Sidebar opens one: Zed restores the workspace it saved for those folders,
    /// and a remote project connects through Zed's connection modal first. The workspace once it
    /// is open and shown; why not, when it could not open, shows as a toast.
    fn open_closed(
        &self,
        key: &ProjectGroupKey,
        paths: PathList,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<Option<Entity<Workspace>>> {
        let Some(multi_workspace) = self.multi_workspace.upgrade() else {
            return Task::ready(None);
        };
        let shown = multi_workspace.read(cx).workspace().clone();
        let modal_workspace = shown.clone();
        let opened = multi_workspace.update(cx, |multi_workspace, cx| {
            multi_workspace.find_or_create_workspace(
                paths,
                key.host(),
                Some(key.clone()),
                move |options, window, cx| {
                    remote_connection::connect_with_modal(&modal_workspace, options, window, cx)
                },
                None,
                OpenMode::Activate,
                None,
                window,
                cx,
            )
        });
        let name = key.display_name(&HashMap::default());
        cx.spawn_in(window, async move |_, cx| {
            let result = opened.await;
            remote_connection::dismiss_connection_modal(&shown, cx);
            match result {
                Ok(workspace) => Some(workspace),
                Err(error) => {
                    let message = format!("Could not open {name}: {error:#}");
                    shown.update(cx, |workspace, cx| {
                        workspace.show_toast(
                            Toast::new(NotificationId::unique::<ClosedProject>(), message),
                            cx,
                        );
                    });
                    None
                }
            }
        })
    }

    /// Moves a project one place up or down in the rail's order, the one dragging sets (#602),
    /// past a projectless group as well as a project.
    fn move_project(
        &mut self,
        key: &ProjectGroupKey,
        up: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let headers: Vec<String> = self
            .snapshot
            .groups
            .iter()
            .map(order::group_place)
            .collect();
        let place = order::header_place(key, None);
        let Some(at) = headers.iter().position(|header| *header == place) else {
            return;
        };
        let neighbour = if up { at.checked_sub(1) } else { Some(at + 1) };
        let Some(neighbour) = neighbour.and_then(|neighbour| headers.get(neighbour)) else {
            return;
        };
        self.saved_order.headers = marley_rail::move_to(&headers, &place, neighbour, up);
        self.order_changed(window, cx);
    }

    /// Opens the prompt for a new projectless group's name (#600).
    fn new_group(&self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(multi_workspace) = self.multi_workspace.upgrade() else {
            return;
        };
        let workspace = multi_workspace.read(cx).workspace().clone();
        groups::prompt(&workspace, self.multi_workspace.clone(), None, window, cx);
    }

    /// Opens the prompt for the projectless group `group`'s new name, holding `name` (#600).
    fn rename_group(&self, group: Uuid, name: String, window: &mut Window, cx: &mut Context<Self>) {
        let Some(multi_workspace) = self.multi_workspace.upgrade() else {
            return;
        };
        let workspace = multi_workspace.read(cx).workspace().clone();
        groups::prompt(
            &workspace,
            self.multi_workspace.clone(),
            Some((group, name)),
            window,
            cx,
        );
    }

    /// Removes the projectless group `group` and its workspace, once Zed's prompts and #550's
    /// question about a working agent let its items close.
    fn remove_group(
        &self,
        group: Uuid,
        workspace: &WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(workspace) = workspace.upgrade() else {
            groups::forget(group, cx);
            return;
        };
        let removed = self.multi_workspace.update(cx, |multi_workspace, cx| {
            multi_workspace.remove([workspace], RemovalIntent::CloseProject, window, cx)
        });
        let Some(removed) = removed.log_err() else {
            return;
        };
        cx.spawn(async move |_, cx| {
            if removed.await? {
                cx.update(|cx| groups::forget(group, cx));
            }
            anyhow::Ok(())
        })
        .detach_and_log_err(cx);
    }

    /// Runs `action` with the window's Home group, making the group first when the window has
    /// none (#600).
    fn in_home(
        &self,
        action: impl FnOnce(
            &Self,
            &WeakEntity<Workspace>,
            &mut Window,
            &mut Context<Self>,
        ) -> anyhow::Result<()>
        + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let multi_workspace = self
            .multi_workspace
            .upgrade()
            .context("the window is gone")?;
        let home = groups::groups_of(multi_workspace.read(cx), cx)
            .into_iter()
            .find(|(group, _)| group.home);
        if let Some((_, workspace)) = home {
            return action(self, &workspace.downgrade(), window, cx);
        }
        let rail = cx.entity().downgrade();
        groups::make(
            &multi_workspace,
            "",
            true,
            move |workspace, window, cx| {
                rail.update(cx, |rail, cx| {
                    action(rail, &workspace.downgrade(), window, cx)
                })
                .flatten()
                .log_err();
            },
            window,
            cx,
        );
        Ok(())
    }

    /// A right-click on the rail's empty space (#600). The rail's own focus on a mouse-down would
    /// take the keyboard from the menu, as `ui::right_click_menu` keeps it from doing.
    fn empty_space_menu(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.stop_propagation();
        window.prevent_default();
        self.deploy_empty_menu(event.position, window, cx);
    }

    /// Opens the empty space's menu at `position`: New Group…, then what a project's `+` makes,
    /// made in the window's Home group (#600).
    fn deploy_empty_menu(
        &mut self,
        position: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let rail = cx.entity().downgrade();
        let search_path = agents::launcher(cx).search_path;
        let menu = ContextMenu::build(window, cx, move |menu, _, _| {
            let group_rail = rail.clone();
            let terminal_rail = rail.clone();
            let browser_rail = rail.clone();
            let menu = menu
                .entry("New Group…", None, move |window, cx| {
                    group_rail
                        .update(cx, |rail, cx| rail.new_group(window, cx))
                        .log_err();
                })
                .separator()
                .entry("New Terminal", None, move |window, cx| {
                    terminal_rail
                        .update(cx, |rail, cx| {
                            rail.in_home(
                                |rail, workspace, window, cx| {
                                    rail.new_terminal(workspace, window, cx)
                                },
                                window,
                                cx,
                            )
                        })
                        .flatten()
                        .log_err();
                })
                .entry("New Browser Tab", None, move |window, cx| {
                    browser_rail
                        .update(cx, |rail, cx| {
                            rail.in_home(
                                |rail, workspace, window, cx| {
                                    rail.new_browser_tab(workspace, window, cx)
                                },
                                window,
                                cx,
                            )
                        })
                        .flatten()
                        .log_err();
                });
            let agents = agents::installed_clis(search_path.as_deref());
            if agents.is_empty() {
                return menu;
            }
            agents
                .into_iter()
                .fold(menu.separator().header("Agent CLIs"), |menu, kind| {
                    let rail = rail.clone();
                    menu.item(
                        ContextMenuEntry::new(kind.display_name())
                            .icon(agents::cli_icon(kind))
                            .icon_color(Color::Muted)
                            .handler(move |window, cx| {
                                rail.update(cx, |rail, cx| {
                                    rail.in_home(
                                        move |rail, workspace, window, cx| {
                                            rail.new_agent(workspace, kind, window, cx)
                                        },
                                        window,
                                        cx,
                                    )
                                })
                                .flatten()
                                .log_err();
                            }),
                    )
                })
        });
        let subscription = cx.subscribe_in(&menu, window, |rail, _, _: &DismissEvent, _, cx| {
            rail.empty_menu = None;
            cx.notify();
        });
        window.focus(&menu.focus_handle(cx), cx);
        self.empty_menu = Some((menu, position, subscription));
        cx.notify();
    }

    fn toggle_expanded(
        &mut self,
        key: &ProjectGroupKey,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.multi_workspace
            .update(cx, |multi_workspace, cx| {
                if let Some(group) = multi_workspace.group_state_by_key_mut(key) {
                    group.expanded = !group.expanded;
                }
                multi_workspace.serialize(cx);
            })
            .log_err();
        // Collapsing emits no event the rail hears.
        self.refresh(window, cx);
    }

    /// Lists the turns of the terminal `id` under its row, or folds them (#509).
    fn toggle_turns(&mut self, id: u64, window: &mut Window, cx: &mut Context<Self>) {
        if !self.turns_open.remove(&id) {
            self.turns_open.insert(id);
        }
        self.refresh(window, cx);
    }

    /// Marks the rows whose turns are listed, and forgets the terminals that closed (#509).
    fn note_turns_open(&mut self, snapshot: &mut Snapshot) {
        self.turns_open
            .retain(|id| snapshot.terminals.contains_key(id));
        for terminal in snapshot
            .rail
            .projects
            .iter_mut()
            .flat_map(|project| project.terminals.iter_mut())
        {
            terminal.turns_open = self.turns_open.contains(&terminal.id);
        }
    }

    /// Opens the turn `sha` of the terminal `id` in Zed's commit view, in the terminal's project,
    /// which it shows (#509). The view diffs the turn's commit against its parent, the turn's
    /// start.
    fn open_turn(
        &self,
        id: u64,
        sha: String,
        workspace: &WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let repository = Turns::repository_of(id, &sha, cx).context("the turn is gone")?;
        let workspace = self.activate_workspace(workspace, window, cx)?;
        git_ui::commit_view::CommitView::open(
            sha,
            repository,
            workspace.downgrade(),
            None,
            None,
            window,
            cx,
        );
        Ok(())
    }

    fn render_header(&self, window: &Window, cx: &Context<Self>) -> impl IntoElement {
        let header = h_flex()
            .h(platform_title_bar_height(window))
            .w_full()
            .flex_none()
            .gap_1()
            .px_1()
            .border_b_1()
            .border_color(cx.theme().colors().border);
        // While a sidebar is open on the left, the title bar leaves its window controls to it.
        #[cfg(target_os = "macos")]
        let header = if window.is_fullscreen() {
            header
        } else {
            header.pl(px(ui::utils::TRAFFIC_LIGHT_PADDING))
        };
        #[cfg(not(target_os = "macos"))]
        let header = if window.is_fullscreen() {
            header
        } else {
            header.children(platform_title_bar::render_left_window_controls(
                cx.button_layout(),
                Box::new(workspace::CloseWindow),
                window,
            ))
        };
        let brain_shown = self.brain_shown();
        header
            .map(|header| {
                if self.brain.connected {
                    header.child(self.render_view_switch(cx))
                } else {
                    header.child(
                        Label::new("PROJECTS")
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    )
                }
            })
            .child(div().flex_1().h_full().when(!brain_shown, |space| {
                space.on_mouse_down(MouseButton::Right, cx.listener(Self::empty_space_menu))
            }))
            .map(|header| match self.shown_brain() {
                Some(brain) => header.child(Self::render_new_page(brain.downgrade())),
                None => header.child(self.render_add_project().into_any_element()),
            })
    }

    /// The header's Projects and Brain buttons while Rusty is connected (#644); Projects carries
    /// the rail's attention dot while Brain shows.
    fn render_view_switch(&self, cx: &Context<Self>) -> impl IntoElement {
        let brain_shown = self.brain_shown();
        let attention = brain_shown
            && (marley_rail::has_attention(&self.snapshot.rail)
                || !self.snapshot.rail.inbox.is_empty());
        h_flex()
            .gap_0p5()
            .child(
                IconButton::new("marley-rail-view-projects", IconName::ListTree)
                    .icon_size(IconSize::Small)
                    .toggle_state(!brain_shown)
                    .when(attention, |button| {
                        button.indicator(Indicator::dot().color(Color::Accent))
                    })
                    .tooltip(|_, cx| Tooltip::for_action("Projects", &ToggleBrainView, cx))
                    .on_click(cx.listener(|rail, _, window, cx| {
                        rail.show_view(RailView::Projects, window, cx);
                    })),
            )
            .child(
                IconButton::new("marley-rail-view-brain", IconName::BookCopy)
                    .icon_size(IconSize::Small)
                    .toggle_state(brain_shown)
                    .tooltip(|_, cx| Tooltip::for_action("Brain", &ToggleBrainView, cx))
                    .on_click(cx.listener(|rail, _, window, cx| {
                        rail.show_view(RailView::Brain, window, cx);
                    })),
            )
    }

    /// The header's end while the Brain view shows: a new page in the selected row's folder.
    fn render_new_page(brain: WeakEntity<BrainView>) -> AnyElement {
        IconButton::new("marley-rail-new-page", IconName::Plus)
            .icon_size(IconSize::Small)
            .tooltip(Tooltip::text("New Page"))
            .on_click(move |_, window, cx| {
                brain
                    .update(cx, |brain, cx| brain.new_page_here(window, cx))
                    .log_err();
            })
            .into_any_element()
    }

    /// The rows, and under them the empty space a right-click opens a menu on (#600).
    /// The rail's rows: each project's header and the rows under it in one block, which a dragged
    /// header lands on (#602).
    fn render_blocks(&self, cx: &Context<Self>) -> Vec<AnyElement> {
        let search_path = agents::launcher(cx).search_path;
        let rail = &self.snapshot.rail;
        let mut blocks: Vec<(AnyElement, Option<DraggedRailHeader>, Vec<AnyElement>)> = Vec::new();
        for (position, row) in marley_rail::rail_rows(rail).into_iter().enumerate() {
            let run = marley_rail::run(rail, &row);
            if let Row::Project(row) = row {
                let Some(group) = self.snapshot.groups.get(row.index) else {
                    continue;
                };
                let index = row.index;
                let drag = run.map(|run| DraggedRailHeader::new(group, position, run, &row.name));
                let icon = group
                    .key
                    .path_list()
                    .paths()
                    .first()
                    .and_then(|root| self.project_icons.get(root))
                    .and_then(|icon| icon.image.clone());
                let header = v_flex()
                    // A line between one project's rows and the next project, as Warp's tab list
                    // draws one between its tabs.
                    .when(position > 0, |project| {
                        project.child(
                            div()
                                .debug_selector(move || format!("marley-rail-divider-{index}"))
                                .py_1p5()
                                .child(Divider::horizontal()),
                        )
                    })
                    .child(self.render_project_row(
                        row,
                        group,
                        icon,
                        search_path.clone(),
                        drag.clone(),
                        cx,
                    ))
                    .into_any_element();
                blocks.push((header, drag, Vec::new()));
            } else if let Some(element) = self.render_row(position, row, run, cx)
                && let Some((_, _, rows)) = blocks.last_mut()
            {
                rows.push(element);
            }
        }
        let mut rendered: Vec<AnyElement> = blocks
            .into_iter()
            .map(|(header, drag, rows)| Self::header_block(header, rows, drag, cx))
            .collect();
        rendered.extend(self.render_containers(cx));
        rendered.extend(self.render_harness(cx));
        rendered
    }

    /// The harness's sessions (#534), under a Harness header after the containers: the
    /// connection's state, which a click on the header folds the rows under, and a row per
    /// session, muted while the connection is not up, which opens the session's tab. Outside the
    /// keys and the filter, as the containers are.
    fn render_harness(&self, cx: &Context<Self>) -> Option<AnyElement> {
        if self.snapshot.rail.filtering {
            return None;
        }
        let connection = Harness::connection(cx);
        let runtime = Harness::runtime(cx);
        if connection.is_none() && runtime.is_none() {
            return None;
        }
        // The runtime Marley runs itself (#632) speaks first: `rh mcp` answers while it is down.
        let (says, says_color) = match (&runtime, &connection) {
            (Some(Runtime::Missing(reason)), _) => (reason.to_string(), Color::Error),
            (Some(Runtime::Stopped(reason)), _) => {
                (format!("runtime stopped: {reason}"), Color::Error)
            }
            (Some(Runtime::Starting), _) => ("starting the runtime".to_string(), Color::Muted),
            (_, Some(Connection::Connected)) => ("connected".to_string(), Color::Muted),
            (_, Some(Connection::Down(reason))) => (format!("not running: {reason}"), Color::Error),
            (_, Some(Connection::Connecting) | None) => ("connecting".to_string(), Color::Muted),
        };
        let stale = connection != Some(Connection::Connected)
            || runtime.is_some_and(|runtime| runtime != Runtime::Running);
        let quiet_after_ms = MarleySettings::get_global(cx)
            .no_update_after_minutes
            .saturating_mul(60_000);
        let now = crate::harness::now_ms();
        let rows: Vec<AnyElement> = if Harness::folded(cx) {
            Vec::new()
        } else {
            Harness::seats(cx)
                .seats()
                .iter()
                .map(|seat| harness_row(seat, stale, quiet_after_ms, now, cx))
                .collect()
        };
        Some(
            v_flex()
                .debug_selector(|| "marley-rail-harness".into())
                .pt_2()
                .gap_0p5()
                .child(
                    h_flex()
                        .id("marley-rail-harness-header")
                        .px_3()
                        .py_1()
                        .gap_2()
                        .cursor_pointer()
                        .child(
                            Label::new("HARNESS")
                                .size(LabelSize::XSmall)
                                .color(Color::Muted),
                        )
                        .child(
                            div().min_w_0().child(
                                Label::new(says.clone())
                                    .size(LabelSize::XSmall)
                                    .color(says_color)
                                    .truncate(),
                            ),
                        )
                        // A reason is longer than the rail is wide (#632).
                        .tooltip(Tooltip::text(says))
                        .on_click(|_, _, cx| Harness::toggle_folded(cx)),
                )
                .children(rows)
                .into_any_element(),
        )
    }

    /// The container ports no project's folder holds (#614), under a Containers label after the
    /// projects. Their rows open in the shown project, and stay out of the keys and the filter.
    fn render_containers(&self, cx: &Context<Self>) -> Option<AnyElement> {
        if self.snapshot.rail.filtering {
            return None;
        }
        let found = Ports::containers(cx);
        if found.is_empty() {
            return None;
        }
        let shown = self
            .multi_workspace
            .upgrade()?
            .read(cx)
            .workspace()
            .downgrade();
        let rows = found.into_iter().map(|listener| {
            let snapshot = port_snapshot(listener, "", None);
            let row = PortRow {
                project: usize::MAX,
                port: snapshot.port,
                pid: snapshot.pid,
                title: snapshot.title,
                url: snapshot.url,
                tooltip: snapshot.tooltip,
                service: None,
                container: snapshot.container,
                state: None,
                selected: false,
                highlight: Vec::new(),
            };
            Self::render_port_row(row, shown.clone(), None, false, cx).into_any_element()
        });
        Some(
            v_flex()
                .debug_selector(|| "marley-rail-containers".into())
                .pt_2()
                .gap_0p5()
                .child(
                    h_flex().px_3().py_1().child(
                        Label::new("CONTAINERS")
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    ),
                )
                .children(rows)
                .into_any_element(),
        )
    }

    /// A row under a project's header, draggable when its kind reorders (#602).
    fn render_row(
        &self,
        position: usize,
        row: Row,
        run: Option<marley_rail::Run>,
        cx: &Context<Self>,
    ) -> Option<AnyElement> {
        match row {
            Row::Project(_) => None,
            Row::Terminal(row) => self.snapshot.terminals.get(&row.id).map(|terminal| {
                let drag = run.map(|run| {
                    let selection = Selection::Terminal(row.id);
                    DraggedRailRow::new(selection, row.project, position, run, &row.title)
                });
                let targets = self.move_targets(row.project);
                Self::render_terminal_row(row, terminal, drag, targets, cx).into_any_element()
            }),
            Row::Browser(row) => self.snapshot.browsers.get(&row.id).map(|browser| {
                let favicon = self.snapshot.favicons.get(&row.id).cloned();
                let drag = run.map(|run| {
                    let selection = Selection::Browser(row.id);
                    DraggedRailRow::new(selection, row.project, position, run, &row.title)
                });
                Self::render_browser_row(row, browser, favicon, drag, cx).into_any_element()
            }),
            Row::Thread(row) => self.snapshot.threads.get(&row.key).map(|thread| {
                let drag = run.map(|run| {
                    let selection = Selection::Thread(row.key.clone());
                    DraggedRailRow::new(selection, row.project, position, run, &row.title)
                });
                Self::render_thread_row(row, thread, drag, cx).into_any_element()
            }),
            Row::Port(row) => self.snapshot.groups.get(row.project).map(|group| {
                Self::render_port_row(
                    row,
                    group.workspace.clone(),
                    Some(group.key.clone()),
                    group.closed,
                    cx,
                )
                .into_any_element()
            }),
            Row::Worktree(row) => Some(Self::render_worktree_row(row, cx).into_any_element()),
        }
    }

    fn render_rows(&self, rows: Vec<AnyElement>, cx: &Context<Self>) -> impl IntoElement {
        v_flex()
            .id("marley-rail-rows")
            .flex_1()
            .overflow_y_scroll()
            .px_2()
            .py_1p5()
            .gap_0p5()
            .when(rows.is_empty() && self.snapshot.rail.filtering, |list| {
                list.child(
                    div()
                        .debug_selector(|| "marley-rail-no-matches".into())
                        .px_2()
                        .py_1()
                        .child(
                            Label::new("No matches")
                                .size(LabelSize::Small)
                                .color(Color::Muted),
                        ),
                )
            })
            .children(rows)
            // The empty space under the last row takes a right-click (#600).
            .child(
                div()
                    .debug_selector(|| "marley-rail-empty".into())
                    .flex_1()
                    .min_h_8()
                    .on_mouse_down(MouseButton::Right, cx.listener(Self::empty_space_menu)),
            )
    }

    fn render_filter(&self, cx: &Context<Self>) -> impl IntoElement {
        let filtering = self.snapshot.rail.filtering;
        h_flex()
            .w_full()
            .flex_none()
            .gap_2()
            .px_3()
            .py_2()
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .child(
                Icon::new(IconName::MagnifyingGlass)
                    .size(IconSize::Small)
                    .color(Color::Muted),
            )
            .child(
                div()
                    .debug_selector(|| "marley-rail-filter".into())
                    .min_w_0()
                    .flex_1()
                    .child(self.filter_editor.clone()),
            )
            .map(|field| {
                if filtering {
                    field.child(
                        div()
                            .debug_selector(|| "marley-rail-filter-clear".into())
                            .child(
                                IconButton::new("marley-rail-filter-clear", IconName::Close)
                                    .icon_size(IconSize::Small)
                                    .tooltip(Tooltip::text("Clear Filter"))
                                    .on_click(cx.listener(|rail, _, window, cx| {
                                        rail.clear_filter(window, cx);
                                    })),
                            ),
                    )
                } else {
                    field.child(KeyBinding::for_action_in(
                        &FocusSidebarFilter,
                        &self.focus_handle,
                        cx,
                    ))
                }
            })
    }

    fn render_add_project(&self) -> impl IntoElement {
        let multi_workspace = self.multi_workspace.clone();
        let focus_handle = self.focus_handle.clone();
        div()
            .debug_selector(|| "marley-rail-add-project".into())
            .child(
                PopoverMenu::new("marley-rail-add-project")
                    .trigger(
                        IconButton::new("marley-rail-add-project-button", IconName::FolderAdd)
                            .icon_size(IconSize::Small)
                            .tooltip(Tooltip::text("Add Project")),
                    )
                    .menu(move |window, cx| {
                        let multi_workspace = multi_workspace.upgrade()?.read(cx);
                        let workspace = multi_workspace.workspace().downgrade();
                        let groups = multi_workspace.project_group_keys();
                        Some(SidebarRecentProjects::popover(
                            workspace,
                            groups,
                            focus_handle.clone(),
                            window,
                            cx,
                        ))
                    })
                    .with_handle(self.add_project_menu.clone())
                    .anchor(Anchor::TopRight),
            )
    }

    fn render_project_row(
        &self,
        row: ProjectRow,
        group: &GroupEntry,
        icon: Option<Arc<RenderImage>>,
        agent_search_path: Option<OsString>,
        drag: Option<DraggedRailHeader>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        // Element ids follow the workspace, not the row's position, so an open menu stays with
        // its project when another group is inserted above it. A closed group has none, so its
        // ids follow its folders (#606).
        let closed = group.closed;
        let id = if closed {
            closed_id(&group.key)
        } else {
            group.workspace.entity_id()
        };
        let index = row.index;
        let last = index + 1 == self.snapshot.groups.len();
        let filtering = self.snapshot.rail.filtering;
        let folds = self.header_folds(index, closed);
        let workspace = group.workspace.clone();
        let key = group.key.clone();
        let open_key = group.key.clone();
        let projectless = group.group;
        let header_menu = HeaderMenu {
            rail: cx.entity().downgrade(),
            key: group.key.clone(),
            workspace: group.workspace.clone(),
            group: projectless.map(|group| (group, row.name.clone())),
            at: (index == 0, last),
            closed,
            archived: archived_threads(&group.key, cx),
        };
        // A project's name reads as a section label, as Warp's tab list labels its tabs; a closed
        // one's is dimmed.
        let name_color = if closed {
            Color::Disabled
        } else if row.selected {
            Color::Default
        } else {
            Color::Muted
        };
        let header =
            row_frame(("marley-rail-project", id), row.selected, cx)
                .h_8()
                .gap_1()
                .px_1()
                // While the filter decides which rows show, the header does not fold, and a closed
                // one with no rows has nothing to fold.
                .when(!filtering && folds, |header| {
                    header.child(
                        div()
                            .debug_selector(move || format!("marley-rail-disclosure-{index}"))
                            .child(
                                Disclosure::new(("marley-rail-disclosure", id), row.expanded)
                                    .on_click(cx.listener(move |rail, _, window, cx| {
                                        // The closed header under the chevron would open its project.
                                        if closed {
                                            cx.stop_propagation();
                                        }
                                        match projectless {
                                            Some(group) => groups::toggle_expanded(group, cx),
                                            None => rail.toggle_expanded(&key, window, cx),
                                        }
                                    })),
                            ),
                    )
                })
                // A closed header keeps the chevron's room, unseen, so its name lines up with an open
                // one's.
                .when(!filtering && !folds, |header| {
                    header.child(div().invisible().child(
                        Disclosure::new(("marley-rail-disclosure", id), false).disabled(true),
                    ))
                })
                .children(header_icon(index, icon, projectless.is_some()).map(|icon| {
                    if closed {
                        div().opacity(0.5).child(icon).into_any_element()
                    } else {
                        icon
                    }
                }))
                .child(project_name(
                    row.name,
                    row.highlight,
                    name_color,
                    row.summary,
                ))
                .child(Self::render_header_end(
                    index,
                    id,
                    group,
                    row.attention,
                    agent_search_path,
                    cx,
                ))
                .on_click(cx.listener(move |rail, _, window, cx| {
                    if closed {
                        rail.open_closed_project(&open_key, window, cx);
                    } else {
                        rail.activate_workspace(&workspace, window, cx).log_err();
                    }
                }));
        let header = Self::takes_terminals(header, index, closed, cx);
        let header = Self::draggable_header(header, drag, cx);
        right_click_menu(("marley-rail-project-context", id))
            // A closed header's tooltip is built only while its menu is closed, so the menu's
            // opening drops it (#618), as the port row's (#615).
            .trigger(move |menu_open, _, _| {
                div()
                    .debug_selector(move || format!("marley-rail-project-{index}"))
                    .child(header.when(closed && !menu_open, |header| {
                        header.tooltip(Tooltip::text("Not open. Click to open it."))
                    }))
            })
            .menu(move |window, cx| header_menu.build(window, cx))
    }

    /// Whether a header folds: an open project's does, and a closed one's with threads or ports
    /// under it (#617).
    fn header_folds(&self, index: usize, closed: bool) -> bool {
        !closed
            || self
                .snapshot
                .rail
                .projects
                .get(index)
                .is_some_and(|project| !project.threads.is_empty() || !project.ports.is_empty())
    }

    /// A header's end: its branch's changed lines and pull request (#531), its attention dot, and
    /// its `+`, which a closed project has not (#606).
    fn render_header_end(
        index: usize,
        id: EntityId,
        group: &GroupEntry,
        attention: bool,
        agent_search_path: Option<OsString>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        h_flex()
            .flex_none()
            .gap_1()
            .children(
                group
                    .git
                    .as_ref()
                    .and_then(|git| git.counts.clone())
                    // A branch with nothing changed shows no counts, not `+0 −0`.
                    .filter(|(_, added, removed)| added + removed > 0)
                    .map(|(base, added, removed)| {
                        DiffStat::new(
                            ("marley-rail-project-lines", id),
                            added as usize,
                            removed as usize,
                        )
                        .label_size(LabelSize::XSmall)
                        .tooltip(format!("Lines changed since the branch left {base}"))
                    }),
            )
            .children(
                group
                    .git
                    .as_ref()
                    .and_then(|git| git.pull_request.as_ref())
                    .map(|found| pull_request_chip(id, found)),
            )
            .when(attention, |slot| {
                slot.child(
                    div()
                        .debug_selector(move || format!("marley-rail-attention-{index}"))
                        .child(Indicator::dot().color(Color::Accent)),
                )
            })
            .when(!group.closed, |slot| {
                slot.child(Self::render_project_menu(
                    index,
                    id,
                    group,
                    agent_search_path,
                    cx,
                ))
            })
    }

    /// A projectless group's right-click menu (#600): Rename Group… and Remove Group.
    fn group_context_menu(
        rail: WeakEntity<Self>,
        group: Uuid,
        name: String,
        workspace: WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<ContextMenu> {
        ContextMenu::build(window, cx, move |menu, _, _| {
            let rename_rail = rail.clone();
            menu.entry("Rename Group…", None, move |window, cx| {
                rename_rail
                    .update(cx, |rail, cx| {
                        rail.rename_group(group, name.clone(), window, cx);
                    })
                    .log_err();
            })
            .entry("Remove Group", None, move |window, cx| {
                rail.update(cx, |rail, cx| {
                    rail.remove_group(group, &workspace, window, cx);
                })
                .log_err();
            })
        })
    }

    /// A project row's right-click menu: Move Project Up and Down, disabled at the ends `at`
    /// (first, last), then Clear Browser Data… (#581) and Remove Project (#507).
    fn project_context_menu(
        header: HeaderMenu,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<ContextMenu> {
        let HeaderMenu {
            rail,
            key,
            workspace,
            at: (first, last),
            closed,
            archived,
            ..
        } = header;
        ContextMenu::build(window, cx, move |menu, _, _| {
            let menu = [
                ("Move Project Up", true, first),
                ("Move Project Down", false, last),
            ]
            .into_iter()
            .fold(menu, |menu, (label, up, at_the_end)| {
                let (rail, key) = (rail.clone(), key.clone());
                menu.item(ContextMenuEntry::new(label).disabled(at_the_end).handler(
                    move |window, cx| {
                        rail.update(cx, |rail, cx| {
                            rail.move_project(&key, up, window, cx);
                        })
                        .log_err();
                    },
                ))
            });
            let closed_key = closed.then(|| key.clone());
            let menu = Self::archived_threads_menu(
                menu.separator(),
                &rail,
                &workspace,
                closed_key,
                archived.clone(),
            );
            // Clearing resets the project's browser (#581), which a closed project has not started
            // (#606); removing the project is what stops it (#507).
            menu.separator()
                .item(
                    ContextMenuEntry::new("Clear Browser Data…")
                        .disabled(closed)
                        .handler(move |window, cx| {
                            workspace
                                .update(cx, |workspace, cx| {
                                    browser::clear_project_browser_data(workspace, window, cx);
                                })
                                .log_err();
                        }),
                )
                .item(
                    ContextMenuEntry::new("Remove Project").handler(move |window, cx| {
                        rail.update(cx, |rail, cx| rail.remove_project(&key, window, cx))
                            .log_err();
                    }),
                )
        })
    }

    /// Archived Threads (#616): a submenu of the project's archived threads; choosing one opens it,
    /// which brings it back. One level only: Zed's menu closes a nested submenu's parents only
    /// when the submenu itself was clicked. Unlisted while there are none.
    fn archived_threads_menu(
        menu: ContextMenu,
        rail: &WeakEntity<Self>,
        workspace: &WeakEntity<Workspace>,
        closed: Option<ProjectGroupKey>,
        archived: Vec<ArchivedThread>,
    ) -> ContextMenu {
        if archived.is_empty() {
            return menu.item(ContextMenuEntry::new("No Archived Threads").disabled(true));
        }
        let (rail, workspace) = (rail.clone(), workspace.clone());
        menu.submenu("Archived Threads", move |menu, _, _| {
            archived.iter().fold(menu, |menu, thread| {
                let label = format!("{} · {}", thread.title, thread.archived_at);
                let (rail, workspace, thread) = (rail.clone(), workspace.clone(), thread.clone());
                let closed = closed.clone();
                menu.entry(label, None, move |window, cx| {
                    let thread = thread.clone();
                    rail.update(cx, |rail, cx| {
                        let opening = ThreadToOpen {
                            thread_id: thread.thread_id,
                            agent: thread.agent,
                            work_dirs: thread.work_dirs,
                            title: Some(thread.title),
                        };
                        rail.open_thread_with(&workspace, closed.as_ref(), opening, window, cx)
                    })
                    .flatten()
                    .log_err();
                })
            })
        })
    }

    fn render_project_menu(
        index: usize,
        id: EntityId,
        group: &GroupEntry,
        agent_search_path: Option<OsString>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let rail = cx.entity().downgrade();
        let workspace = group.workspace.clone();
        // A projectless group has no folder for a thread (#600).
        let projectless = group.group.is_some();
        div()
            .debug_selector(move || format!("marley-rail-project-menu-{index}"))
            .child(
                PopoverMenu::new(("marley-rail-project-menu", id))
                    .trigger(
                        IconButton::new(("marley-rail-project-plus", id), IconName::Plus)
                            .icon_size(IconSize::Small)
                            .tooltip(Tooltip::text("New in this project")),
                    )
                    .menu(move |window, cx| {
                        let rail = rail.clone();
                        let workspace = workspace.clone();
                        let agent_search_path = agent_search_path.clone();
                        Some(ContextMenu::build(window, cx, move |menu, _, cx| {
                            let terminal_rail = rail.clone();
                            let terminal_workspace = workspace.clone();
                            let browser_rail = rail.clone();
                            let browser_workspace = workspace.clone();
                            let cli_rail = rail.clone();
                            let cli_workspace = workspace.clone();
                            let menu = menu
                                .entry("New Terminal", None, move |window, cx| {
                                    terminal_rail
                                        .update(cx, |rail, cx| {
                                            rail.new_terminal(&terminal_workspace, window, cx)
                                        })
                                        .flatten()
                                        .log_err();
                                })
                                .entry("New Browser Tab", None, move |window, cx| {
                                    browser_rail
                                        .update(cx, |rail, cx| {
                                            rail.new_browser_tab(&browser_workspace, window, cx)
                                        })
                                        .flatten()
                                        .log_err();
                                });
                            let menu = if projectless {
                                menu
                            } else {
                                menu.submenu("New Agent Thread", move |menu, _, cx| {
                                    Self::agent_menu(menu, &rail, &workspace, cx)
                                })
                            };
                            let menu = Self::worktree_agent_entries(
                                menu,
                                &cli_workspace,
                                agent_search_path.as_deref(),
                                cx,
                            );
                            let menu = Self::agent_cli_entries(
                                menu,
                                &cli_rail,
                                &cli_workspace,
                                agent_search_path.as_deref(),
                            );
                            Self::launch_entries(menu, &cli_rail, &cli_workspace, cx)
                        }))
                    })
                    .anchor(Anchor::TopRight),
            )
    }

    /// The New Agent Thread entries: the Zed Agent, then every agent the project's agent servers
    /// list, as the Agent Panel's own menu names and orders them.
    fn agent_menu(
        menu: ContextMenu,
        rail: &WeakEntity<Self>,
        workspace: &WeakEntity<Workspace>,
        cx: &App,
    ) -> ContextMenu {
        let choices = workspace
            .upgrade()
            .map(|workspace| agents::thread_agents(workspace.read(cx).project(), cx))
            .unwrap_or_default();
        choices.into_iter().fold(menu, |menu, (agent, name, icon)| {
            let rail = rail.clone();
            let workspace = workspace.clone();
            let entry = ContextMenuEntry::new(name);
            let entry = match icon {
                AgentIcon::Named(icon) => entry.icon(icon),
                AgentIcon::Svg(path) => entry.custom_icon_svg(path),
            };
            menu.item(entry.icon_color(Color::Muted).handler(move |window, cx| {
                rail.update(cx, |rail, cx| {
                    rail.new_agent_thread(&workspace, &agent, window, cx)
                })
                .flatten()
                .log_err();
            }))
        })
    }

    /// The agent CLIs the search path holds, one entry each under a header, after New Agent
    /// Thread and New Agent in Worktree; nothing when none is installed.
    fn agent_cli_entries(
        menu: ContextMenu,
        rail: &WeakEntity<Self>,
        workspace: &WeakEntity<Workspace>,
        search_path: Option<&OsStr>,
    ) -> ContextMenu {
        let agents = agents::installed_clis(search_path);
        if agents.is_empty() {
            return menu;
        }
        agents
            .into_iter()
            .fold(menu.separator().header("Agent CLIs"), |menu, kind| {
                let rail = rail.clone();
                let workspace = workspace.clone();
                menu.item(
                    ContextMenuEntry::new(kind.display_name())
                        .icon(agents::cli_icon(kind))
                        .icon_color(Color::Muted)
                        .handler(move |window, cx| {
                            rail.update(cx, |rail, cx| {
                                rail.new_agent(&workspace, kind, window, cx)
                            })
                            .flatten()
                            .log_err();
                        }),
                )
            })
    }

    /// New Agent in Worktree (#510): a submenu of the installed agent CLIs, right after New Agent
    /// Thread (#598), for a local project whose folder is a git repository.
    fn worktree_agent_entries(
        menu: ContextMenu,
        workspace: &WeakEntity<Workspace>,
        search_path: Option<&OsStr>,
        cx: &App,
    ) -> ContextMenu {
        let offered = workspace
            .upgrade()
            .is_some_and(|workspace| worktree_agents::offered(workspace.read(cx), cx));
        let agents = agents::installed_clis(search_path);
        if !offered || agents.is_empty() {
            return menu;
        }
        let workspace = workspace.clone();
        menu.submenu("New Agent in Worktree", move |menu, _, _| {
            agents.iter().fold(menu, |menu, kind| {
                let (workspace, kind) = (workspace.clone(), *kind);
                menu.item(
                    ContextMenuEntry::new(kind.display_name())
                        .icon(agents::cli_icon(kind))
                        .icon_color(Color::Muted)
                        .handler(move |window, cx| {
                            worktree_agents::open_prompt(&workspace, kind, window, cx).log_err();
                        }),
                )
            })
        })
    }

    /// A linked worktree's row (#510): the branch icon, its name and branch, muted while its
    /// workspace is not open, with the commits its branch has that its base lacks (#511), and its
    /// drift's chip (#560); a click shows the workspace, or opens it, and its menu offers Review
    /// and Merge (#511).
    fn render_worktree_row(row: WorktreeRow, cx: &Context<Self>) -> impl IntoElement {
        let path = row.path.clone();
        let selector = format!("marley-rail-worktree-{}", row.name);
        let chip = row
            .drift
            .as_ref()
            .and_then(|drift| drift_chip(&row.path, &row.name, drift, cx));
        let icon = Icon::new(IconName::GitBranch)
            .size(IconSize::Small)
            .color(Color::Muted)
            .into_any_element();
        let color = if row.open {
            Color::Default
        } else {
            Color::Muted
        };
        let ahead = row.drift.as_ref().and_then(DriftSnapshot::ahead_words);
        let line = match (row.branch, ahead) {
            (Some(branch), Some(ahead)) => Some(format!("{branch} · {ahead}")),
            (branch, ahead) => branch.or(ahead),
        };
        let item = row_card(
            SharedString::from(format!("marley-rail-worktree-{}", row.path)),
            format!("{selector}-icon"),
            row.selected,
            icon,
            row_label(row.name, row.highlight, color),
            line.into_iter().collect(),
            cx,
        )
        .children(chip)
        .on_click(cx.listener(move |rail, _: &ClickEvent, window, cx| {
            rail.open_row(Selection::Worktree(path.clone()), window, cx)
                .log_err();
        }));
        let rail = cx.entity().downgrade();
        let menu_path = row.path.clone();
        right_click_menu(SharedString::from(format!(
            "marley-rail-worktree-menu-{}",
            row.path
        )))
        .trigger(move |_, _, _| div().debug_selector(move || selector).pl_2().child(item))
        .menu(move |window, cx| {
            let (merge, remove) = rail.upgrade().map_or((None, None), |rail| {
                let rail = rail.read(cx);
                (
                    Some(rail.merge_line(&menu_path, cx)),
                    Some(rail.remove_line(&menu_path)),
                )
            });
            let (review_rail, review_path) = (rail.clone(), menu_path.clone());
            let (merge_rail, merge_path) = (rail.clone(), menu_path.clone());
            let (remove_rail, remove_path) = (rail.clone(), menu_path.clone());
            ContextMenu::build(window, cx, move |menu, _, _| {
                let menu = menu.entry("Review", None, move |window, cx| {
                    review_rail
                        .update(cx, |rail, cx| {
                            rail.review_worktree(&review_path, window, cx);
                        })
                        .log_err();
                });
                let menu = match merge {
                    Some(MergeLine::Merge(words)) => menu.entry(words, None, move |window, cx| {
                        merge_rail
                            .update(cx, |rail, cx| rail.merge_worktree(&merge_path, window, cx))
                            .log_err();
                    }),
                    Some(MergeLine::Note(note)) => menu.label(note),
                    None => menu,
                };
                match remove {
                    Some(RemoveLine::Remove) => {
                        menu.separator().entry("Remove…", None, move |window, cx| {
                            remove_rail
                                .update(cx, |rail, cx| {
                                    rail.remove_worktree(&remove_path, window, cx);
                                })
                                .log_err();
                        })
                    }
                    Some(RemoveLine::Note(note)) => menu.separator().label(note),
                    None => menu,
                }
            })
        })
    }

    fn render_thread_row(
        row: ThreadRow,
        thread: &ThreadEntry,
        drag: Option<DraggedRailRow>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let key = row.key.clone();
        let thread_id = thread.thread_id;
        let rail = cx.entity().downgrade();
        let title = thread
            .title
            .clone()
            .unwrap_or_else(|| SharedString::new_static("Untitled thread"));
        let icon = match &thread.icon {
            AgentIcon::Named(icon) => Icon::new(*icon),
            AgentIcon::Svg(path) => Icon::from_external_svg(path.clone()),
        };
        let subtitle = thread_subtitle(&thread.agent_name, row.status);
        // The status mark shows until the pointer is over the row, and Archive while it is (#605),
        // as a terminal row's bell gives way to its close button.
        let archive = IconButton::new(
            SharedString::from(format!("marley-rail-thread-archive-{key}")),
            IconName::Archive,
        )
        .icon_size(IconSize::Small)
        .icon_color(Color::Muted)
        .tooltip(Tooltip::text("Archive Thread"))
        .on_click(move |_, _, cx| {
            // The row under the button would open the thread it archives.
            cx.stop_propagation();
            Self::archive_thread(thread_id, cx);
        });
        let end = h_flex()
            .flex_none()
            .relative()
            .child(
                h_flex()
                    .debug_selector({
                        let key = key.clone();
                        move || format!("marley-rail-thread-archive-{key}")
                    })
                    .visible_on_hover(ROW_GROUP)
                    .child(archive),
            )
            .children(thread_status_mark(row.status, row.attention).map(|mark| {
                h_flex()
                    .absolute()
                    .inset_0()
                    .justify_center()
                    .group_hover(ROW_GROUP, Styled::invisible)
                    .child(mark)
            }));
        let card = row_card(
            SharedString::from(format!("marley-rail-thread-{key}")),
            format!("marley-rail-thread-icon-{key}"),
            row.selected,
            icon.size(IconSize::Small)
                .color(Color::Muted)
                .into_any_element(),
            row_label(row.title, row.highlight, Color::Default),
            vec![subtitle],
            cx,
        )
        .child(end)
        .on_click(cx.listener(move |rail, _, window, cx| {
            rail.open_thread(&key, window, cx).log_err();
        }));
        let card = Self::draggable_row(card, drag, cx);
        right_click_menu(SharedString::from(format!(
            "marley-rail-thread-menu-{}",
            row.key
        )))
        .trigger(move |_, _, _| {
            div()
                .debug_selector(move || format!("marley-rail-thread-{}", row.key))
                .pl_2()
                .child(card)
        })
        .menu(move |window, cx| {
            let (rail, title) = (rail.clone(), title.clone());
            ContextMenu::build(window, cx, move |menu, _, _| {
                let (rail, title) = (rail.clone(), title.clone());
                menu.entry("Archive Thread", None, move |_, cx| {
                    Self::archive_thread(thread_id, cx);
                })
                .entry("Delete Thread…", None, move |window, cx| {
                    rail.update(cx, |rail, cx| {
                        rail.delete_thread(thread_id, &title, window, cx);
                    })
                    .log_err();
                })
            })
        })
    }

    /// Archives a thread as Zed's thread history does (#605): the store keeps it archived, and
    /// the rail, which lists no archived thread, drops its row at the refresh the store's change
    /// brings.
    fn archive_thread(thread_id: ThreadId, cx: &mut App) {
        if let Some(store) = ThreadMetadataStore::try_global(cx) {
            store.update(cx, |store, cx| store.archive(thread_id, None, cx));
        }
    }

    fn render_terminal_row(
        row: TerminalRow,
        terminal: &TerminalEntry,
        drag: Option<DraggedRailRow>,
        targets: Vec<(usize, String)>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let id = row.id;
        let (workspace, view) = (terminal.workspace.clone(), terminal.view.clone());
        let (close_workspace, close_view) = (terminal.workspace.clone(), terminal.view.clone());
        let close = div()
            .debug_selector(move || format!("marley-rail-terminal-close-{id}"))
            .child(
                IconButton::new(("marley-rail-terminal-close", id), IconName::Close)
                    .icon_size(IconSize::Small)
                    .icon_color(Color::Muted)
                    .tooltip(Tooltip::text("Close Terminal"))
                    .on_click(cx.listener(move |_, _, window, cx| {
                        // The row under the button would show the terminal it closes.
                        cx.stop_propagation();
                        Self::close_terminal(&close_workspace, &close_view, window, cx).log_err();
                    })),
            );
        let icon = div()
            .when(row.agent.is_some(), |icon| {
                icon.debug_selector(move || format!("marley-rail-agent-{id}"))
            })
            .child(rail_terminal_icon(&row, cx))
            .into_any_element();
        // The bell shows until the pointer is over the row, and the close button while it is.
        let end = h_flex()
            .flex_none()
            .relative()
            .child(h_flex().visible_on_hover(ROW_GROUP).child(close))
            .when(row.bell, |end| {
                end.child(
                    h_flex()
                        .absolute()
                        .inset_0()
                        .justify_center()
                        .group_hover(ROW_GROUP, Styled::invisible)
                        .child(
                            div()
                                .debug_selector(move || format!("marley-rail-bell-{id}"))
                                .child(Indicator::dot().color(Color::Accent)),
                        ),
                )
            });
        let marks = terminal_marks(&row, cx);
        let item = row_card(
            ("marley-rail-terminal", id),
            format!("marley-rail-terminal-icon-{id}"),
            row.selected,
            icon,
            row_label(row.title, row.highlight, Color::Default),
            row.subtitle
                .into_iter()
                .chain(row.activity)
                .map(RowLine::muted)
                .chain(row.command.map(command_line))
                .chain(row.running_error.map(|error| RowLine {
                    text: error.line,
                    state: None,
                    color: Color::Error,
                    cut: Cut::End,
                }))
                .collect(),
            cx,
        )
        .children(marks)
        .child(end)
        .on_click(cx.listener(move |rail, event: &ClickEvent, window, cx| {
            // The second click of a double-click renames, as a tab's does.
            if event.click_count() == 2 {
                rail.rename_terminal(&workspace, &view, window, cx)
                    .log_err();
            } else {
                rail.activate_terminal(&workspace, &view, window, cx)
                    .log_err();
            }
        }));
        let item = Self::draggable_row(item, drag, cx);
        let nested = row.worktree.is_some();
        let turns = Self::render_turns(id, row.turns, row.turns_open, &terminal.workspace, cx);
        let rail = cx.entity().downgrade();
        let (menu_workspace, menu_view) = (terminal.workspace.clone(), terminal.view.clone());
        let menu = right_click_menu(("marley-rail-terminal-menu", id))
            .trigger(move |_, _, _| {
                div()
                    .debug_selector(move || format!("marley-rail-terminal-{id}"))
                    .map(|row| if nested { row.pl_6() } else { row.pl_2() })
                    .child(item)
            })
            .menu(move |window, cx| {
                Self::terminal_context_menu(
                    &rail,
                    &menu_workspace,
                    &menu_view,
                    (id, &targets),
                    window,
                    cx,
                )
            });
        v_flex().child(menu).children(turns)
    }

    /// A terminal row's right-click menu: Rename, Move to Project (#613) when the window has
    /// another open project, and Close. `moves` is the terminal's id and the projects it may move
    /// to, by their place in the rail and their names.
    fn terminal_context_menu(
        rail: &WeakEntity<Self>,
        workspace: &WeakEntity<Workspace>,
        view: &WeakEntity<TerminalView>,
        moves: (u64, &[(usize, String)]),
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<ContextMenu> {
        let (rename_rail, rename_workspace, rename_view) =
            (rail.clone(), workspace.clone(), view.clone());
        let (close_workspace, close_view) = (workspace.clone(), view.clone());
        let (terminal, targets) = (moves.0, moves.1.to_vec());
        let move_rail = rail.clone();
        ContextMenu::build(window, cx, move |menu, _, _| {
            let menu = menu.entry("Rename", None, move |window, cx| {
                rename_rail
                    .update(cx, |rail, cx| {
                        rail.rename_terminal(&rename_workspace, &rename_view, window, cx)
                    })
                    .flatten()
                    .log_err();
            });
            let targets = targets.clone();
            let move_rail = move_rail.clone();
            menu.when(!targets.is_empty(), move |menu| {
                menu.submenu("Move to Project", move |menu, _, _| {
                    targets.iter().fold(menu, |menu, (project, name)| {
                        let (rail, project) = (move_rail.clone(), *project);
                        menu.entry(name.clone(), None, move |window, cx| {
                            rail.update(cx, |rail, cx| {
                                rail.move_terminal(terminal, project, window, cx);
                            })
                            .log_err();
                        })
                    })
                })
            })
            .entry("Close", None, move |window, cx| {
                Self::close_terminal(&close_workspace, &close_view, window, cx).log_err();
            })
        })
    }

    /// The open projects a terminal of the project at `project` may move to: every other one of
    /// the window that is not closed, projectless groups (#600) included, by its place in the
    /// rail and its name (#613).
    fn move_targets(&self, project: usize) -> Vec<(usize, String)> {
        self.snapshot
            .groups
            .iter()
            .enumerate()
            .filter(|(index, group)| *index != project && !group.closed)
            .filter_map(|(index, _)| {
                let name = self.snapshot.rail.projects.get(index)?.name.clone();
                Some((index, name))
            })
            .collect()
    }

    /// Moves a terminal to the project at `project` (#613): its own view goes to that project's
    /// active pane through Zed's `move_item`, so its shell and what Marley keeps for it stay, and
    /// the project shows. Deferred, since it updates panes and workspaces the rail's update must
    /// not hold.
    fn move_terminal(
        &self,
        terminal: u64,
        project: usize,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let Some(entry) = self.snapshot.terminals.get(&terminal) else {
            return;
        };
        let Some(target) = self
            .snapshot
            .groups
            .get(project)
            .filter(|group| !group.closed)
            .map(|group| group.workspace.clone())
        else {
            return;
        };
        let (source, view) = (entry.workspace.clone(), entry.view.clone());
        let rail = cx.entity().downgrade();
        window.defer(cx, move |window, cx| {
            let (Some(source), Some(view), Some(destination_workspace)) =
                (source.upgrade(), view.upgrade(), target.upgrade())
            else {
                return;
            };
            if source == destination_workspace {
                return;
            }
            let Some(source_pane) = source.read(cx).pane_for(&view) else {
                return;
            };
            let destination = destination_workspace.read(cx).active_pane().clone();
            let index = destination.read(cx).items_len();
            workspace::move_item(
                &source_pane,
                &destination,
                view.entity_id(),
                index,
                true,
                window,
                cx,
            );
            rail.update(cx, |rail, cx| {
                rail.activate_workspace(&target, window, cx).log_err();
            })
            .log_err();
        });
    }

    /// Under a terminal's card, its turns (#509): a "Turns (N)" line whose disclosure lists them,
    /// newest first, each with its title, the files it changed, and whether it failed or a
    /// harness started it. A click on a turn opens its diff.
    fn render_turns(
        id: u64,
        turns: Vec<TurnSnapshot>,
        open: bool,
        workspace: &WeakEntity<Workspace>,
        cx: &Context<Self>,
    ) -> Option<AnyElement> {
        if turns.is_empty() {
            return None;
        }
        let hover = cx.theme().colors().ghost_element_hover;
        let header = h_flex()
            .id(("marley-rail-turns", id))
            .debug_selector(move || format!("marley-rail-turns-{id}"))
            .h_6()
            .gap_1()
            .px_1()
            .rounded_md()
            .cursor_pointer()
            .hover(|style| style.bg(hover))
            .child(
                Disclosure::new(("marley-rail-turns-disclosure", id), open).on_click(
                    cx.listener(move |rail, _, window, cx| rail.toggle_turns(id, window, cx)),
                ),
            )
            .child(
                Label::new(format!("Turns ({})", turns.len()))
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
            .on_click(cx.listener(move |rail, _, window, cx| rail.toggle_turns(id, window, cx)));
        let rows = open.then(|| {
            turns.into_iter().enumerate().map(|(index, turn)| {
                let files = if turn.files == 1 {
                    "1 file".to_string()
                } else {
                    format!("{} files", turn.files)
                };
                let (sha, workspace) = (turn.sha, workspace.clone());
                h_flex()
                    .id(("marley-rail-turn", index))
                    .debug_selector(move || format!("marley-rail-turn-{id}-{index}"))
                    .h_6()
                    .gap_1()
                    .pl_6()
                    .pr_1()
                    .rounded_md()
                    .cursor_pointer()
                    .hover(|style| style.bg(hover))
                    .tooltip(Tooltip::text(turn.title.clone()))
                    .child(
                        div()
                            .min_w_0()
                            .child(Label::new(turn.title).size(LabelSize::XSmall).truncate()),
                    )
                    .child(
                        h_flex()
                            .flex_none()
                            .gap_1()
                            .child(
                                Label::new(format!("· {files}"))
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted),
                            )
                            .when(turn.failed, |marks| {
                                marks.child(
                                    Label::new("· failed")
                                        .size(LabelSize::XSmall)
                                        .color(Color::Error),
                                )
                            })
                            .when(turn.injected, |marks| {
                                marks.child(
                                    Label::new("· injected")
                                        .size(LabelSize::XSmall)
                                        .color(Color::Muted),
                                )
                            }),
                    )
                    .on_click(cx.listener(move |rail, _, window, cx| {
                        rail.open_turn(id, sha.clone(), &workspace, window, cx)
                            .log_err();
                    }))
            })
        });
        Some(
            v_flex()
                .id(("marley-rail-turn-list", id))
                .pl_4()
                .child(header)
                .children(rows.into_iter().flatten())
                .into_any_element(),
        )
    }
}

impl Rail {
    /// A Browser tab's row (#504): the page's icon, a spinner while it loads, or the globe; the
    /// title and the host; the tray's picks, the page's annotations and the agent's mark; and
    /// the close button over the row.
    fn render_browser_row(
        row: BrowserRow,
        browser: &BrowserEntry,
        favicon: Option<Arc<Image>>,
        drag: Option<DraggedRailRow>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let id = row.id;
        let (workspace, view) = (browser.workspace.clone(), browser.view.clone());
        let (close_workspace, close_view) = (browser.workspace.clone(), browser.view.clone());
        let icon = if row.loading {
            Icon::new(IconName::LoadCircle)
                .size(IconSize::Small)
                .color(Color::Muted)
                .with_rotate_animation(2)
                .into_any_element()
        } else if let Some(favicon) = favicon {
            div()
                .debug_selector(move || format!("marley-rail-browser-favicon-{id}"))
                .child(img(favicon).size_3p5())
                .into_any_element()
        } else {
            Icon::new(IconName::ToolWeb)
                .size(IconSize::Small)
                .color(Color::Muted)
                .into_any_element()
        };
        let count = |icon: IconName, count: usize, selector: &'static str| {
            (count > 0).then(|| {
                h_flex()
                    .debug_selector(move || format!("{selector}-{id}"))
                    .gap_0p5()
                    .child(Icon::new(icon).size(IconSize::XSmall).color(Color::Muted))
                    .child(
                        Label::new(count.to_string())
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    )
            })
        };
        let close = div()
            .debug_selector(move || format!("marley-rail-browser-close-{id}"))
            .child(
                IconButton::new(("marley-rail-browser-close", id), IconName::Close)
                    .icon_size(IconSize::Small)
                    .icon_color(Color::Muted)
                    .tooltip(Tooltip::text("Close Browser Tab"))
                    .on_click(cx.listener(move |_, _, window, cx| {
                        // The row under the button would show the tab it closes.
                        cx.stop_propagation();
                        Self::close_browser(&close_workspace, &close_view, window, cx).log_err();
                    })),
            );
        let end = h_flex()
            .flex_none()
            .gap_1()
            .children(count(
                IconName::Crosshair,
                row.picks,
                "marley-rail-browser-picks",
            ))
            .children(count(
                IconName::Pencil,
                row.annotations,
                "marley-rail-browser-annotations",
            ))
            .when(row.agent_unseen, |end| {
                end.child(
                    div()
                        .debug_selector(move || format!("marley-rail-browser-agent-{id}"))
                        .child(
                            Icon::new(IconName::Sparkle)
                                .size(IconSize::XSmall)
                                .color(Color::Accent),
                        ),
                )
            })
            .child(h_flex().visible_on_hover(ROW_GROUP).child(close));
        let item = row_card(
            ("marley-rail-browser", id),
            format!("marley-rail-browser-icon-{id}"),
            row.selected,
            icon,
            row_label(row.title, row.highlight, Color::Default),
            row.host.into_iter().collect(),
            cx,
        )
        .child(end)
        .on_click(cx.listener(move |rail, _: &ClickEvent, window, cx| {
            rail.activate_browser(&workspace, &view, window, cx)
                .log_err();
        }));
        let item = Self::draggable_row(item, drag, cx);
        div()
            .debug_selector(move || format!("marley-rail-browser-{id}"))
            .pl_2()
            .child(item)
    }
}

impl Rail {
    /// A port row's hover buttons: Open, Copy URL and Stop (a container's, a quiet restarted
    /// service's, or the process's). They lie over the row's end rather than keep room beside it
    /// (#618), on the row's own shade made opaque over the rail's background, so they cover the
    /// text under them and leave the text the row's width while hidden.
    fn port_row_buttons(
        row: &PortRow,
        workspace: &WeakEntity<Workspace>,
        closed: Option<&ProjectGroupKey>,
        cx: &Context<Self>,
    ) -> Div {
        let (port, pid) = (row.port, row.pid);
        let key = (u64::from(port) << 32) | u64::from(pid);
        let colors = cx.theme().colors();
        let fill = if row.selected {
            colors.ghost_element_selected
        } else {
            colors.ghost_element_hover
        };
        let shade = colors.panel_background.blend(fill);
        let button = |id: &'static str, icon: IconName, tooltip: &'static str| {
            IconButton::new((id, key), icon)
                .icon_size(IconSize::Small)
                .icon_color(Color::Muted)
                .tooltip(Tooltip::text(tooltip))
        };
        let (open_workspace, open_url) = (workspace.clone(), row.url.clone());
        let open_closed = closed.cloned();
        let open = button(
            "marley-rail-port-open",
            IconName::ToolWeb,
            "Open in a Browser Tab",
        )
        .on_click(cx.listener(move |rail, _, window, cx| {
            // The row under the button would open it a second time.
            cx.stop_propagation();
            rail.open_port(
                &open_workspace,
                open_closed.as_ref(),
                open_url.clone(),
                window,
                cx,
            )
            .log_err();
        }));
        let copy_url = row.url.clone();
        let copy = button("marley-rail-port-copy", IconName::Copy, "Copy URL").on_click(
            move |_, _, cx| {
                cx.stop_propagation();
                cx.write_to_clipboard(ClipboardItem::new_string(copy_url.clone()));
            },
        );
        let (stop_title, stop_meta) = stop_words(pid, row.service.as_ref(), row.container.as_ref());
        let is_container = row.container.is_some();
        // A restarted service's row kept while its port is quiet has no process (#615).
        let kept = (pid == 0).then(|| row.service.clone()).flatten();
        let stop = IconButton::new(("marley-rail-port-stop", key), IconName::Stop)
            .icon_size(IconSize::Small)
            .icon_color(Color::Muted)
            .tooltip(move |_, cx| Tooltip::with_meta(stop_title, None, stop_meta.clone(), cx))
            .on_click(cx.listener(move |rail, _, window, cx| {
                cx.stop_propagation();
                if is_container {
                    rail.stop_container_port(port, window, cx);
                } else if let Some(service) = kept.clone() {
                    rail.stop_service_port(service, window, cx);
                } else {
                    rail.stop_port(port, pid, window, cx);
                }
            }));
        h_flex()
            .absolute()
            .top_0()
            .bottom_0()
            .right_0()
            .pl_1()
            .pr_1p5()
            .gap_0p5()
            .rounded_r_md()
            .bg(shade)
            .visible_on_hover(ROW_GROUP)
            .child(
                div()
                    .debug_selector(move || format!("marley-rail-port-open-{port}"))
                    .child(open),
            )
            .child(
                div()
                    .debug_selector(move || format!("marley-rail-port-copy-{port}"))
                    .child(copy),
            )
            .child(
                div()
                    .debug_selector(move || format!("marley-rail-port-stop-{port}"))
                    .child(stop),
            )
    }

    /// A port's row (#521): the port and its process, the URL under them, and on hover Open,
    /// Copy and Stop; its tooltip gives the whole URL and names the process. One click marks the
    /// row, and a double-click or Enter opens it as Open does (#604).
    fn render_port_row(
        row: PortRow,
        workspace: WeakEntity<Workspace>,
        group: Option<ProjectGroupKey>,
        closed: bool,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let (port, pid) = (row.port, row.pid);
        let key = (u64::from(port) << 32) | u64::from(pid);
        let is_container = row.container.is_some();
        // A closed project's row opens the project first (#617).
        let closed = group.clone().filter(|_| closed);
        let end = Self::port_row_buttons(&row, &workspace, closed.as_ref(), cx);
        let url = row.url.clone();
        let (menu_workspace, menu_url, key_id) = (workspace.clone(), row.url.clone(), key);
        let row_closed = closed.clone();
        let tooltip = format!(
            "{}\n{}\nDouble-click, or Enter, to open in a Browser tab.",
            row.url, row.tooltip
        );
        let item = row_card(
            ("marley-rail-port", key),
            format!("marley-rail-port-icon-{port}"),
            row.selected,
            Icon::new(IconName::Server)
                .size(IconSize::Small)
                .color(Color::Muted)
                .into_any_element(),
            row_label(row.title, row.highlight, Color::Default),
            // A service's unit, with its state (#615), or a port's container (#614), gets a line
            // of its own: beside the URL it would squeeze the URL out. A URL too long for the rail
            // loses its middle and a unit's name its start, so the host and port and the name's
            // end stay (#618).
            std::iter::once(RowLine {
                cut: Cut::Middle,
                ..RowLine::muted(url_label(&row.url))
            })
            .chain(row.service.clone().map(|service| RowLine {
                text: service.unit,
                color: if row.state.as_deref() == Some("failed") {
                    Color::Error
                } else {
                    Color::Muted
                },
                state: row.state.clone(),
                cut: Cut::Start,
            }))
            .chain(
                row.container
                    .map(|container| RowLine::muted(container_line(&container))),
            )
            .collect(),
            cx,
        )
        .relative()
        .child(end)
        // One click marks the row and two open it (#604): a click on a server's row to look at it
        // should not start a browser.
        .on_click(cx.listener(move |rail, event: &ClickEvent, window, cx| {
            if event.click_count() >= 2 {
                rail.open_port(&workspace, row_closed.as_ref(), url.clone(), window, cx)
                    .log_err();
            } else {
                rail.mark_row(Selection::Port(port, pid), window, cx);
            }
        }));
        let menu = PortMenu {
            rail: cx.entity().downgrade(),
            workspace: menu_workspace,
            group,
            closed,
            port,
            pid,
            url: menu_url,
            service: row.service,
            is_container,
        };
        right_click_menu(("marley-rail-port-menu", key_id))
            // The row's tooltip is built only while its menu is closed, so it never lies over the
            // menu it opened, as Zed's dock buttons do.
            .trigger(move |menu_open, _, _| {
                div()
                    .debug_selector(move || format!("marley-rail-port-{port}"))
                    .pl_2()
                    .child(item.when(!menu_open, |item| item.tooltip(Tooltip::text(tooltip))))
            })
            .menu(move |window, cx| menu.build(window, cx))
    }
}

/// What a port row's Stop says it does (#603): stop the user's or the system's service, with the
/// command it runs, or signal the process.
/// What a port row's right-click menu acts on (#615).
#[derive(Clone)]
struct PortMenu {
    rail: WeakEntity<Rail>,
    workspace: WeakEntity<Workspace>,
    /// The row's project, none for a container's row listed apart.
    group: Option<ProjectGroupKey>,
    /// That project's key again while it is closed (#617), which Open and Show Logs open first.
    closed: Option<ProjectGroupKey>,
    port: u16,
    pid: u32,
    url: String,
    service: Option<PortService>,
    is_container: bool,
}

impl PortMenu {
    /// Open and Copy URL, then Restart Service, Stop Service and Show Logs for a service, Stop
    /// Container for a container, or Stop Process.
    fn build(&self, window: &mut Window, cx: &mut App) -> Entity<ContextMenu> {
        let menu = self.clone();
        ContextMenu::build(window, cx, move |context, _, _| {
            let (open, copy) = (menu.clone(), menu.url.clone());
            let context = context
                .entry("Open in a Browser Tab", None, move |window, cx| {
                    open.rail
                        .update(cx, |rail, cx| {
                            rail.open_port(
                                &open.workspace,
                                open.closed.as_ref(),
                                open.url.clone(),
                                window,
                                cx,
                            )
                        })
                        .flatten()
                        .log_err();
                })
                .entry("Copy URL", None, move |_, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(copy.clone()));
                })
                .separator();
            match (&menu.service, menu.group.clone()) {
                (Some(service), Some(group)) => {
                    let (restart, stop, logs) = (menu.clone(), menu.clone(), menu.clone());
                    let (stop_service, logs_service) = (service.clone(), service.clone());
                    context
                        .entry("Restart Service", None, move |window, cx| {
                            restart
                                .rail
                                .update(cx, |rail, cx| {
                                    rail.restart_port(&group, restart.port, window, cx);
                                })
                                .log_err();
                        })
                        .entry("Stop Service", None, move |window, cx| {
                            let service = stop_service.clone();
                            stop.rail
                                .update(cx, |rail, cx| rail.stop_service_port(service, window, cx))
                                .log_err();
                        })
                        .entry("Show Logs", None, move |window, cx| {
                            logs.rail
                                .update(cx, |rail, cx| {
                                    rail.show_logs(
                                        &logs.workspace,
                                        logs.closed.as_ref(),
                                        &logs_service,
                                        window,
                                        cx,
                                    )
                                })
                                .flatten()
                                .log_err();
                        })
                }
                _ if menu.is_container => {
                    let stop = menu.clone();
                    context.entry("Stop Container", None, move |window, cx| {
                        stop.rail
                            .update(cx, |rail, cx| {
                                rail.stop_container_port(stop.port, window, cx);
                            })
                            .log_err();
                    })
                }
                _ => {
                    let stop = menu.clone();
                    context.entry("Stop Process", None, move |window, cx| {
                        stop.rail
                            .update(cx, |rail, cx| {
                                rail.stop_port(stop.port, stop.pid, window, cx);
                            })
                            .log_err();
                    })
                }
            }
        })
    }
}

fn stop_words(
    pid: u32,
    service: Option<&PortService>,
    container: Option<&PortContainer>,
) -> (&'static str, String) {
    if let Some(container) = container {
        return (
            "Stop the Container",
            container.name.as_ref().map_or_else(
                || format!("{} did not name it", container.engine),
                |name| format!("{} stop {name}", container.engine),
            ),
        );
    }
    match service {
        Some(service) if service.user => (
            "Stop the User Service",
            format!("systemctl --user stop {}", service.unit),
        ),
        Some(service) => (
            "Stop the System Service",
            format!(
                "systemctl stop {}, which asks for authorization",
                service.unit
            ),
        ),
        None => ("Stop the Process", format!("SIGTERM to pid {pid}")),
    }
}

/// The toast for a service `systemctl` could not stop (#603), with the command that stops it by
/// hand to copy. Marley does not signal its process instead: systemd would start it again.
fn refused_toast(service: &Service, reason: &str) -> Toast {
    let kind = if service.user { "user" } else { "system" };
    let command = ports::hand_command(service);
    let message = format!(
        "Could not stop the {kind} service {}: {reason}. Marley leaves its process alone, since \
         systemd would start it again. To stop it yourself, run: {command}",
        service.unit
    );
    Toast::new(NotificationId::unique::<Ports>(), message).on_click("Copy Command", move |_, cx| {
        cx.write_to_clipboard(ClipboardItem::new_string(command.clone()));
    })
}

/// The project group `key`'s listeners as rows (#521): the port and the process's name as the
/// title, the URL, and a tooltip with the command line, the working directory and the pid.
fn port_snapshots(key: &ProjectGroupKey, filter: &str, cx: &App) -> Vec<PortSnapshot> {
    Ports::of(key, cx)
        .into_iter()
        .map(|found| {
            let state = found
                .listener
                .service
                .as_ref()
                .and_then(|service| ports::unit_word(service, found.listener.pid == 0, cx));
            port_snapshot(found, filter, state)
        })
        .collect()
}

/// One listener as a row's snapshot, `state` its service's word (#615).
fn port_snapshot(found: ProjectListener, filter: &str, state: Option<String>) -> PortSnapshot {
    let listener = found.listener;
    let port = listener.address.port();
    let title = format!(":{port} {}", listener.name);
    let matched = filter_match(filter, &title);
    let service_line = listener
        .service
        .as_ref()
        .map_or_else(String::new, |service| {
            let kind = if service.user { "user" } else { "system" };
            format!("\n{kind} service {}", service.unit)
        });
    let container = found.container.map(|container| PortContainer {
        engine: container.engine.command().to_string(),
        name: container.name,
        target: container.target,
    });
    let tooltip = match &container {
        Some(container) => format!(
            "{}{}\nproxy pid {}",
            listener.command,
            container
                .target
                .as_ref()
                .map_or_else(String::new, |target| format!("\nto {target}")),
            listener.pid
        ),
        None => format!(
            "{}\nin {}\npid {}{service_line}",
            listener.command,
            listener.cwd.display(),
            listener.pid
        ),
    };
    let service = listener.service.map(|service| PortService {
        unit: service.unit,
        user: service.user,
    });
    PortSnapshot {
        port,
        pid: listener.pid,
        url: marley_browser::ports::url(listener.address),
        tooltip,
        service,
        container,
        state,
        title,
        matched,
    }
}

/// A container port's second line: its container's name, else where its proxy sends it.
fn container_line(container: &PortContainer) -> String {
    match (&container.name, &container.target) {
        (Some(name), _) => format!("container {name}"),
        (None, Some(target)) => format!("→ {target}"),
        (None, None) => format!("a {} container", container.engine),
    }
}

/// The toast for a container its engine could not stop, or did not name (#614), with the command
/// that stops it by hand to copy.
fn container_refused_toast(command: String, reason: &str) -> Toast {
    let message =
        format!("Could not stop the container: {reason}. To stop it yourself, run: {command}");
    Toast::new(NotificationId::unique::<Ports>(), message).on_click("Copy Command", move |_, cx| {
        cx.write_to_clipboard(ClipboardItem::new_string(command.clone()));
    })
}

/// Everything in a window the rail follows.
#[derive(Default)]
struct Watched {
    workspaces: Vec<Entity<Workspace>>,
    projects: Vec<Entity<Project>>,
    views: Vec<Entity<TerminalView>>,
    browsers: Vec<Entity<BrowserView>>,
    panels: Vec<Entity<AgentPanel>>,
    threads: Vec<Entity<AcpThread>>,
    agent_servers: Vec<Entity<AgentServerStore>>,
}

impl Watched {
    fn in_window(multi_workspace: &Entity<MultiWorkspace>, cx: &App) -> Self {
        let workspaces: Vec<Entity<Workspace>> =
            multi_workspace.read(cx).workspaces().cloned().collect();
        let views = workspaces
            .iter()
            .flat_map(|workspace| workspace.read(cx).items_of_type::<TerminalView>(cx))
            .collect();
        let browsers = workspaces
            .iter()
            .flat_map(|workspace| workspace.read(cx).items_of_type::<BrowserView>(cx))
            .collect();
        let panels: Vec<Entity<AgentPanel>> = workspaces
            .iter()
            .filter_map(|workspace| workspace.read(cx).panel::<AgentPanel>(cx))
            .collect();
        let threads = panels
            .iter()
            .flat_map(|panel| live_threads(panel, cx))
            .collect();
        let projects: Vec<Entity<Project>> = workspaces
            .iter()
            .map(|workspace| workspace.read(cx).project().clone())
            .collect();
        let agent_servers = projects
            .iter()
            .map(|project| project.read(cx).agent_server_store().clone())
            .collect();
        Self {
            workspaces,
            projects,
            views,
            browsers,
            panels,
            threads,
            agent_servers,
        }
    }
}

/// Keeps each entity's subscriptions, makes them for each new entity, and returns the map for
/// what exists now; whatever the old map still holds is dropped, ending those subscriptions.
/// What the rail keeps in the window's saved sidebar state, beside Zed's sidebar's fields.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct RailState {
    /// The width the user set, if any.
    width: Option<f32>,
    /// Whether the user closed the rail.
    closed: bool,
}

/// The fields of a saved sidebar blob the rail reads. `width` and `width_set_by_user` are the
/// names Zed's sidebar writes and reads; `marley_rail_closed` is the rail's own, which Zed's
/// sidebar ignores.
#[derive(Default, serde::Deserialize)]
struct SavedRail {
    #[serde(default)]
    width: Option<f32>,
    #[serde(default)]
    width_set_by_user: bool,
    #[serde(default)]
    marley_rail_closed: bool,
}

/// The rail's fields in a saved sidebar blob. A blob that cannot be read holds nothing for it.
fn read_rail_state(blob: &str) -> RailState {
    let saved: SavedRail = serde_json::from_str(blob).unwrap_or_default();
    RailState {
        width: saved.width.filter(|_| saved.width_set_by_user),
        closed: saved.marley_rail_closed,
    }
}

/// `zed_state`, the Zed sidebar's saved blob, with the rail's fields written into it and every
/// other field kept, so Zed's sidebar still restores its own state from it.
fn write_rail_state(zed_state: Option<&str>, state: RailState) -> String {
    let mut blob: serde_json::Map<String, serde_json::Value> = zed_state
        .and_then(|zed_state| serde_json::from_str(zed_state).ok())
        .unwrap_or_default();
    blob.insert("width".into(), serde_json::json!(state.width));
    blob.insert("width_set_by_user".into(), state.width.is_some().into());
    blob.insert("marley_rail_closed".into(), state.closed.into());
    serde_json::Value::Object(blob).to_string()
}

/// The rail's projectless groups in a saved sidebar blob (#601): their workspace ids, in order.
#[derive(Default, serde::Deserialize)]
struct SavedGroups {
    #[serde(default)]
    marley_groups: Vec<WorkspaceId>,
}

/// The projectless groups' workspace ids a saved sidebar blob keeps; none when it cannot be read.
fn read_rail_groups(blob: &str) -> Vec<WorkspaceId> {
    serde_json::from_str::<SavedGroups>(blob)
        .map(|saved| saved.marley_groups)
        .unwrap_or_default()
}

/// `blob` with the projectless groups' workspace ids written into it and every other field kept.
fn write_rail_groups(blob: &str, groups: &[WorkspaceId]) -> String {
    let mut fields: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(blob).unwrap_or_default();
    fields.insert("marley_groups".into(), serde_json::json!(groups));
    serde_json::Value::Object(fields).to_string()
}

/// Whether a git store's event can change a project's worktree rows (#510): a repository come or
/// gone, and a repository's worktrees, `HEAD` or branches, never its file statuses, which change
/// with every save.
const fn changes_the_worktrees(event: &GitStoreEvent) -> bool {
    matches!(
        event,
        GitStoreEvent::RepositoryAdded
            | GitStoreEvent::RepositoryRemoved(_)
            | GitStoreEvent::RepositoryUpdated(
                _,
                RepositoryEvent::GitWorktreeListChanged
                    | RepositoryEvent::HeadChanged
                    | RepositoryEvent::BranchListChanged,
                _,
            )
    )
}

fn resubscribe<E: 'static, S>(
    old: &mut HashMap<EntityId, S>,
    entities: &[Entity<E>],
    mut subscribe: impl FnMut(&Entity<E>) -> S,
) -> HashMap<EntityId, S> {
    entities
        .iter()
        .map(|entity| {
            let subscriptions = old
                .remove(&entity.entity_id())
                .unwrap_or_else(|| subscribe(entity));
            (entity.entity_id(), subscriptions)
        })
        .collect()
}

/// A terminal row's icon: its agent CLI's, or the terminal's.
fn terminal_icon(row: &TerminalRow) -> IconName {
    row.agent
        .map_or(IconName::Terminal, |agent| agents::cli_icon(agent.kind))
}

/// A terminal row's icon in the rail: its agent CLI's, or for a shell a prompt, `>_` in the
/// buffer font, as Warp's tab list marks a shell. Zed's terminal icons draw the prompt in a box.
fn rail_terminal_icon(row: &TerminalRow, cx: &App) -> AnyElement {
    if row.agent.is_some() {
        Icon::new(terminal_icon(row))
            .size(IconSize::Small)
            .color(Color::Muted)
            .into_any_element()
    } else {
        Label::new(">_")
            .size(LabelSize::Small)
            .color(Color::Muted)
            .buffer_font(cx)
            .into_any_element()
    }
}

/// A thread's row as Zed's thread list draws it, with the agent's icon: the switcher's rows.
fn thread_item(id: SharedString, row: ThreadRow, icon: &AgentIcon) -> ThreadItem {
    let item = ThreadItem::new(id, row.title)
        .highlight_positions(row.highlight)
        .status(ui_status(row.status))
        .notified(row.attention)
        .selected(row.selected)
        .rounded(true);
    match icon {
        AgentIcon::Named(icon) => item.icon(*icon),
        AgentIcon::Svg(path) => item.custom_icon_from_external_svg(path.clone()),
    }
}

/// The status Zed's thread row draws for a rail status.
const fn ui_status(status: ThreadStatus) -> AgentThreadStatus {
    match status {
        ThreadStatus::Done => AgentThreadStatus::Completed,
        ThreadStatus::Running => AgentThreadStatus::Running,
        ThreadStatus::Waiting => AgentThreadStatus::WaitingForConfirmation,
        ThreadStatus::Error => AgentThreadStatus::Error,
    }
}

/// The events after which a thread's row may read differently: its status, its title, or a
/// confirmation asked or answered. Streamed output changes none of these.
const fn changes_the_row(event: &AcpThreadEvent) -> bool {
    matches!(
        event,
        AcpThreadEvent::StatusChanged
            | AcpThreadEvent::TitleUpdated
            | AcpThreadEvent::ToolAuthorizationRequested(_)
            | AcpThreadEvent::ToolAuthorizationReceived(_)
            | AcpThreadEvent::Stopped(_)
            | AcpThreadEvent::Error
            | AcpThreadEvent::LoadError(_)
            | AcpThreadEvent::Refusal
    )
}

/// The events after which a project's folders differ: one added, removed or moved, or the paths
/// its group is keyed by. A busy project emits many others, and none of them changes a row.
const fn changes_the_folders(event: &project::Event) -> bool {
    matches!(
        event,
        project::Event::WorktreeAdded(_)
            | project::Event::WorktreeRemoved(_)
            | project::Event::WorktreeOrderChanged
            | project::Event::WorktreePathsChanged { .. }
    )
}

/// The agents of a group's workspaces that wait on the user (#508): each Agent Panel
/// conversation's first tool call waiting for confirmation, each terminal's Claude Code that waits
/// on a permission or a question, and each Browser tab's paused click, listed once per page.
fn inbox_entries(project: &str, members: &[Entity<Workspace>], snapshot: &mut Snapshot, cx: &App) {
    let hub = BrowserHub::try_global(cx);
    let marking = system_one::use_mode(INBOX_RISK.name, cx) != SystemOneMode::Off;
    let routing = system_one::use_mode(QUESTION_ROUTE.name, cx) != SystemOneMode::Off;
    for member in members {
        let scope = (marking || routing).then(|| RiskScope::of(member, marking, routing, cx));
        let conversations = member
            .read(cx)
            .panel::<AgentPanel>(cx)
            .map(|panel| panel.read(cx).conversation_views())
            .unwrap_or_default();
        for conversation in conversations {
            if let Some((mut entry, target, waiting)) =
                thread_entry(project, member, &conversation, cx)
            {
                if let Some(scope) = &scope {
                    mark(&mut entry, &waiting, scope, snapshot);
                }
                snapshot.inbox.insert(entry.key.clone(), target);
                snapshot.rail.inbox.push(entry);
            }
        }
        for view in member.read(cx).items_of_type::<TerminalView>(cx) {
            let waiting = cx
                .try_global::<AgentEvents>()
                .and_then(|events| events.seat(view.entity_id()))
                .filter(|seat| seat.state == marley_fleet::State::Waiting);
            let Some(seat) = waiting else {
                continue;
            };
            let id = view.entity_id().as_u64();
            let mut entry = seat_entry(project, id, seat);
            // Codex's waits get their risk and route marks with their answers (#651).
            if let Some(scope) = scope.as_ref().filter(|_| !codex_events::is_codex(seat)) {
                let waiting = seat_waiting(seat, &entry.ask);
                mark(&mut entry, &waiting, scope, snapshot);
            }
            snapshot
                .inbox
                .insert(entry.key.clone(), InboxTarget::Terminal(id));
            snapshot.rail.inbox.push(entry);
        }
        if let Some(hub) = &hub {
            click_entries(
                project,
                member,
                hub.read(cx),
                (marking, routing),
                snapshot,
                cx,
            );
        }
    }
}

/// A harness session's row (#534): its title, then the question it waits on, how long a working
/// session has been quiet (#547's rule), or its state; muted while the connection is not up. A
/// state its labels say was not declared is drawn weaker, its source's word at the line's end,
/// and the labels' progress and most-used quota window get a line each (#640). A click shows its
/// tab.
fn harness_row(
    seat: &marley_fleet::Session,
    stale: bool,
    quiet_after_ms: u64,
    now_ms: u64,
    cx: &Context<Rail>,
) -> AnyElement {
    let signals = Signals::of(&seat.labels);
    let weaker = !signals.declared();
    let (word, color) = match seat.state {
        marley_fleet::State::Starting => ("starting", Color::Muted),
        marley_fleet::State::Working => ("working", Color::Accent),
        marley_fleet::State::Idle => ("idle", Color::Muted),
        marley_fleet::State::Waiting => ("waiting", Color::Warning),
        marley_fleet::State::Error => ("error", Color::Error),
        marley_fleet::State::Done => ("done", Color::Success),
    };
    let silent_ms = now_ms.saturating_sub(seat.last_event_ms);
    let line = match (&seat.question, seat.state) {
        (Some(question), _) => one_line(crate::harness::shown_prompt(&question.prompt)),
        (None, marley_fleet::State::Working)
            if quiet_after_ms > 0 && silent_ms >= quiet_after_ms =>
        {
            format!("no update in {} m", silent_ms / 60_000)
        }
        _ => word.to_string(),
    };
    // The marks sit in the line's end slot, so a long question is cut before them.
    let marks: String = signals
        .source
        .as_ref()
        .filter(|_| weaker)
        .map(|source| {
            source
                .word()
                .chars()
                .take(SOURCE_WORD_CHARS)
                .collect::<String>()
        })
        .into_iter()
        .chain(stale.then(|| "stale".to_string()))
        .fold(String::new(), |mut marks, mark| {
            marks.push_str(" · ");
            marks.push_str(&mark);
            marks
        });
    let mut lines = vec![RowLine {
        text: line,
        state: (!marks.is_empty()).then_some(marks),
        color: Color::Muted,
        cut: Cut::End,
    }];
    if let Some(progress) = &signals.progress {
        let text = match (progress.percent, &progress.activity) {
            (Some(percent), Some(activity)) => format!("{percent} % · {activity}"),
            (Some(percent), None) => format!("{percent} %"),
            (None, Some(activity)) => activity.clone(),
            (None, None) => String::new(),
        };
        lines.push(RowLine::muted(text));
    }
    if let Some(window) = signals.quota.first() {
        lines.push(RowLine {
            text: window.kind.clone(),
            state: Some(format!(" {}", used_words(window, now_ms))),
            color: Color::Muted,
            cut: Cut::End,
        });
    }
    let icon = Indicator::dot()
        .color(if stale || weaker { Color::Muted } else { color })
        .into_any_element();
    let title = Label::new(seat.title.clone())
        .size(LabelSize::Small)
        .color(if stale { Color::Muted } else { Color::Default })
        .truncate()
        .into_any_element();
    let id = seat.id.clone();
    let tooltip = (!signals.is_empty()).then(|| signals_tooltip(&signals, &seat.title, now_ms));
    row_card(
        SharedString::from(format!("marley-rail-harness-{}", seat.id)),
        format!("marley-rail-harness-icon-{}", seat.id),
        false,
        icon,
        title,
        lines,
        cx,
    )
    .when_some(tooltip, |row, (title, meta)| {
        row.tooltip(move |_, cx| Tooltip::with_meta(title.clone(), None, meta.clone(), cx))
    })
    .on_click(cx.listener(move |rail, _, window, cx| rail.open_harness(&id, window, cx)))
    .into_any_element()
}

/// How many characters of a source word Marley does not know the row shows.
const SOURCE_WORD_CHARS: usize = 12;

/// A harness row's tooltip (#640): where its state came from as the title, then every quota
/// window, most used first, and the account.
fn signals_tooltip(signals: &Signals, title: &str, now_ms: u64) -> (SharedString, SharedString) {
    let source = match &signals.source {
        Some(StateSource::Protocol) => "State from the agent's protocol".to_string(),
        Some(StateSource::Reported) => "State reported by the agent".to_string(),
        Some(StateSource::Runtime) => {
            "State from the harness, while it restarts the agent".to_string()
        }
        Some(StateSource::Detected) => {
            "State detected from its screen: shown, not acted on".to_string()
        }
        Some(StateSource::Other(word)) => {
            format!("State source \"{word}\": not known, not acted on")
        }
        None => title.to_string(),
    };
    let meta: Vec<String> = signals
        .quota
        .iter()
        .map(|window| format!("{} {}", window.kind, used_words(window, now_ms)))
        .chain(
            signals
                .account
                .as_ref()
                .map(|account| format!("Account: {account}")),
        )
        .collect();
    (source.into(), meta.join("\n").into())
}

/// How much of a quota window is used and when it resets: `62 % · resets in 1 h 35 m`.
fn used_words(window: &QuotaWindow, now_ms: u64) -> String {
    window.resets_at_ms.map_or_else(
        || format!("{} %", window.percent_used),
        |reset| format!("{} % · {}", window.percent_used, reset_words(reset, now_ms)),
    )
}

/// When a quota window resets, from `now_ms`: `resets in 35 m`, `resets in 1 h 35 m`, `resets in
/// 3 d 4 h`, or `resets now` for one under a minute away or past (#640).
fn reset_words(resets_at_ms: u64, now_ms: u64) -> String {
    let seconds = resets_at_ms.saturating_sub(now_ms) / 1000;
    match seconds {
        0..60 => "resets now".to_string(),
        60..86_400 => format!("resets in {}", marley_rail::waited_words(seconds)),
        _ => format!(
            "resets in {} d {} h",
            seconds / 86_400,
            seconds % 86_400 / 3600
        ),
    }
}

/// Each harness session waiting on a question, as an inbox entry, its options after its prompt
/// (#534).
fn harness_entries(snapshot: &mut Snapshot, cx: &App) {
    for seat in Harness::seats(cx).seats() {
        let Some(question) = &seat.question else {
            continue;
        };
        // A state the agent did not declare is shown on its row and never asks the user (#640).
        if !Signals::of(&seat.labels).declared() {
            continue;
        }
        let mut ask = one_line(crate::harness::shown_prompt(&question.prompt));
        if !question.options.is_empty() {
            ask = format!("{ask} ({})", question.options.join(", "));
        }
        let key = format!("harness:{}:{ask}", seat.id);
        if let std::collections::hash_map::Entry::Vacant(vacant) = snapshot.inbox.entry(key.clone())
        {
            vacant.insert(InboxTarget::Harness(seat.id.clone()));
            snapshot.rail.inbox.push(InboxEntry {
                key,
                kind: InboxKind::Harness,
                agent: seat.title.clone(),
                project: "Harness".to_string(),
                ask,
                waited: String::new(),
                answers: false,
                chips: Vec::new(),
                level: 0,
                route: None,
            });
        }
    }
}

/// A terminal's Claude Code, or Codex (#650), that waits, as an inbox entry.
fn seat_entry(project: &str, id: u64, seat: &marley_fleet::Session) -> InboxEntry {
    let ask = seat.question.as_ref().map_or_else(
        || "Waits for you".to_string(),
        |question| one_line(&question.prompt),
    );
    InboxEntry {
        key: format!("terminal:{id}:{ask}"),
        kind: InboxKind::Terminal,
        agent: agent_events::seat_agent(seat)
            .unwrap_or(AgentKind::Claude)
            .display_name()
            .to_string(),
        project: project.to_string(),
        ask,
        waited: String::new(),
        answers: false,
        chips: Vec::new(),
        level: 0,
        route: None,
    }
}

/// Each Browser tab of `member` that holds an agent's click, as an inbox entry, once a page; with
/// the chip of the click's class while the risk use is on (#568), and the owner's route while the
/// question route is (#570): the click waits because it could pay, delete, send or change an
/// account.
fn click_entries(
    project: &str,
    member: &Entity<Workspace>,
    hub: &BrowserHub,
    (marking, routing): (bool, bool),
    snapshot: &mut Snapshot,
    cx: &App,
) {
    for view in member.read(cx).items_of_type::<BrowserView>(cx) {
        let Some(target) = view.read(cx).target().map(str::to_string) else {
            continue;
        };
        let Some(sentence) = hub.pause_sentence(&target) else {
            continue;
        };
        let key = format!("click:{target}");
        let class_chip = hub.pause_class(&target).and_then(click_chip);
        let route = class_chip.filter(|_| routing).map(|chip| RouteMark {
            route: Route::Owner,
            source: RouteSource::Rule(chip.kind.words()),
        });
        let chips: Vec<Chip> = class_chip.filter(|_| marking).into_iter().collect();
        if let std::collections::hash_map::Entry::Vacant(vacant) = snapshot.inbox.entry(key.clone())
        {
            vacant.insert(InboxTarget::Click(target));
            snapshot.rail.inbox.push(InboxEntry {
                key,
                kind: InboxKind::Click,
                agent: "Browser tab".to_string(),
                project: project.to_string(),
                ask: sentence.to_string(),
                waited: String::new(),
                answers: true,
                level: if marking {
                    risk::level(ToolClass::Other, &chips)
                } else {
                    0
                },
                chips,
                route,
            });
        }
    }
}

/// The chip a held click's class gives its inbox entry (#568).
const fn click_chip(class: Class) -> Option<Chip> {
    let kind = match class {
        Class::Pays => ChipKind::Pays,
        Class::Deletes => ChipKind::Destroys,
        Class::Sends => ChipKind::SendsOut,
        Class::ChangesAccount => ChipKind::ChangesAccount,
        Class::Open | Class::Plain => return None,
    };
    Some(Chip::rules(kind))
}

impl RiskScope {
    /// What `workspace`'s entries are read against: its folders, whether it is on this machine,
    /// the redactor that finds a secret in a line, and which uses read them.
    fn of(workspace: &Entity<Workspace>, marking: bool, routing: bool, cx: &App) -> Self {
        let (folders, local) = system_one::project_of(workspace.read(cx), cx);
        Self {
            folders,
            local,
            redactor: crate::mcp::model_redactor(cx),
            marking,
            routing,
        }
    }
}

/// Reads what `entry` waits on with Marley's rules: while the risk use is on, marks it with its
/// chips and level and keeps beside it what that use logs or asks (#568); while the question
/// route is on, marks who should answer it by the rules and keeps what the route logs or asks
/// (#570), with the chips as its facts whether the risk use is on or not.
fn mark(entry: &mut InboxEntry, waiting: &Waiting, scope: &RiskScope, snapshot: &mut Snapshot) {
    let root = scope.folders.first();
    let cwd = waiting
        .cwd
        .as_ref()
        .map(|cwd| root.map_or_else(|| cwd.clone(), |root| root.join(cwd)))
        .or_else(|| root.cloned());
    let action = Action {
        tool: waiting.tool,
        line: &waiting.line,
        paths: &waiting.paths,
        cwd: cwd.as_deref(),
        folders: &scope.folders,
        home: Some(util::paths::home_dir().as_path()),
        secret: scope.redactor.redact(&waiting.line).count > 0,
    };
    let chips = risk::classify(&action);
    let found = if chips.is_empty() {
        "nothing".to_string()
    } else {
        chips
            .iter()
            .map(|chip| chip.kind.words())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let project = system_one::project_name(&scope.folders);
    if scope.marking {
        let nouls: Vec<&str> = chips.iter().filter_map(|chip| chip.kind.noul()).collect();
        let asking = Asking {
            subject: entry.key.clone(),
            facts: vec![
                ("tool", waiting.tool_name.clone()),
                ("agent", waiting.agent.clone()),
                ("project", project.clone()),
                ("code found", found.clone()),
            ],
            texts: vec![("ask", entry.ask.clone())],
            verdict: (!nouls.is_empty()).then(|| system_one::nouls_verdict(&nouls)),
            project: project.clone(),
            folders: scope.folders.clone(),
            local: scope.local,
        };
        snapshot.inbox_risk.insert(
            entry.key.clone(),
            RiskAsking {
                ask: entry.ask.clone(),
                tool: waiting.tool,
                asking,
            },
        );
        entry.level = risk::level(waiting.tool, &chips);
        entry.chips.clone_from(&chips);
    }
    if scope.routing {
        let class = route::classify(&route::Facts {
            action: &action,
            tool_name: &waiting.tool_name,
            chips: &chips,
            options: &waiting.options,
        });
        entry.route = class.mark();
        let mut texts = vec![("ask", entry.ask.clone())];
        if !waiting.options.is_empty() {
            texts.push(("options", waiting.options.join(", ")));
        }
        if let Some(prompt) = &waiting.prompt {
            texts.push(("prompt", prompt.clone()));
        }
        let asking = Asking {
            subject: entry.key.clone(),
            facts: vec![
                ("tool", waiting.tool_name.clone()),
                ("agent", waiting.agent.clone()),
                ("project", project.clone()),
                ("chips", found),
            ],
            texts,
            verdict: match class {
                route::Class::Owner(_) => Some(system_one::choice_verdict("route", "owner")),
                route::Class::CouldProceed(_) => {
                    Some(system_one::choice_verdict("route", "agent_proceeds"))
                }
                route::Class::Open => None,
            },
            project,
            folders: scope.folders.clone(),
            local: scope.local,
        };
        snapshot.inbox_route.insert(
            entry.key.clone(),
            RouteAsking {
                ask: entry.ask.clone(),
                asking,
            },
        );
    }
}

/// What an Agent Panel tool call waits to do (#568): its kind's class, the command it runs or its
/// label as plain text, the paths it names, and a terminal tool's `cd`.
fn thread_waiting(call: &acp_thread::ToolCall, agent: &str, cx: &App) -> Waiting {
    let tool = match call.kind {
        acp::ToolKind::Read
        | acp::ToolKind::Search
        | acp::ToolKind::Fetch
        | acp::ToolKind::Think => ToolClass::Read,
        acp::ToolKind::Edit | acp::ToolKind::Move => ToolClass::Write,
        acp::ToolKind::Delete => ToolClass::Delete,
        acp::ToolKind::Execute => ToolClass::Execute,
        _ => ToolClass::Other,
    };
    let text = |key: &str| {
        call.raw_input
            .as_ref()
            .and_then(|input| input.get(key))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
    };
    let line = text("command").unwrap_or_else(|| unescape(call.label.read(cx).source()));
    let mut paths: Vec<PathBuf> = call
        .locations
        .iter()
        .map(|location| location.path.clone())
        .collect();
    paths.extend(
        ["path", "file_path", "abs_path"]
            .into_iter()
            .filter_map(text)
            .map(PathBuf::from),
    );
    Waiting {
        tool,
        tool_name: call.tool_name.as_ref().map_or_else(
            || format!("{:?}", call.kind).to_lowercase(),
            ToString::to_string,
        ),
        line: one_line(&line),
        paths,
        cwd: text("cd").map(PathBuf::from),
        agent: format!("{agent} in the Agent Panel"),
        options: Vec::new(),
        prompt: None,
    }
}

/// What a terminal's Claude Code waits on (#568): the tool and its preview of a
/// `Permission for <Tool: preview>` wait, or a question.
fn seat_waiting(seat: &marley_fleet::Session, ask: &str) -> Waiting {
    let cwd = seat.labels.get(claude_events::CWD_LABEL).map(PathBuf::from);
    let agent = "Claude Code in a terminal".to_string();
    let prompt = seat
        .labels
        .get(claude_events::PROMPT_LABEL)
        .filter(|prompt| !prompt.is_empty())
        .cloned();
    let Some(asked) = ask.strip_prefix("Permission for ") else {
        return Waiting {
            tool: ToolClass::Question,
            tool_name: "AskUserQuestion".to_string(),
            line: ask.to_string(),
            paths: Vec::new(),
            cwd,
            agent,
            options: seat
                .question
                .as_ref()
                .map(|question| question.options.clone())
                .unwrap_or_default(),
            prompt,
        };
    };
    let (name, preview) = asked.split_once(": ").unwrap_or((asked, ""));
    let tool = ToolClass::of_claude_tool(name);
    let paths = if tool == ToolClass::Write && !preview.is_empty() {
        vec![PathBuf::from(preview)]
    } else {
        Vec::new()
    };
    Waiting {
        tool,
        tool_name: name.to_string(),
        line: preview.to_string(),
        paths,
        cwd,
        agent,
        options: Vec::new(),
        prompt,
    }
}

/// A label's Markdown source as plain text: the backslash before an escaped character dropped,
/// and `&lt;` read back.
fn unescape(markdown: &str) -> String {
    let mut plain = String::with_capacity(markdown.len());
    let mut characters = markdown.chars();
    while let Some(character) = characters.next() {
        if character == '\\' {
            plain.extend(characters.next());
        } else {
            plain.push(character);
        }
    }
    plain.replace("&lt;", "<")
}

/// A model's reading of who should answer an entry (#570): the choice at or above the floor, and
/// `unclear` for `cannot_tell`, a reading under the floor, no signal, a refusal or no answer at
/// all; none while the use is off or for a rule's row.
fn route_reading(reading: &Reading) -> Option<(Route, Option<f64>)> {
    match reading {
        Reading::Model(reads) => Some(
            reads
                .iter()
                .find(|read| read.key == "route")
                .and_then(|read| match &read.signal {
                    Signal::Choice { option, confidence } => {
                        Some((route::route_of_choice(option), Some(*confidence)))
                    }
                    Signal::Noul { .. } | Signal::Score { .. } | Signal::Nothing(_) => None,
                })
                .unwrap_or((Route::Unclear, None)),
        ),
        Reading::Refused(_) | Reading::Unavailable(_) => Some((Route::Unclear, None)),
        Reading::Off | Reading::Rules(_) => None,
    }
}

/// A model's reading of a tool entry (#568): each noul that holds as a chip, with its
/// probability, and the urgency's expected level, rounded.
fn risk_reading(reading: &Reading) -> RiskReading {
    let Reading::Model(reads) = reading else {
        return RiskReading::default();
    };
    let mut found = RiskReading::default();
    for read in reads {
        match &read.signal {
            Signal::Noul {
                holds: true,
                probability,
            } => {
                if let Some(kind) = ChipKind::from_noul(&read.key) {
                    found.chips.push((kind, *probability));
                }
            }
            Signal::Score { score, .. } if read.key == "urgency" => {
                found.urgency = (1..=risk::TOP_LEVEL)
                    .rev()
                    .find(|level| *score >= f64::from(*level) - 0.5);
            }
            _ => {}
        }
    }
    found
}

/// A conversation's first tool call waiting for confirmation, as an inbox entry, with the
/// allow-once and deny-once options that answer it in place. A sandbox escalation gets no
/// buttons: the panel gates its Allow behind a check the inbox would go around. The agent's name
/// and icon are the ones its thread's row shows.
fn thread_entry(
    project: &str,
    member: &Entity<Workspace>,
    conversation: &Entity<ConversationView>,
    cx: &App,
) -> Option<(InboxEntry, InboxTarget, Waiting)> {
    let view = conversation.read(cx);
    let (session, tool_call, options) = view.pending_tool_call(cx)?;
    let thread_view = view.thread_view(&session)?;
    let thread_view = thread_view.read(cx);
    let (_, call) = thread_view.thread.read(cx).tool_call(&tool_call)?;
    let ask = one_line(call.label.read(cx).source());
    let option = |kind| {
        options
            .first_option_of_kind(kind)
            .map(|option| (option.option_id.clone(), option.kind))
    };
    let answers = option(acp::PermissionOptionKind::AllowOnce)
        .zip(option(acp::PermissionOptionKind::RejectOnce))
        .filter(|_| call.sandbox_authorization_details.is_none())
        .map(<[_; 2]>::from);
    let thread_key = view.parent_id().to_key_string();
    let key = format!("thread:{thread_key}:{tool_call}");
    let workspace_project = member.read(cx).project();
    let icon = agents::thread_icon(&thread_view.agent_id, workspace_project, cx);
    let agent = agents::thread_agent_name(&thread_view.agent_id, workspace_project, cx).to_string();
    let waiting = thread_waiting(call, &agent, cx);
    let entry = InboxEntry {
        key,
        kind: InboxKind::Thread,
        agent,
        project: project.to_string(),
        ask,
        waited: String::new(),
        answers: answers.is_some(),
        chips: Vec::new(),
        level: 0,
        route: None,
    };
    let target = InboxTarget::Thread {
        thread_key,
        conversation: conversation.downgrade(),
        session,
        tool_call,
        answers,
        icon,
    };
    Some((entry, target, waiting))
}

/// `text` on one line: its runs of whitespace, line breaks included, as single spaces.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The root thread of each conversation the panel holds, shown or kept in the background.
fn live_threads(panel: &Entity<AgentPanel>, cx: &App) -> Vec<Entity<AcpThread>> {
    panel
        .read(cx)
        .conversation_views()
        .into_iter()
        .filter_map(|conversation| {
            let view = conversation.read(cx).root_thread_view()?;
            Some(view.read(cx).thread.clone())
        })
        .collect()
}

/// The status of every thread the workspaces' panels hold, by thread id, and the thread each
/// panel shows.
fn live_statuses(
    workspaces: &[Entity<Workspace>],
    cx: &App,
) -> (HashMap<ThreadId, ThreadStatus>, HashSet<ThreadId>) {
    let mut statuses = HashMap::default();
    let mut active = HashSet::default();
    for panel in workspaces
        .iter()
        .filter_map(|workspace| workspace.read(cx).panel::<AgentPanel>(cx))
    {
        let panel = panel.read(cx);
        active.extend(panel.active_thread_id(cx));
        // A conversation still connecting has no thread yet, and nothing to report.
        let loaded = panel
            .conversation_views()
            .into_iter()
            .filter_map(|conversation| {
                let view = conversation.read(cx).root_thread_view()?;
                Some((conversation, view))
            });
        for (conversation, view) in loaded {
            let conversation = conversation.read(cx);
            let thread = view.read(cx).thread.read(cx);
            statuses.insert(
                conversation.parent_id(),
                marley_rail::thread_status(
                    conversation.root_thread_has_pending_tool_call(cx),
                    thread.had_error(),
                    thread.status() == acp_thread::ThreadStatus::Generating,
                ),
            );
        }
    }
    (statuses, active)
}

/// A group's threads, newest first. The metadata store files a thread under the main worktree
/// paths of the project it ran in; rows written before those were kept are found by their folder
/// paths, and a row whose paths disagree with its group by its workspace's own roots. Drafts are
/// listed only while their panel shows them, so a new thread appears at once. A closed group
/// (#617) has no `listed` workspace: its rows open the project first, and their agents' icons and
/// names are read against the shown workspace's project, whose agent servers are the user's
/// settings and extensions as every local project's are.
fn group_threads(
    group: &ProjectGroup,
    listed: Option<&Entity<Workspace>>,
    shown: &Entity<Workspace>,
    cx: &App,
) -> Vec<(ThreadSnapshot, ThreadEntry)> {
    let Some(store) = ThreadMetadataStore::try_global(cx) else {
        return Vec::new();
    };
    // A projectless group (#600) has no folder, and Zed archives such a workspace's threads.
    if group.key.path_list().paths().is_empty() {
        return Vec::new();
    }
    let store = store.read(cx);
    let host = group.key.host();
    let members: Vec<(PathList, &Entity<Workspace>)> = group
        .workspaces
        .iter()
        .map(|workspace| (PathList::new(&workspace.read(cx).root_paths(cx)), workspace))
        .collect();
    let (statuses, active) = live_statuses(&group.workspaces, cx);
    let mut seen: HashSet<ThreadId> = HashSet::default();
    let mut rows: Vec<&ThreadMetadata> = store
        .entries_for_main_worktree_path(group.key.path_list(), host.as_ref())
        .chain(store.entries_for_path(group.key.path_list(), host.as_ref()))
        .chain(
            members
                .iter()
                .filter(|(paths, _)| !paths.paths().is_empty())
                .flat_map(|(paths, _)| store.entries_for_path(paths, host.as_ref())),
        )
        .filter(|row| seen.insert(row.thread_id))
        .filter(|row| !row.is_draft() || active.contains(&row.thread_id))
        .collect();
    rows.sort_by_key(|row| Reverse(row.interacted_at.unwrap_or(row.updated_at)));
    rows.into_iter()
        .map(|row| {
            let workspace = members
                .iter()
                .find(|(paths, _)| paths == row.folder_paths())
                .map(|(_, workspace)| *workspace)
                .or(listed);
            let project = workspace.unwrap_or(shown).read(cx).project();
            let key = row.thread_id.to_key_string();
            (
                ThreadSnapshot {
                    key,
                    title: row.display_title().to_string(),
                    status: statuses.get(&row.thread_id).copied().unwrap_or_default(),
                    attention: false,
                    matched: None,
                },
                ThreadEntry {
                    workspace: workspace.map_or_else(WeakEntity::new_invalid, Entity::downgrade),
                    thread_id: row.thread_id,
                    agent: Agent::from(row.agent_id.clone()),
                    work_dirs: row.folder_paths().clone(),
                    title: row.title(),
                    icon: agents::thread_icon(&row.agent_id, project, cx),
                    agent_name: agents::thread_agent_name(&row.agent_id, project, cx),
                    closed: listed.is_none().then(|| group.key.clone()),
                },
            )
        })
        .collect()
}

/// One terminal's row: an agent row when a known agent CLI runs in its foreground, labelled with
/// the CLI's own title and its status, else the terminal's title and working directory.
fn terminal_snapshot(
    view: &Entity<TerminalView>,
    root: Option<&Path>,
    home: &Path,
    foreground_command: ForegroundCommand,
    last_output: Option<Instant>,
    now: Instant,
    cx: &App,
) -> TerminalSnapshot {
    let terminal_view = view.read(cx);
    let terminal = terminal_view.terminal();
    // An agent event the user has not seen lights the same dot as a bell (#538).
    let bell = terminal_view.has_bell() || crate::notifications::unread(view.entity_id(), cx);
    let kind = foreground_command(terminal, cx)
        .as_deref()
        .and_then(marley_agent::agent_kind_of)
        .or_else(|| remote_claude(view, cx));
    // Once Claude Code has sent its hook events (#519), or Codex's App Server its thread (#650),
    // they say what it is doing.
    let seat = kind.and_then(|kind| agent_events::seat_of(view.entity_id(), kind, cx));
    let shown = agent_events::stop_kind_shown(cx);
    let flag = crate::stall::flag_shown(cx);
    let agent = kind.map(|kind| TerminalAgent {
        kind,
        status: seat.map_or_else(
            || {
                // No output seen yet counts as quiet: the agent is at its prompt, as far as the
                // rail knows.
                let quiet_for =
                    last_output.map_or(Duration::MAX, |at| now.saturating_duration_since(at));
                marley_agent::agent_status(quiet_for, bell)
            },
            |seat| claude_events::seat_status(seat.state),
        ),
        mark: agent_mark(kind, terminal.read(cx), seat),
    });
    let (title, subtitle) = agent.map_or_else(
        || {
            (
                terminal_view.tab_content_text(0, cx).to_string(),
                marley_rail::working_directory_label(
                    terminal.read(cx).working_directory().as_deref(),
                    root,
                    Some(home),
                ),
            )
        },
        |agent| {
            // A name the user gave the terminal wins over the CLI's own title.
            let title = terminal_view.custom_title().map_or_else(
                || agent_title(&terminal.read(cx).breadcrumb_text, agent.kind),
                ToString::to_string,
            );
            let status = seat.map_or_else(
                || marley_agent::status_line(agent.kind, agent.status),
                |seat| match agent.kind {
                    AgentKind::Codex => codex_line(seat, cx),
                    _ => claude_events::seat_line(
                        seat,
                        agent_events::now_ms(),
                        no_update_after_ms(cx),
                        shown,
                        flag,
                    ),
                },
            );
            (title, Some(status))
        },
    );
    TerminalSnapshot {
        id: view.entity_id().as_u64(),
        title,
        subtitle,
        bell,
        agent,
        activity: seat.and_then(|seat| {
            if codex_events::is_codex(seat) {
                codex_events::seat_activity(seat)
            } else {
                claude_events::seat_activity(seat, shown)
            }
        }),
        flag: seat
            .filter(|seat| seat.state == marley_fleet::State::Working)
            .and_then(|seat| marley_agent::stall::tooltip(&seat.labels, flag)),
        // A plain terminal's running or last command (#551); an agent's rows say what it does.
        command: agent
            .is_none()
            .then(|| command_snapshot(terminal.read(cx)))
            .flatten(),
        running_error: crate::running_errors::mark(view.entity_id(), cx),
        // The seat's own events drive the agent's class in the rail's order (#542).
        reporting: reporting(seat, cx),
        turns: Turns::of(view.entity_id().as_u64(), cx)
            .iter()
            .rev()
            .map(|turn| TurnSnapshot {
                title: turn.title.clone(),
                files: turn.files,
                failed: turn.failed,
                injected: turn.injected,
                sha: turn.sha.clone(),
            })
            .collect(),
        turns_open: false,
        worktree: None,
        matched: None,
    }
}

/// The most characters of a command a row's line shows.
const COMMAND_LINE_MAX: usize = 60;

/// `terminal`'s last block for its row's line: the command on one line and cut, whether it runs,
/// its exit and how long it took, and whether it reads a password (#551).
fn command_snapshot(terminal: &Terminal) -> Option<CommandSnapshot> {
    let anchored = terminal.marley_anchored();
    let block = anchored.blocks().last()?;
    // An agent's block is the agent's session, which its own row told (#551's watch agrees).
    if marley_agent::agent_kind_of(&block.command).is_some() {
        return None;
    }
    let one_line = block
        .command
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if one_line.is_empty() {
        return None;
    }
    let text = if one_line.chars().count() > COMMAND_LINE_MAX {
        let mut cut: String = one_line.chars().take(COMMAND_LINE_MAX - 1).collect();
        cut.push('…');
        cut
    } else {
        one_line
    };
    let running = block.state != marley_terminal::BlockState::Finished;
    let duration = anchored
        .times(block.index)
        .and_then(|times| times.finished?.duration_since(times.started).ok())
        .map(marley_terminal::duration_label);
    Some(CommandSnapshot {
        text,
        running,
        exit_code: (!running).then_some(block.exit_code.0).flatten(),
        duration,
        password: running && terminal.marley_foreground_reads_password(),
    })
}

/// A command's line under its terminal's row: `cargo build · running`, `· done · 45 s`,
/// `· exit 101 · 4 m 12 s` in the error color, or `· waiting for a password` (#551).
fn command_line(command: CommandSnapshot) -> RowLine {
    let CommandSnapshot {
        text,
        running,
        exit_code,
        duration,
        password,
    } = command;
    let took = duration
        .map(|duration| format!(" · {duration}"))
        .unwrap_or_default();
    let (state, color) = match (running, password, exit_code) {
        (true, true, _) => (" · waiting for a password".to_string(), Color::Muted),
        (true, false, _) => (" · running".to_string(), Color::Muted),
        (false, _, Some(code)) if code != 0 => (format!(" · exit {code}{took}"), Color::Error),
        (false, _, _) => (format!(" · done{took}"), Color::Muted),
    };
    RowLine {
        text,
        state: Some(state),
        color,
        cut: Cut::End,
    }
}

/// Claude Code, for a remote terminal whose host's session has sent its hook events (#543): the
/// terminal's own foreground is ssh.
fn remote_claude(view: &Entity<TerminalView>, cx: &App) -> Option<AgentKind> {
    let remote = crate::remote::is_remote(view.read(cx).terminal().read(cx));
    let seat = agent_events::seat_of(view.entity_id(), AgentKind::Claude, cx);
    (remote && seat.is_some()).then_some(AgentKind::Claude)
}

/// The full access or bypass mark on an agent's row (#532): for Codex with a thread that reports
/// its sandbox, the thread's (#650); otherwise Claude Code's reported mode, or the arguments.
fn agent_mark(
    kind: AgentKind,
    terminal: &Terminal,
    seat: Option<&marley_fleet::Session>,
) -> Option<marley_agent::PermissionMark> {
    let label = |key: &str| {
        seat.and_then(|seat| seat.labels.get(key))
            .map(String::as_str)
    };
    if kind == AgentKind::Codex
        && let Some(sandbox) = label(codex_events::SANDBOX_LABEL)
    {
        return marley_agent::thread_mark(
            sandbox,
            label(codex_events::APPROVAL_LABEL).unwrap_or_default(),
        );
    }
    marley_agent::permission_mark(
        kind,
        &terminal.marley_foreground_argv().unwrap_or_default(),
        label(claude_events::PERMISSION_MODE_LABEL),
    )
}

/// A Codex row's line from its App Server's thread (#650): the state's words, and the thread's
/// token use once Codex reported it, such as `working · 41k tokens`.
fn codex_line(seat: &marley_fleet::Session, cx: &App) -> String {
    let words = codex_events::seat_words(seat, agent_events::now_ms(), no_update_after_ms(cx));
    match codex_events::tokens(seat) {
        Some(tokens) => format!("{words} · {} tokens", crate::fleet::compact(tokens)),
        None => words,
    }
}

/// Where a terminal agent's status comes from, for its class in the rail's order (#542).
fn reporting(seat: Option<&marley_fleet::Session>, cx: &App) -> Reporting {
    let Some(seat) = seat else {
        return Reporting::Timer;
    };
    let after = no_update_after_ms(cx);
    let stale = seat.state == marley_fleet::State::Working
        && after > 0
        && marley_fleet::is_stale(seat, agent_events::now_ms(), after);
    if stale {
        Reporting::Stale
    } else {
        Reporting::Events
    }
}

/// How long a working Claude Code may go without an event before its row says so, from
/// `marley.no_update_after_minutes`; 0 is never (#547).
fn no_update_after_ms(cx: &App) -> u64 {
    MarleySettings::get_global(cx)
        .no_update_after_minutes
        .saturating_mul(60_000)
}

/// An agent row's title: the title the CLI set over OSC, else the agent's name.
fn agent_title(breadcrumb: &str, kind: AgentKind) -> String {
    if breadcrumb.trim().is_empty() {
        kind.display_name().to_string()
    } else {
        breadcrumb.to_string()
    }
}

/// The workspace a group's row stands for: its last active one, or for a projectless group
/// (#600), whose key every group shares, its own.
fn listed_workspace(
    multi_workspace: &MultiWorkspace,
    group: &ProjectGroup,
    projectless: bool,
    cx: &App,
) -> Option<Entity<Workspace>> {
    if projectless {
        return group.workspaces.first().cloned();
    }
    multi_workspace
        .last_active_workspace_for_group(&group.key, cx)
        .or_else(|| group.workspaces.first().cloned())
}

/// A projectless group as the rail lists it: its id and its workspace's id.
type Projectless = Option<(Uuid, Option<WorkspaceId>)>;

/// The groups the rail lists: the window's projects with an open workspace, then its projectless
/// groups (#600), each on its own workspace under the empty key, in the order the window saved
/// them (#601) and then as made; their names, and each projectless group's ids. The headers the
/// user placed by dragging go first, in that order (#602).
fn rail_groups(
    multi_workspace: &MultiWorkspace,
    saved_order: &SavedOrder,
    cx: &App,
) -> (Vec<ProjectGroup>, Vec<String>, Vec<Projectless>) {
    // A group with no open workspace, as a restart leaves every one but the shown, is listed as
    // closed (#606).
    let mut groups: Vec<ProjectGroup> = multi_workspace.project_groups(cx);
    let mut names = crate::group_names(&groups);
    let mut projectless = vec![None; groups.len()];
    let mut marley_groups = groups::groups_of(multi_workspace, cx);
    marley_groups.sort_by_key(|(group, _)| {
        saved_order
            .groups
            .iter()
            .position(|saved| Some(*saved) == group.database_id)
            .unwrap_or(usize::MAX)
    });
    for (group, workspace) in marley_groups {
        groups.push(ProjectGroup {
            key: ProjectGroupKey::default(),
            workspaces: vec![workspace],
            expanded: group.expanded,
        });
        names.push(group.name);
        projectless.push(Some((group.id, group.database_id)));
    }
    let mut listed: Vec<(ProjectGroup, String, Projectless)> = groups
        .into_iter()
        .zip(names)
        .zip(projectless)
        .map(|((group, name), id)| (group, name, id))
        .collect();
    marley_rail::place(&mut listed, &saved_order.headers, |(group, _, id)| {
        Some(order::header_place(&group.key, id.map(|(group, _)| group)))
    });
    let mut groups = Vec::with_capacity(listed.len());
    let mut names = Vec::with_capacity(listed.len());
    let mut projectless = Vec::with_capacity(listed.len());
    for (group, name, id) in listed {
        groups.push(group);
        names.push(name);
        projectless.push(id);
    }
    (groups, names, projectless)
}

/// The window, read once. A group the window holds no workspace of, as a restart leaves every
/// one but the shown, is listed closed, its header alone (#606).
fn build_snapshot(
    multi_workspace: &Entity<MultiWorkspace>,
    foreground_command: ForegroundCommand,
    terminal_output: &HashMap<EntityId, Instant>,
    filter: &str,
    saved_order: &SavedOrder,
    window: &Window,
    cx: &App,
) -> Snapshot {
    let multi_workspace = multi_workspace.read(cx);
    let (groups, names, projectless) = rail_groups(multi_workspace, saved_order, cx);
    let displayed = multi_workspace.workspace();
    let home = util::paths::home_dir().as_path();
    let now = cx.background_executor().now();
    let mut snapshot = Snapshot::default();
    let mut displayed_worktree = None;
    for ((group, name), id) in groups.iter().zip(names).zip(projectless) {
        let Some(workspace) = listed_workspace(multi_workspace, group, id.is_some(), cx) else {
            push_closed(&mut snapshot, group, name, displayed, filter, cx);
            continue;
        };
        let (worktrees, tags) = group_worktrees(group, &workspace, filter, &mut snapshot, cx);
        if group.workspaces.contains(displayed) {
            displayed_worktree = tags.get(&displayed.entity_id()).cloned();
        }
        let mut terminals = Vec::new();
        let mut browsers = Vec::new();
        for member in &group.workspaces {
            let worktree = tags.get(&member.entity_id()).cloned();
            // Subtitles read against the member's own first root: a linked worktree's, not the
            // main repository's the group is keyed by.
            let root = member
                .read(cx)
                .project()
                .read(cx)
                .visible_worktrees(cx)
                .find_map(|worktree| worktree.read(cx).root_dir());
            for view in member.read(cx).items_of_type::<TerminalView>(cx) {
                let id = view.entity_id().as_u64();
                let mut terminal = terminal_snapshot(
                    &view,
                    root.as_deref(),
                    home,
                    foreground_command,
                    terminal_output.get(&view.entity_id()).copied(),
                    now,
                    cx,
                );
                terminal.matched = terminal_match(filter, &terminal);
                terminal.worktree.clone_from(&worktree);
                if terminal.agent.is_some() {
                    snapshot.agent_terminals.insert(view.entity_id());
                }
                terminals.push(terminal);
                snapshot
                    .places
                    .insert(Selection::Terminal(id), order::terminal_place(&view, cx));
                snapshot.terminals.insert(
                    id,
                    TerminalEntry {
                        workspace: member.downgrade(),
                        view: view.downgrade(),
                    },
                );
            }
            browsers.extend(member_browsers(member, filter, &mut snapshot, cx));
        }
        let threads = listed_threads(
            &mut snapshot,
            group,
            Some(&workspace),
            displayed,
            filter,
            cx,
        );
        inbox_entries(&name, &group.workspaces, &mut snapshot, cx);
        let matched = filter_match(filter, &name);
        snapshot.rail.projects.push(ProjectSnapshot {
            name,
            expanded: group.expanded,
            terminals,
            browsers,
            threads,
            ports: port_snapshots(&group.key, filter, cx),
            worktrees,
            matched,
            closed: false,
        });
        snapshot.groups.push(GroupEntry {
            key: group.key.clone(),
            workspace: workspace.downgrade(),
            source: git_source(&workspace, cx),
            git: None,
            group: id.map(|(group, _)| group),
            database_id: id.and_then(|(_, database_id)| database_id),
            closed: false,
        });
    }
    harness_entries(&mut snapshot, cx);
    snapshot.rail.filtering = !filter.is_empty();
    note_focus(
        &mut snapshot,
        &groups,
        displayed,
        displayed_worktree,
        window,
        cx,
    );
    snapshot
}

/// A group's threads as its rows list them, their entities noted in `snapshot`.
fn listed_threads(
    snapshot: &mut Snapshot,
    group: &ProjectGroup,
    listed: Option<&Entity<Workspace>>,
    shown: &Entity<Workspace>,
    filter: &str,
    cx: &App,
) -> Vec<ThreadSnapshot> {
    let mut threads = Vec::new();
    for (mut thread, entry) in group_threads(group, listed, shown, cx) {
        thread.matched = filter_match(filter, &thread.title);
        snapshot.threads.insert(thread.key.clone(), entry);
        threads.push(thread);
    }
    threads
}

/// A group the window holds no workspace of (#606), closed: its header, and under it its threads
/// and the ports listening in its folders (#617), which need no workspace to be listed.
fn push_closed(
    snapshot: &mut Snapshot,
    group: &ProjectGroup,
    name: String,
    shown: &Entity<Workspace>,
    filter: &str,
    cx: &App,
) {
    let matched = filter_match(filter, &name);
    let threads = listed_threads(snapshot, group, None, shown, filter, cx);
    snapshot.rail.projects.push(ProjectSnapshot {
        name,
        expanded: group.expanded,
        terminals: Vec::new(),
        browsers: Vec::new(),
        threads,
        ports: port_snapshots(&group.key, filter, cx),
        worktrees: Vec::new(),
        matched,
        closed: true,
    });
    snapshot.groups.push(GroupEntry {
        key: group.key.clone(),
        workspace: WeakEntity::new_invalid(),
        source: None,
        git: None,
        group: None,
        database_id: None,
        closed: true,
    });
}

/// What the window shows, for the one selected row: the displayed workspace's project, its active
/// terminal or Browser tab, its worktree (#510), and the Agent Panel's thread.
fn note_focus(
    snapshot: &mut Snapshot,
    groups: &[ProjectGroup],
    displayed: &Entity<Workspace>,
    worktree: Option<String>,
    window: &Window,
    cx: &App,
) {
    let panel = displayed.read(cx).panel::<AgentPanel>(cx);
    let panel_thread = panel
        .as_ref()
        .and_then(|panel| panel.read(cx).active_thread_id(cx))
        .map(|thread_id| thread_id.to_key_string());
    snapshot.shown_thread = panel_thread
        .clone()
        .filter(|_| AgentPanel::is_visible(displayed, cx));
    let (terminal, terminal_focused, browser) = active_rows(displayed, window, cx);
    snapshot.rail.focus = Focus {
        cursor: None,
        project: groups
            .iter()
            .position(|group| group.workspaces.contains(displayed)),
        terminal,
        browser,
        worktree,
        terminal_focused,
        thread: panel_thread.filter(|_| {
            panel
                .as_ref()
                .is_some_and(|panel| panel.focus_handle(cx).contains_focused(window, cx))
        }),
    };
}

/// A group's worktree rows (#510), their entities put in `snapshot`, and the folder of each
/// member that is a listed linked worktree, which its terminals list under.
fn group_worktrees(
    group: &ProjectGroup,
    workspace: &Entity<Workspace>,
    filter: &str,
    snapshot: &mut Snapshot,
    cx: &App,
) -> (Vec<WorktreeSnapshot>, HashMap<EntityId, String>) {
    let gits: Vec<(Entity<Workspace>, MemberGit)> = group
        .workspaces
        .iter()
        .filter_map(|member| Some((member.clone(), member_git(member, cx)?)))
        .collect();
    let rows = worktree_rows(&gits, filter);
    let tags = gits
        .iter()
        .filter(|(_, git)| git.linked)
        .map(|(member, git)| (member.entity_id(), git.root.display().to_string()))
        .filter(|(_, path)| rows.iter().any(|(row, _)| row.path == *path))
        .collect();
    // The checkouts of one repository share its branches and its config; the main checkout's
    // repository, when it is open, is the one whose folder the drift's git runs in (#560).
    let main = gits.iter().find_map(|(_, git)| git.main.clone());
    let repository = gits
        .iter()
        .find(|(_, git)| main.as_ref() == Some(&git.root))
        .or_else(|| gits.first())
        .map(|(_, git)| git.repository.clone());
    let rows = rows
        .into_iter()
        .map(|(row, facts)| {
            snapshot.worktrees.insert(
                row.path.clone(),
                WorktreeEntry {
                    member: facts.member,
                    project: workspace.downgrade(),
                    path: PathBuf::from(&row.path),
                    name: row.name.clone(),
                    branch: facts.branch,
                    commit: facts.commit,
                    main: main.clone(),
                    repository: repository.clone(),
                },
            );
            row
        })
        .collect();
    (rows, tags)
}

/// A member workspace's place in its repository (#510): its first folder, whether that is a
/// linked worktree, the repository's main checkout, and the linked worktrees the repository
/// lists, which leave out the member itself.
struct MemberGit {
    root: PathBuf,
    linked: bool,
    main: Option<PathBuf>,
    /// The member's branch as git names it, none when it is detached, and its `HEAD`.
    branch: Option<String>,
    head: Option<String>,
    repository: WeakEntity<Repository>,
    others: Vec<git::repository::Worktree>,
}

/// What a worktree's row holds beyond what it shows: its open workspace, and its branch and
/// `HEAD` as git names them, which its drift is read for (#560).
struct WorktreeFacts {
    member: Option<WeakEntity<Workspace>>,
    branch: Option<String>,
    commit: Option<String>,
}

/// `member`'s place in the repository whose folder is its first, when it is one.
fn member_git(member: &Entity<Workspace>, cx: &App) -> Option<MemberGit> {
    let project = member.read(cx).project().read(cx);
    let root = project.visible_worktrees(cx).next()?.read(cx).abs_path();
    let repository = project
        .git_store()
        .read(cx)
        .repositories()
        .values()
        .find(|repository| *repository.read(cx).work_directory_abs_path == *root)?;
    let snapshot = repository.read(cx);
    Some(MemberGit {
        root: root.to_path_buf(),
        linked: snapshot.is_linked_worktree(),
        main: snapshot.main_worktree_abs_path().map(Path::to_path_buf),
        branch: snapshot
            .branch
            .as_ref()
            .map(|branch| branch.name().to_string()),
        head: snapshot
            .head_commit
            .as_ref()
            .map(|commit| commit.sha.to_string()),
        repository: repository.downgrade(),
        others: snapshot.linked_worktrees().to_vec(),
    })
}

/// A project's pull request as its row shows it: GitHub's pull request icon and its number, in
/// its state's color, with the state, title and link in the tooltip (#531).
fn pull_request_chip(id: EntityId, found: &PullRequest) -> impl IntoElement {
    let color = match found.state {
        PullRequestState::Open => Color::Success,
        PullRequestState::Draft => Color::Muted,
        PullRequestState::Merged => Color::Accent,
        PullRequestState::Closed => Color::Error,
    };
    let tooltip = format!(
        "{} pull request #{}: {}\n{}",
        found.state.words(),
        found.number,
        found.title,
        found.url
    );
    h_flex()
        .id(("marley-rail-pull-request", id))
        .gap_0p5()
        .child(
            Icon::new(IconName::PullRequest)
                .size(IconSize::XSmall)
                .color(color),
        )
        .child(
            Label::new(format!("#{}", found.number))
                .size(LabelSize::XSmall)
                .color(color),
        )
        .tooltip(Tooltip::text(tooltip))
}

/// A project's read off the main thread (#531): its branch's pull request from `gh` when `github`
/// names its repository, then its lines changed against that pull request's base, else `kept`'s,
/// else `default`.
async fn read_project_git(
    root: &Path,
    branch: Option<String>,
    github: Option<String>,
    kept: Option<PullRequest>,
    default: Option<String>,
) -> (
    Option<anyhow::Result<Option<PullRequest>>>,
    anyhow::Result<Option<(String, u32, u32)>>,
) {
    let asked = match (github, branch) {
        (Some(repository), Some(branch)) => {
            Some(github::pull_request(root, &repository, &branch).await)
        }
        _ => None,
    };
    let pull_request = match &asked {
        Some(Ok(found)) => found.clone(),
        _ => kept,
    };
    let base = pull_request.map(|found| found.base).or(default);
    let counts = match base {
        Some(base) => worktree_git::changed_lines(root, &base).await,
        None => Ok(None),
    };
    (asked, counts)
}

/// The checkout of a project whose folder is a repository's main checkout, which its row's
/// counts and pull request are read for (#531).
fn git_source(workspace: &Entity<Workspace>, cx: &App) -> Option<GitSource> {
    member_git(workspace, cx)
        .filter(|git| !git.linked)
        .map(|git| GitSource {
            root: git.root,
            branch: git.branch,
            head: git.head,
            repository: git.repository,
        })
}

/// The `owner/repo` of `repository`'s `origin` on GitHub, none for another host.
fn github_repository(repository: &Entity<Repository>, cx: &App) -> Option<String> {
    let url = repository.read(cx).remote_origin_url.clone()?;
    let registry = git::GitHostingProviderRegistry::try_global(cx)?;
    let (provider, parsed) = git::parse_git_remote_url(registry, &url)?;
    (provider.name() == "GitHub").then(|| format!("{}/{}", parsed.owner, parsed.repo))
}

/// A commit's first seven characters, as git abbreviates one.
fn short_commit(sha: &str) -> String {
    sha.chars().take(7).collect()
}

/// `1 commit`, or `N commits`.
fn commits(count: u32) -> String {
    if count == 1 {
        "1 commit".to_string()
    } else {
        format!("{count} commits")
    }
}

/// A group's worktree rows (#510), each with its open workspace: the linked worktrees the open
/// members' repositories list, and each open member that is a linked worktree, since a linked
/// repository lists the main checkout but not itself; none for the main checkout, and none under
/// its `.claude/worktrees/`, which are Claude Code's own.
fn worktree_rows(
    gits: &[(Entity<Workspace>, MemberGit)],
    filter: &str,
) -> Vec<(WorktreeSnapshot, WorktreeFacts)> {
    let main = gits.iter().find_map(|(_, git)| git.main.clone());
    let scratch = main
        .as_ref()
        .map(|main| main.join(".claude").join("worktrees"));
    // Each worktree's folder, branch and `HEAD`.
    let mut found: Vec<(PathBuf, Option<String>, Option<String>)> = Vec::new();
    let mut add = |path: &Path, branch: Option<String>, commit: Option<String>| {
        let listed = main.as_deref() != Some(path)
            && !scratch
                .as_ref()
                .is_some_and(|scratch| path.starts_with(scratch))
            && !found.iter().any(|(known, ..)| known == path);
        if listed {
            found.push((path.to_path_buf(), branch, commit));
        }
    };
    for (_, git) in gits {
        for other in git.others.iter().filter(|other| !other.is_main) {
            add(
                &other.path,
                other.branch_name().map(str::to_string),
                Some(other.sha.to_string()),
            );
        }
        if git.linked {
            add(&git.root, git.branch.clone(), git.head.clone());
        }
    }
    found
        .into_iter()
        .map(|(path, branch_name, commit)| {
            // The row shows the branch, or the short commit of a detached worktree.
            let branch = branch_name
                .clone()
                .or_else(|| commit.as_deref().map(short_commit));
            let member = gits
                .iter()
                .find(|(_, git)| git.linked && git.root == path)
                .map(|(member, _)| member.downgrade());
            let name = main
                .as_deref()
                .and_then(|main| project::linked_worktree_short_name(main, &path))
                .map(|name| name.to_string())
                .or_else(|| {
                    path.file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                })
                .unwrap_or_else(|| path.display().to_string());
            let matched = filter_match(filter, &name).or_else(|| {
                branch
                    .as_deref()
                    .and_then(|branch| filter_match(filter, branch))
                    .map(|_| Vec::new())
            });
            let row = WorktreeSnapshot {
                path: path.display().to_string(),
                name,
                branch,
                open: member.is_some(),
                matched,
                drift: None,
            };
            let facts = WorktreeFacts {
                member,
                branch: branch_name,
                commit,
            };
            (row, facts)
        })
        .collect()
}

/// One worktree's part of a drift run (#560): what its row knows of it, and what the rail kept
/// from the last run.
struct DriftRead {
    path: String,
    branch: Option<String>,
    commit: Option<String>,
    kept: Option<DriftState>,
}

/// Reads who merges the repository's branches (#511) and each worktree's base in `main`, #510's
/// record else `default`, and runs its summary unless the base and the tips are the ones its kept
/// drift was read for (#560).
async fn read_drifts(
    main: &Path,
    reads: Vec<DriftRead>,
    branches: &[git::repository::Branch],
    default: Option<String>,
    fs: &dyn Fs,
) -> Vec<(String, DriftState)> {
    // Merge reads it again when it is chosen, so an unreadable owner offers it and refuses there.
    let owner = worktree_git::merge_owner(main, fs)
        .await
        .log_err()
        .unwrap_or_default();
    let mut states = Vec::with_capacity(reads.len());
    for read in reads {
        let recorded_base = match &read.branch {
            Some(branch) => worktree_git::recorded_base(main, branch)
                .await
                .log_err()
                .flatten(),
            None => None,
        };
        let recorded = recorded_base.is_some();
        let base = recorded_base.or_else(|| default.clone());
        let tips = drift_tips(read.commit.as_deref(), base.as_deref(), branches);
        let state = match read.kept {
            Some(kept) if kept.base == base && same_tips(kept.tips.as_ref(), tips) => DriftState {
                recorded,
                owner,
                ..kept
            },
            _ => {
                let drift = match (base.as_deref(), tips) {
                    (Some(base), Some((tip, base_tip))) => {
                        worktree_git::summary(main, tip, base_tip, base)
                            .await
                            .log_err()
                            .map(drift_snapshot)
                    }
                    _ => None,
                };
                let tips = tips.map(|(tip, base_tip)| (tip.to_string(), base_tip.to_string()));
                DriftState {
                    base,
                    recorded,
                    owner,
                    tips,
                    drift,
                }
            }
        };
        states.push((read.path, state));
    }
    states
}

/// The tips a worktree's drift is read for (#560): its `HEAD`, and its base's tip from the local
/// branches, or the base itself when #510 recorded a detached main checkout's commit.
fn drift_tips<'a>(
    commit: Option<&'a str>,
    base: Option<&'a str>,
    branches: &'a [git::repository::Branch],
) -> Option<(&'a str, &'a str)> {
    let (commit, base) = (commit?, base?);
    let base_tip = match branches
        .iter()
        .find(|branch| !branch.is_remote() && branch.name() == base)
    {
        Some(branch) => branch.most_recent_commit.as_ref()?.sha.as_ref(),
        None => worktree_git::is_commit(base).then_some(base)?,
    };
    Some((commit, base_tip))
}

/// Whether the tips a drift was read for are `tips`.
fn same_tips(kept: Option<&(String, String)>, tips: Option<(&str, &str)>) -> bool {
    kept.map(|(tip, base_tip)| (tip.as_str(), base_tip.as_str())) == tips
}

/// A drift as the rail's model holds it.
fn drift_snapshot(drift: Drift) -> DriftSnapshot {
    DriftSnapshot {
        ahead: drift.ahead,
        behind: drift.behind,
        conflicts: drift.conflicts,
        base: drift.base,
        base_commit: drift.base_commit,
    }
}

/// The rows the displayed workspace's active center item makes current: a terminal's, with
/// whether it holds the focus, or a Browser tab's (#504).
fn active_rows(
    displayed: &Entity<Workspace>,
    window: &Window,
    cx: &App,
) -> (Option<u64>, bool, Option<u64>) {
    let active_item = displayed.read(cx).active_item(cx);
    let terminal = active_item
        .as_ref()
        .and_then(|item| item.downcast::<TerminalView>());
    let browser = active_item.and_then(|item| item.downcast::<BrowserView>());
    let focused = terminal
        .as_ref()
        .is_some_and(|view| crate::rich_input::holds_focus(view.read(cx), window, cx));
    (
        terminal.map(|view| view.entity_id().as_u64()),
        focused,
        browser.map(|view| view.entity_id().as_u64()),
    )
}

/// The Browser tabs of `member` as rows show them (#504), each tab's entities and page icon kept
/// in `snapshot`. The hub is read without being made: with none, no browser has started.
fn member_browsers(
    member: &Entity<Workspace>,
    filter: &str,
    snapshot: &mut Snapshot,
    cx: &App,
) -> Vec<BrowserSnapshot> {
    let hub = BrowserHub::try_global(cx);
    let hub = hub.as_ref();
    let mut browsers = Vec::new();
    for view in member.read(cx).items_of_type::<BrowserView>(cx) {
        let id = view.entity_id().as_u64();
        let (mut browser, favicon) = browser_snapshot(&view, hub, cx);
        browser.matched = filter_match(filter, &browser.title);
        browsers.push(browser);
        if let Some(favicon) = favicon {
            snapshot.favicons.insert(id, favicon);
        }
        snapshot
            .places
            .insert(Selection::Browser(id), order::browser_place(&view, cx));
        snapshot.browsers.insert(
            id,
            BrowserEntry {
                workspace: member.downgrade(),
                view: view.downgrade(),
            },
        );
    }
    browsers
}

/// A Browser tab as its row shows it (#504), and its page's icon once one was read. Without a hub
/// nothing has started the browser, and the tab has no page yet.
fn browser_snapshot(
    view: &Entity<BrowserView>,
    hub: Option<&Entity<BrowserHub>>,
    cx: &App,
) -> (BrowserSnapshot, Option<Arc<Image>>) {
    let tab = view.read(cx);
    let page = tab.target().zip(hub.map(|hub| hub.read(cx)));
    let host = page
        .and_then(|(target, hub)| hub.url(target))
        .and_then(|url| host_and_port(&url));
    let favicon = page.and_then(|(target, hub)| hub.favicon(target));
    let snapshot = BrowserSnapshot {
        id: view.entity_id().as_u64(),
        title: tab.tab_content_text(0, cx).to_string(),
        host,
        loading: page.is_some_and(|(target, hub)| hub.is_loading(target)),
        picks: page.map_or(0, |(target, hub)| hub.pick_count(target)),
        annotations: page.map_or(0, |(target, hub)| hub.annotations(target).len()),
        agent_unseen: page.is_some_and(|(target, hub)| hub.agent_unseen(target)),
        icon: favicon.as_ref().map(|favicon| favicon.id()),
        matched: None,
    };
    (snapshot, favicon)
}

/// An http or https URL's host, with its port when the URL names one: a Browser row's second
/// line.
fn host_and_port(url: &str) -> Option<String> {
    let url = url::Url::parse(url).ok()?;
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }
    let host = url.host_str()?;
    Some(
        url.port()
            .map_or_else(|| host.to_string(), |port| format!("{host}:{port}")),
    )
}

/// Where the rail's filter matches `text`, by the matcher Zed's Threads Sidebar uses; `None`
/// without a filter.
fn filter_match(filter: &str, text: &str) -> Option<Vec<usize>> {
    (!filter.is_empty())
        .then(|| fuzzy_match_positions(filter, text))
        .flatten()
}

/// Where the rail's filter matches a terminal: its title, or else its running or last command
/// (#551) or the failure that command printed (#572), which highlight nothing in the title.
fn terminal_match(filter: &str, terminal: &TerminalSnapshot) -> Option<Vec<usize>> {
    filter_match(filter, &terminal.title).or_else(|| {
        terminal
            .command
            .as_ref()
            .and_then(|command| filter_match(filter, &command.text))
            .or_else(|| {
                terminal
                    .running_error
                    .as_ref()
                    .and_then(|error| filter_match(filter, &error.line))
            })
            .map(|_| Vec::new())
    })
}

/// A project header's name, and after it a collapsed project's agents by state (#542).
fn project_name(
    name: String,
    highlight: Vec<usize>,
    color: Color,
    summary: Option<String>,
) -> impl IntoElement {
    h_flex()
        .min_w_0()
        .flex_1()
        .gap_1p5()
        .child(div().min_w_0().child(row_label(name, highlight, color)))
        .children(summary.map(|summary| {
            div().flex_none().child(
                Label::new(summary)
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
        }))
}

/// A row's name or title, with the characters the filter matched highlighted.
fn row_label(text: String, highlight: Vec<usize>, color: Color) -> AnyElement {
    if highlight.is_empty() {
        Label::new(text)
            .size(LabelSize::Small)
            .color(color)
            .truncate()
            .into_any_element()
    } else {
        HighlightedLabel::new(text, highlight)
            .size(LabelSize::Small)
            .color(color)
            .truncate()
            .into_any_element()
    }
}

/// The hover group of a rail row, for what shows only while the pointer is over the row.
const ROW_GROUP: &str = "marley-rail-row";

/// A shade a step lighter than what it is drawn over, in a dark theme, and darker in a light
/// one: the rows' icon circles and the selected row's border, as Warp's tab list draws them.
fn raised(cx: &App) -> Hsla {
    cx.theme().colors().text.opacity(0.1)
}

/// What every rail row is drawn in. The selected row is a card, a raised fill inside a border;
/// the others keep the border, clear, so moving the selection moves nothing.
fn row_frame(id: impl Into<ElementId>, selected: bool, cx: &App) -> Stateful<Div> {
    let colors = cx.theme().colors();
    let (border, fill, hover) = (
        raised(cx),
        colors.ghost_element_selected,
        colors.ghost_element_hover,
    );
    h_flex()
        .id(id)
        .group(ROW_GROUP)
        .w_full()
        .flex_none()
        .rounded_md()
        .border_1()
        .cursor_pointer()
        .map(|row| {
            if selected {
                row.border_color(border).bg(fill)
            } else {
                row.border_color(gpui::transparent_black())
                    .hover(|style| style.bg(hover))
            }
        })
}

/// An agent row's permission mark (#532): a pill in the warning color, whose tooltip says what
/// the agent runs without and where Marley read it. It stays in sight, however the agent was
/// started.
fn permission_chip(id: u64, mark: marley_agent::PermissionMark, cx: &App) -> Stateful<Div> {
    div()
        .id(("marley-rail-mark", id))
        .debug_selector(move || format!("marley-rail-mark-{id}"))
        .flex_none()
        .px_1()
        .rounded_sm()
        .border_1()
        .border_color(cx.theme().colors().border)
        .child(
            Label::new(mark.words())
                .size(LabelSize::XSmall)
                .color(Color::Warning),
        )
        .tooltip(Tooltip::text(mark.tooltip()))
}

/// A worktree row's drift (#560): a pill with how far its branch is behind its base, muted, or
/// how many files a merge would stop on, in the warning color with Zed's conflict icon; its
/// tooltip says the rest. None while the branch is up to date and merges cleanly.
fn drift_chip(path: &str, name: &str, drift: &DriftSnapshot, cx: &App) -> Option<Stateful<Div>> {
    let words = drift.words()?;
    let conflicted = drift.conflicted();
    let color = if conflicted {
        Color::Warning
    } else {
        Color::Muted
    };
    let selector = format!("marley-rail-drift-{name}");
    let chip = h_flex()
        .id(SharedString::from(format!("marley-rail-drift-{path}")))
        .debug_selector(move || selector)
        .flex_none()
        .gap_0p5()
        .px_1()
        .rounded_sm()
        .border_1()
        .border_color(cx.theme().colors().border)
        .when(conflicted, |chip| {
            chip.child(
                Icon::new(IconName::GitMergeConflict)
                    .size(IconSize::XSmall)
                    .color(color),
            )
        })
        .child(Label::new(words).size(LabelSize::XSmall).color(color))
        .tooltip(Tooltip::text(drift.tooltip()));
    Some(chip)
}

/// A stall or loop flag's mark (#569), which stays while the pointer is over the row, for its
/// tooltip.
fn stall_flag(id: u64, flag: String) -> Stateful<Div> {
    div()
        .id(("marley-rail-flag", id))
        .debug_selector(move || format!("marley-rail-flag-{id}"))
        .flex_none()
        .child(
            Icon::new(IconName::Warning)
                .size(IconSize::XSmall)
                .color(Color::Warning),
        )
        .tooltip(Tooltip::text(flag))
}

/// A terminal row's marks before its end: an agent's permission chip, its stall flag (#569) and
/// a running command's failure (#572).
fn terminal_marks(row: &TerminalRow, cx: &App) -> Vec<Stateful<Div>> {
    let id = row.id;
    row.agent
        .and_then(|agent| agent.mark)
        .map(|mark| permission_chip(id, mark, cx))
        .into_iter()
        .chain(row.flag.clone().map(|flag| stall_flag(id, flag)))
        .chain(
            row.running_error
                .as_ref()
                .map(|error| running_error_mark(id, error)),
        )
        .collect()
}

/// The mark of a failure a running command printed and kept running after (#572), as a failed
/// thread's; a `?` beside it when a reading in `suggest` found it.
fn running_error_mark(id: u64, error: &RunningError) -> Stateful<Div> {
    h_flex()
        .id(("marley-rail-running-error", id))
        .debug_selector(move || format!("marley-rail-running-error-{id}"))
        .flex_none()
        .child(
            Icon::new(IconName::Close)
                .size(IconSize::Small)
                .color(Color::Error),
        )
        .when(error.questioned, |mark| {
            mark.child(Label::new("?").size(LabelSize::XSmall).color(Color::Error))
        })
        .tooltip(Tooltip::text(error.line.clone()))
}

/// A line under a row's title, and its color: muted unless it says a command failed (#551). A
/// command's `state` follows it and stays whole while a long command is cut.
struct RowLine {
    text: String,
    state: Option<String>,
    color: Color,
    cut: Cut,
}

impl RowLine {
    const fn muted(text: String) -> Self {
        Self {
            text,
            state: None,
            color: Color::Muted,
            cut: Cut::End,
        }
    }
}

/// Where a row's line loses its text when it does not fit (#618).
#[derive(Clone, Copy)]
enum Cut {
    End,
    /// For a name whose end tells it apart, as a unit's.
    Start,
    /// For a URL, whose host and port are at its two ends.
    Middle,
}

/// A port row's URL as its line shows it (#618): a local server's without `http://`, since its
/// host and port are what tell it apart. Open and Copy keep the whole URL.
fn url_label(url: &str) -> String {
    match url.strip_prefix("http://") {
        Some(rest) if rest.starts_with("127.0.0.1:") || rest.starts_with("localhost:") => {
            rest.to_string()
        }
        _ => url.to_string(),
    }
}

impl From<String> for RowLine {
    fn from(text: String) -> Self {
        Self::muted(text)
    }
}

/// A terminal's or a thread's row, laid out as Warp's tab list lays out a tab: a round icon, then
/// the title over the `lines` under it, at one height with a second line or without. An agent row
/// whose events give it a third line (#519) is taller, and a harness session's row with its
/// progress and quota lines (#640) the tallest. `icon_selector` names the icon's container for
/// the driven tests. The caller adds the row's end and its clicks.
fn row_card(
    id: impl Into<ElementId>,
    icon_selector: String,
    selected: bool,
    icon: AnyElement,
    title: AnyElement,
    lines: Vec<impl Into<RowLine>>,
    cx: &App,
) -> Stateful<Div> {
    row_frame(id, selected, cx)
        .map(|row| match lines.len() {
            0 | 1 => row.h_11(),
            2 => row.h(rems(3.5)),
            _ => row.h(rems(4.5)),
        })
        .gap_2p5()
        .pl_2()
        .pr_1p5()
        .child(
            h_flex()
                .debug_selector(move || icon_selector)
                .flex_none()
                .size_7()
                .justify_center()
                .rounded_full()
                .bg(raised(cx))
                .child(icon),
        )
        .child(
            v_flex()
                .min_w_0()
                .flex_1()
                .child(title)
                .children(lines.into_iter().map(|line| {
                    let line: RowLine = line.into();
                    h_flex()
                        .min_w_0()
                        .child(div().min_w_0().child({
                            let label = Label::new(line.text)
                                .size(LabelSize::XSmall)
                                .color(line.color);
                            match line.cut {
                                Cut::End => label.truncate(),
                                Cut::Start => label.truncate_start(),
                                Cut::Middle => label.truncate_middle(),
                            }
                        }))
                        .when_some(line.state, |row, state| {
                            row.child(
                                div().flex_none().child(
                                    Label::new(state).size(LabelSize::XSmall).color(line.color),
                                ),
                            )
                        })
                })),
        )
}

/// A thread row's second line: the agent, then what it is doing, as an agent CLI's terminal row
/// reads (`marley_agent::status_line`).
fn thread_subtitle(agent_name: &str, status: ThreadStatus) -> String {
    format!("{agent_name} · {}", status.label())
}

/// What a thread row shows at its end, as Zed's thread row shows it: a spinner while the thread
/// runs, a warning while it waits for a confirmation, an error mark after a failed run, and
/// otherwise the attention dot, or nothing.
fn thread_status_mark(status: ThreadStatus, attention: bool) -> Option<AnyElement> {
    match status {
        ThreadStatus::Running => Some(
            Icon::new(IconName::LoadCircle)
                .size(IconSize::Small)
                .color(Color::Muted)
                .with_rotate_animation(2)
                .into_any_element(),
        ),
        ThreadStatus::Waiting => Some(
            Icon::new(IconName::Warning)
                .size(IconSize::XSmall)
                .color(Color::Warning)
                .into_any_element(),
        ),
        ThreadStatus::Error => Some(
            Icon::new(IconName::Close)
                .size(IconSize::Small)
                .color(Color::Error)
                .into_any_element(),
        ),
        ThreadStatus::Done => {
            attention.then(|| Indicator::dot().color(Color::Accent).into_any_element())
        }
    }
}

impl EventEmitter<SidebarEvent> for Rail {}

impl Focusable for Rail {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// The `MultiWorkspace` calls these while it is being updated, so none of them reads it.
impl Sidebar for Rail {
    fn width(&self, _cx: &App) -> Pixels {
        self.width
    }

    fn set_width(&mut self, width: Option<Pixels>, cx: &mut Context<Self>) {
        // The resize handle passes the raw pointer position, and `None` to reset.
        self.width_set_by_user = width.is_some();
        self.width = width.unwrap_or(DEFAULT_WIDTH).clamp(MIN_WIDTH, MAX_WIDTH);
        // One width for both layouts: the Zed sidebar the rail keeps takes the rail's clamped
        // width too, or the reset.
        if let Some((sidebar, _)) = &self.zed_sidebar {
            let width = width.map(|_| self.width);
            sidebar.update(cx, |sidebar, cx| sidebar.set_width(width, cx));
        }
        cx.notify();
    }

    fn has_notifications(&self, _cx: &App) -> bool {
        marley_rail::has_attention(&self.snapshot.rail)
    }

    // `SidebarSide`'s default is the left, the rail's side.
    fn side(&self, _cx: &App) -> SidebarSide {
        SidebarSide::default()
    }

    // `true` would silence every thread's OS notification in the window while the rail is
    // open, the Agent Panel's terminal threads included, which the rail does not list.
    fn is_threads_list_view_active(&self) -> bool {
        false
    }

    // Runs deferred, outside the `MultiWorkspace`'s update, so the overlay can be set here.
    fn toggle_thread_switcher(
        &mut self,
        select_last: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(open) = &self.switcher {
            open.view
                .update(cx, |switcher, cx| switcher.step(!select_last, cx));
            return;
        }
        let entries = self.switcher_entries();
        if entries.len() < 2 {
            return;
        }
        let return_focus = window.focused(cx);
        let view = cx.new(|cx| RailSwitcher::new(entries, select_last, window, cx));
        let events = cx.subscribe_in(
            &view,
            window,
            |rail, _, event: &SwitcherEvent, window, cx| rail.switcher_ended(event, window, cx),
        );
        let overlay = AnyView::from(view.clone());
        self.multi_workspace
            .update(cx, |multi_workspace, cx| {
                multi_workspace.set_sidebar_overlay(Some(overlay), cx);
            })
            .log_err();
        // The overlay neither focuses nor dismisses what it shows, and the release of `ctrl`
        // reaches only the focused view.
        window.focus(&view.focus_handle(cx), cx);
        self.switcher = Some(OpenSwitcher {
            view,
            return_focus,
            _events: events,
        });
    }

    // Runs deferred, outside the `MultiWorkspace`'s update, so the project can be shown here.
    fn cycle_project(&mut self, forward: bool, window: &mut Window, cx: &mut Context<Self>) {
        let project = marley_rail::cycle_project(&self.snapshot.rail, forward);
        self.open_row(project, window, cx).log_err();
    }

    // Runs deferred, as `cycle_project` does.
    fn cycle_thread(&mut self, forward: bool, window: &mut Window, cx: &mut Context<Self>) {
        let row = marley_rail::cycle_row(&self.snapshot.rail, forward);
        self.open_row(row, window, cx).log_err();
    }

    // The window's saved sidebar state stays Zed's sidebar's while the rail stands in for it,
    // with the rail's own fields added. This runs inside the `MultiWorkspace`'s update, so it
    // reads the rail's fields and never the `MultiWorkspace`.
    fn serialized_state(&self, cx: &App) -> Option<String> {
        let zed_state = self
            .zed_sidebar
            .as_ref()
            .and_then(|(sidebar, _)| sidebar.read(cx).serialized_state(cx))
            .or_else(|| self.zed_sidebar_state.clone());
        let blob = write_rail_state(zed_state.as_deref(), self.rail_state());
        let blob = write_rail_groups(&blob, &self.group_ids());
        Some(order::write_rail_order(&blob, &self.saved_order))
    }

    fn restore_serialized_state(
        &mut self,
        state: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.zed_sidebar_state = Some(state.to_string());
        // The window's projectless groups come back once the restore is done: opening a
        // workspace into the window updates the `MultiWorkspace` this runs inside (#601).
        self.saved_order.groups = read_rail_groups(state);
        (self.saved_order.headers, self.saved_order.rows) = order::read_rail_order(state);
        if !self.saved_order.groups.is_empty() {
            let multi_workspace = self.multi_workspace.clone();
            let saved_groups = self.saved_order.groups.clone();
            window.defer(cx, move |window, cx| {
                if let Some(multi_workspace) = multi_workspace.upgrade() {
                    groups::reopen(&multi_workspace, &saved_groups, window, cx);
                }
            });
        }
        let saved = read_rail_state(state);
        if let Some(width) = saved.width {
            self.width = px(width).clamp(MIN_WIDTH, MAX_WIDTH);
            self.width_set_by_user = true;
        }
        if saved.closed {
            self.closed = true;
            // The restore runs inside the `MultiWorkspace`'s update, and closing reads the rail,
            // so the close waits until neither is being updated: a `defer_in` on the rail would
            // still run inside the rail's own update.
            let multi_workspace = self.multi_workspace.clone();
            window.defer(cx, move |window, cx| {
                let close = |multi_workspace: &mut MultiWorkspace,
                             cx: &mut Context<MultiWorkspace>| {
                    multi_workspace.close_sidebar(window, cx);
                };
                multi_workspace.update(cx, close).log_err();
            });
        }
    }
}

impl Render for Rail {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let brain = self.shown_brain().cloned();
        v_flex()
            .id("marley-rail")
            // Nothing moves under the pointer: the order the rail showed when it came over is held
            // until it leaves (#542).
            .on_hover(cx.listener(|rail, hovered: &bool, window, cx| {
                // gpui reads no hover while anything is dragged, so a drag begun in the rail reads
                // as the pointer leaving it: the order holds until the drag ends (#602).
                if !hovered && rail.hold.dragging {
                    return;
                }
                // gpui reports the pointer over the rail only with no drag in flight, so a flag
                // still set then is stale.
                rail.hold.dragging = false;
                rail.hold.order = hovered.then(|| marley_rail::held_order(&rail.snapshot.rail));
                if !hovered {
                    rail.refresh(window, cx);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|rail, _, window, cx| rail.end_drag(window, cx)),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|rail, _, window, cx| rail.end_drag(window, cx)),
            )
            // Zed binds left and right for lists only in the `menu` context.
            .key_context("MarleyRail menu")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_previous))
            .on_action(cx.listener(Self::select_first))
            .on_action(cx.listener(Self::select_last))
            .on_action(cx.listener(Self::select_parent))
            .on_action(cx.listener(Self::select_child))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::focus_filter))
            .on_action(cx.listener(Self::cancel))
            .size_full()
            .bg(cx.theme().colors().panel_background)
            .child(self.render_header(window, cx))
            .map(|rail| match brain {
                Some(brain) => rail.child(brain),
                None => rail
                    .child(self.render_filter(cx))
                    .children(self.render_inbox(cx))
                    .child(self.render_rows(self.render_blocks(cx), cx)),
            })
            .children(self.empty_menu.as_ref().map(|(menu, position, _)| {
                deferred(
                    anchored()
                        .position(*position)
                        .anchor(Anchor::TopLeft)
                        .child(menu.clone()),
                )
                .with_priority(1)
            }))
    }
}

#[cfg(test)]
#[path = "rail_tests.rs"]
mod tests;
