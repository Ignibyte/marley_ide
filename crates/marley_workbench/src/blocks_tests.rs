//! Driven tests for the block keys: a real PTY's blocks in a focused terminal, walked with the
//! keys and the actions.

use std::time::Duration;

use gpui::{AppContext as _, Entity, TestAppContext, VisualTestContext};
use terminal::Terminal;
use util::path;
use workspace::SplitDirection;

use super::*;
use crate::marley_workbench_tests::{init_test, open_projects};

/// Three finished blocks of five lines, 200 lines of other output, then a running block at the
/// bottom. The shell then waits: a child that exits at once can leave its last bytes unread
/// (L-claude-470-a-pty-test-child-that-exits-can-lose-its-last-bytes-001).
const SCRIPT: &str = r#"f() { printf '\033Pp%s\033\\' "$1"; }
f 'init;id=1'
for n in 1 2 3; do f 'precmd;exit=0'; echo "\$ command $n"; f "preexec;command=command $n"; seq 1 5; done
f 'precmd;exit=0'; seq 1 200
f 'precmd;exit=0'; echo '$ last'; f 'preexec;command=last'
sleep 60"#;

/// A terminal view for `workspace` over a real PTY running [`SCRIPT`], not yet placed.
async fn blocks_terminal(
    workspace: &Entity<Workspace>,
    cx: &mut VisualTestContext,
) -> (Entity<Terminal>, Entity<TerminalView>) {
    let program = "/bin/sh".to_string();
    let args = vec!["-c".to_string(), SCRIPT.to_string()];
    let builder = cx
        .update(|_, cx| {
            terminal::TerminalBuilder::new(
                None,
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
                collections::HashMap::default(),
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
        cx.new(|cx| {
            TerminalView::new(
                terminal.clone(),
                workspace.weak_handle(),
                None,
                workspace.project().downgrade(),
                window,
                cx,
            )
        })
    });
    (terminal, view)
}

/// Draws frames until `terminal` holds [`SCRIPT`]'s four blocks, and returns where each starts.
async fn wait_for_blocks(terminal: &Entity<Terminal>, cx: &mut VisualTestContext) -> Vec<u64> {
    for _ in 0..500 {
        redraw(cx);
        let starts: Vec<u64> = terminal.read_with(cx, |terminal, _| {
            terminal
                .blocks()
                .iter()
                .filter_map(|block| block.prompt_line)
                .collect()
        });
        if starts.len() == 4 {
            return starts;
        }
        cx.background_executor
            .timer(Duration::from_millis(10))
            .await;
    }
    panic!("the terminal never held its four blocks");
}

/// A focused terminal view in `workspace`'s center over [`SCRIPT`], with its four blocks in.
async fn terminal_with_blocks(
    workspace: &Entity<Workspace>,
    cx: &mut VisualTestContext,
) -> (Entity<Terminal>, Vec<u64>) {
    let (terminal, view) = blocks_terminal(workspace, cx).await;
    workspace.update_in(cx, |workspace, window, cx| {
        workspace.add_item_to_active_pane(Box::new(view), None, true, window, cx);
    });
    let starts = wait_for_blocks(&terminal, cx).await;
    (terminal, starts)
}

/// Draws a frame, in which the terminal takes its queued scrolls.
fn redraw(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}

fn display_offset(terminal: &Entity<Terminal>, cx: &VisualTestContext) -> usize {
    terminal.read_with(cx, |terminal, _| terminal.last_content().display_offset)
}

/// The index of `terminal`'s selected block (#554).
fn selected(terminal: &Entity<Terminal>, cx: &VisualTestContext) -> Option<usize> {
    cx.read(|cx| MarleyBlockSelection::selected(terminal, cx))
}

/// Whether the first line of `terminal`'s block at `index` is on the screen.
fn shows_block(terminal: &Entity<Terminal>, index: usize, cx: &VisualTestContext) -> bool {
    terminal.read_with(cx, |terminal, _| {
        let content = terminal.last_content();
        let top = content.marley_screen_top - content.display_offset as u64;
        terminal.blocks().get(index).is_some_and(|block| {
            let start = block.prompt_line.unwrap_or(block.output_start);
            (top..top + content.screen_lines as u64).contains(&start)
        })
    })
}

#[gpui::test]
async fn the_block_keys_walk_the_focused_terminals_blocks(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    init_test(cx);
    cx.update(|cx| {
        init(cx);
        crate::load_keymap(cx);
    });
    let (_, workspaces, cx) = open_projects(&[path!("/alpha")], cx).await;
    let (terminal, _) = terminal_with_blocks(&workspaces[0], cx).await;
    assert_eq!(display_offset(&terminal, cx), 0);
    let last = terminal.read_with(cx, |terminal, _| terminal.blocks().len()) - 1;

    // The Marley keymap's key, in the terminal: each press selects the block before, scrolled
    // into view when it is not (#554), and two presses before a frame count as two.
    cx.simulate_keystrokes("secondary-up secondary-up");
    redraw(cx);
    assert_eq!(selected(&terminal, cx), Some(last - 1));
    assert!(shows_block(&terminal, last - 1, cx));
    // On to the first block; from it, nothing further back.
    for expected in (0..last - 1).rev().chain([0]) {
        cx.simulate_keystrokes("secondary-up");
        redraw(cx);
        assert_eq!(selected(&terminal, cx), Some(expected));
        assert!(shows_block(&terminal, expected, cx), "block {expected}");
    }
    // And the actions forward; past the last block the selection ends.
    for expected in 1..=last {
        cx.dispatch_action(NextBlock);
        redraw(cx);
        assert_eq!(selected(&terminal, cx), Some(expected));
        assert!(shows_block(&terminal, expected, cx), "block {expected}");
    }
    cx.dispatch_action(NextBlock);
    redraw(cx);
    assert_eq!(selected(&terminal, cx), None);
    // With nothing selected, the key forward scrolls block by block to the live screen.
    for _ in 0..=last {
        cx.simulate_keystrokes("secondary-down");
        redraw(cx);
    }
    assert_eq!(display_offset(&terminal, cx), 0);
}

#[gpui::test]
async fn the_block_keys_leave_a_terminal_without_focus_alone(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    init_test(cx);
    cx.update(init);
    let (_, workspaces, cx) = open_projects(&[path!("/alpha")], cx).await;
    let workspace = workspaces[0].clone();
    let (terminal, _) = terminal_with_blocks(&workspace, cx).await;
    // An empty pane beside the terminal takes the focus.
    workspace.update_in(cx, |workspace, window, cx| {
        let pane = workspace.active_pane().clone();
        workspace.split_pane(pane, SplitDirection::Right, window, cx);
    });
    cx.dispatch_action(PreviousBlock);
    redraw(cx);
    assert_eq!(display_offset(&terminal, cx), 0);
}

#[gpui::test]
async fn the_block_keys_work_in_the_terminal_panel(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    init_test(cx);
    cx.update(|cx| {
        init(cx);
        crate::load_keymap(cx);
    });
    let (_, workspaces, cx) = open_projects(&[path!("/alpha")], cx).await;
    let workspace = workspaces[0].clone();
    let loading =
        cx.update(|window, cx| TerminalPanel::load(workspace.downgrade(), window.to_async(cx)));
    let panel = loading.await.expect("the Terminal Panel loads");
    workspace.update_in(cx, |workspace, window, cx| {
        workspace.add_panel(panel.clone(), window, cx);
    });
    let (terminal, view) = blocks_terminal(&workspace, cx).await;
    let pane = panel.read_with(cx, |panel, _| panel.panes()[0].clone());
    pane.update_in(cx, |pane, window, cx| {
        pane.add_item(Box::new(view), true, true, None, window, cx);
    });
    workspace.update_in(cx, |workspace, window, cx| {
        workspace.focus_panel::<TerminalPanel>(window, cx);
    });
    wait_for_blocks(&terminal, cx).await;
    let last = terminal.read_with(cx, |terminal, _| terminal.blocks().len()) - 1;
    // The key selects the last block (#554), which already shows.
    cx.simulate_keystrokes("secondary-up");
    redraw(cx);
    assert_eq!(selected(&terminal, cx), Some(last));
}
