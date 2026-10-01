//! The block keys (T1c) and the block menu (#554).
//!
//! The keys select a block of the focused terminal: `ctrl-up` takes the newest block and then the
//! one before, `ctrl-down` the one after, clearing past the last; while one is selected, `up` and
//! `down` move the selection too, Escape ends it, and `ctrl-shift-i` types its command at the
//! prompt. The selection lives in `MarleyBlockSelection` with the count of inputs the terminal had
//! noted, so the first key typed to the shell ends it. With no block selected, `ctrl-down`
//! scrolls to the start of the next block, as the keys did before. The actions are caught at the
//! workspace's root, as `routing` catches its own, and act on whichever terminal view holds focus,
//! in the center or in the Terminal Panel.
//!
//! A right-click on a block selects it and adds a Block section to Zed's terminal menu: Send to
//! Agent (#555), Bookmark and Find in Block (#559), the copies of its command, its output, both,
//! or the block as Markdown, and
//! Reinput, with or without
//! `sudo`, under Rerun's rule: a command the shell's hook reported, while that shell waits at its
//! prompt.

use std::sync::Arc;
use std::time::SystemTime;

use gpui::{App, ClipboardItem, Context, Entity, Focusable as _, InteractiveElement as _, Window};
use terminal::Terminal;
use terminal_view::{
    MarleyBlockSelection, MarleyFooterContext, MarleyTerminalBlockMenu, TerminalView,
    terminal_panel::TerminalPanel,
};
use ui::{ContextMenu, ContextMenuEntry};
use workspace::Workspace;

use crate::{ClearBlockSelection, NextBlock, PreviousBlock, ReinputBlock, SendBlockToAgent};

/// What a copy item of the block menu puts on the clipboard.
#[derive(Clone, Copy)]
enum Copied {
    Command,
    Output,
    Both,
    Markdown,
}

/// Installs the block keys on every workspace and the block menu on every terminal. [`crate::init`]
/// calls it once, before any window opens.
pub fn init(cx: &mut App) {
    cx.set_global(MarleyBlockSelection::default());
    cx.set_global(MarleyTerminalBlockMenu(Arc::new(block_menu)));
    // A closed terminal's selection goes with it.
    cx.observe_new(
        |view: &mut TerminalView, _, cx: &mut Context<TerminalView>| {
            let terminal = view.terminal().entity_id();
            cx.on_release(move |_, cx| {
                cx.default_global::<MarleyBlockSelection>()
                    .0
                    .remove(&terminal);
            })
            .detach();
        },
    )
    .detach();
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action_renderer(|div, _, _, cx| {
            div.on_action(cx.listener(|workspace, _: &PreviousBlock, window, cx| {
                step(workspace, false, window, cx);
            }))
            .on_action(cx.listener(|workspace, _: &NextBlock, window, cx| {
                step(workspace, true, window, cx);
            }))
            .on_action(
                cx.listener(|workspace, _: &ClearBlockSelection, window, cx| {
                    clear_focused(workspace, window, cx);
                }),
            )
            .on_action(cx.listener(|workspace, _: &ReinputBlock, window, cx| {
                reinput_focused(workspace, window, cx);
            }))
            .on_action(cx.listener(|workspace, _: &SendBlockToAgent, window, cx| {
                send_focused(workspace, window, cx);
            }))
        });
    })
    .detach();
}

/// Moves the focused terminal's selection back or forward a block, as the block keys do. On the
/// alternate screen, which has no blocks, the key goes on to the program.
fn step(workspace: &Workspace, forward: bool, window: &mut Window, cx: &mut Context<Workspace>) {
    let Some(view) = focused_terminal(workspace, window, cx) else {
        cx.propagate();
        return;
    };
    let terminal = view.read(cx).terminal().clone();
    // The content is otherwise the last frame's: a key pressed before the next frame then starts
    // where the one before it went.
    terminal.update(cx, |terminal, cx| terminal.sync(window, cx));
    if terminal
        .read(cx)
        .last_content()
        .mode
        .contains(terminal::Modes::ALT_SCREEN)
    {
        cx.propagate();
        return;
    }
    // The key is the block keys' now, not the program's (#563).
    let (workspace_entity, focus) = (cx.entity(), view.focus_handle(cx));
    let (action, did): (&dyn gpui::Action, _) = if forward {
        (&NextBlock, "moved to the next block")
    } else {
        (&PreviousBlock, "moved to the previous block")
    };
    crate::shortcut_note::taken(action, did, &focus, &workspace_entity, window, cx);
    let count = terminal.read(cx).blocks().len();
    let target = match (MarleyBlockSelection::selected(&terminal, cx), forward) {
        (None, false) => count.checked_sub(1),
        (None, true) => {
            scroll_to_block(&terminal, cx);
            return;
        }
        (Some(index), false) => Some(index.saturating_sub(1)),
        (Some(index), true) => (index + 1 < count).then_some(index + 1),
    };
    match target {
        Some(index) => {
            select(&terminal, index, cx);
            reveal(&terminal, index, cx);
        }
        None => clear(&terminal, cx),
    }
    view.update(cx, |_, cx| cx.notify());
}

/// Selects the block at `index` of `terminal`, until the next input reaches it.
pub(crate) fn select(terminal: &Entity<Terminal>, index: usize, cx: &mut App) {
    let inputs = terminal.read(cx).marley_anchored().inputs();
    cx.default_global::<MarleyBlockSelection>()
        .0
        .insert(terminal.entity_id(), (index, inputs));
}

/// Ends `terminal`'s block selection.
fn clear(terminal: &Entity<Terminal>, cx: &mut App) {
    cx.default_global::<MarleyBlockSelection>()
        .0
        .remove(&terminal.entity_id());
}

/// Scrolls `terminal` so the first line of its block at `index` shows, when it does not.
pub(crate) fn reveal(terminal: &Entity<Terminal>, index: usize, cx: &mut App) {
    let Some(start) = terminal
        .read(cx)
        .blocks()
        .get(index)
        .map(|block| block.prompt_line.unwrap_or(block.output_start))
    else {
        return;
    };
    reveal_line(terminal, start, cx);
}

/// Scrolls `terminal` so the absolute line `start` is its top row, when it does not show (#620).
pub(crate) fn reveal_line(terminal: &Entity<Terminal>, start: u64, cx: &mut App) {
    terminal.update(cx, |terminal, _| {
        let content = terminal.last_content();
        let display_offset = u64::try_from(content.display_offset).unwrap_or(u64::MAX);
        let top = content.marley_screen_top.saturating_sub(display_offset);
        let screen_lines = u64::try_from(content.screen_lines).unwrap_or(u64::MAX);
        if (top..top.saturating_add(screen_lines)).contains(&start) {
            return;
        }
        let history = content.total_lines.saturating_sub(content.screen_lines);
        let above = content.marley_screen_top.saturating_sub(start);
        let offset = usize::try_from(above).unwrap_or(usize::MAX).min(history);
        terminal.scroll_to_bottom();
        terminal.scroll_up_by(offset);
    });
}

/// Scrolls `terminal` to the start of the next block, when there is one below its top.
fn scroll_to_block(terminal: &Entity<Terminal>, cx: &mut App) {
    terminal.update(cx, |terminal, _| {
        let content = terminal.last_content();
        let history = content.total_lines.saturating_sub(content.screen_lines);
        let offset = marley_terminal::block_scroll(
            terminal.blocks(),
            content.marley_screen_top,
            content.display_offset,
            history,
            true,
        );
        if let Some(offset) = offset {
            terminal.scroll_to_bottom();
            terminal.scroll_up_by(offset);
        }
    });
}

/// Ends the focused terminal's block selection, as Escape does while one is selected.
fn clear_focused(workspace: &Workspace, window: &Window, cx: &mut Context<Workspace>) {
    let Some(view) = focused_terminal(workspace, window, cx) else {
        cx.propagate();
        return;
    };
    let terminal = view.read(cx).terminal().clone();
    clear(&terminal, cx);
    view.update(cx, |_, cx| cx.notify());
}

/// Types the focused terminal's selected block's command at the prompt, as `ctrl-shift-i` does.
fn reinput_focused(workspace: &Workspace, window: &Window, cx: &mut Context<Workspace>) {
    let Some(view) = focused_terminal(workspace, window, cx) else {
        cx.propagate();
        return;
    };
    let terminal = view.read(cx).terminal().clone();
    let Some(index) = MarleyBlockSelection::selected(&terminal, cx) else {
        cx.propagate();
        return;
    };
    reinput(&terminal, index, false, cx);
    view.update(cx, |_, cx| cx.notify());
}

/// Sends the focused terminal's selected block to an agent, as `ctrl-shift-enter` does (#555).
fn send_focused(workspace: &Workspace, window: &Window, cx: &mut Context<Workspace>) {
    let Some(view) = focused_terminal(workspace, window, cx) else {
        cx.propagate();
        return;
    };
    let terminal = view.read(cx).terminal().clone();
    let Some(index) = MarleyBlockSelection::selected(&terminal, cx) else {
        cx.propagate();
        return;
    };
    crate::send_block::send(&view, index, window, cx);
}

/// Types Ctrl-U and the command of `terminal`'s block at `index`, with `sudo ` first when asked,
/// and no return, only where Rerun is offered: a command the shell's hook reported, while that
/// shell waits at its prompt.
fn reinput(terminal: &Entity<Terminal>, index: usize, sudo: bool, cx: &mut App) {
    terminal.update(cx, |terminal, _| {
        let line = {
            let anchored = terminal.marley_anchored();
            let Some(block) = anchored.blocks().get(index) else {
                return;
            };
            if !anchored.rerun_offered(block) {
                return;
            }
            let sudo = if sudo { "sudo " } else { "" };
            format!("\u{15}{sudo}{}", block.command)
        };
        terminal.input(line.into_bytes());
    });
}

/// Puts `copied` of `terminal`'s block at `index` on the clipboard.
fn copy(terminal: &Entity<Terminal>, index: usize, copied: Copied, cx: &App) {
    let text = {
        let terminal = terminal.read(cx);
        let Some(block) = terminal.blocks().get(index) else {
            return;
        };
        let output = terminal.block_output(block);
        match copied {
            Copied::Command => block.command.clone(),
            Copied::Output => output.unwrap_or_default(),
            Copied::Both => format!("{}\n{}", block.command, output.unwrap_or_default()),
            Copied::Markdown => {
                let took = terminal.marley_anchored().times(index).and_then(|times| {
                    times
                        .finished
                        .unwrap_or_else(SystemTime::now)
                        .duration_since(times.started)
                        .ok()
                });
                block.markdown(output.as_deref(), took)
            }
        }
    };
    cx.write_to_clipboard(ClipboardItem::new_string(text));
}

/// The hook Zed's terminal menu asks for a right-click on the block at `index`: selects it and
/// adds the Block section.
fn block_menu(
    context: &MarleyFooterContext,
    index: usize,
    menu: ContextMenu,
    _: &mut Window,
    cx: &mut App,
) -> ContextMenu {
    let terminal = context.terminal.clone();
    select(&terminal, index, cx);
    let offered = {
        let anchored = terminal.read(cx).marley_anchored();
        anchored
            .blocks()
            .get(index)
            .is_some_and(|block| anchored.rerun_offered(block))
    };
    let copy_item = |label: &'static str, copied: Copied| {
        let terminal = terminal.clone();
        ContextMenuEntry::new(label).handler(move |_, cx| copy(&terminal, index, copied, cx))
    };
    let reinput_item = |label: &'static str, sudo: bool| {
        let terminal = terminal.clone();
        ContextMenuEntry::new(label)
            .disabled(!offered)
            .handler(move |_, cx| reinput(&terminal, index, sudo, cx))
    };
    let send_item = {
        let view = context.view.clone();
        ContextMenuEntry::new("Send to Agent").handler(move |window, cx| {
            if let Some(view) = view.upgrade() {
                crate::send_block::send(&view, index, window, cx);
            }
        })
    };
    let bookmark_item = {
        let view = context.view.clone();
        let label = if crate::bookmarks::is_bookmarked(&terminal, index, cx) {
            "Remove Bookmark"
        } else {
            "Bookmark"
        };
        ContextMenuEntry::new(label).handler(move |_, cx| {
            if let Some(view) = view.upgrade() {
                crate::bookmarks::toggle(&view, index, cx);
            }
        })
    };
    let find_item = {
        let view = context.view.clone();
        ContextMenuEntry::new("Find in Block").handler(move |window, cx| {
            if let Some(view) = view.upgrade() {
                crate::bookmarks::find_in(&view, index, window, cx);
            }
        })
    };
    // A failed block whose output names a failing place offers to go there (#620).
    let jump_item = crate::failures::first(&terminal, index, cx).map(|_| {
        let view = context.view.clone();
        ContextMenuEntry::new("Jump to First Failure").handler(move |window, cx| {
            if let Some(view) = view.upgrade() {
                crate::failures::jump(&view, index, window, cx);
            }
        })
    });
    let menu = menu.separator().header("Block");
    let menu = match jump_item {
        Some(item) => menu.item(item),
        None => menu,
    };
    menu.item(send_item)
        .item(bookmark_item)
        .item(find_item)
        .item(copy_item("Copy Command", Copied::Command))
        .item(copy_item("Copy Output", Copied::Output))
        .item(copy_item("Copy Both", Copied::Both))
        .item(copy_item("Copy as Markdown", Copied::Markdown))
        .item(reinput_item("Reinput", false))
        .item(reinput_item("Reinput with sudo", true))
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
