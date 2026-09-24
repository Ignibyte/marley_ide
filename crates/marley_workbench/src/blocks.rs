//! The block keys (T1c), which scroll the focused terminal from block to block.
//!
//! They move to the start of the previous or the next block. The actions are caught at the
//! workspace's root, as `routing` catches its own, and act on whichever terminal view holds
//! focus, in the center or in the Terminal Panel.

use gpui::{App, Context, Entity, Focusable as _, InteractiveElement as _, Window};
use terminal_view::{TerminalView, terminal_panel::TerminalPanel};
use workspace::Workspace;

use crate::{NextBlock, PreviousBlock};

/// Installs the block keys on every workspace. [`crate::init`] calls it once, before any window
/// opens.
pub fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action_renderer(|div, _, _, cx| {
            div.on_action(cx.listener(|workspace, _: &PreviousBlock, window, cx| {
                scroll_to_block(workspace, false, window, cx);
            }))
            .on_action(cx.listener(|workspace, _: &NextBlock, window, cx| {
                scroll_to_block(workspace, true, window, cx);
            }))
        });
    })
    .detach();
}

/// Scrolls the focused terminal view to the start of the next or the previous block, when there
/// is one that way.
fn scroll_to_block(
    workspace: &Workspace,
    forward: bool,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let Some(view) = focused_terminal(workspace, window, cx) else {
        return;
    };
    let terminal = view.read(cx).terminal().clone();
    terminal.update(cx, |terminal, cx| {
        // The content is otherwise the last frame's: a key pressed before the next frame then
        // starts where the one before it went.
        terminal.sync(window, cx);
        let content = terminal.last_content();
        let history = content.total_lines.saturating_sub(content.screen_lines);
        let offset = marley_terminal::block_scroll(
            terminal.blocks(),
            content.marley_screen_top,
            content.display_offset,
            history,
            forward,
        );
        if let Some(offset) = offset {
            terminal.scroll_to_bottom();
            terminal.scroll_up_by(offset);
        }
    });
}

/// The terminal view that holds focus: the active item of a center pane or of a Terminal Panel
/// pane.
pub(crate) fn focused_terminal(
    workspace: &Workspace,
    window: &Window,
    cx: &App,
) -> Option<Entity<TerminalView>> {
    let panel_panes = workspace
        .panel::<TerminalPanel>(cx)
        .map(|panel| {
            panel
                .read(cx)
                .panes()
                .into_iter()
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    workspace
        .panes()
        .iter()
        .chain(&panel_panes)
        .filter_map(|pane| pane.read(cx).active_item()?.downcast::<TerminalView>())
        .find(|view| view.focus_handle(cx).contains_focused(window, cx))
}

#[cfg(test)]
#[path = "blocks_tests.rs"]
mod tests;
