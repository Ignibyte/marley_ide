//! Driven tests for the agent bar: a real PTY whose shell becomes `claude`, a link to `sleep` on
//! a scratch PATH, in a folder the project's fake repository covers.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use fs::{FakeFs, Fs};
use gpui::{Entity, Focusable as _, TestAppContext, VisualTestContext};
use project::Project;
use serde_json::json;
use terminal::Terminal;
use terminal_view::{MarleyFooterContext, TerminalView};
use workspace::{MultiWorkspace, Workspace};

use super::*;
use crate::marley_workbench_tests::init_test;

/// A scratch folder holding `claude`, a link to `sleep`, so a script can become an agent by
/// name, as `claude` is started from the PATH.
fn fake_claude_bin() -> tempfile::TempDir {
    let path = std::env::var_os("PATH").expect("a PATH");
    let sleep = std::env::split_paths(&path)
        .map(|dir| dir.join("sleep"))
        .find(|candidate| candidate.is_file())
        .expect("sleep on the PATH");
    let bin = tempfile::tempdir().expect("a scratch directory");
    std::os::unix::fs::symlink(sleep, bin.path().join("claude")).expect("the link");
    bin
}

/// A real scratch folder, by its canonical path, which a process reports as its directory.
fn scratch_folder() -> (tempfile::TempDir, PathBuf) {
    let folder = tempfile::tempdir().expect("a scratch folder");
    let path = folder.path().canonicalize().expect("its canonical path");
    (folder, path)
}

/// A window whose project is `folder`, which is also a fake repository on branch `main`.
async fn workspace_over<'a>(
    folder: &Path,
    cx: &'a mut TestAppContext,
) -> (Entity<Workspace>, &'a mut VisualTestContext) {
    let fs = FakeFs::new(cx.executor());
    fs.insert_tree(folder, json!({ ".git": {}, "src": {} }))
        .await;
    fs.set_branch_name(&folder.join(".git"), Some("main"));
    let project = Project::test(Arc::<FakeFs>::clone(&fs), [folder], cx).await;
    cx.update(|cx| <dyn Fs>::set_global(fs, cx));
    let (multi_workspace, cx) =
        cx.add_window_view(|window, cx| MultiWorkspace::test_new(project, window, cx));
    let workspace =
        multi_workspace.read_with(cx, |multi_workspace, _| multi_workspace.workspace().clone());
    cx.run_until_parked();
    (workspace, cx)
}

/// A terminal view in `workspace`'s center over a real PTY that runs `script` in `folder`, with
/// `bin` first on the PATH.
async fn terminal_running(
    script: &str,
    folder: &Path,
    bin: &Path,
    workspace: &Entity<Workspace>,
    cx: &mut VisualTestContext,
) -> (Entity<Terminal>, Entity<TerminalView>) {
    let program = "/bin/sh".to_string();
    let args = vec!["-c".to_string(), script.to_string()];
    let inherited = std::env::var_os("PATH").expect("a PATH");
    let path = std::env::join_paths(
        std::iter::once(bin.to_path_buf()).chain(std::env::split_paths(&inherited)),
    )
    .expect("a PATH");
    let mut env = collections::HashMap::default();
    env.insert(
        "PATH".to_string(),
        path.into_string().expect("a UTF-8 PATH"),
    );
    let builder = cx
        .update(|_, cx| {
            terminal::TerminalBuilder::new(
                Some(folder.to_path_buf()),
                terminal::TerminalMode::task(task::SpawnInTerminal {
                    command: Some(program.clone()),
                    args: args.clone(),
                    ..Default::default()
                }),
                task::Shell::WithArguments {
                    program,
                    args,
                    title_override: None,
                },
                env,
                terminal::terminal_settings::CursorShape::default(),
                terminal::terminal_settings::AlternateScroll::On,
                None,
                vec![],
                Duration::ZERO,
                false,
                0,
                cx,
                vec![],
                util::paths::PathStyle::local(),
            )
        })
        .await
        .expect("the terminal starts");
    let terminal = cx.update(|_, cx| cx.new(|cx| builder.subscribe(cx)));
    let view = workspace.update_in(cx, |workspace, window, cx| {
        let view = cx.new(|cx| {
            TerminalView::new(
                terminal.clone(),
                workspace.weak_handle(),
                None,
                workspace.project().downgrade(),
                window,
                cx,
            )
        });
        workspace.add_item_to_active_pane(Box::new(view.clone()), None, true, window, cx);
        view
    });
    (terminal, view)
}

fn redraw(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}

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
