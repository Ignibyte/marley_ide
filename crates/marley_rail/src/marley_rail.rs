//! The Marley rail's row model: which rows the rail shows for a window, in what order, and which
//! single row is selected.
//!
//! Pure and gpui-free, so every decision is unit-tested in milliseconds. The gpui side
//! (`marley_workbench`) builds a [`RailSnapshot`] from the live window, with the keyboard's row
//! and what the filter matched, and renders the [`Row`]s this crate returns; it decides no
//! ordering, visibility or selection of its own.

// gate:21 runs Zed's dylint lints (`tooling/lints`) with these as errors in the Marley crates;
// Zed's crates keep them at warn (CONSTITUTION §0).
#![cfg_attr(
    dylint_lib = "lints",
    deny(
        async_block_without_await,
        blocking_io_on_foreground,
        entity_update_in_render,
        map_lookup_then_insert,
        notify_in_render,
        owned_string_into_shared,
        shared_string_from_str_literal
    )
)]

use std::path::Path;

use marley_agent::risk::Chip;
use marley_agent::route::RouteMark;
use marley_agent::{AgentKind, AgentStatus, PermissionMark};

/// One project group as the rail sees it: Zed's project group flattened to what a row shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSnapshot {
    /// The group's display name (disambiguated against the window's other groups).
    pub name: String,
    /// Whether the group is expanded (Zed keeps this per group in the `MultiWorkspace`).
    pub expanded: bool,
    /// The group's center terminals, in the order the rail lists them.
    pub terminals: Vec<TerminalSnapshot>,
    /// The group's Browser tabs, in the order its workspaces list them (#504).
    pub browsers: Vec<BrowserSnapshot>,
    /// The group's agent threads, in the order the rail lists them (newest first).
    pub threads: Vec<ThreadSnapshot>,
    /// The ports the group's processes listen on, by port (#521).
    pub ports: Vec<PortSnapshot>,
    /// The linked worktrees of the group's repository, each a row with its terminals under it
    /// (#510).
    pub worktrees: Vec<WorktreeSnapshot>,
    /// Where the filter matched the name, as the byte offsets of the matched characters; `None`
    /// when it did not. Read only while [`RailSnapshot::filtering`].
    pub matched: Option<Vec<usize>>,
    /// Whether the window holds no workspace of the group, as after a restart, which reopens only
    /// the shown one (#606): its header is listed, dimmed, with nothing under it.
    pub closed: bool,
}

/// One linked worktree of a project's repository (#510).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeSnapshot {
    /// The worktree's folder: the row's identity, and what its terminals'
    /// [`TerminalSnapshot::worktree`] names.
    pub path: String,
    /// The name Zed gives a linked worktree.
    pub name: String,
    /// Its branch, or its short commit when it is detached.
    pub branch: Option<String>,
    /// Whether the window has its workspace open.
    pub open: bool,
    /// Where the filter matched the name, as for [`ProjectSnapshot::matched`]; empty when it
    /// matched the branch alone.
    pub matched: Option<Vec<usize>>,
    /// What its branch would meet merging its base, once Marley has read it (#560).
    pub drift: Option<DriftSnapshot>,
}

/// How many conflicting files a drift's tooltip names before it counts the rest.
const TOOLTIP_FILES: usize = 20;

/// What a worktree's branch would meet merging its base (#560).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriftSnapshot {
    /// The commits on the branch the base does not have (#511).
    pub ahead: u32,
    /// The commits on the base the branch does not have.
    pub behind: u32,
    /// The files a merge would stop on, none for a clean merge; `None` when git cannot say.
    pub conflicts: Option<Vec<String>>,
    /// The base: a branch, or a commit.
    pub base: String,
    /// The base's tip, abbreviated.
    pub base_commit: String,
}

impl DriftSnapshot {
    /// Whether a merge would stop.
    #[must_use]
    pub fn conflicted(&self) -> bool {
        self.conflicts
            .as_ref()
            .is_some_and(|files| !files.is_empty())
    }

    /// The chip's words, `1 conflict`, `3 conflicts` or `2 behind`; none for a branch up to date
    /// with its base that merges cleanly, and none for a branch with no commits of its own (#511):
    /// one merged, whose base holds a merge commit it lacks, or one not started.
    #[must_use]
    pub fn words(&self) -> Option<String> {
        if self.ahead == 0 {
            return None;
        }
        let conflicts = self.conflicts.as_ref().map_or(0, Vec::len);
        match conflicts {
            0 => (self.behind > 0).then(|| format!("{} behind", self.behind)),
            1 => Some("1 conflict".to_string()),
            _ => Some(format!("{conflicts} conflicts")),
        }
    }

    /// The row's count (#511), `2 ahead of main`, while the branch has commits its base lacks.
    #[must_use]
    pub fn ahead_words(&self) -> Option<String> {
        (self.ahead > 0).then(|| format!("{} ahead of {}", self.ahead, self.base_name()))
    }

    /// The base as the row names it: a branch, or a recorded commit abbreviated.
    fn base_name(&self) -> &str {
        // A base #510 recorded as a detached main checkout's commit is its own tip.
        if self.base.starts_with(&self.base_commit) {
            &self.base_commit
        } else {
            &self.base
        }
    }

    /// The chip's tooltip: the commits behind, the base and its tip, and the files a merge would
    /// stop on, twenty at most.
    #[must_use]
    pub fn tooltip(&self) -> String {
        let commits = if self.behind == 1 {
            "1 commit".to_string()
        } else {
            format!("{} commits", self.behind)
        };
        // A base #510 recorded as a detached main checkout's commit is its own tip.
        let base = if self.base.starts_with(&self.base_commit) {
            self.base_commit.clone()
        } else {
            format!("{} ({} at {})", self.base, self.base, self.base_commit)
        };
        let mut lines = vec![format!("{commits} behind {base}")];
        match &self.conflicts {
            Some(files) if !files.is_empty() => {
                lines.push("A merge would stop on:".to_string());
                lines.extend(files.iter().take(TOOLTIP_FILES).cloned());
                if files.len() > TOOLTIP_FILES {
                    lines.push(format!("and {} more", files.len() - TOOLTIP_FILES));
                }
            }
            Some(_) => lines.push("It merges cleanly.".to_string()),
            None => lines.push("This git cannot tell whether it merges cleanly.".to_string()),
        }
        lines.join("\n")
    }
}

/// One center terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalSnapshot {
    /// The terminal view's entity id: the row's identity across rebuilds.
    pub id: u64,
    /// The title the terminal's tab shows.
    pub title: String,
    /// A second line under the title (see [`working_directory_label`]).
    pub subtitle: Option<String>,
    /// Whether the terminal rang its bell and nothing has cleared it yet.
    pub bell: bool,
    /// The agent CLI in the terminal's foreground, if one is.
    pub agent: Option<TerminalAgent>,
    /// A third line under an agent's status, from the agent's own events (#519): the tool in
    /// flight, what the agent waits on, its last message or its error.
    pub activity: Option<String>,
    /// The agent's stall or loop flag (#569), as the warning mark's tooltip says it; `None` when
    /// the row carries no flag.
    pub flag: Option<String>,
    /// A plain terminal's running or last command (#551); `None` for an agent's row or before
    /// the first command.
    pub command: Option<CommandSnapshot>,
    /// The failure a running command printed and did not end on (#572); `None` while none is
    /// open.
    pub running_error: Option<RunningError>,
    /// Where the agent's status comes from, which its class in the rail's order reads (#542).
    pub reporting: Reporting,
    /// The turns of the terminal's Claude Code that changed the tree (#509), newest first.
    pub turns: Vec<TurnSnapshot>,
    /// Whether the row's turns are listed under it.
    pub turns_open: bool,
    /// The folder of the linked worktree whose workspace holds the terminal, which lists it under
    /// that worktree's row (#510); `None` for the main checkout's.
    pub worktree: Option<String>,
    /// Where the filter matched the title, as for [`ProjectSnapshot::matched`].
    pub matched: Option<Vec<usize>>,
}

/// A plain terminal's running or last command, for its row's line (#551).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSnapshot {
    /// The command's text, on one line and cut to the row.
    pub text: String,
    /// Whether it still runs.
    pub running: bool,
    /// Its exit code, once it finished and the shell reported one.
    pub exit_code: Option<i32>,
    /// How long it took, once it finished, as `marley_terminal::duration_label` says it.
    pub duration: Option<String>,
    /// Whether it reads a password now, with the PTY's echo off.
    pub password: bool,
}

/// A failure a running command printed and kept running after, for its row's mark (#572).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunningError {
    /// The first failure line, as printed.
    pub line: String,
    /// Whether a reading in `suggest`, not a shape, found it, which the mark shows with a `?`.
    pub questioned: bool,
}

/// One turn of a terminal's Claude Code that changed the tree (#509).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnSnapshot {
    /// The prompt on one line, the slash command, or what injected it.
    pub title: String,
    /// How many files it changed.
    pub files: usize,
    /// Whether it ended in an error.
    pub failed: bool,
    /// Whether a harness's prompt started it.
    pub injected: bool,
    /// Its commit, in full: what a click on its row opens.
    pub sha: String,
}

/// One Browser tab (#504).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserSnapshot {
    /// The tab's view entity id: the row's identity across rebuilds.
    pub id: u64,
    /// The title the tab shows.
    pub title: String,
    /// The page's host and port, for an http or https page.
    pub host: Option<String>,
    /// Whether the page's main frame is loading.
    pub loading: bool,
    /// How many picks the tab holds in its tray.
    pub picks: usize,
    /// How many annotations the page has, the user's and the agents'.
    pub annotations: usize,
    /// Whether an agent acted in the page since a tab last drew it.
    pub agent_unseen: bool,
    /// The page's icon's identity, once one was read, so a new icon changes the snapshot.
    pub icon: Option<u64>,
    /// Where the filter matched the title, as for [`ProjectSnapshot::matched`].
    pub matched: Option<Vec<usize>>,
}

/// A port a process of the group listens on (#521).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortSnapshot {
    /// The port.
    pub port: u16,
    /// The process that listens: with the port, the row's identity across rebuilds.
    pub pid: u32,
    /// The row's title: the port and the process's name.
    pub title: String,
    /// The URL that reaches it.
    pub url: String,
    /// The process's command line, working directory and pid, for the row's tooltip.
    pub tooltip: String,
    /// The systemd service the process runs in, when it runs in one (#603).
    pub service: Option<PortService>,
    /// The container that publishes the port, when a container does (#614).
    pub container: Option<PortContainer>,
    /// Where the filter matched the title, as for [`ProjectSnapshot::matched`].
    pub matched: Option<Vec<usize>>,
}

/// A container that publishes a port (#614), which the row's Stop stops.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortContainer {
    /// The engine's command, `docker` or `podman`.
    pub engine: String,
    /// The container's name, when the engine said it.
    pub name: Option<String>,
    /// The container's address and port, when known.
    pub target: Option<String>,
}

/// The systemd service a port's process runs in (#603), which the row's Stop stops.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortService {
    /// The unit's name.
    pub unit: String,
    /// Whether the user's own manager runs it, rather than the system's.
    pub user: bool,
}

/// An agent CLI running in a terminal, and what it is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalAgent {
    /// Which agent it is.
    pub kind: AgentKind,
    /// Whether it is working or waiting on the user.
    pub status: AgentStatus,
    /// Whether it runs without its permission prompts, and where Marley read it (#532).
    pub mark: Option<PermissionMark>,
}

/// One agent thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadSnapshot {
    /// The thread's stable id as a string: the row's identity across rebuilds.
    pub key: String,
    /// The thread's title.
    pub title: String,
    /// What the thread is doing, as far as its live conversation says.
    pub status: ThreadStatus,
    /// Whether a run ended while the thread was not shown and nothing has shown it since.
    pub attention: bool,
    /// Where the filter matched the title, as for [`ProjectSnapshot::matched`].
    pub matched: Option<Vec<usize>>,
}

/// What an agent thread is doing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ThreadStatus {
    /// Idle, or not loaded: nothing is running and nothing waits on the user.
    #[default]
    Done,
    /// The agent is working.
    Running,
    /// The agent waits for the user to allow or deny a tool call.
    Waiting,
    /// The last run failed.
    Error,
}

impl ThreadStatus {
    /// The word a thread row shows for the status, after the agent's name, as an agent CLI's
    /// row shows its own (`marley_agent::status_line`).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Done => "idle",
            Self::Running => "working",
            Self::Waiting => "waiting",
            Self::Error => "failed",
        }
    }
}

/// A live thread's status from what its conversation reports.
///
/// A pending confirmation outranks an error, which outranks a running agent: the user has to
/// act on a confirmation before anything else happens, and an error is what the last run left.
#[must_use]
pub const fn thread_status(
    waiting_for_confirmation: bool,
    errored: bool,
    generating: bool,
) -> ThreadStatus {
    if waiting_for_confirmation {
        ThreadStatus::Waiting
    } else if errored {
        ThreadStatus::Error
    } else if generating {
        ThreadStatus::Running
    } else {
        ThreadStatus::Done
    }
}

/// Whether a thread's row carries the attention dot after this rebuild.
///
/// A run that just ended, running before and done or failed now, lights it unless the thread is
/// shown; once lit it stays lit until the thread is shown. `previous` is the status the last
/// rebuild saw, `None` for a thread seen for the first time.
#[must_use]
pub fn thread_attention(
    previous: Option<ThreadStatus>,
    current: ThreadStatus,
    shown: bool,
    noted: bool,
) -> bool {
    if shown {
        return false;
    }
    let run_ended = previous == Some(ThreadStatus::Running)
        && matches!(current, ThreadStatus::Done | ThreadStatus::Error);
    noted || run_ended
}

/// What the window shows right now; the one selected row follows it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Focus {
    /// The index of the displayed workspace's project group, if the rail lists it.
    pub project: Option<usize>,
    /// The displayed workspace's active center item, when that item is a terminal.
    pub terminal: Option<u64>,
    /// The displayed workspace's active center item, when that item is a Browser tab (#504).
    pub browser: Option<u64>,
    /// The displayed workspace's folder, when it is a listed linked worktree (#510).
    pub worktree: Option<String>,
    /// Whether that terminal holds the window's focus.
    pub terminal_focused: bool,
    /// The thread the displayed workspace's Agent Panel shows, while the panel holds focus.
    pub thread: Option<String>,
    /// The row the keyboard is on, while the rail holds focus.
    pub cursor: Option<Selection>,
}

/// Where an entry of the rail's inbox waits (#508).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InboxKind {
    /// An Agent Panel thread's tool call, waiting for confirmation.
    Thread,
    /// An agent CLI in a terminal, waiting on a permission or a question.
    Terminal,
    /// An agent's click a Browser tab holds (#571).
    Click,
}

/// An agent that waits on the user, as the rail's inbox lists it (#508).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboxEntry {
    /// What waits and on what: the entry's identity across rebuilds.
    pub key: String,
    /// Where it waits.
    pub kind: InboxKind,
    /// The agent: `Claude Code`, an Agent Panel agent's name, or `Browser tab` for a held click,
    /// whose `ask` names the caller.
    pub agent: String,
    /// The project it waits in.
    pub project: String,
    /// What it asks, on one line.
    pub ask: String,
    /// How long it has waited, in words: `now`, `3 m`.
    pub waited: String,
    /// Whether it answers in place, with Allow and Deny, or Allow and Refuse for a click.
    pub answers: bool,
    /// What its action would do, as Marley's rules and a model's reading mark it (#568); none
    /// while the inbox's risk use is off.
    pub chips: Vec<Chip>,
    /// Its level, from 1 to 5, which orders the inbox while the use is on (#568).
    pub level: u8,
    /// Who should answer it, and where that came from (#570); none while the question route's use
    /// is off, or while nothing has said.
    pub route: Option<RouteMark>,
}

/// How long an inbox entry has waited, in words, from `seconds`: `now` under a minute, then
/// `3 m`, then `1 h 5 m`.
#[must_use]
pub fn waited_words(seconds: u64) -> String {
    let minutes = seconds / 60;
    match minutes {
        0 => "now".to_string(),
        1..=59 => format!("{minutes} m"),
        _ => format!("{} h {} m", minutes / 60, minutes % 60),
    }
}

/// The window, as the rail sees it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RailSnapshot {
    /// The project groups in the window's order.
    pub projects: Vec<ProjectSnapshot>,
    /// What the window shows.
    pub focus: Focus,
    /// Whether the filter holds text, so the rail shows only what it matched.
    pub filtering: bool,
    /// The agents that wait on the user, the one that has waited longest first (#508).
    pub inbox: Vec<InboxEntry>,
    /// Whether a model's chips in the inbox show as suggestions, with a question mark, rather
    /// than with a dashed border (#568).
    pub inbox_suggests: bool,
    /// Whether a route a model read shows as a suggestion, with a question mark (#570).
    pub route_suggests: bool,
    /// The order projects and rows are listed in (#542).
    pub order: RailOrder,
    /// The order the rail showed when the pointer came over it, kept while it stays (#542).
    pub held: Option<Held>,
}

/// The single selected row.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Selection {
    /// Nothing to select: the window shows no project the rail lists.
    None,
    /// A project header, by its index.
    Project(usize),
    /// A terminal row, by the terminal's id.
    Terminal(u64),
    /// A Browser tab's row, by the tab's view id (#504).
    Browser(u64),
    /// A thread row, by the thread's key.
    Thread(String),
    /// A port's row, by the port and the pid that listens on it (#521).
    Port(u16, u32),
    /// A linked worktree's row, by its folder (#510).
    Worktree(String),
}

/// A project group's header row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRow {
    /// The group's index in [`RailSnapshot::projects`].
    pub index: usize,
    /// The display name.
    pub name: String,
    /// Whether the group's terminals are listed under it.
    pub expanded: bool,
    /// Whether this is the selected row.
    pub selected: bool,
    /// Whether a row the rail is not showing under it needs the user: a terminal's bell, or a
    /// thread's attention dot or wait for a confirmation.
    pub attention: bool,
    /// The byte offsets of the characters the filter matched, to highlight.
    pub highlight: Vec<usize>,
    /// For a collapsed project, its agents by state, most needing the user first: `1 waiting, 2
    /// working` (#542); `None` with nothing to count.
    pub summary: Option<String>,
}

/// A terminal row under its project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalRow {
    /// The group's index in [`RailSnapshot::projects`].
    pub project: usize,
    /// The terminal view's entity id.
    pub id: u64,
    /// The title.
    pub title: String,
    /// The second line, if any.
    pub subtitle: Option<String>,
    /// Whether the bell dot shows.
    pub bell: bool,
    /// The agent CLI the terminal runs, which makes the row an agent row.
    pub agent: Option<TerminalAgent>,
    /// The third line, if any (see [`TerminalSnapshot::activity`]).
    pub activity: Option<String>,
    /// The warning mark's tooltip, when the agent is flagged (see [`TerminalSnapshot::flag`]).
    pub flag: Option<String>,
    /// The command's line, if any (see [`TerminalSnapshot::command`]).
    pub command: Option<CommandSnapshot>,
    /// The running command's failure, if any (see [`TerminalSnapshot::running_error`]).
    pub running_error: Option<RunningError>,
    /// The turns listed under the row (see [`TerminalSnapshot::turns`]).
    pub turns: Vec<TurnSnapshot>,
    /// Whether the turns are listed.
    pub turns_open: bool,
    /// The folder of the worktree whose row the row sits under (#510), if one.
    pub worktree: Option<String>,
    /// Whether this is the selected row.
    pub selected: bool,
    /// The byte offsets of the title's characters the filter matched, to highlight.
    pub highlight: Vec<usize>,
}

/// A linked worktree's row under its project (#510).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeRow {
    /// The group's index in [`RailSnapshot::projects`].
    pub project: usize,
    /// The worktree's folder.
    pub path: String,
    /// The name.
    pub name: String,
    /// The branch, the second line.
    pub branch: Option<String>,
    /// Whether its workspace is open.
    pub open: bool,
    /// What its branch would meet merging its base (see [`WorktreeSnapshot::drift`]).
    pub drift: Option<DriftSnapshot>,
    /// Whether this is the selected row.
    pub selected: bool,
    /// The byte offsets of the name's characters the filter matched, to highlight.
    pub highlight: Vec<usize>,
}

/// A Browser tab's row under its project (#504).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserRow {
    /// The group's index in [`RailSnapshot::projects`].
    pub project: usize,
    /// The tab's view entity id.
    pub id: u64,
    /// The title.
    pub title: String,
    /// The page's host and port, the second line.
    pub host: Option<String>,
    /// Whether the page is loading, which draws the spinner.
    pub loading: bool,
    /// How many picks the tab holds.
    pub picks: usize,
    /// How many annotations the page has.
    pub annotations: usize,
    /// Whether the agent's mark shows.
    pub agent_unseen: bool,
    /// Whether this is the selected row.
    pub selected: bool,
    /// The byte offsets of the title's characters the filter matched, to highlight.
    pub highlight: Vec<usize>,
}

/// An agent thread row under its project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadRow {
    /// The group's index in [`RailSnapshot::projects`].
    pub project: usize,
    /// The thread's key.
    pub key: String,
    /// The title.
    pub title: String,
    /// What the thread is doing.
    pub status: ThreadStatus,
    /// Whether the attention dot shows.
    pub attention: bool,
    /// Whether this is the selected row.
    pub selected: bool,
    /// The byte offsets of the title's characters the filter matched, to highlight.
    pub highlight: Vec<usize>,
}

/// A port's row under its project (#521).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortRow {
    /// The group's index in [`RailSnapshot::projects`].
    pub project: usize,
    /// The port.
    pub port: u16,
    /// The process that listens on it.
    pub pid: u32,
    /// The title.
    pub title: String,
    /// The URL, the second line.
    pub url: String,
    /// The tooltip.
    pub tooltip: String,
    /// The service its process runs in (see [`PortSnapshot::service`]).
    pub service: Option<PortService>,
    /// The container that publishes it (see [`PortSnapshot::container`]).
    pub container: Option<PortContainer>,
    /// Whether this is the selected row.
    pub selected: bool,
    /// The byte offsets of the title's characters the filter matched, to highlight.
    pub highlight: Vec<usize>,
}

/// One row of the rail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Row {
    /// A project group's header.
    Project(ProjectRow),
    /// A terminal under its project.
    Terminal(TerminalRow),
    /// A Browser tab under its project, after the project's terminals (#504).
    Browser(BrowserRow),
    /// An agent thread under its project, after the project's terminals and Browser tabs.
    Thread(ThreadRow),
    /// A port under its project, after everything else under it (#521).
    Port(PortRow),
    /// A linked worktree under its project, after the main checkout's terminals, with its own
    /// terminals under it (#510).
    Worktree(WorktreeRow),
}

/// A row the switcher lists: a terminal or a thread, never a header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwitcherRow {
    /// A terminal, unselected and unhighlighted; boxed, as a terminal's row is the larger by far.
    Terminal(Box<TerminalRow>),
    /// A thread, unselected and unhighlighted.
    Thread(ThreadRow),
}

impl SwitcherRow {
    /// The row the rail opens for it.
    #[must_use]
    pub fn selection(&self) -> Selection {
        match self {
            Self::Terminal(row) => Selection::Terminal(row.id),
            Self::Thread(row) => Selection::Thread(row.key.clone()),
        }
    }
}

/// The one selected row.
///
/// While the rail holds focus, the keyboard's row wins when it is shown. While the displayed
/// workspace's Agent Panel holds focus, the thread it shows wins when that thread's row is shown.
/// Otherwise the active terminal wins when its row is shown, and otherwise the displayed
/// workspace's project header does when the rail shows it. So a window whose project the rail
/// shows has exactly one selected row, and a stale focus never selects a row that is not there.
#[must_use]
pub fn selection(snapshot: &RailSnapshot) -> Selection {
    let rows = walk(snapshot);
    if let Some(cursor) = &snapshot.focus.cursor
        && rows.iter().any(|row| row.selection() == *cursor)
    {
        return cursor.clone();
    }
    let Some(index) = snapshot.focus.project else {
        return Selection::None;
    };
    let thread = snapshot.focus.thread.clone().map(Selection::Thread);
    let terminal = snapshot.focus.terminal.map(Selection::Terminal);
    let browser = snapshot.focus.browser.map(Selection::Browser);
    let worktree = snapshot.focus.worktree.clone().map(Selection::Worktree);
    [
        thread,
        terminal,
        browser,
        worktree,
        Some(Selection::Project(index)),
    ]
    .into_iter()
    .flatten()
    .find(|wanted| {
        rows.iter()
            .any(|row| row.project() == index && row.selection() == *wanted)
    })
    .unwrap_or(Selection::None)
}

/// A row the rail shows, before it is drawn, with the index of the project it sits under.
#[derive(Clone, Copy)]
enum Shown<'a> {
    Project(usize, &'a ProjectSnapshot),
    Terminal(usize, &'a TerminalSnapshot),
    Browser(usize, &'a BrowserSnapshot),
    Thread(usize, &'a ThreadSnapshot),
    Port(usize, &'a PortSnapshot),
    Worktree(usize, &'a WorktreeSnapshot),
}

impl<'a> Shown<'a> {
    const fn project(self) -> usize {
        match self {
            Self::Project(index, _)
            | Self::Terminal(index, _)
            | Self::Browser(index, _)
            | Self::Thread(index, _)
            | Self::Port(index, _)
            | Self::Worktree(index, _) => index,
        }
    }

    fn selection(self) -> Selection {
        match self {
            Self::Project(index, _) => Selection::Project(index),
            Self::Terminal(_, terminal) => Selection::Terminal(terminal.id),
            Self::Browser(_, browser) => Selection::Browser(browser.id),
            Self::Thread(_, thread) => Selection::Thread(thread.key.clone()),
            Self::Port(_, port) => Selection::Port(port.port, port.pid),
            Self::Worktree(_, worktree) => Selection::Worktree(worktree.path.clone()),
        }
    }

    /// Where the filter matched the row's own name or title.
    fn matched(self) -> Option<&'a [usize]> {
        match self {
            Self::Project(_, project) => project.matched.as_deref(),
            Self::Terminal(_, terminal) => terminal.matched.as_deref(),
            Self::Browser(_, browser) => browser.matched.as_deref(),
            Self::Thread(_, thread) => thread.matched.as_deref(),
            Self::Port(_, port) => port.matched.as_deref(),
            Self::Worktree(_, worktree) => worktree.matched.as_deref(),
        }
    }
}

/// Where a terminal agent's status comes from (#542).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Reporting {
    /// The quiet timer, for an agent that sends no events (or no agent).
    #[default]
    Timer,
    /// The agent's own events (#519), so a waiting or failed status is its own word.
    Events,
    /// Its events said it works and have stopped past the setting's minutes, its row's `no
    /// update in N m` (#547).
    Stale,
}

/// How much a row needs the user, most first (#542).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Attention {
    /// An agent waits on the user, or failed and the user has not looked since.
    NeedsYou,
    /// An agent or a program finished while the user looked elsewhere.
    DoneUnseen,
    /// An agent works.
    Working,
    /// An agent said it works and has sent nothing for a while.
    NotReporting,
    /// Everything else.
    Idle,
}

/// The order the rail lists projects and rows in (#542).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RailOrder {
    /// What needs the user first; ties keep the window's order.
    #[default]
    Attention,
    /// The window's order, as before.
    Window,
}

/// The order the rail showed when the pointer came over it, which it keeps while the pointer
/// stays, so a row never moves under it (#542). What appeared since goes last.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Held {
    /// The projects' window indices, in the order shown.
    pub projects: Vec<usize>,
    /// The terminals' ids, in the order shown.
    pub terminals: Vec<u64>,
    /// The threads' keys, in the order shown.
    pub threads: Vec<String>,
}

/// How much `terminal` needs the user. An agent that sends no events cannot say it waits, so only
/// its bell lifts it (#542's D4).
#[must_use]
pub fn terminal_attention(terminal: &TerminalSnapshot) -> Attention {
    let status = terminal.agent.map(|agent| agent.status);
    let reports = terminal.reporting != Reporting::Timer;
    match (status, reports) {
        (Some(AgentStatus::Waiting), true) => Attention::NeedsYou,
        (Some(AgentStatus::Failed), true) if terminal.bell => Attention::NeedsYou,
        (Some(AgentStatus::Working), true) if terminal.reporting == Reporting::Stale => {
            Attention::NotReporting
        }
        (Some(AgentStatus::Working), _) => Attention::Working,
        _ if terminal.bell => Attention::DoneUnseen,
        _ => Attention::Idle,
    }
}

/// How much `thread` needs the user: a wait on a confirmation, a failure or an end not yet seen.
#[must_use]
pub const fn thread_attention_class(thread: &ThreadSnapshot) -> Attention {
    match thread.status {
        ThreadStatus::Waiting => Attention::NeedsYou,
        ThreadStatus::Error if thread.attention => Attention::NeedsYou,
        ThreadStatus::Done if thread.attention => Attention::DoneUnseen,
        ThreadStatus::Running => Attention::Working,
        ThreadStatus::Error | ThreadStatus::Done => Attention::Idle,
    }
}

/// A project's class: its most demanding row's.
#[must_use]
pub fn project_attention(project: &ProjectSnapshot) -> Attention {
    project
        .terminals
        .iter()
        .map(terminal_attention)
        .chain(project.threads.iter().map(thread_attention_class))
        .min()
        .unwrap_or(Attention::Idle)
}

/// The projects' window indices in the order the rail lists them.
fn project_order(snapshot: &RailSnapshot) -> Vec<usize> {
    let mut order: Vec<usize> = (0..snapshot.projects.len()).collect();
    match (snapshot.order, &snapshot.held) {
        (RailOrder::Window, _) => {}
        (RailOrder::Attention, Some(held)) => {
            order.sort_by_key(|index| held_position(&held.projects, index));
        }
        (RailOrder::Attention, None) => {
            order.sort_by_key(|index| {
                snapshot
                    .projects
                    .get(*index)
                    .map_or(Attention::Idle, project_attention)
            });
        }
    }
    order
}

/// Where `item` sat in a held order; what was not there goes last, in the order it came.
fn held_position<T: PartialEq>(held: &[T], item: &T) -> usize {
    held.iter()
        .position(|shown| shown == item)
        .unwrap_or(usize::MAX)
}

/// `terminals` in the order the rail lists them under their project.
fn arrange_terminals<'a>(
    snapshot: &RailSnapshot,
    mut terminals: Vec<&'a TerminalSnapshot>,
) -> Vec<&'a TerminalSnapshot> {
    match (snapshot.order, &snapshot.held) {
        (RailOrder::Window, _) => {}
        (RailOrder::Attention, Some(held)) => {
            terminals.sort_by_key(|terminal| held_position(&held.terminals, &terminal.id));
        }
        (RailOrder::Attention, None) => {
            terminals.sort_by_key(|terminal| terminal_attention(terminal));
        }
    }
    terminals
}

/// `threads` in the order the rail lists them under their project.
fn arrange_threads<'a>(
    snapshot: &RailSnapshot,
    mut threads: Vec<&'a ThreadSnapshot>,
) -> Vec<&'a ThreadSnapshot> {
    match (snapshot.order, &snapshot.held) {
        (RailOrder::Window, _) => {}
        (RailOrder::Attention, Some(held)) => {
            threads.sort_by_key(|thread| held_position(&held.threads, &thread.key));
        }
        (RailOrder::Attention, None) => {
            threads.sort_by_key(|thread| thread_attention_class(thread));
        }
    }
    threads
}

/// The order the rail shows now, to hold while the pointer is over it (#542).
#[must_use]
pub fn held_order(snapshot: &RailSnapshot) -> Held {
    let mut held = Held::default();
    for shown in walk(snapshot) {
        match shown {
            Shown::Project(index, _) => held.projects.push(index),
            Shown::Terminal(_, terminal) => held.terminals.push(terminal.id),
            Shown::Thread(_, thread) => held.threads.push(thread.key.clone()),
            Shown::Worktree(..) | Shown::Browser(..) | Shown::Port(..) => {}
        }
    }
    held
}

/// The rows a dragged row may be dropped among, so a drop only reorders (#602). Under the
/// attention order a row keeps to its class, which the rail sorts first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Run {
    /// The headers, projects' and projectless groups' alike.
    Headers(Option<Attention>),
    /// A project's terminals in one section: its main checkout's, or one worktree's.
    Terminals {
        /// The project's index in [`RailSnapshot::projects`].
        project: usize,
        /// The worktree's folder, `None` for the main checkout.
        worktree: Option<String>,
        /// The class, under the attention order.
        class: Option<Attention>,
    },
    /// A project's Browser tabs, which the attention order does not sort.
    Browsers {
        /// The project's index in [`RailSnapshot::projects`].
        project: usize,
    },
    /// A project's threads.
    Threads {
        /// The project's index in [`RailSnapshot::projects`].
        project: usize,
        /// The class, under the attention order.
        class: Option<Attention>,
    },
}

/// The run `row` moves within when it is dragged; `None` for a worktree's or a port's row, which do
/// not move.
#[must_use]
pub fn run(snapshot: &RailSnapshot, row: &Row) -> Option<Run> {
    let attention = snapshot.order == RailOrder::Attention;
    match row {
        Row::Project(row) => {
            let project = snapshot.projects.get(row.index)?;
            Some(Run::Headers(attention.then(|| project_attention(project))))
        }
        Row::Terminal(row) => {
            let project = snapshot.projects.get(row.project)?;
            let terminal = project
                .terminals
                .iter()
                .find(|terminal| terminal.id == row.id)?;
            Some(Run::Terminals {
                project: row.project,
                worktree: row.worktree.clone(),
                class: attention.then(|| terminal_attention(terminal)),
            })
        }
        Row::Browser(row) => Some(Run::Browsers {
            project: row.project,
        }),
        Row::Thread(row) => {
            let project = snapshot.projects.get(row.project)?;
            let thread = project
                .threads
                .iter()
                .find(|thread| thread.key == row.key)?;
            Some(Run::Threads {
                project: row.project,
                class: attention.then(|| thread_attention_class(thread)),
            })
        }
        Row::Port(_) | Row::Worktree(_) => None,
    }
}

/// Sorts `items` by where each one's place sits in `places`, the order the user left (#602); an
/// item with no place there keeps its order after the placed ones.
pub fn place<T>(items: &mut [T], places: &[String], place_of: impl Fn(&T) -> Option<String>) {
    items.sort_by_cached_key(|item| {
        place_of(item)
            .and_then(|place| places.iter().position(|placed| *placed == place))
            .unwrap_or(usize::MAX)
    });
}

/// `order` with `moved` taken out and put back just before `target`, or just after it (#602); the
/// order unchanged when either is not in it, or they are the same.
#[must_use]
pub fn move_to<T: PartialEq + Clone>(order: &[T], moved: &T, target: &T, before: bool) -> Vec<T> {
    if moved == target || !order.contains(moved) || !order.contains(target) {
        return order.to_vec();
    }
    let mut result: Vec<T> = order
        .iter()
        .filter(|item| *item != moved)
        .cloned()
        .collect();
    let at = result
        .iter()
        .position(|item| item == target)
        .map_or(result.len(), |at| if before { at } else { at + 1 });
    result.insert(at, moved.clone());
    result
}

/// A collapsed project's agents by state, most needing the user first: `1 waiting, 2 working`;
/// shells and idle agents are not counted.
fn summary(project: &ProjectSnapshot) -> Option<String> {
    const WORDS: [&str; 5] = ["waiting", "failed", "done", "working", "not reporting"];
    let mut counts = [0usize; 5];
    let terminals = project.terminals.iter().filter_map(|terminal| {
        let status = terminal.agent?.status;
        let reports = terminal.reporting != Reporting::Timer;
        match (status, reports) {
            (AgentStatus::Waiting, true) => Some(0),
            (AgentStatus::Failed, true) if terminal.bell => Some(1),
            (AgentStatus::Working, true) if terminal.reporting == Reporting::Stale => Some(4),
            (AgentStatus::Working, _) => Some(3),
            _ if terminal.bell => Some(2),
            _ => None,
        }
    });
    let threads = project
        .threads
        .iter()
        .filter_map(|thread| match thread.status {
            ThreadStatus::Waiting => Some(0),
            ThreadStatus::Error if thread.attention => Some(1),
            ThreadStatus::Done if thread.attention => Some(2),
            ThreadStatus::Running => Some(3),
            ThreadStatus::Error | ThreadStatus::Done => None,
        });
    for kind in terminals.chain(threads) {
        if let Some(count) = counts.get_mut(kind) {
            *count += 1;
        }
    }
    let parts: Vec<String> = counts
        .iter()
        .zip(WORDS)
        .filter(|(count, _)| **count > 0)
        .map(|(count, word)| format!("{count} {word}"))
        .collect();
    (!parts.is_empty()).then(|| parts.join(", "))
}

/// Every row the rail shows, in its order: each shown project's header, then the main
/// checkout's terminals, each linked worktree's row with its terminals (#510), the Browser tabs,
/// the threads and the ports shown under it. The rows, the selection and the keyboard all read
/// this one walk.
fn walk(snapshot: &RailSnapshot) -> Vec<Shown<'_>> {
    let mut rows = Vec::new();
    for index in project_order(snapshot) {
        let Some(project) = snapshot.projects.get(index) else {
            continue;
        };
        let terminals = arrange_terminals(
            snapshot,
            project
                .terminals
                .iter()
                .filter(|terminal| worktree_of(project, terminal).is_none())
                .filter(|terminal| terminal_shows(snapshot, project, terminal))
                .collect(),
        )
        .into_iter()
        .map(|terminal| Shown::Terminal(index, terminal));
        let worktrees = project.worktrees.iter().flat_map(|worktree| {
            let terminals: Vec<Shown<'_>> = arrange_terminals(
                snapshot,
                project
                    .terminals
                    .iter()
                    .filter(|terminal| terminal.worktree.as_deref() == Some(worktree.path.as_str()))
                    .filter(|terminal| terminal_shows(snapshot, project, terminal))
                    .collect(),
            )
            .into_iter()
            .map(|terminal| Shown::Terminal(index, terminal))
            .collect();
            // A shown terminal keeps its worktree's row above it.
            let row = (row_shows(snapshot, project, worktree.matched.as_deref())
                || !terminals.is_empty())
            .then_some(Shown::Worktree(index, worktree));
            row.into_iter().chain(terminals)
        });
        let browsers = project
            .browsers
            .iter()
            .filter(|browser| row_shows(snapshot, project, browser.matched.as_deref()))
            .map(|browser| Shown::Browser(index, browser));
        let threads = arrange_threads(
            snapshot,
            project
                .threads
                .iter()
                .filter(|thread| row_shows(snapshot, project, thread.matched.as_deref()))
                .collect(),
        )
        .into_iter()
        .map(|thread| Shown::Thread(index, thread));
        let ports = project
            .ports
            .iter()
            .filter(|port| row_shows(snapshot, project, port.matched.as_deref()))
            .map(|port| Shown::Port(index, port));
        let under: Vec<Shown<'_>> = terminals
            .chain(worktrees)
            .chain(browsers)
            .chain(threads)
            .chain(ports)
            .collect();
        // The filter shows a project for its own name or for a row under it.
        if snapshot.filtering && project.matched.is_none() && under.is_empty() {
            continue;
        }
        rows.push(Shown::Project(index, project));
        rows.extend(under);
    }
    rows
}

/// Whether a terminal or thread row under `project` shows: under an expanded project without a
/// filter, and with one when the project's name or the row's own text matched, whatever the fold.
const fn row_shows(
    snapshot: &RailSnapshot,
    project: &ProjectSnapshot,
    matched: Option<&[usize]>,
) -> bool {
    if snapshot.filtering {
        project.matched.is_some() || matched.is_some()
    } else {
        project.expanded
    }
}

/// The listed worktree whose workspace holds `terminal`, if one does.
fn worktree_of<'a>(
    project: &'a ProjectSnapshot,
    terminal: &TerminalSnapshot,
) -> Option<&'a WorktreeSnapshot> {
    let path = terminal.worktree.as_deref()?;
    project
        .worktrees
        .iter()
        .find(|worktree| worktree.path == path)
}

/// Whether a terminal row shows: as [`row_shows`] says, and with a filter also when its
/// worktree's name or branch matched.
fn terminal_shows(
    snapshot: &RailSnapshot,
    project: &ProjectSnapshot,
    terminal: &TerminalSnapshot,
) -> bool {
    row_shows(snapshot, project, terminal.matched.as_deref())
        || (snapshot.filtering
            && worktree_of(project, terminal).is_some_and(|worktree| worktree.matched.is_some()))
}

/// Every shown row, as the selection it would be, in the rail's order.
fn shown(snapshot: &RailSnapshot) -> Vec<Selection> {
    walk(snapshot).into_iter().map(Shown::selection).collect()
}

/// The first shown row whose own name or title the filter matched, where the keyboard's row goes
/// as the filter changes; nothing without a filter, or when nothing matched.
#[must_use]
pub fn first_match(snapshot: &RailSnapshot) -> Selection {
    walk(snapshot)
        .into_iter()
        .find(|row| snapshot.filtering && row.matched().is_some())
        .map_or(Selection::None, Shown::selection)
}

/// The shown row after the selected one, or before it, staying on the last or the first. With
/// nothing selected it is the first row going forward and the last going back.
#[must_use]
pub fn step(snapshot: &RailSnapshot, forward: bool) -> Selection {
    let rows = shown(snapshot);
    let current = selection(snapshot);
    let next = match rows.iter().position(|row| *row == current) {
        Some(at) if forward => rows.get(at + 1).or_else(|| rows.get(at)),
        Some(at) => at
            .checked_sub(1)
            .and_then(|before| rows.get(before))
            .or_else(|| rows.get(at)),
        None if forward => rows.first(),
        None => rows.last(),
    };
    next.cloned().unwrap_or(Selection::None)
}

/// The first shown row.
#[must_use]
pub fn first_row(snapshot: &RailSnapshot) -> Selection {
    shown(snapshot)
        .into_iter()
        .next()
        .unwrap_or(Selection::None)
}

/// The last shown row.
#[must_use]
pub fn last_row(snapshot: &RailSnapshot) -> Selection {
    shown(snapshot).pop().unwrap_or(Selection::None)
}

/// Zed's Next and Previous Project in the rail.
///
/// The shown project header after the selected row's project, or before it, wrapping at the
/// ends. With nothing selected it is the first header going forward and the last going back.
#[must_use]
pub fn cycle_project(snapshot: &RailSnapshot, forward: bool) -> Selection {
    let project = match parent(snapshot, &selection(snapshot)) {
        worktree @ Selection::Worktree(_) => parent(snapshot, &worktree),
        project => project,
    };
    // A closed project is passed over: going to it would open it (#606).
    cycle(snapshot, &project, forward, |row| {
        matches!(row, Selection::Project(index)
            if !snapshot.projects.get(*index).is_some_and(|project| project.closed))
    })
}

/// Zed's Next and Previous Thread in the rail, which reach terminals, Browser tabs and threads
/// alike, and not ports, which are no place to switch to (#521).
///
/// The shown row under a project after the selected row, or before it, passing over project
/// headers and wrapping at the ends. With nothing selected it is the first such row going
/// forward and the last going back.
#[must_use]
pub fn cycle_row(snapshot: &RailSnapshot, forward: bool) -> Selection {
    cycle(snapshot, &selection(snapshot), forward, |row| {
        matches!(
            row,
            Selection::Terminal(_) | Selection::Browser(_) | Selection::Thread(_)
        )
    })
}

/// The first shown row after `from`, or before it, that `wanted` takes, going once round the
/// rail: past the last row comes the first, and `from` itself comes last. When `from` is not
/// shown, the search starts at the first row going forward and at the last going back.
fn cycle(
    snapshot: &RailSnapshot,
    from: &Selection,
    forward: bool,
    wanted: impl Fn(&Selection) -> bool,
) -> Selection {
    let mut rows = shown(snapshot);
    if !forward {
        rows.reverse();
    }
    if let Some(at) = rows.iter().position(|row| row == from) {
        rows.rotate_left(at + 1);
    }
    rows.into_iter().find(wanted).unwrap_or(Selection::None)
}

/// The row a row sits under: a linked worktree's terminal under the worktree's row (#510), any
/// other row under its project's header; a header is its own.
#[must_use]
pub fn parent(snapshot: &RailSnapshot, selection: &Selection) -> Selection {
    let owner = match selection {
        Selection::None | Selection::Project(_) => return selection.clone(),
        Selection::Terminal(id) => {
            let under = snapshot.projects.iter().find_map(|project| {
                let terminal = project
                    .terminals
                    .iter()
                    .find(|terminal| terminal.id == *id)?;
                Some(worktree_of(project, terminal))
            });
            if let Some(Some(worktree)) = under {
                return Selection::Worktree(worktree.path.clone());
            }
            snapshot
                .projects
                .iter()
                .position(|project| project.terminals.iter().any(|terminal| terminal.id == *id))
        }
        Selection::Browser(id) => snapshot
            .projects
            .iter()
            .position(|project| project.browsers.iter().any(|browser| browser.id == *id)),
        Selection::Thread(key) => snapshot
            .projects
            .iter()
            .position(|project| project.threads.iter().any(|thread| thread.key == *key)),
        Selection::Port(port, pid) => snapshot.projects.iter().position(|project| {
            project
                .ports
                .iter()
                .any(|shown| shown.port == *port && shown.pid == *pid)
        }),
        Selection::Worktree(path) => snapshot.projects.iter().position(|project| {
            project
                .worktrees
                .iter()
                .any(|worktree| worktree.path == *path)
        }),
    };
    owner.map_or(Selection::None, Selection::Project)
}

/// The rows, in display order.
///
/// Each shown project's header comes first, then the terminals, the Browser tabs, the threads
/// and the ports shown under it.
/// Without a filter a folded project shows its header alone. With one, a project shows when its
/// name or a row under it matched, with every row when its name did, and each row carries the
/// matched characters.
#[must_use]
pub fn rail_rows(snapshot: &RailSnapshot) -> Vec<Row> {
    let selected = selection(snapshot);
    let highlight = |matched: Option<&[usize]>| {
        matched
            .filter(|_| snapshot.filtering)
            .map(<[usize]>::to_vec)
            .unwrap_or_default()
    };
    walk(snapshot)
        .into_iter()
        .map(|row| match row {
            Shown::Project(index, project) => Row::Project(ProjectRow {
                index,
                name: project.name.clone(),
                expanded: project.expanded,
                selected: selected == Selection::Project(index),
                attention: hidden_rows_need_the_user(snapshot, project),
                highlight: highlight(project.matched.as_deref()),
                summary: (!project.expanded).then(|| summary(project)).flatten(),
            }),
            Shown::Terminal(index, terminal) => Row::Terminal(TerminalRow {
                project: index,
                id: terminal.id,
                title: terminal.title.clone(),
                subtitle: terminal.subtitle.clone(),
                bell: terminal.bell,
                agent: terminal.agent,
                activity: terminal.activity.clone(),
                flag: terminal.flag.clone(),
                command: terminal.command.clone(),
                running_error: terminal.running_error.clone(),
                turns: terminal.turns.clone(),
                turns_open: terminal.turns_open,
                worktree: snapshot
                    .projects
                    .get(index)
                    .and_then(|project| worktree_of(project, terminal))
                    .map(|worktree| worktree.path.clone()),
                selected: selected == Selection::Terminal(terminal.id),
                highlight: highlight(terminal.matched.as_deref()),
            }),
            Shown::Browser(index, browser) => Row::Browser(BrowserRow {
                project: index,
                id: browser.id,
                title: browser.title.clone(),
                host: browser.host.clone(),
                loading: browser.loading,
                picks: browser.picks,
                annotations: browser.annotations,
                agent_unseen: browser.agent_unseen,
                selected: selected == Selection::Browser(browser.id),
                highlight: highlight(browser.matched.as_deref()),
            }),
            Shown::Thread(index, thread) => Row::Thread(ThreadRow {
                project: index,
                key: thread.key.clone(),
                title: thread.title.clone(),
                status: thread.status,
                attention: thread.attention,
                selected: matches!(&selected, Selection::Thread(key) if *key == thread.key),
                highlight: highlight(thread.matched.as_deref()),
            }),
            Shown::Port(index, port) => Row::Port(PortRow {
                project: index,
                port: port.port,
                pid: port.pid,
                title: port.title.clone(),
                url: port.url.clone(),
                tooltip: port.tooltip.clone(),
                service: port.service.clone(),
                container: port.container.clone(),
                selected: selected == Selection::Port(port.port, port.pid),
                highlight: highlight(port.matched.as_deref()),
            }),
            Shown::Worktree(index, worktree) => Row::Worktree(WorktreeRow {
                project: index,
                path: worktree.path.clone(),
                name: worktree.name.clone(),
                branch: worktree.branch.clone(),
                open: worktree.open,
                drift: worktree.drift.clone(),
                selected: matches!(&selected, Selection::Worktree(path) if *path == worktree.path),
                highlight: highlight(worktree.matched.as_deref()),
            }),
        })
        .collect()
}

/// The terminal or thread row that holds the window's focus, the one the user is working in.
///
/// That is the focused Agent Panel's thread, else the displayed workspace's active terminal while
/// it holds focus, each when the displayed project lists it. The fold and the filter do not
/// matter.
#[must_use]
pub fn window_row(snapshot: &RailSnapshot) -> Option<Selection> {
    let project = snapshot.projects.get(snapshot.focus.project?)?;
    let thread = snapshot
        .focus
        .thread
        .as_ref()
        .filter(|key| project.threads.iter().any(|thread| thread.key == **key))
        .map(|key| Selection::Thread(key.clone()));
    thread.or_else(|| {
        snapshot
            .focus
            .terminal
            .filter(|id| {
                snapshot.focus.terminal_focused
                    && project.terminals.iter().any(|terminal| terminal.id == *id)
            })
            .map(Selection::Terminal)
    })
}

/// Every terminal and thread for the switcher.
///
/// The rows the window showed come first, the most recent first by `shown_at`, which ranks each
/// row by when the window last showed it; the rest follow in the rail's order. The fold and the
/// filter do not matter.
#[must_use]
pub fn switcher_rows(
    snapshot: &RailSnapshot,
    shown_at: impl Fn(&Selection) -> Option<u64>,
) -> Vec<SwitcherRow> {
    let mut rows: Vec<SwitcherRow> = Vec::new();
    for (index, project) in snapshot.projects.iter().enumerate() {
        rows.extend(project.terminals.iter().map(|terminal| {
            SwitcherRow::Terminal(Box::new(TerminalRow {
                project: index,
                id: terminal.id,
                title: terminal.title.clone(),
                subtitle: terminal.subtitle.clone(),
                bell: terminal.bell,
                agent: terminal.agent,
                activity: terminal.activity.clone(),
                flag: terminal.flag.clone(),
                command: terminal.command.clone(),
                running_error: terminal.running_error.clone(),
                turns: terminal.turns.clone(),
                turns_open: terminal.turns_open,
                worktree: None,
                selected: false,
                highlight: Vec::new(),
            }))
        }));
        rows.extend(project.threads.iter().map(|thread| {
            SwitcherRow::Thread(ThreadRow {
                project: index,
                key: thread.key.clone(),
                title: thread.title.clone(),
                status: thread.status,
                attention: thread.attention,
                selected: false,
                highlight: Vec::new(),
            })
        }));
    }
    // A stable sort, so the rows never shown keep the rail's order after the others.
    rows.sort_by_key(|row| std::cmp::Reverse(shown_at(&row.selection())));
    rows
}

/// Whether a row under `project` that the rail is not showing needs the user.
fn hidden_rows_need_the_user(snapshot: &RailSnapshot, project: &ProjectSnapshot) -> bool {
    let hidden = |matched: Option<&[usize]>| !row_shows(snapshot, project, matched);
    project
        .terminals
        .iter()
        .any(|terminal| terminal.bell && !terminal_shows(snapshot, project, terminal))
        || project
            .threads
            .iter()
            .any(|thread| thread_needs_the_user(thread) && hidden(thread.matched.as_deref()))
}

/// Whether a thread needs the user: its attention dot, or a wait for a confirmation.
fn thread_needs_the_user(thread: &ThreadSnapshot) -> bool {
    thread.attention || thread.status == ThreadStatus::Waiting
}

/// Whether anything in the project needs the user: a terminal's bell, or a thread's attention
/// dot or wait for a confirmation.
fn project_needs_the_user(project: &ProjectSnapshot) -> bool {
    project.terminals.iter().any(|terminal| terminal.bell)
        || project.threads.iter().any(thread_needs_the_user)
}

/// Whether anything listed needs the user, collapsed or not: the rail's notification flag.
#[must_use]
pub fn has_attention(snapshot: &RailSnapshot) -> bool {
    snapshot.projects.iter().any(project_needs_the_user)
}

/// The second line of a terminal row: its working directory relative to the project root, or with
/// the home directory written as `~` when it is outside the project.
///
/// `None` at the root itself, where the project header already says where the terminal is, and when
/// the directory is unknown: a terminal that cannot tell reports no path or an empty one.
#[must_use]
pub fn working_directory_label(
    working_directory: Option<&Path>,
    project_root: Option<&Path>,
    home: Option<&Path>,
) -> Option<String> {
    let working_directory = working_directory.filter(|path| !path.as_os_str().is_empty())?;
    if let Some(root) = project_root
        && let Ok(relative) = working_directory.strip_prefix(root)
    {
        return (!relative.as_os_str().is_empty()).then(|| relative.display().to_string());
    }
    if let Some(home) = home
        && let Ok(relative) = working_directory.strip_prefix(home)
    {
        return Some(if relative.as_os_str().is_empty() {
            "~".to_string()
        } else {
            format!("~/{}", relative.display())
        });
    }
    Some(working_directory.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn terminal(id: u64, bell: bool) -> TerminalSnapshot {
        TerminalSnapshot {
            id,
            title: format!("terminal {id}"),
            subtitle: None,
            bell,
            agent: None,
            activity: None,
            flag: None,
            command: None,
            running_error: None,
            reporting: Reporting::Timer,
            turns: Vec::new(),
            turns_open: false,
            worktree: None,
            matched: None,
        }
    }

    fn project(name: &str, expanded: bool, terminals: Vec<TerminalSnapshot>) -> ProjectSnapshot {
        ProjectSnapshot {
            name: name.to_string(),
            expanded,
            terminals,
            browsers: Vec::new(),
            threads: Vec::new(),
            ports: Vec::new(),
            worktrees: Vec::new(),
            matched: None,
            closed: false,
        }
    }

    fn thread(key: &str, status: ThreadStatus, attention: bool) -> ThreadSnapshot {
        ThreadSnapshot {
            key: key.to_string(),
            title: format!("thread {key}"),
            status,
            attention,
            matched: None,
        }
    }

    fn with_threads(mut project: ProjectSnapshot, threads: Vec<ThreadSnapshot>) -> ProjectSnapshot {
        project.threads = threads;
        project
    }

    fn window(
        projects: Vec<ProjectSnapshot>,
        project: Option<usize>,
        terminal: Option<u64>,
    ) -> RailSnapshot {
        RailSnapshot {
            projects,
            focus: Focus {
                project,
                terminal,
                browser: None,
                worktree: None,
                terminal_focused: false,
                thread: None,
                cursor: None,
            },
            filtering: false,
            inbox: Vec::new(),
            inbox_suggests: false,
            route_suggests: false,
            order: RailOrder::Window,
            held: None,
        }
    }

    fn panel_focused(mut snapshot: RailSnapshot, thread: &str) -> RailSnapshot {
        snapshot.focus.thread = Some(thread.to_string());
        snapshot
    }

    /// Two projects, each with a terminal and threads: `marley` has `a` (running) and `b`, `rusty`
    /// has `c`.
    fn two_projects_with_threads(first_expanded: bool) -> Vec<ProjectSnapshot> {
        vec![
            with_threads(
                project("marley", first_expanded, vec![terminal(1, false)]),
                vec![
                    thread("a", ThreadStatus::Running, false),
                    thread("b", ThreadStatus::Done, false),
                ],
            ),
            with_threads(
                project("rusty", true, vec![terminal(3, false)]),
                vec![thread("c", ThreadStatus::Done, false)],
            ),
        ]
    }

    fn two_projects(first_expanded: bool) -> Vec<ProjectSnapshot> {
        vec![
            project(
                "marley",
                first_expanded,
                vec![terminal(1, false), terminal(2, true)],
            ),
            project("rusty", true, vec![terminal(3, false)]),
        ]
    }

    fn selected_rows(rows: &[Row]) -> usize {
        rows.iter()
            .filter(|row| match row {
                Row::Project(row) => row.selected,
                Row::Terminal(row) => row.selected,
                Row::Browser(row) => row.selected,
                Row::Thread(row) => row.selected,
                Row::Port(row) => row.selected,
                Row::Worktree(row) => row.selected,
            })
            .count()
    }

    #[test]
    fn nothing_is_selected_without_a_listed_displayed_project() {
        assert_eq!(
            selection(&window(two_projects(true), None, Some(1))),
            Selection::None
        );
        assert_eq!(
            selection(&window(two_projects(true), Some(2), None)),
            Selection::None
        );
        assert_eq!(selection(&RailSnapshot::default()), Selection::None);
    }

    #[test]
    fn the_active_terminal_is_selected_when_its_row_is_visible() {
        assert_eq!(
            selection(&window(two_projects(true), Some(0), Some(2))),
            Selection::Terminal(2)
        );
    }

    #[test]
    fn the_project_header_is_selected_otherwise() {
        // No active terminal.
        assert_eq!(
            selection(&window(two_projects(true), Some(0), None)),
            Selection::Project(0)
        );
        // The active terminal belongs to another project's list.
        assert_eq!(
            selection(&window(two_projects(true), Some(0), Some(3))),
            Selection::Project(0)
        );
        // The project is collapsed, so the terminal's row is hidden.
        assert_eq!(
            selection(&window(two_projects(false), Some(0), Some(1))),
            Selection::Project(0)
        );
    }

    #[test]
    fn rows_list_each_header_then_its_terminals_when_expanded() {
        let rows = rail_rows(&window(two_projects(true), Some(1), Some(3)));
        assert_eq!(
            rows,
            vec![
                Row::Project(ProjectRow {
                    index: 0,
                    name: "marley".into(),
                    expanded: true,
                    selected: false,
                    attention: false,
                    highlight: Vec::new(),
                    summary: None,
                }),
                Row::Terminal(TerminalRow {
                    project: 0,
                    id: 1,
                    title: "terminal 1".into(),
                    subtitle: None,
                    bell: false,
                    agent: None,
                    activity: None,
                    flag: None,
                    command: None,
                    running_error: None,
                    turns: Vec::new(),
                    turns_open: false,
                    worktree: None,
                    selected: false,
                    highlight: Vec::new(),
                }),
                Row::Terminal(TerminalRow {
                    project: 0,
                    id: 2,
                    title: "terminal 2".into(),
                    subtitle: None,
                    bell: true,
                    agent: None,
                    activity: None,
                    flag: None,
                    command: None,
                    running_error: None,
                    turns: Vec::new(),
                    turns_open: false,
                    worktree: None,
                    selected: false,
                    highlight: Vec::new(),
                }),
                Row::Project(ProjectRow {
                    index: 1,
                    name: "rusty".into(),
                    expanded: true,
                    selected: false,
                    attention: false,
                    highlight: Vec::new(),
                    summary: None,
                }),
                Row::Terminal(TerminalRow {
                    project: 1,
                    id: 3,
                    title: "terminal 3".into(),
                    subtitle: None,
                    bell: false,
                    agent: None,
                    activity: None,
                    flag: None,
                    command: None,
                    running_error: None,
                    turns: Vec::new(),
                    turns_open: false,
                    worktree: None,
                    selected: true,
                    highlight: Vec::new(),
                }),
            ]
        );
    }

    #[test]
    fn a_collapsed_project_hides_its_terminals_and_carries_their_bell() {
        let rows = rail_rows(&window(two_projects(false), Some(0), None));
        assert_eq!(
            rows,
            vec![
                Row::Project(ProjectRow {
                    index: 0,
                    name: "marley".into(),
                    expanded: false,
                    selected: true,
                    attention: true,
                    highlight: Vec::new(),
                    summary: None,
                }),
                Row::Project(ProjectRow {
                    index: 1,
                    name: "rusty".into(),
                    expanded: true,
                    selected: false,
                    attention: false,
                    highlight: Vec::new(),
                    summary: None,
                }),
                Row::Terminal(TerminalRow {
                    project: 1,
                    id: 3,
                    title: "terminal 3".into(),
                    subtitle: None,
                    bell: false,
                    agent: None,
                    activity: None,
                    flag: None,
                    command: None,
                    running_error: None,
                    turns: Vec::new(),
                    turns_open: false,
                    worktree: None,
                    selected: false,
                    highlight: Vec::new(),
                }),
            ]
        );
        // A collapsed project with no bell raises no attention.
        let quiet = window(
            vec![project("quiet", false, vec![terminal(9, false)])],
            None,
            None,
        );
        assert_eq!(
            rail_rows(&quiet),
            vec![Row::Project(ProjectRow {
                index: 0,
                name: "quiet".into(),
                expanded: false,
                selected: false,
                attention: false,
                highlight: Vec::new(),
                summary: None,
            })]
        );
    }

    #[test]
    fn exactly_one_row_is_selected_whenever_a_listed_project_is_displayed() {
        for expanded in [true, false] {
            for project in [None, Some(0), Some(1), Some(5)] {
                for terminal in [None, Some(1), Some(2), Some(3), Some(42)] {
                    let rows = rail_rows(&window(two_projects(expanded), project, terminal));
                    let expected = usize::from(matches!(project, Some(0 | 1)));
                    assert_eq!(
                        selected_rows(&rows),
                        expected,
                        "expanded {expanded} project {project:?} terminal {terminal:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn attention_is_any_bell_in_any_project() {
        assert!(has_attention(&window(two_projects(true), None, None)));
        assert!(has_attention(&window(two_projects(false), None, None)));
        let quiet = window(
            vec![
                project("a", true, vec![terminal(1, false)]),
                project("b", false, vec![terminal(2, false)]),
            ],
            None,
            None,
        );
        assert!(!has_attention(&quiet));
        let late_bell = window(
            vec![
                project("a", true, vec![terminal(1, false)]),
                project("b", true, vec![terminal(2, false), terminal(3, true)]),
            ],
            None,
            None,
        );
        assert!(has_attention(&late_bell));
        assert!(!has_attention(&RailSnapshot::default()));
    }

    #[test]
    fn each_thread_status_has_its_word() {
        assert_eq!(
            [
                ThreadStatus::Done,
                ThreadStatus::Running,
                ThreadStatus::Waiting,
                ThreadStatus::Error,
            ]
            .map(ThreadStatus::label),
            ["idle", "working", "waiting", "failed"]
        );
    }

    #[test]
    fn a_thread_status_ranks_a_confirmation_over_an_error_over_a_run() {
        for waiting in [false, true] {
            for errored in [false, true] {
                for generating in [false, true] {
                    let expected = if waiting {
                        ThreadStatus::Waiting
                    } else if errored {
                        ThreadStatus::Error
                    } else if generating {
                        ThreadStatus::Running
                    } else {
                        ThreadStatus::Done
                    };
                    assert_eq!(
                        thread_status(waiting, errored, generating),
                        expected,
                        "waiting {waiting} errored {errored} generating {generating}"
                    );
                }
            }
        }
        assert_eq!(ThreadStatus::default(), ThreadStatus::Done);
    }

    #[test]
    fn a_run_that_ends_unseen_lights_the_dot_until_the_thread_is_shown() {
        use ThreadStatus::{Done, Error, Running, Waiting};
        // A run that ends while the thread is not shown lights the dot, cleanly or not.
        assert!(thread_attention(Some(Running), Done, false, false));
        assert!(thread_attention(Some(Running), Error, false, false));
        // Anything else leaves it dark: no run ended.
        assert!(!thread_attention(None, Done, false, false));
        assert!(!thread_attention(Some(Done), Done, false, false));
        assert!(!thread_attention(Some(Running), Running, false, false));
        assert!(!thread_attention(Some(Running), Waiting, false, false));
        assert!(!thread_attention(Some(Waiting), Done, false, false));
        // A lit dot stays lit until the thread is shown, and showing it clears it.
        assert!(thread_attention(Some(Done), Done, false, true));
        assert!(!thread_attention(Some(Done), Done, true, true));
        assert!(!thread_attention(Some(Running), Done, true, false));
    }

    #[test]
    fn threads_follow_their_projects_terminals_when_expanded() {
        let rows = rail_rows(&window(two_projects_with_threads(true), Some(1), None));
        assert_eq!(
            rows,
            vec![
                Row::Project(ProjectRow {
                    index: 0,
                    name: "marley".into(),
                    expanded: true,
                    selected: false,
                    attention: false,
                    highlight: Vec::new(),
                    summary: None,
                }),
                Row::Terminal(TerminalRow {
                    project: 0,
                    id: 1,
                    title: "terminal 1".into(),
                    subtitle: None,
                    bell: false,
                    agent: None,
                    activity: None,
                    flag: None,
                    command: None,
                    running_error: None,
                    turns: Vec::new(),
                    turns_open: false,
                    worktree: None,
                    selected: false,
                    highlight: Vec::new(),
                }),
                Row::Thread(ThreadRow {
                    project: 0,
                    key: "a".into(),
                    title: "thread a".into(),
                    status: ThreadStatus::Running,
                    attention: false,
                    selected: false,
                    highlight: Vec::new(),
                }),
                Row::Thread(ThreadRow {
                    project: 0,
                    key: "b".into(),
                    title: "thread b".into(),
                    status: ThreadStatus::Done,
                    attention: false,
                    selected: false,
                    highlight: Vec::new(),
                }),
                Row::Project(ProjectRow {
                    index: 1,
                    name: "rusty".into(),
                    expanded: true,
                    selected: true,
                    attention: false,
                    highlight: Vec::new(),
                    summary: None,
                }),
                Row::Terminal(TerminalRow {
                    project: 1,
                    id: 3,
                    title: "terminal 3".into(),
                    subtitle: None,
                    bell: false,
                    agent: None,
                    activity: None,
                    flag: None,
                    command: None,
                    running_error: None,
                    turns: Vec::new(),
                    turns_open: false,
                    worktree: None,
                    selected: false,
                    highlight: Vec::new(),
                }),
                Row::Thread(ThreadRow {
                    project: 1,
                    key: "c".into(),
                    title: "thread c".into(),
                    status: ThreadStatus::Done,
                    attention: false,
                    selected: false,
                    highlight: Vec::new(),
                }),
            ]
        );
        // Folded, the project lists neither its terminals nor its threads.
        let folded = rail_rows(&window(two_projects_with_threads(false), None, None));
        assert!(
            !folded
                .iter()
                .any(|row| matches!(row, Row::Thread(row) if row.project == 0)),
            "{folded:?}"
        );
    }

    #[test]
    fn a_folded_header_carries_a_hidden_threads_dot_or_wait() {
        let header_attention = |threads: Vec<ThreadSnapshot>| {
            let snapshot = window(
                vec![with_threads(
                    project("marley", false, vec![terminal(1, false)]),
                    threads,
                )],
                None,
                None,
            );
            matches!(rail_rows(&snapshot).first(), Some(Row::Project(row)) if row.attention)
        };
        assert!(header_attention(vec![thread(
            "a",
            ThreadStatus::Done,
            true
        )]));
        assert!(header_attention(vec![thread(
            "a",
            ThreadStatus::Waiting,
            false
        )]));
        assert!(!header_attention(vec![
            thread("a", ThreadStatus::Running, false),
            thread("b", ThreadStatus::Error, false),
        ]));
        // Expanded, the thread's own row shows it and the header stays plain.
        let expanded = window(
            vec![with_threads(
                project("marley", true, Vec::new()),
                vec![thread("a", ThreadStatus::Waiting, true)],
            )],
            None,
            None,
        );
        assert!(matches!(
            rail_rows(&expanded).first(),
            Some(Row::Project(row)) if !row.attention
        ));
    }

    #[test]
    fn a_threads_dot_or_wait_raises_the_rails_attention() {
        let rail = |threads: Vec<ThreadSnapshot>| {
            window(
                vec![with_threads(project("marley", true, Vec::new()), threads)],
                None,
                None,
            )
        };
        assert!(has_attention(&rail(vec![thread(
            "a",
            ThreadStatus::Done,
            true
        )])));
        assert!(has_attention(&rail(vec![thread(
            "a",
            ThreadStatus::Waiting,
            false
        )])));
        assert!(!has_attention(&rail(vec![
            thread("a", ThreadStatus::Running, false),
            thread("b", ThreadStatus::Error, false),
            thread("c", ThreadStatus::Done, false),
        ])));
    }

    #[test]
    fn the_focused_panels_thread_is_selected_when_its_row_is_visible() {
        // The panel's thread outranks the active terminal.
        let focused = panel_focused(
            window(two_projects_with_threads(true), Some(0), Some(1)),
            "b",
        );
        assert_eq!(selection(&focused), Selection::Thread("b".into()));
        let rows = rail_rows(&focused);
        assert!(
            rows.iter()
                .any(|row| matches!(row, Row::Thread(row) if row.key == "b" && row.selected))
        );
        assert_eq!(selected_rows(&rows), 1);
        // A thread another project lists falls back to the terminal, then to the header.
        assert_eq!(
            selection(&panel_focused(
                window(two_projects_with_threads(true), Some(0), Some(1)),
                "c"
            )),
            Selection::Terminal(1)
        );
        assert_eq!(
            selection(&panel_focused(
                window(two_projects_with_threads(true), Some(0), None),
                "c"
            )),
            Selection::Project(0)
        );
        // Folded, the thread's row is hidden, so the header is selected.
        assert_eq!(
            selection(&panel_focused(
                window(two_projects_with_threads(false), Some(0), Some(1)),
                "a"
            )),
            Selection::Project(0)
        );
        // No displayed project, nothing selected.
        assert_eq!(
            selection(&panel_focused(
                window(two_projects_with_threads(true), None, None),
                "a"
            )),
            Selection::None
        );
    }

    #[test]
    fn exactly_one_row_is_selected_with_threads_listed() {
        for expanded in [true, false] {
            for project in [None, Some(0), Some(1), Some(5)] {
                for terminal in [None, Some(1), Some(3), Some(42)] {
                    for focused in [None, Some("a"), Some("b"), Some("c"), Some("zz")] {
                        let mut snapshot =
                            window(two_projects_with_threads(expanded), project, terminal);
                        snapshot.focus.thread = focused.map(str::to_string);
                        let expected = usize::from(matches!(project, Some(0 | 1)));
                        assert_eq!(
                            selected_rows(&rail_rows(&snapshot)),
                            expected,
                            "expanded {expanded} project {project:?} terminal {terminal:?} thread {focused:?}"
                        );
                    }
                }
            }
        }
    }

    fn with_cursor(mut snapshot: RailSnapshot, cursor: Selection) -> RailSnapshot {
        snapshot.focus.cursor = Some(cursor);
        snapshot
    }

    #[test]
    fn the_keyboards_row_is_selected_while_it_is_shown() {
        let snapshot = with_cursor(
            window(two_projects_with_threads(true), Some(0), Some(1)),
            Selection::Thread("b".to_string()),
        );
        assert_eq!(selection(&snapshot), Selection::Thread("b".to_string()));
        assert_eq!(selected_rows(&rail_rows(&snapshot)), 1);
        // A cursor on a folded project's row is not shown, so the usual rule holds.
        let folded = with_cursor(
            window(two_projects_with_threads(false), Some(1), Some(3)),
            Selection::Thread("b".to_string()),
        );
        assert_eq!(selection(&folded), Selection::Terminal(3));
    }

    #[test]
    fn step_walks_the_shown_rows_and_stays_at_the_ends() {
        let mut snapshot = window(two_projects_with_threads(true), Some(0), None);
        let order = [
            Selection::Project(0),
            Selection::Terminal(1),
            Selection::Thread("a".to_string()),
            Selection::Thread("b".to_string()),
            Selection::Project(1),
            Selection::Terminal(3),
            Selection::Thread("c".to_string()),
        ];
        for pair in order.windows(2) {
            snapshot.focus.cursor = Some(pair[0].clone());
            assert_eq!(step(&snapshot, true), pair[1]);
            snapshot.focus.cursor = Some(pair[1].clone());
            assert_eq!(step(&snapshot, false), pair[0]);
        }
        snapshot.focus.cursor = Some(order[6].clone());
        assert_eq!(step(&snapshot, true), order[6], "stays on the last");
        snapshot.focus.cursor = Some(order[0].clone());
        assert_eq!(step(&snapshot, false), order[0], "stays on the first");
        assert_eq!(first_row(&snapshot), order[0]);
        assert_eq!(last_row(&snapshot), order[6]);
    }

    #[test]
    fn step_starts_at_an_end_when_nothing_is_selected() {
        let nothing = window(two_projects(false), None, None);
        assert_eq!(step(&nothing, true), Selection::Project(0));
        assert_eq!(step(&nothing, false), Selection::Terminal(3));
        let empty = RailSnapshot::default();
        assert_eq!(step(&empty, true), Selection::None);
        assert_eq!(first_row(&empty), Selection::None);
        assert_eq!(last_row(&empty), Selection::None);
    }

    /// `two_projects_with_threads`, then `zed` with terminal 5 and no thread.
    fn three_projects(first_expanded: bool) -> Vec<ProjectSnapshot> {
        let mut projects = two_projects_with_threads(first_expanded);
        projects.push(project("zed", true, vec![terminal(5, false)]));
        projects
    }

    #[test]
    fn next_and_previous_project_go_round_the_shown_headers() {
        let mut snapshot = window(three_projects(true), Some(0), None);
        let headers = [
            Selection::Project(0),
            Selection::Project(1),
            Selection::Project(2),
        ];
        for (at, header) in headers.iter().enumerate() {
            snapshot.focus.cursor = Some(header.clone());
            assert_eq!(cycle_project(&snapshot, true), headers[(at + 1) % 3]);
            assert_eq!(cycle_project(&snapshot, false), headers[(at + 2) % 3]);
        }
        // A row goes from its project's header, so going back leaves its project.
        snapshot.focus.cursor = Some(Selection::Thread("b".to_string()));
        assert_eq!(cycle_project(&snapshot, true), Selection::Project(1));
        assert_eq!(cycle_project(&snapshot, false), Selection::Project(2));
    }

    #[test]
    fn next_and_previous_thread_go_round_the_terminals_and_threads() {
        let mut snapshot = window(three_projects(true), Some(0), None);
        let rows = [
            Selection::Terminal(1),
            Selection::Thread("a".to_string()),
            Selection::Thread("b".to_string()),
            Selection::Terminal(3),
            Selection::Thread("c".to_string()),
            Selection::Terminal(5),
        ];
        for (at, row) in rows.iter().enumerate() {
            snapshot.focus.cursor = Some(row.clone());
            assert_eq!(cycle_row(&snapshot, true), rows[(at + 1) % 6]);
            assert_eq!(cycle_row(&snapshot, false), rows[(at + 5) % 6]);
        }
        // From a header: the first row under it going forward, the last row above it going
        // back, and from the first header round to the last row.
        snapshot.focus.cursor = Some(Selection::Project(1));
        assert_eq!(cycle_row(&snapshot, true), Selection::Terminal(3));
        assert_eq!(
            cycle_row(&snapshot, false),
            Selection::Thread("b".to_string())
        );
        snapshot.focus.cursor = Some(Selection::Project(0));
        assert_eq!(cycle_row(&snapshot, false), Selection::Terminal(5));
    }

    #[test]
    fn cycling_passes_over_what_the_fold_and_the_filter_hide() {
        // `marley` is folded: its rows are passed over, and its header is still reached.
        let folded = with_cursor(
            window(three_projects(false), Some(2), Some(5)),
            Selection::Terminal(5),
        );
        assert_eq!(cycle_row(&folded, true), Selection::Terminal(3));
        assert_eq!(cycle_project(&folded, true), Selection::Project(0));
        // The filter shows only the threads, and so hides `zed`, which has none.
        let threads = with_cursor(
            filtered(window(three_projects(true), Some(0), None), "thread"),
            Selection::Thread("c".to_string()),
        );
        assert_eq!(
            cycle_row(&threads, true),
            Selection::Thread("a".to_string())
        );
        assert_eq!(cycle_project(&threads, true), Selection::Project(0));
    }

    #[test]
    fn cycling_from_nothing_starts_at_an_end_and_finds_nothing_in_an_empty_rail() {
        let nothing = window(three_projects(true), None, None);
        assert_eq!(cycle_project(&nothing, true), Selection::Project(0));
        assert_eq!(cycle_project(&nothing, false), Selection::Project(2));
        assert_eq!(cycle_row(&nothing, true), Selection::Terminal(1));
        assert_eq!(cycle_row(&nothing, false), Selection::Terminal(5));
        // A lone project or row reaches itself; a rail of headers has no row to reach.
        let lone = window(
            vec![project("marley", true, vec![terminal(1, false)])],
            Some(0),
            Some(1),
        );
        assert_eq!(cycle_project(&lone, true), Selection::Project(0));
        assert_eq!(cycle_row(&lone, true), Selection::Terminal(1));
        let headers = window(vec![project("marley", true, Vec::new())], Some(0), None);
        assert_eq!(cycle_row(&headers, false), Selection::None);
        let empty = RailSnapshot::default();
        assert_eq!(cycle_project(&empty, true), Selection::None);
        assert_eq!(cycle_row(&empty, true), Selection::None);
    }

    #[test]
    fn a_rows_parent_is_its_projects_header() {
        let snapshot = window(two_projects_with_threads(true), Some(0), None);
        assert_eq!(
            parent(&snapshot, &Selection::Terminal(3)),
            Selection::Project(1)
        );
        assert_eq!(
            parent(&snapshot, &Selection::Thread("a".to_string())),
            Selection::Project(0)
        );
        assert_eq!(
            parent(&snapshot, &Selection::Project(1)),
            Selection::Project(1)
        );
        assert_eq!(parent(&snapshot, &Selection::None), Selection::None);
        assert_eq!(parent(&snapshot, &Selection::Terminal(99)), Selection::None);
    }

    /// The snapshot as the rail builds it with `query` in the filter: each name and title that
    /// contains it matched at the characters it covers.
    fn filtered(mut snapshot: RailSnapshot, query: &str) -> RailSnapshot {
        let matched = |text: &str| {
            text.find(query).map(|start| {
                text[start..start + query.len()]
                    .char_indices()
                    .map(|(offset, _)| start + offset)
                    .collect()
            })
        };
        snapshot.filtering = true;
        for project in &mut snapshot.projects {
            project.matched = matched(&project.name);
            for terminal in &mut project.terminals {
                terminal.matched = matched(&terminal.title);
            }
            for thread in &mut project.threads {
                thread.matched = matched(&thread.title);
            }
        }
        snapshot
    }

    /// The rows as lines: a header's name, or a terminal's or thread's title, indented.
    fn outline(snapshot: &RailSnapshot) -> Vec<String> {
        rail_rows(snapshot)
            .into_iter()
            .map(|row| match row {
                Row::Project(row) => row.name,
                Row::Terminal(row) => format!("  {}", row.title),
                Row::Browser(row) => format!("  {}", row.title),
                Row::Thread(row) => format!("  {}", row.title),
                Row::Port(row) => format!("  {}", row.title),
                Row::Worktree(row) => format!("  {}", row.name),
            })
            .collect()
    }

    #[test]
    fn a_filter_shows_every_row_of_a_project_whose_name_matched() {
        // `marley` is folded, and the filter shows its rows anyway.
        let snapshot = filtered(
            window(two_projects_with_threads(false), Some(0), None),
            "marl",
        );
        assert_eq!(
            outline(&snapshot),
            ["marley", "  terminal 1", "  thread a", "  thread b"]
        );
    }

    #[test]
    fn a_filter_shows_a_project_for_the_rows_under_it_that_matched() {
        let snapshot = window(two_projects_with_threads(true), Some(0), None);
        assert_eq!(
            outline(&filtered(snapshot.clone(), "thread c")),
            ["rusty", "  thread c"]
        );
        assert_eq!(
            outline(&filtered(snapshot, "terminal 1")),
            ["marley", "  terminal 1"]
        );
    }

    #[test]
    fn a_filter_that_matches_nothing_shows_and_selects_nothing() {
        let snapshot = filtered(
            with_cursor(
                window(two_projects(true), Some(0), Some(1)),
                Selection::Terminal(1),
            ),
            "zzz",
        );
        assert!(rail_rows(&snapshot).is_empty());
        assert_eq!(selection(&snapshot), Selection::None);
        assert_eq!(first_match(&snapshot), Selection::None);
        assert_eq!(step(&snapshot, true), Selection::None);
    }

    #[test]
    fn filtered_rows_carry_the_characters_that_matched() {
        let snapshot = window(two_projects_with_threads(true), Some(0), None);
        let highlights = |query: &str| -> Vec<(String, Vec<usize>)> {
            rail_rows(&filtered(snapshot.clone(), query))
                .into_iter()
                .map(|row| match row {
                    Row::Project(row) => (row.name, row.highlight),
                    Row::Terminal(row) => (row.title, row.highlight),
                    Row::Browser(row) => (row.title, row.highlight),
                    Row::Thread(row) => (row.title, row.highlight),
                    Row::Port(row) => (row.title, row.highlight),
                    Row::Worktree(row) => (row.name, row.highlight),
                })
                .collect()
        };
        assert_eq!(
            highlights("ar"),
            [
                ("marley".to_string(), vec![1, 2]),
                ("terminal 1".to_string(), vec![]),
                ("thread a".to_string(), vec![]),
                ("thread b".to_string(), vec![]),
            ]
        );
        assert_eq!(
            highlights("l 1"),
            [
                ("marley".to_string(), vec![]),
                ("terminal 1".to_string(), vec![7, 8, 9]),
            ]
        );
        assert_eq!(
            highlights("d b"),
            [
                ("marley".to_string(), vec![]),
                ("thread b".to_string(), vec![5, 6, 7]),
            ]
        );
        // Without the filter nothing is highlighted, whatever the snapshot says matched.
        let mut unfiltered = filtered(snapshot, "ar");
        unfiltered.filtering = false;
        for row in rail_rows(&unfiltered) {
            let highlight = match row {
                Row::Project(row) => row.highlight,
                Row::Terminal(row) => row.highlight,
                Row::Browser(row) => row.highlight,
                Row::Thread(row) => row.highlight,
                Row::Port(row) => row.highlight,
                Row::Worktree(row) => row.highlight,
            };
            assert!(highlight.is_empty());
        }
    }

    #[test]
    fn a_shown_header_carries_the_attention_of_the_rows_the_filter_hides() {
        // `marley`'s terminal 2 rang its bell.
        let snapshot = window(two_projects(true), None, None);
        let attention = |query: &str| -> Vec<bool> {
            rail_rows(&filtered(snapshot.clone(), query))
                .into_iter()
                .filter_map(|row| match row {
                    Row::Project(row) => Some(row.attention),
                    Row::Terminal(_)
                    | Row::Browser(_)
                    | Row::Thread(_)
                    | Row::Port(_)
                    | Row::Worktree(_) => None,
                })
                .collect()
        };
        assert_eq!(attention("terminal 1"), [true], "the bell's row is hidden");
        assert_eq!(attention("marley"), [false], "the bell's row shows");
        // A thread waiting on a confirmation counts too.
        let waiting = window(
            vec![with_threads(
                project("waiting", true, vec![terminal(1, false)]),
                vec![thread("w", ThreadStatus::Waiting, false)],
            )],
            None,
            None,
        );
        assert!(matches!(
            rail_rows(&filtered(waiting, "terminal")).first(),
            Some(Row::Project(ProjectRow {
                attention: true,
                ..
            }))
        ));
    }

    #[test]
    fn first_match_is_the_first_shown_row_that_matched() {
        let snapshot = window(two_projects_with_threads(true), Some(1), Some(3));
        assert_eq!(
            first_match(&filtered(snapshot.clone(), "rusty")),
            Selection::Project(1)
        );
        // `marley` shows for its threads, which matched, while its header did not.
        assert_eq!(
            first_match(&filtered(snapshot.clone(), "thread")),
            Selection::Thread("a".to_string())
        );
        assert_eq!(
            first_match(&filtered(snapshot.clone(), "terminal 3")),
            Selection::Terminal(3)
        );
        assert_eq!(first_match(&snapshot), Selection::None, "no filter");
    }

    #[test]
    fn the_selection_and_the_keyboard_keep_to_the_filtered_rows() {
        // The window shows `rusty`'s terminal 3.
        let snapshot = window(two_projects_with_threads(true), Some(1), Some(3));
        // The filter hides the terminal, so its project's header is the selected row.
        let threads = filtered(snapshot.clone(), "thread");
        assert_eq!(selection(&threads), Selection::Project(1));
        assert_eq!(
            shown(&threads),
            [
                Selection::Project(0),
                Selection::Thread("a".to_string()),
                Selection::Thread("b".to_string()),
                Selection::Project(1),
                Selection::Thread("c".to_string()),
            ]
        );
        // A focused panel's thread gives way to the terminal when the filter hides it.
        let panel = filtered(panel_focused(snapshot.clone(), "c"), "terminal");
        assert_eq!(selection(&panel), Selection::Terminal(3));
        // The filter hides the project too: nothing is selected.
        let marley = filtered(snapshot, "marley");
        assert_eq!(selection(&marley), Selection::None);
        // The keyboard walks only what shows, and a cursor on a hidden row gives way.
        let walked = with_cursor(threads, Selection::Thread("b".to_string()));
        assert_eq!(step(&walked, true), Selection::Project(1));
        let hidden = with_cursor(marley, Selection::Terminal(3));
        assert_eq!(step(&hidden, true), Selection::Project(0));
    }

    fn terminal_focused(mut snapshot: RailSnapshot) -> RailSnapshot {
        snapshot.focus.terminal_focused = true;
        snapshot
    }

    #[test]
    fn the_window_row_is_the_focused_panels_thread_else_the_focused_terminal() {
        let unfocused = window(two_projects_with_threads(true), Some(1), Some(3));
        assert_eq!(
            window_row(&unfocused),
            None,
            "the terminal does not hold focus"
        );
        let snapshot = terminal_focused(unfocused);
        assert_eq!(window_row(&snapshot), Some(Selection::Terminal(3)));
        assert_eq!(
            window_row(&panel_focused(snapshot.clone(), "c")),
            Some(Selection::Thread("c".to_string()))
        );
        // A thread or terminal the displayed project does not list is not the window's row.
        assert_eq!(
            window_row(&panel_focused(snapshot, "a")),
            Some(Selection::Terminal(3))
        );
        assert_eq!(
            window_row(&terminal_focused(window(
                two_projects(true),
                Some(1),
                Some(1)
            ))),
            None
        );
        assert_eq!(
            window_row(&terminal_focused(window(two_projects(true), None, Some(1)))),
            None
        );
        // The fold and the filter do not change what the window shows.
        let folded = terminal_focused(window(two_projects_with_threads(false), Some(0), Some(1)));
        assert_eq!(
            window_row(&filtered(folded, "zzz")),
            Some(Selection::Terminal(1))
        );
    }

    #[test]
    fn the_switcher_lists_the_recently_shown_first_then_the_rails_order() {
        let snapshot = filtered(
            window(two_projects_with_threads(false), Some(1), Some(3)),
            "zzz",
        );
        let order = |shown: &[Selection]| -> Vec<Selection> {
            switcher_rows(&snapshot, |row| {
                shown
                    .iter()
                    .position(|shown| shown == row)
                    .map(|at| at as u64)
            })
            .iter()
            .map(SwitcherRow::selection)
            .collect()
        };
        // Nothing shown yet: every terminal and thread in the rail's order, whatever the fold
        // and the filter, and no header.
        let rail_order = vec![
            Selection::Terminal(1),
            Selection::Thread("a".to_string()),
            Selection::Thread("b".to_string()),
            Selection::Terminal(3),
            Selection::Thread("c".to_string()),
        ];
        assert_eq!(order(&[]), rail_order);
        let unmarked = switcher_rows(&snapshot, |_| None);
        assert!(unmarked.iter().all(|row| match row {
            SwitcherRow::Terminal(row) => !row.selected && row.highlight.is_empty(),
            SwitcherRow::Thread(row) => !row.selected && row.highlight.is_empty(),
        }));
        // The window showed terminal 3, then thread b: b is the most recent.
        assert_eq!(
            order(&[Selection::Terminal(3), Selection::Thread("b".to_string())]),
            [
                Selection::Thread("b".to_string()),
                Selection::Terminal(3),
                Selection::Terminal(1),
                Selection::Thread("a".to_string()),
                Selection::Thread("c".to_string()),
            ]
        );
    }

    #[test]
    fn an_agent_terminals_row_carries_its_agent() {
        let agent = TerminalAgent {
            kind: AgentKind::Claude,
            status: AgentStatus::Waiting,
            mark: None,
        };
        let mut claude = terminal(7, false);
        claude.agent = Some(agent);
        let snapshot = window(
            vec![project("marley", true, vec![claude])],
            Some(0),
            Some(7),
        );
        let rows = rail_rows(&snapshot);
        assert!(
            matches!(&rows[..], [Row::Project(_), Row::Terminal(row)] if row.agent == Some(agent) && row.selected),
            "{rows:?}"
        );
        // An agent's quiet spell raises no attention; only a bell does.
        assert!(!has_attention(&snapshot));
    }

    #[test]
    fn the_working_directory_reads_relative_to_the_project_or_home() {
        let root = PathBuf::from("/srv/stacks/marley_ide");
        let home = PathBuf::from("/home/user");
        let label = |path: Option<&str>, root: Option<&Path>, home: Option<&Path>| {
            working_directory_label(path.map(Path::new), root, home)
        };
        assert_eq!(label(None, Some(&root), Some(&home)), None);
        assert_eq!(label(Some(""), Some(&root), Some(&home)), None);
        assert_eq!(
            label(Some("/srv/stacks/marley_ide"), Some(&root), Some(&home)),
            None
        );
        assert_eq!(
            label(
                Some("/srv/stacks/marley_ide/crates/marley_rail"),
                Some(&root),
                Some(&home)
            ),
            Some("crates/marley_rail".to_string())
        );
        assert_eq!(
            label(Some("/home/user/notes"), Some(&root), Some(&home)),
            Some("~/notes".to_string())
        );
        assert_eq!(
            label(Some("/home/user"), Some(&root), Some(&home)),
            Some("~".to_string())
        );
        assert_eq!(
            label(Some("/tmp/scratch"), Some(&root), Some(&home)),
            Some("/tmp/scratch".to_string())
        );
        assert_eq!(
            label(Some("/home/user/x"), None, Some(&home)),
            Some("~/x".to_string())
        );
        assert_eq!(
            label(Some("/home/user/x"), None, None),
            Some("/home/user/x".to_string())
        );
    }
}
