//! Driven tests for the desktop notifications: a real PTY prints the escapes once the test sends
//! it a line.

use std::time::Duration;

use gpui::{Entity, SystemNotificationResponse, TestAppContext, VisualTestContext};
use terminal::Terminal;
use terminal_view::TerminalView;
use workspace::{SplitDirection, Workspace};

use super::*;
use crate::marley_workbench_tests::{
    init_test, redraw, scratch_folder, terminal_running, workspace_over,
};

/// The escapes a program prints for two notifications: OSC 777 with a title, OSC 9 without.
const TWO_NOTIFICATIONS: &str = r"\033]777;notify;Build;done\007\033]9;tests passed\007";

/// A terminal in `workspace` whose script prints `escapes` after each line it is sent.
async fn notifying_terminal(
    escapes: &str,
    workspace: &Entity<Workspace>,
    cx: &mut VisualTestContext,
) -> (Entity<Terminal>, Entity<TerminalView>, tempfile::TempDir) {
    let (folder, path) = scratch_folder();
    let script = format!("while read line; do printf '{escapes}'; done");
    let (terminal, view) = terminal_running(&script, &path, &path, workspace, cx).await;
    (terminal, view, folder)
}

/// Sends the script a line and draws frames until the view's bell is set, which the
/// notification does whether or not it goes to the desktop.
async fn send_a_line_and_wait(
    terminal: &Entity<Terminal>,
    view: &Entity<TerminalView>,
    cx: &mut VisualTestContext,
) {
    view.update(cx, TerminalView::clear_bell);
    terminal.update(cx, |terminal, _| terminal.input(b"\r".to_vec()));
    for _ in 0..300 {
        redraw(cx);
        if view.read_with(cx, |view, _| view.has_bell()) {
            return;
        }
        cx.background_executor
            .timer(Duration::from_millis(10))
            .await;
    }
    panic!("the terminal's bell was never set");
}

fn focus_a_pane_beside(workspace: &Entity<Workspace>, cx: &mut VisualTestContext) {
    workspace.update_in(cx, |workspace, window, cx| {
        let pane = workspace.active_pane().clone();
        workspace.split_pane(pane, SplitDirection::Right, window, cx);
    });
}

fn terminal_is_focused(view: &Entity<TerminalView>, cx: &mut VisualTestContext) -> bool {
    cx.update(|window, cx| view.focus_handle(cx).contains_focused(window, cx))
}

#[gpui::test]
async fn a_notification_from_a_terminal_out_of_focus_goes_to_the_desktop(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    init_test(cx);
    cx.update(|cx| init(cx));
    let (_folder, folder) = scratch_folder();
    let (workspace, cx) = workspace_over(&folder, cx).await;
    let (terminal, view, _scratch) = notifying_terminal(TWO_NOTIFICATIONS, &workspace, cx).await;
    focus_a_pane_beside(&workspace, cx);
    assert!(!terminal_is_focused(&view, cx));

    send_a_line_and_wait(&terminal, &view, cx).await;
    let shown = cx.shown_system_notifications();
    assert_eq!(shown.len(), 2, "{shown:?}");
    assert_eq!(
        (shown[0].title.as_ref(), shown[0].body.as_ref()),
        ("Build", "done")
    );
    // OSC 9 gives no title, so the terminal's tab names it.
    let tab = view.read_with(cx, |view, cx| view.tab_content_text(0, cx));
    assert_eq!(
        (&shown[1].title, shown[1].body.as_ref()),
        (&tab, "tests passed")
    );
}

#[gpui::test]
async fn the_focused_terminal_of_the_active_window_goes_to_no_desktop(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    init_test(cx);
    cx.update(|cx| init(cx));
    let (_folder, folder) = scratch_folder();
    let (workspace, cx) = workspace_over(&folder, cx).await;
    let (terminal, view, _scratch) = notifying_terminal(TWO_NOTIFICATIONS, &workspace, cx).await;
    redraw(cx);
    assert!(terminal_is_focused(&view, cx));
    assert!(cx.update(|window, _| window.is_window_active()));

    send_a_line_and_wait(&terminal, &view, cx).await;
    assert_eq!(cx.shown_system_notifications(), []);

    // The same terminal, focused in a window that is not active, is out of sight.
    cx.deactivate_window();
    send_a_line_and_wait(&terminal, &view, cx).await;
    assert_eq!(cx.shown_system_notifications().len(), 2);
}

#[gpui::test]
async fn answering_a_notification_shows_its_terminal(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    init_test(cx);
    cx.update(|cx| init(cx));
    let (_folder, folder) = scratch_folder();
    let (workspace, cx) = workspace_over(&folder, cx).await;
    let (terminal, view, _scratch) = notifying_terminal(TWO_NOTIFICATIONS, &workspace, cx).await;
    focus_a_pane_beside(&workspace, cx);
    send_a_line_and_wait(&terminal, &view, cx).await;
    let tag = cx.shown_system_notifications()[0].tag.clone();

    cx.simulate_system_notification_response(SystemNotificationResponse {
        tag,
        action_id: None,
    });
    redraw(cx);
    assert!(terminal_is_focused(&view, cx));
    assert!(!view.read_with(cx, |view, _| view.has_bell()));
}

#[gpui::test]
async fn a_click_on_a_notification_from_a_closed_terminal_does_nothing(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    init_test(cx);
    cx.update(|cx| init(cx));
    let (_folder, folder) = scratch_folder();
    let (workspace, cx) = workspace_over(&folder, cx).await;
    let (terminal, view, _scratch) = notifying_terminal(TWO_NOTIFICATIONS, &workspace, cx).await;
    focus_a_pane_beside(&workspace, cx);
    send_a_line_and_wait(&terminal, &view, cx).await;
    let tag = cx.shown_system_notifications()[0].tag.clone();

    // The terminal closes before the click.
    let terminal_pane = workspace.read_with(cx, |workspace, _| workspace.panes()[0].clone());
    let view_id = view.entity_id();
    drop((terminal, view));
    terminal_pane
        .update_in(cx, |pane, window, cx| {
            pane.close_item_by_id(view_id, workspace::SaveIntent::Skip, window, cx)
        })
        .await
        .expect("the terminal closes");
    redraw(cx);
    let active_pane = workspace.read_with(cx, |workspace, _| workspace.active_pane().clone());

    cx.simulate_system_notification_response(SystemNotificationResponse {
        tag,
        action_id: None,
    });
    redraw(cx);
    assert_eq!(
        workspace.read_with(cx, |workspace, _| workspace.active_pane().clone()),
        active_pane
    );
}

#[gpui::test]
async fn a_response_to_a_notification_marley_did_not_post_does_nothing(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    init_test(cx);
    cx.update(|cx| init(cx));
    let (_folder, folder) = scratch_folder();
    let (workspace, cx) = workspace_over(&folder, cx).await;
    let (_terminal, view, _scratch) = notifying_terminal(TWO_NOTIFICATIONS, &workspace, cx).await;
    focus_a_pane_beside(&workspace, cx);
    cx.simulate_system_notification_response(SystemNotificationResponse {
        tag: SharedString::new_static("another-apps"),
        action_id: None,
    });
    redraw(cx);
    assert!(!terminal_is_focused(&view, cx));
}
