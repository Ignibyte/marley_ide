//! The Marley rail's row model: which rows the rail shows for a window, in what order, and which
//! single row is selected.
//!
//! Pure and gpui-free, so every decision is unit-tested in milliseconds. The gpui side
//! (`marley_workbench`) builds a [`RailSnapshot`] from the live window and renders the [`Row`]s
//! this crate returns; it keeps no selection or ordering state of its own.

use std::path::Path;

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
    /// The thread the displayed workspace's Agent Panel shows, while the panel holds focus.
    pub thread: Option<String>,
}

/// The window, as the rail sees it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RailSnapshot {
    /// The project groups in the window's order.
    pub projects: Vec<ProjectSnapshot>,
    /// What the window shows.
    pub focus: Focus,
}

/// The single selected row.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Whether a row hidden by the collapse needs the user: a terminal's bell, or a thread's
    /// attention dot or wait for a confirmation.
    pub attention: bool,
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
    /// Whether this is the selected row.
    pub selected: bool,
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

/// The one selected row.
///
/// While the displayed workspace's Agent Panel holds focus, the thread it shows wins when that
/// thread's row is visible. Otherwise the active terminal wins when its row is visible, and
/// otherwise the displayed workspace's project header does, so a window showing a project always
/// has exactly one selected row and a stale focus never selects a row that is not there.
#[must_use]
pub fn selection(snapshot: &RailSnapshot) -> Selection {
    let Some((index, project)) = snapshot
        .focus
        .project
        .and_then(|index| Some((index, snapshot.projects.get(index)?)))
    else {
        return Selection::None;
    };
    if project.expanded {
        if let Some(key) = &snapshot.focus.thread
            && project.threads.iter().any(|thread| thread.key == *key)
        {
            return Selection::Thread(key.clone());
        }
        if let Some(id) = snapshot.focus.terminal
            && project.terminals.iter().any(|terminal| terminal.id == id)
        {
            return Selection::Terminal(id);
        }
    }
    Selection::Project(index)
}

/// The rows, in display order: each project's header, then its terminals and its threads when it
/// is expanded.
#[must_use]
pub fn rail_rows(snapshot: &RailSnapshot) -> Vec<Row> {
    let selected = selection(snapshot);
    let mut rows = Vec::new();
    for (index, project) in snapshot.projects.iter().enumerate() {
        rows.push(Row::Project(ProjectRow {
            index,
            name: project.name.clone(),
            expanded: project.expanded,
            selected: selected == Selection::Project(index),
            attention: !project.expanded && project_needs_the_user(project),
        }));
        if project.expanded {
            rows.extend(project.terminals.iter().map(|terminal| {
                Row::Terminal(TerminalRow {
                    project: index,
                    id: terminal.id,
                    title: terminal.title.clone(),
                    subtitle: terminal.subtitle.clone(),
                    bell: terminal.bell,
                    selected: selected == Selection::Terminal(terminal.id),
                })
            }));
            rows.extend(project.threads.iter().map(|thread| {
                Row::Thread(ThreadRow {
                    project: index,
                    key: thread.key.clone(),
                    title: thread.title.clone(),
                    status: thread.status,
                    attention: thread.attention,
                    selected: matches!(&selected, Selection::Thread(key) if *key == thread.key),
                })
            }));
        }
    }
    rows
}

/// Whether anything in the project needs the user: a terminal's bell, or a thread's attention
/// dot or wait for a confirmation.
fn project_needs_the_user(project: &ProjectSnapshot) -> bool {
    project.terminals.iter().any(|terminal| terminal.bell)
        || project
            .threads
            .iter()
            .any(|thread| thread.attention || thread.status == ThreadStatus::Waiting)
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
        }
    }

    fn project(name: &str, expanded: bool, terminals: Vec<TerminalSnapshot>) -> ProjectSnapshot {
        ProjectSnapshot {
            name: name.to_string(),
            expanded,
            terminals,
            threads: Vec::new(),
        }
    }

    fn thread(key: &str, status: ThreadStatus, attention: bool) -> ThreadSnapshot {
        ThreadSnapshot {
            key: key.to_string(),
            title: format!("thread {key}"),
            status,
            attention,
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
                thread: None,
            },
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
                }),
                Row::Terminal(TerminalRow {
                    project: 0,
                    id: 1,
                    title: "terminal 1".into(),
                    subtitle: None,
                    bell: false,
                    selected: false,
                }),
                Row::Terminal(TerminalRow {
                    project: 0,
                    id: 2,
                    title: "terminal 2".into(),
                    subtitle: None,
                    bell: true,
                    selected: false,
                }),
                Row::Project(ProjectRow {
                    index: 1,
                    name: "rusty".into(),
                    expanded: true,
                    selected: false,
                    attention: false,
                }),
                Row::Terminal(TerminalRow {
                    project: 1,
                    id: 3,
                    title: "terminal 3".into(),
                    subtitle: None,
                    bell: false,
                    selected: true,
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
                }),
                Row::Project(ProjectRow {
                    index: 1,
                    name: "rusty".into(),
                    expanded: true,
                    selected: false,
                    attention: false,
                }),
                Row::Terminal(TerminalRow {
                    project: 1,
                    id: 3,
                    title: "terminal 3".into(),
                    subtitle: None,
                    bell: false,
                    selected: false,
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
                }),
                Row::Terminal(TerminalRow {
                    project: 0,
                    id: 1,
                    title: "terminal 1".into(),
                    subtitle: None,
                    bell: false,
                    selected: false,
                }),
                Row::Thread(ThreadRow {
                    project: 0,
                    key: "a".into(),
                    title: "thread a".into(),
                    status: ThreadStatus::Running,
                    attention: false,
                    selected: false,
                }),
                Row::Thread(ThreadRow {
                    project: 0,
                    key: "b".into(),
                    title: "thread b".into(),
                    status: ThreadStatus::Done,
                    attention: false,
                    selected: false,
                }),
                Row::Project(ProjectRow {
                    index: 1,
                    name: "rusty".into(),
                    expanded: true,
                    selected: true,
                    attention: false,
                }),
                Row::Terminal(TerminalRow {
                    project: 1,
                    id: 3,
                    title: "terminal 3".into(),
                    subtitle: None,
                    bell: false,
                    selected: false,
                }),
                Row::Thread(ThreadRow {
                    project: 1,
                    key: "c".into(),
                    title: "thread c".into(),
                    status: ThreadStatus::Done,
                    attention: false,
                    selected: false,
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
