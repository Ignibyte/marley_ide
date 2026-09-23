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

/// What the window shows right now; the one selected row follows it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Focus {
    /// The index of the displayed workspace's project group, if the rail lists it.
    pub project: Option<usize>,
    /// The displayed workspace's active center item, when that item is a terminal.
    pub terminal: Option<u64>,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    /// Nothing to select: the window shows no project the rail lists.
    None,
    /// A project header, by its index.
    Project(usize),
    /// A terminal row, by the terminal's id.
    Terminal(u64),
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
    /// Whether a terminal hidden by the collapse rang its bell.
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

/// One row of the rail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Row {
    /// A project group's header.
    Project(ProjectRow),
    /// A terminal under its project.
    Terminal(TerminalRow),
}

/// The one selected row.
///
/// The displayed workspace's active terminal wins when its row is visible; otherwise the displayed
/// workspace's project header does, so a window showing a project always has exactly one selected
/// row and a stale focus never selects a row that is not there.
#[must_use]
pub fn selection(snapshot: &RailSnapshot) -> Selection {
    let Some((index, project)) = snapshot
        .focus
        .project
        .and_then(|index| Some((index, snapshot.projects.get(index)?)))
    else {
        return Selection::None;
    };
    match snapshot.focus.terminal {
        Some(id)
            if project.expanded && project.terminals.iter().any(|terminal| terminal.id == id) =>
        {
            Selection::Terminal(id)
        }
        _ => Selection::Project(index),
    }
}

/// The rows, in display order: each project's header, then its terminals when it is expanded.
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
            attention: !project.expanded && project.terminals.iter().any(|terminal| terminal.bell),
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
        }
    }
    rows
}

/// Whether any listed terminal rang its bell, collapsed or not: the rail's notification flag.
#[must_use]
pub fn has_attention(snapshot: &RailSnapshot) -> bool {
    snapshot
        .projects
        .iter()
        .any(|project| project.terminals.iter().any(|terminal| terminal.bell))
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
        }
    }

    fn window(
        projects: Vec<ProjectSnapshot>,
        project: Option<usize>,
        terminal: Option<u64>,
    ) -> RailSnapshot {
        RailSnapshot {
            projects,
            focus: Focus { project, terminal },
        }
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
