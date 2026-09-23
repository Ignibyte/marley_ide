//! Driven tests for the terminal routing. They start real shells and tasks, as Zed's own
//! Terminal Panel tests do, so each lets the executor park while a process works.

use std::time::Duration;

use fs::FakeFs;
use gpui::{Focusable as _, TestAppContext, VisualTestContext};
use project::Project;
use settings::MarleyLayout;
use task::{Shell, TaskId};
use terminal_view::TerminalView;
use workspace::MultiWorkspace;

use super::*;
use crate::marley_workbench_tests::{init_test, init_zed_sidebar, set_layout};

/// A window in `layout` over an empty project, with the Terminal Panel loaded and added the way
/// `crates/zed` adds it: `load` installs Zed's task provider before `add_panel` announces the
/// panel, so the routing's provider has to win that order.
async fn open_workspace(
    layout: MarleyLayout,
    cx: &mut TestAppContext,
) -> (
    Entity<Workspace>,
    Entity<TerminalPanel>,
    &mut VisualTestContext,
) {
    cx.executor().allow_parking();
    init_test(cx);
    cx.update(crate::init);
    set_layout(layout, cx);
    let project = Project::test(FakeFs::new(cx.executor()), [], cx).await;
    let (multi_workspace, cx) =
        cx.add_window_view(|window, cx| MultiWorkspace::test_new(project, window, cx));
    let workspace =
        multi_workspace.read_with(cx, |multi_workspace, _| multi_workspace.workspace().clone());
    let loading =
        cx.update(|window, cx| TerminalPanel::load(workspace.downgrade(), window.to_async(cx)));
    let panel = loading.await.expect("the Terminal Panel loads");
    workspace.update_in(cx, |workspace, window, cx| {
        workspace.add_panel(panel.clone(), window, cx);
    });
    cx.run_until_parked();
    (workspace, panel, cx)
}

/// A task that runs `echo` once. The label keeps it from reusing another task's terminal.
fn echo_task(label: &str) -> SpawnInTerminal {
    SpawnInTerminal {
        id: TaskId(label.to_string()),
        label: label.to_string(),
        full_label: label.to_string(),
        command: Some("echo".to_string()),
        ..SpawnInTerminal::default()
    }
}

/// Runs `task` the way the task picker does and waits for it to finish.
async fn run(
    task: SpawnInTerminal,
    workspace: &Entity<Workspace>,
    cx: &mut VisualTestContext,
) -> Option<anyhow::Result<ExitStatus>> {
    let running = workspace.update_in(cx, |workspace, window, cx| {
        workspace.spawn_in_terminal(task, window, cx)
    });
    let exit = running.await;
    cx.run_until_parked();
    exit
}

fn succeeded(exit: Option<anyhow::Result<ExitStatus>>) -> bool {
    exit.expect("the task reports an exit")
        .expect("the task starts")
        .success()
}

/// Dispatches `action` from the workspace's center pane, below the workspace's root and its
/// capture listeners, as a key press there would.
fn dispatch_from_center(
    workspace: &Entity<Workspace>,
    action: impl gpui::Action,
    cx: &mut VisualTestContext,
) {
    workspace.update_in(cx, |workspace, window, cx| {
        workspace.active_pane().focus_handle(cx).focus(window, cx);
    });
    cx.update(|window, _| window.refresh());
    cx.dispatch_action(action);
    cx.run_until_parked();
}

fn center_terminals(
    workspace: &Entity<Workspace>,
    cx: &VisualTestContext,
) -> Vec<Entity<Terminal>> {
    workspace.read_with(cx, |workspace, cx| {
        workspace
            .items_of_type::<TerminalView>(cx)
            .map(|view| view.read(cx).terminal().clone())
            .collect()
    })
}

fn panel_terminals(panel: &Entity<TerminalPanel>, cx: &VisualTestContext) -> usize {
    panel.read_with(cx, |panel, cx| {
        panel
            .panes()
            .into_iter()
            .map(|pane| pane.read(cx).items_len())
            .sum()
    })
}

fn bottom_dock_open(workspace: &Entity<Workspace>, cx: &VisualTestContext) -> bool {
    workspace.read_with(cx, |workspace, cx| {
        workspace.bottom_dock().read(cx).is_open()
    })
}

/// The directory `terminal`'s shell reports once it has started, given five seconds to start.
async fn shell_directory(terminal: &Entity<Terminal>, cx: &VisualTestContext) -> Option<PathBuf> {
    for _ in 0..100 {
        let directory = terminal.read_with(cx, |terminal, _| terminal.working_directory());
        if directory.is_some() {
            return directory;
        }
        cx.background_executor
            .timer(Duration::from_millis(50))
            .await;
        cx.run_until_parked();
    }
    None
}

#[gpui::test]
async fn a_task_in_the_marley_layout_runs_in_a_center_terminal(cx: &mut TestAppContext) {
    let (workspace, panel, cx) = open_workspace(MarleyLayout::Marley, cx).await;
    // The task asks for the panel, as a task does unless it names the center.
    assert!(succeeded(run(echo_task("echo"), &workspace, cx).await));
    assert_eq!(center_terminals(&workspace, cx).len(), 1);
    // A rerun replaces the task's terminal, as Zed's rules say, in the center.
    assert!(succeeded(run(echo_task("echo"), &workspace, cx).await));
    assert_eq!(center_terminals(&workspace, cx).len(), 1);
    assert_eq!(panel_terminals(&panel, cx), 0);
    assert!(!bottom_dock_open(&workspace, cx));
}

#[gpui::test]
async fn a_task_that_cannot_start_reports_its_error_and_opens_nothing(cx: &mut TestAppContext) {
    let (workspace, panel, cx) = open_workspace(MarleyLayout::Marley, cx).await;
    // A missing program is the shell's to report, with exit status 127; a missing shell means
    // there is no terminal to run the task in.
    let task = SpawnInTerminal {
        shell: Shell::Program("__nonexistent_shell__".to_string()),
        ..echo_task("missing")
    };
    let exit = run(task, &workspace, cx).await;
    assert!(matches!(exit, Some(Err(_))), "{exit:?}");
    assert!(center_terminals(&workspace, cx).is_empty());
    assert_eq!(panel_terminals(&panel, cx), 0);
}

#[gpui::test]
async fn a_task_in_the_zed_layout_opens_where_its_reveal_target_says(cx: &mut TestAppContext) {
    let (workspace, panel, cx) = open_workspace(MarleyLayout::Zed, cx).await;
    assert!(succeeded(run(echo_task("dock"), &workspace, cx).await));
    assert_eq!(panel_terminals(&panel, cx), 1);
    assert!(center_terminals(&workspace, cx).is_empty());

    let task = SpawnInTerminal {
        reveal_target: RevealTarget::Center,
        ..echo_task("center")
    };
    assert!(succeeded(run(task, &workspace, cx).await));
    assert_eq!(panel_terminals(&panel, cx), 1);
    assert_eq!(center_terminals(&workspace, cx).len(), 1);
}

#[gpui::test]
async fn new_terminal_in_the_marley_layout_opens_in_the_center(cx: &mut TestAppContext) {
    let (workspace, panel, cx) = open_workspace(MarleyLayout::Marley, cx).await;
    dispatch_from_center(&workspace, NewTerminal::default(), cx);
    assert_eq!(center_terminals(&workspace, cx).len(), 1);
    dispatch_from_center(&workspace, NewTerminal { local: true }, cx);
    assert_eq!(center_terminals(&workspace, cx).len(), 2);
    assert_eq!(panel_terminals(&panel, cx), 0);
    assert!(!bottom_dock_open(&workspace, cx));
}

#[gpui::test]
async fn open_terminal_in_the_marley_layout_starts_a_center_shell_in_its_directory(
    cx: &mut TestAppContext,
) {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let expected = directory
        .path()
        .canonicalize()
        .expect("the directory exists");
    let (workspace, panel, cx) = open_workspace(MarleyLayout::Marley, cx).await;
    let action = OpenTerminal {
        working_directory: directory.path().to_path_buf(),
        local: false,
    };
    dispatch_from_center(&workspace, action, cx);
    let terminals = center_terminals(&workspace, cx);
    assert_eq!(terminals.len(), 1);
    assert_eq!(panel_terminals(&panel, cx), 0);
    assert!(!bottom_dock_open(&workspace, cx));
    assert_eq!(shell_directory(&terminals[0], cx).await, Some(expected));
}

#[gpui::test]
async fn in_the_zed_layout_both_actions_open_in_the_terminal_panel(cx: &mut TestAppContext) {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let (workspace, panel, cx) = open_workspace(MarleyLayout::Zed, cx).await;
    dispatch_from_center(&workspace, NewTerminal::default(), cx);
    assert_eq!(panel_terminals(&panel, cx), 1);
    assert!(bottom_dock_open(&workspace, cx));
    let action = OpenTerminal {
        working_directory: directory.path().to_path_buf(),
        local: false,
    };
    dispatch_from_center(&workspace, action, cx);
    assert_eq!(panel_terminals(&panel, cx), 2);
    assert!(center_terminals(&workspace, cx).is_empty());
}

#[gpui::test]
async fn a_layout_switch_routes_the_next_terminal_and_task(cx: &mut TestAppContext) {
    let (workspace, panel, cx) = open_workspace(MarleyLayout::Zed, cx).await;
    // The switch back to the Zed layout builds Zed's sidebar.
    init_zed_sidebar(cx);
    dispatch_from_center(&workspace, NewTerminal::default(), cx);
    assert_eq!(center_terminals(&workspace, cx).len(), 0);
    assert_eq!(panel_terminals(&panel, cx), 1);

    set_layout(MarleyLayout::Marley, cx);
    dispatch_from_center(&workspace, NewTerminal::default(), cx);
    assert!(succeeded(run(echo_task("in marley"), &workspace, cx).await));
    assert_eq!(center_terminals(&workspace, cx).len(), 2);
    assert_eq!(panel_terminals(&panel, cx), 1);

    // Back in the Zed layout a task that asks for the panel gets it again. (Zed itself sends
    // New Terminal to the center while a center terminal is focused, so a task shows the switch.)
    set_layout(MarleyLayout::Zed, cx);
    assert!(succeeded(run(echo_task("in zed"), &workspace, cx).await));
    assert_eq!(center_terminals(&workspace, cx).len(), 2);
    assert_eq!(panel_terminals(&panel, cx), 2);

    // In the Marley layout again, that task reruns in the center: its terminal leaves the panel.
    set_layout(MarleyLayout::Marley, cx);
    assert!(succeeded(run(echo_task("in zed"), &workspace, cx).await));
    assert_eq!(center_terminals(&workspace, cx).len(), 3);
    assert_eq!(panel_terminals(&panel, cx), 1);
}
