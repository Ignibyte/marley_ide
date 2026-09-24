//! Driven tests for the agent bar: a real PTY whose shell becomes `claude`, a link to `sleep` on
//! a scratch PATH, in a folder the project's fake repository covers.

use std::path::Path;
use std::time::Duration;

use gpui::{Entity, Focusable as _, TestAppContext, VisualTestContext};
use terminal::Terminal;
use terminal_view::{MarleyFooterContext, TerminalView};
use workspace::Workspace;

use super::*;
use crate::marley_workbench_tests::{
    fake_claude_bin, init_test, redraw, scratch_folder, terminal_running, workspace_over,
};

/// What the bar shows for `view` now.
fn current_contents(
    view: &Entity<TerminalView>,
    workspace: &Entity<Workspace>,
    cx: &mut VisualTestContext,
) -> Option<BarContents> {
    cx.update(|_, cx| {
        let terminal = view.read(cx).terminal().clone();
        let project = workspace.read(cx).project().downgrade();
        let workspace = workspace.downgrade();
        let focus_handle = view.focus_handle(cx);
        let context = MarleyFooterContext {
            view: view.downgrade(),
            terminal: &terminal,
            project: &project,
            workspace: &workspace,
            focus_handle: &focus_handle,
        };
        contents(&context, cx)
    })
}

/// Sends a space, which the tty echoes, so the terminal reads its foreground process again with
/// the output, until the bar has an agent to show.
async fn wait_for_the_bar(
    terminal: &Entity<Terminal>,
    view: &Entity<TerminalView>,
    workspace: &Entity<Workspace>,
    cx: &mut VisualTestContext,
) -> BarContents {
    for _ in 0..300 {
        terminal.update(cx, |terminal, _| terminal.input(b" ".to_vec()));
        redraw(cx);
        if let Some(contents) = current_contents(view, workspace, cx) {
            redraw(cx);
            return contents;
        }
        cx.background_executor
            .timer(Duration::from_millis(20))
            .await;
    }
    panic!("the terminal never had an agent in its foreground");
}

#[gpui::test]
async fn the_bar_shows_the_agent_its_folder_and_its_branch(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    init_test(cx);
    cx.update(init);
    let (_folder, folder) = scratch_folder();
    let bin = fake_claude_bin();
    let (workspace, cx) = workspace_over(&folder, cx).await;
    let (terminal, view) =
        terminal_running("exec claude 60", &folder, bin.path(), &workspace, cx).await;
    let contents = wait_for_the_bar(&terminal, &view, &workspace, cx).await;
    assert_eq!(
        contents,
        BarContents {
            agent: AgentKind::Claude,
            folder: Some(folder),
            branch: Some(SharedString::new_static("main")),
        }
    );
    for selector in [
        "marley-agent-bar",
        "marley-agent-bar-folder",
        "marley-agent-bar-branch",
    ] {
        assert!(cx.debug_bounds(selector).is_some(), "{selector}");
    }
}

#[gpui::test]
async fn without_an_agent_there_is_no_bar_and_every_row_is_the_terminals(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    init_test(cx);
    cx.update(init);
    let (_folder, folder) = scratch_folder();
    let bin = fake_claude_bin();
    let (workspace, cx) = workspace_over(&folder, cx).await;
    // The shell waits for a line before it becomes the agent.
    let (terminal, view) = terminal_running(
        "read line; exec claude 60",
        &folder,
        bin.path(),
        &workspace,
        cx,
    )
    .await;
    for _ in 0..10 {
        terminal.update(cx, |terminal, _| terminal.input(b" ".to_vec()));
        redraw(cx);
        cx.background_executor
            .timer(Duration::from_millis(20))
            .await;
    }
    assert_eq!(current_contents(&view, &workspace, cx), None);
    assert!(cx.debug_bounds("marley-agent-bar").is_none());
    let lines = |cx: &mut VisualTestContext| {
        terminal.read_with(cx, |terminal, _| terminal.last_content().screen_lines)
    };
    let without_the_bar = lines(cx);

    terminal.update(cx, |terminal, _| terminal.input(b"\r".to_vec()));
    wait_for_the_bar(&terminal, &view, &workspace, cx).await;
    redraw(cx);
    assert!(cx.debug_bounds("marley-agent-bar").is_some());
    let with_the_bar = lines(cx);
    assert!(
        with_the_bar < without_the_bar,
        "{with_the_bar} lines with the bar, {without_the_bar} without"
    );
}

#[test]
fn the_branch_comes_from_the_innermost_repository() {
    let outer = SharedString::new_static("main");
    let inner = SharedString::new_static("feature");
    let repositories = [
        (Path::new("/work"), Some(&outer)),
        (Path::new("/work/vendor/lib"), Some(&inner)),
    ];
    assert_eq!(
        branch_for(Path::new("/work/vendor/lib/src"), repositories),
        Some(inner.clone())
    );
    assert_eq!(
        branch_for(Path::new("/work/src"), repositories),
        Some(outer.clone())
    );
    // A path that only shares a prefix is not inside, and a folder outside every repository
    // has no branch.
    assert_eq!(branch_for(Path::new("/workshop"), repositories), None);
    assert_eq!(branch_for(Path::new("/elsewhere"), repositories), None);
    // A repository with no branch checked out names none.
    assert_eq!(
        branch_for(Path::new("/detached/src"), [(Path::new("/detached"), None)]),
        None
    );
}
