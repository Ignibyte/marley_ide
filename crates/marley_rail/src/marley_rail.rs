//! The Marley rail's row model: which rows the rail shows for a window, in what order, and which
//! single row is selected.
//!
//! Pure and gpui-free, so every decision is unit-tested in milliseconds. The gpui side
//! (`marley_workbench`) builds a [`RailSnapshot`] from the live window, with the keyboard's row
//! and what the filter matched, and renders the [`Row`]s this crate returns; it decides no
//! ordering, visibility or selection of its own.

use std::path::Path;

use marley_agent::{AgentKind, AgentStatus};

/// One project group as the rail sees it: Zed's project group flattened to what a row shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSnapshot {
    /// The group's display name (disambiguated against the window's other groups).
    pub name: String,
    /// Whether the group is expanded (Zed keeps this per group in the `MultiWorkspace`).
    pub expanded: bool,
    /// The group's center terminals, in the order the rail lists them.
    pub terminals: Vec<TerminalSnapshot>,
    /// The group's agent threads, in the order the rail lists them (newest first).
    pub threads: Vec<ThreadSnapshot>,
    /// Where the filter matched the name, as the byte offsets of the matched characters; `None`
    /// when it did not. Read only while [`RailSnapshot::filtering`].
    pub matched: Option<Vec<usize>>,
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
    /// Where the filter matched the title, as for [`ProjectSnapshot::matched`].
    pub matched: Option<Vec<usize>>,
}

/// An agent CLI running in a terminal, and what it is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalAgent {
    /// Which agent it is.
    pub kind: AgentKind,
    /// Whether it is working or waiting on the user.
    pub status: AgentStatus,
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
    /// Whether that terminal holds the window's focus.
    pub terminal_focused: bool,
    /// The thread the displayed workspace's Agent Panel shows, while the panel holds focus.
    pub thread: Option<String>,
    /// The row the keyboard is on, while the rail holds focus.
    pub cursor: Option<Selection>,
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
    /// A thread row, by the thread's key.
    Thread(String),
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

/// One row of the rail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Row {
    /// A project group's header.
    Project(ProjectRow),
    /// A terminal under its project.
    Terminal(TerminalRow),
    /// An agent thread under its project, after the project's terminals.
    Thread(ThreadRow),
}

/// A row the switcher lists: a terminal or a thread, never a header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwitcherRow {
    /// A terminal, unselected and unhighlighted.
    Terminal(TerminalRow),
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
    [thread, terminal, Some(Selection::Project(index))]
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
    Thread(usize, &'a ThreadSnapshot),
}

impl<'a> Shown<'a> {
    const fn project(self) -> usize {
        match self {
            Self::Project(index, _) | Self::Terminal(index, _) | Self::Thread(index, _) => index,
        }
    }

    fn selection(self) -> Selection {
        match self {
            Self::Project(index, _) => Selection::Project(index),
            Self::Terminal(_, terminal) => Selection::Terminal(terminal.id),
            Self::Thread(_, thread) => Selection::Thread(thread.key.clone()),
        }
    }

    /// Where the filter matched the row's own name or title.
    fn matched(self) -> Option<&'a [usize]> {
        match self {
            Self::Project(_, project) => project.matched.as_deref(),
            Self::Terminal(_, terminal) => terminal.matched.as_deref(),
            Self::Thread(_, thread) => thread.matched.as_deref(),
        }
    }
}

/// Every row the rail shows, in its order: each shown project's header, then the terminals and
/// the threads shown under it. The rows, the selection and the keyboard all read this one walk.
fn walk(snapshot: &RailSnapshot) -> Vec<Shown<'_>> {
    let mut rows = Vec::new();
    for (index, project) in snapshot.projects.iter().enumerate() {
        let terminals = project
            .terminals
            .iter()
            .filter(|terminal| row_shows(snapshot, project, terminal.matched.as_deref()))
            .map(|terminal| Shown::Terminal(index, terminal));
        let threads = project
            .threads
            .iter()
            .filter(|thread| row_shows(snapshot, project, thread.matched.as_deref()))
            .map(|thread| Shown::Thread(index, thread));
        let under: Vec<Shown<'_>> = terminals.chain(threads).collect();
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
    let project = parent(snapshot, &selection(snapshot));
    cycle(snapshot, &project, forward, |row| {
        matches!(row, Selection::Project(_))
    })
}

/// Zed's Next and Previous Thread in the rail, which reach terminals and threads alike.
///
/// The shown terminal or thread row after the selected row, or before it, passing over project
/// headers and wrapping at the ends. With nothing selected it is the first such row going
/// forward and the last going back.
#[must_use]
pub fn cycle_row(snapshot: &RailSnapshot, forward: bool) -> Selection {
    cycle(snapshot, &selection(snapshot), forward, |row| {
        matches!(row, Selection::Terminal(_) | Selection::Thread(_))
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

/// The project header a row sits under; a header is its own.
#[must_use]
pub fn parent(snapshot: &RailSnapshot, selection: &Selection) -> Selection {
    let owner = match selection {
        Selection::None | Selection::Project(_) => return selection.clone(),
        Selection::Terminal(id) => snapshot
            .projects
            .iter()
            .position(|project| project.terminals.iter().any(|terminal| terminal.id == *id)),
        Selection::Thread(key) => snapshot
            .projects
            .iter()
            .position(|project| project.threads.iter().any(|thread| thread.key == *key)),
    };
    owner.map_or(Selection::None, Selection::Project)
}

/// The rows, in display order.
///
/// Each shown project's header comes first, then the terminals and the threads shown under it.
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
            }),
            Shown::Terminal(index, terminal) => Row::Terminal(TerminalRow {
                project: index,
                id: terminal.id,
                title: terminal.title.clone(),
                subtitle: terminal.subtitle.clone(),
                bell: terminal.bell,
                agent: terminal.agent,
                selected: selected == Selection::Terminal(terminal.id),
                highlight: highlight(terminal.matched.as_deref()),
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
            SwitcherRow::Terminal(TerminalRow {
                project: index,
                id: terminal.id,
                title: terminal.title.clone(),
                subtitle: terminal.subtitle.clone(),
                bell: terminal.bell,
                agent: terminal.agent,
                selected: false,
                highlight: Vec::new(),
            })
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
        .any(|terminal| terminal.bell && hidden(terminal.matched.as_deref()))
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
            matched: None,
        }
    }

    fn project(name: &str, expanded: bool, terminals: Vec<TerminalSnapshot>) -> ProjectSnapshot {
        ProjectSnapshot {
            name: name.to_string(),
            expanded,
            terminals,
            threads: Vec::new(),
            matched: None,
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
                terminal_focused: false,
                thread: None,
                cursor: None,
            },
            filtering: false,
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
                Row::Thread(row) => row.selected,
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
                }),
                Row::Terminal(TerminalRow {
                    project: 0,
                    id: 1,
                    title: "terminal 1".into(),
                    subtitle: None,
                    bell: false,
                    agent: None,
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
                }),
                Row::Terminal(TerminalRow {
                    project: 1,
                    id: 3,
                    title: "terminal 3".into(),
                    subtitle: None,
                    bell: false,
                    agent: None,
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
                }),
                Row::Project(ProjectRow {
                    index: 1,
                    name: "rusty".into(),
                    expanded: true,
                    selected: false,
                    attention: false,
                    highlight: Vec::new(),
                }),
                Row::Terminal(TerminalRow {
                    project: 1,
                    id: 3,
                    title: "terminal 3".into(),
                    subtitle: None,
                    bell: false,
                    agent: None,
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
                }),
                Row::Terminal(TerminalRow {
                    project: 0,
                    id: 1,
                    title: "terminal 1".into(),
                    subtitle: None,
                    bell: false,
                    agent: None,
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
                }),
                Row::Terminal(TerminalRow {
                    project: 1,
                    id: 3,
                    title: "terminal 3".into(),
                    subtitle: None,
                    bell: false,
                    agent: None,
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
                Row::Thread(row) => format!("  {}", row.title),
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
                    Row::Thread(row) => (row.title, row.highlight),
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
                Row::Thread(row) => row.highlight,
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
                    Row::Terminal(_) | Row::Thread(_) => None,
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
