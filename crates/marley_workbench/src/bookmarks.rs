//! Bookmarks on blocks and find within a block (#559).
//!
//! A bookmark marks a block for the session: `ctrl-shift-b` toggles the focused terminal's
//! selected block, else its newest block in view, and a hovered block's Bookmark button or its
//! Block menu toggles that one. A marked block shows the bookmark before its pill and a tick at
//! the terminal's right edge; `alt-up` and `alt-down` scroll to the marked block before or after
//! the viewport's top, and go on to the program in a terminal with no bookmark. Find in Block holds
//! Zed's search bar to one block until the bar closes. The marks live in
//! `terminal_view::MarleyBlockMarks`, which the view and its element read.
//!
//! The element asks one hook for a block's chip and one for its extra hover buttons, so this module
//! sets both: the chip is the bookmark and then `send_block`'s Ask the agent, and the buttons are
//! Bookmark, Find, and then `workflows`' Save as Workflow.

use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, Focusable as _, Window};
use marley_terminal::AnchoredBlock;
use terminal::Terminal;
use terminal_view::{MarleyBlockChip, MarleyBlockExtras, MarleyBlockMarks, TerminalView};
use ui::{IconButton, IconName, IconSize, Tooltip, prelude::*};
use workspace::{Workspace, searchable::SearchEvent};

use crate::blocks::focused_terminal;
use crate::{FindInBlock, NextBookmark, PreviousBookmark, ToggleBookmark};

/// Installs the marks, the element's chip and button hooks, and the actions on every workspace;
/// [`crate::init`] calls it once, before any window opens.
pub fn init(cx: &mut App) {
    cx.set_global(MarleyBlockMarks::default());
    cx.set_global(MarleyBlockChip(Arc::new(chip)));
    cx.set_global(MarleyBlockExtras(Arc::new(block_buttons)));
    // A terminal's marks go with it, so a task's rerun in a new terminal starts with none.
    cx.observe_new(|_: &mut Terminal, _, cx: &mut Context<Terminal>| {
        let terminal = cx.entity_id();
        cx.on_release(move |_, cx| {
            let marks = cx.default_global::<MarleyBlockMarks>();
            marks.bookmarks.remove(&terminal);
            marks.search_scopes.remove(&terminal);
        })
        .detach();
    })
    .detach();
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action_renderer(|div, _, _, cx| {
            div.on_action(cx.listener(|workspace, _: &ToggleBookmark, window, cx| {
                let Some((view, block)) = focused_block(workspace, window, cx) else {
                    cx.propagate();
                    return;
                };
                toggle(&view, block, cx);
            }))
            .on_action(cx.listener(|workspace, _: &PreviousBookmark, window, cx| {
                jump(workspace, false, window, cx);
            }))
            .on_action(cx.listener(|workspace, _: &NextBookmark, window, cx| {
                jump(workspace, true, window, cx);
            }))
            .on_action(cx.listener(|workspace, _: &FindInBlock, window, cx| {
                let Some((view, block)) = focused_block(workspace, window, cx) else {
                    cx.propagate();
                    return;
                };
                find_in(&view, block, window, cx);
            }))
        });
    })
    .detach();
}

/// The focused terminal view and the block a key acts on: its selected block, else its newest
/// block in view.
fn focused_block(
    workspace: &Workspace,
    window: &Window,
    cx: &App,
) -> Option<(Entity<TerminalView>, usize)> {
    let view = focused_terminal(workspace, window, cx)?;
    let block = crate::block_filter::block_to_filter(&view, cx)?;
    Some((view, block))
}

/// Whether `terminal`'s block at `index` is bookmarked.
pub(crate) fn is_bookmarked(terminal: &Entity<Terminal>, index: usize, cx: &App) -> bool {
    cx.try_global::<MarleyBlockMarks>()
        .and_then(|marks| marks.bookmarks.get(&terminal.entity_id()))
        .is_some_and(|marked| marked.contains(&index))
}

/// Bookmarks the block at `index` of `view`'s terminal, or removes its bookmark.
pub(crate) fn toggle(view: &Entity<TerminalView>, index: usize, cx: &mut App) {
    let terminal = view.read(cx).terminal().entity_id();
    let bookmarks = &mut cx.default_global::<MarleyBlockMarks>().bookmarks;
    let marked = bookmarks.entry(terminal).or_default();
    if !marked.remove(&index) {
        marked.insert(index);
    }
    if marked.is_empty() {
        bookmarks.remove(&terminal);
    }
    view.update(cx, |_, cx| cx.notify());
}

/// Scrolls the focused terminal so the bookmarked block before or after its viewport's top starts
/// at the top. With no bookmark, or on the alternate screen, the key goes on to the program.
fn jump(workspace: &Workspace, forward: bool, window: &mut Window, cx: &mut Context<Workspace>) {
    let Some(view) = focused_terminal(workspace, window, cx) else {
        cx.propagate();
        return;
    };
    let terminal = view.read(cx).terminal().clone();
    let marked = MarleyBlockMarks::bookmarked(&terminal, cx);
    if marked.is_empty() {
        cx.propagate();
        return;
    }
    // The content is otherwise the last frame's, as the block keys find.
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
    terminal.update(cx, |terminal, _| {
        let blocks: Vec<AnchoredBlock> = terminal
            .blocks()
            .iter()
            .filter(|block| marked.contains(&block.index))
            .cloned()
            .collect();
        let content = terminal.last_content();
        let history = content.total_lines.saturating_sub(content.screen_lines);
        let offset = marley_terminal::block_scroll(
            &blocks,
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
    view.update(cx, |_, cx| cx.notify());
}

/// Holds `view`'s search to its block at `index` and opens the search bar with the focus in its
/// query, or, when the search is held to that block already, searches the whole terminal again.
/// An open bar searches again at once. It runs after the current update, since the key's action
/// runs while the workspace is being updated.
pub(crate) fn find_in(view: &Entity<TerminalView>, index: usize, window: &Window, cx: &mut App) {
    let view = view.clone();
    window.defer(cx, move |window, cx| {
        let terminal = view.read(cx).terminal().entity_id();
        let scopes = &mut cx.default_global::<MarleyBlockMarks>().search_scopes;
        let held = scopes.get(&terminal) == Some(&index);
        if held {
            scopes.remove(&terminal);
        } else {
            scopes.insert(terminal, index);
            view.focus_handle(cx).dispatch_action(
                &zed_actions::buffer_search::Deploy::find(),
                window,
                cx,
            );
        }
        view.update(cx, |_, cx| {
            cx.emit(SearchEvent::MatchesInvalidated);
            cx.notify();
        });
    });
}

/// The element's chip hook: the bookmark on a marked block, then Ask the agent where
/// `send_block` offers it.
fn chip(
    view: &Entity<TerminalView>,
    terminal: &Entity<Terminal>,
    index: usize,
    cx: &App,
) -> Option<AnyElement> {
    let ask = crate::send_block::chip(view, terminal, index, cx);
    if !is_bookmarked(terminal, index, cx) {
        return ask;
    }
    Some(
        h_flex()
            .gap_1()
            .child(
                Icon::new(IconName::Bookmark)
                    .size(IconSize::XSmall)
                    .color(Color::Accent),
            )
            .children(ask)
            .into_any_element(),
    )
}

/// The element's hook for a block's extra hover buttons: Bookmark and Find, then Save as Workflow
/// where `workflows` offers it.
fn block_buttons(
    view: &Entity<TerminalView>,
    terminal: &Entity<Terminal>,
    index: usize,
    cx: &App,
) -> Vec<AnyElement> {
    let marked = is_bookmarked(terminal, index, cx);
    let bookmark = {
        let view = view.clone();
        IconButton::new(("marley-block-bookmark", index), IconName::Bookmark)
            .icon_size(IconSize::XSmall)
            .toggle_state(marked)
            .tooltip(Tooltip::text(if marked {
                "Remove Bookmark"
            } else {
                "Bookmark"
            }))
            .on_click(move |_, _, cx| toggle(&view, index, cx))
            .into_any_element()
    };
    let find = {
        let view = view.clone();
        IconButton::new(("marley-block-find", index), IconName::MagnifyingGlass)
            .icon_size(IconSize::XSmall)
            .tooltip(Tooltip::text("Find in Block"))
            .on_click(move |_, window, cx| find_in(&view, index, window, cx))
            .into_any_element()
    };
    let mut buttons = vec![bookmark, find];
    buttons.extend(crate::workflows::block_buttons(view, terminal, index, cx));
    buttons
}
