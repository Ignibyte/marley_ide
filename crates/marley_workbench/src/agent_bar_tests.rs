//! Driven tests for the agent bar: a real PTY whose shell becomes `claude`, a link to `sleep` on
//! a scratch PATH, in a folder the project's fake repository covers.

use std::path::{Path, PathBuf};
use std::time::Duration;

use gpui::{Entity, Focusable as _, Modifiers, TestAppContext, VisualTestContext};
use terminal::Terminal;
use terminal_view::{MarleyFooterContext, TerminalView};
use workspace::notifications::NotificationId;
use workspace::{SplitDirection, Workspace};

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
        "marley-attach-file",
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

/// A `claude` for the installer: it logs each run's arguments to `claude.log` beside it, and
/// fails, saying why, when `fail` is set.
fn fake_claude_cli(dir: &Path, fail: bool) -> PathBuf {
    let path = dir.join("claude-cli");
    let failure = if fail {
        "echo 'no network' >&2\nexit 1\n"
    } else {
        ""
    };
    std::fs::write(
        &path,
        format!("#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$(dirname \"$0\")/claude.log\"\n{failure}"),
    )
    .expect("the script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .expect("executable");
    }
    path
}

/// Claude Code's state in `scratch/config`, Marley's plugin going to `scratch/market`, and
/// `claude` the installer's fake.
fn plugin_in(scratch: &Path, fail: bool) -> ClaudePlugin {
    ClaudePlugin {
        marketplace_dir: scratch.join("market"),
        config_dir: scratch.join("config"),
        claude: Some(fake_claude_cli(scratch, fail)),
        installed: None,
        installing: false,
    }
}

/// Draws frames until `done` holds of the plugin's state.
async fn wait_for_plugin(cx: &mut VisualTestContext, done: impl Fn(&ClaudePlugin) -> bool) {
    for _ in 0..300 {
        redraw(cx);
        if cx.update(|_, cx| done(cx.global::<ClaudePlugin>())) {
            return;
        }
        cx.background_executor
            .timer(Duration::from_millis(10))
            .await;
    }
    panic!("the plugin's state never came");
}

/// A window whose terminal runs a stand-in `claude`, with Marley's plugin set up as `plugin`
/// says, once Claude Code's list has been read.
async fn claude_with_plugin<'a>(
    plugin: ClaudePlugin,
    folder: &Path,
    bin: &Path,
    cx: &'a mut TestAppContext,
) -> (Entity<Workspace>, &'a mut VisualTestContext) {
    cx.executor().allow_parking();
    init_test(cx);
    cx.update(init);
    cx.update(|cx| claude_plugin::set_up(plugin, cx));
    let (workspace, cx) = workspace_over(folder, cx).await;
    let (terminal, view) = terminal_running("exec claude 60", folder, bin, &workspace, cx).await;
    wait_for_the_bar(&terminal, &view, &workspace, cx).await;
    wait_for_plugin(cx, |plugin| plugin.installed.is_some()).await;
    (workspace, cx)
}

fn click_the_chip(cx: &mut VisualTestContext) {
    let chip = cx
        .debug_bounds("marley-claude-plugin-chip")
        .expect("the chip");
    cx.simulate_click(chip.center(), Modifiers::none());
}

#[gpui::test]
async fn the_chip_installs_marleys_plugin_for_claude_code(cx: &mut TestAppContext) {
    let scratch = tempfile::tempdir().expect("a scratch directory");
    let (_folder, folder) = scratch_folder();
    let bin = fake_claude_bin();
    let plugin = plugin_in(scratch.path(), false);
    let market = plugin.marketplace_dir.clone();
    let (workspace, cx) = claude_with_plugin(plugin, &folder, bin.path(), cx).await;

    click_the_chip(cx);
    wait_for_plugin(cx, |plugin| !plugin.installing).await;
    let log = std::fs::read_to_string(scratch.path().join("claude.log")).expect("the log");
    assert_eq!(
        log,
        format!(
            "plugin marketplace add {}\nplugin install marley@marley\n",
            market.display()
        )
    );
    assert!(market.join("marley/hooks/notify.sh").is_file());
    assert_eq!(
        cx.update(|_, cx| cx.global::<ClaudePlugin>().installed),
        Some(true)
    );
    redraw(cx);
    assert!(cx.debug_bounds("marley-claude-plugin-chip").is_none());
    assert!(workspace.read_with(cx, |workspace, _| {
        workspace.has_notification(&NotificationId::unique::<ClaudePlugin>())
    }));
}

#[gpui::test]
async fn a_marketplace_claude_code_knows_is_not_added_again(cx: &mut TestAppContext) {
    let scratch = tempfile::tempdir().expect("a scratch directory");
    let (_folder, folder) = scratch_folder();
    let bin = fake_claude_bin();
    let plugin = plugin_in(scratch.path(), false);
    let plugins = plugin.config_dir.join("plugins");
    std::fs::create_dir_all(&plugins).expect("the directory");
    std::fs::write(plugins.join("known_marketplaces.json"), r#"{"marley": {}}"#).expect("written");
    let (_, cx) = claude_with_plugin(plugin, &folder, bin.path(), cx).await;

    click_the_chip(cx);
    wait_for_plugin(cx, |plugin| !plugin.installing).await;
    let log = std::fs::read_to_string(scratch.path().join("claude.log")).expect("the log");
    assert_eq!(log, "plugin install marley@marley\n");
}

#[gpui::test]
async fn with_marleys_plugin_installed_there_is_no_chip(cx: &mut TestAppContext) {
    let scratch = tempfile::tempdir().expect("a scratch directory");
    let (_folder, folder) = scratch_folder();
    let bin = fake_claude_bin();
    let plugin = plugin_in(scratch.path(), false);
    let plugins = plugin.config_dir.join("plugins");
    std::fs::create_dir_all(&plugins).expect("the directory");
    std::fs::write(
        plugins.join("installed_plugins.json"),
        r#"{"plugins": {"marley@marley": []}}"#,
    )
    .expect("written");
    let (_, cx) = claude_with_plugin(plugin, &folder, bin.path(), cx).await;
    assert!(cx.debug_bounds("marley-agent-bar").is_some());
    assert!(cx.debug_bounds("marley-claude-plugin-chip").is_none());
}

#[gpui::test]
async fn a_failed_install_keeps_the_chip_and_says_why(cx: &mut TestAppContext) {
    let scratch = tempfile::tempdir().expect("a scratch directory");
    let (_folder, folder) = scratch_folder();
    let bin = fake_claude_bin();
    let (workspace, cx) =
        claude_with_plugin(plugin_in(scratch.path(), true), &folder, bin.path(), cx).await;

    click_the_chip(cx);
    wait_for_plugin(cx, |plugin| !plugin.installing).await;
    assert_eq!(
        cx.update(|_, cx| cx.global::<ClaudePlugin>().installed),
        Some(false)
    );
    redraw(cx);
    assert!(cx.debug_bounds("marley-claude-plugin-chip").is_some());
    let (errors, toast) = workspace.read_with(cx, |workspace, _| {
        (
            workspace.notification_ids().len(),
            workspace.has_notification(&NotificationId::unique::<ClaudePlugin>()),
        )
    });
    assert_eq!((errors, toast), (1, false));
}

/// A window whose focused terminal runs a stand-in `claude`, once the bar shows, with the PTY's
/// log emptied of the spaces that brought the bar up.
async fn claude_to_attach_to<'a>(
    folder: &Path,
    bin: &Path,
    cx: &'a mut TestAppContext,
) -> (
    Entity<Workspace>,
    Entity<Terminal>,
    &'a mut VisualTestContext,
) {
    cx.executor().allow_parking();
    init_test(cx);
    cx.update(init);
    let (workspace, cx) = workspace_over(folder, cx).await;
    let (terminal, view) = terminal_running("exec claude 60", folder, bin, &workspace, cx).await;
    wait_for_the_bar(&terminal, &view, &workspace, cx).await;
    written(&terminal, cx);
    (workspace, terminal, cx)
}

/// What the terminal sent its PTY since the last look.
fn written(terminal: &Entity<Terminal>, cx: &mut VisualTestContext) -> Vec<Vec<u8>> {
    terminal.update(cx, |terminal, _| terminal.take_pty_write_log())
}

fn click_attach_file(cx: &mut VisualTestContext) {
    let button = cx.debug_bounds("marley-attach-file").expect("Attach File");
    cx.simulate_click(button.center(), Modifiers::none());
    assert!(cx.did_prompt_for_paths(), "the chooser opens");
}

#[gpui::test]
async fn attach_file_types_the_chosen_paths_as_a_drop_does(cx: &mut TestAppContext) {
    let (_folder, folder) = scratch_folder();
    let bin = fake_claude_bin();
    let (_, terminal, cx) = claude_to_attach_to(&folder, bin.path(), cx).await;
    click_attach_file(cx);
    let spaced = folder.join("notes on the plan.md");
    let plain = folder.join("src/main.rs");
    cx.simulate_path_prompt_response(|options| {
        assert!(
            options.files && options.multiple && !options.directories,
            "{options:?}"
        );
        Some(vec![spaced.clone(), plain.clone()])
    });
    redraw(cx);
    // Each path whole, quoted when it needs to be, with a space either side, in one write.
    let expected = format!(" '{}' {} ", spaced.display(), plain.display());
    assert_eq!(written(&terminal, cx), [expected.into_bytes()]);
}

#[gpui::test]
async fn a_cancelled_chooser_types_nothing(cx: &mut TestAppContext) {
    let (_folder, folder) = scratch_folder();
    let bin = fake_claude_bin();
    let (_, terminal, cx) = claude_to_attach_to(&folder, bin.path(), cx).await;
    click_attach_file(cx);
    cx.simulate_path_prompt_response(|_| None);
    redraw(cx);
    assert_eq!(written(&terminal, cx), Vec::<Vec<u8>>::new());
}

#[gpui::test]
async fn the_action_attaches_to_the_focused_terminal_only(cx: &mut TestAppContext) {
    let (_folder, folder) = scratch_folder();
    let bin = fake_claude_bin();
    let (workspace, terminal, cx) = claude_to_attach_to(&folder, bin.path(), cx).await;
    cx.dispatch_action(AttachFile);
    let chosen = folder.join("README.md");
    cx.simulate_path_prompt_response(|_| Some(vec![chosen.clone()]));
    redraw(cx);
    assert_eq!(
        written(&terminal, cx),
        [format!(" {} ", chosen.display()).into_bytes()]
    );

    // An empty pane beside the terminal takes the focus, and the action asks for nothing.
    workspace.update_in(cx, |workspace, window, cx| {
        let pane = workspace.active_pane().clone();
        workspace.split_pane(pane, SplitDirection::Right, window, cx);
    });
    cx.dispatch_action(AttachFile);
    redraw(cx);
    assert!(!cx.did_prompt_for_paths());
}
