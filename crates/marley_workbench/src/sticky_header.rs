//! The sticky command header (#529).
//!
//! While a terminal is scrolled back into a block whose first row is above the view, the block's
//! command is pinned over the top row, and a click on it scrolls to the block's start, as Warp's
//! header does.
//!
//! The terminal's element picks the block (`marley_terminal::sticky_block`) and asks the
//! `MarleyStickyHeader` hook this module sets for the row; `marley.sticky_command_header` turns it
//! off.

use std::sync::Arc;

use gpui::{AnyElement, App, Entity, MouseButton};
use marley_terminal::BlockState;
use settings::Settings as _;
use terminal::Terminal;
use terminal_view::{MarleyStickyHeader, TerminalView};
use ui::prelude::*;

use crate::{BlockHeaders, MarleySettings};

/// Sets the header's hook; [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    cx.set_global(MarleyStickyHeader(Arc::new(header)));
}

/// The row pinned over the view's top row for its block at `index`: the command's first line in
/// the buffer font, cut to fit, the block's state, and an arrow, over the terminal's background.
/// None while the setting is off.
fn header(
    view: &Entity<TerminalView>,
    terminal: &Entity<Terminal>,
    index: usize,
    cx: &App,
) -> Option<AnyElement> {
    if !MarleySettings::get_global(cx).sticky_command_header {
        return None;
    }
    let block = terminal.read(cx).blocks().get(index)?;
    let mut lines = block.command.trim().lines();
    let first = lines.next().unwrap_or_default();
    // With Marley's headers on, the pinned row reads as the header it stands for (#629).
    let prompt = match MarleySettings::get_global(cx).block_headers {
        BlockHeaders::Native => "",
        BlockHeaders::ShellPrompt => "$ ",
    };
    let command = if lines.next().is_some() {
        format!("{prompt}{first} …")
    } else {
        format!("{prompt}{first}")
    };
    let state = match (block.state, block.exit_code.0) {
        (BlockState::Pending | BlockState::Running, _) => Label::new("running")
            .size(LabelSize::Small)
            .color(Color::Muted)
            .into_any_element(),
        (BlockState::Finished, Some(0)) => Icon::new(IconName::Check)
            .size(IconSize::Small)
            .color(Color::Success)
            .into_any_element(),
        (BlockState::Finished, Some(code)) => Label::new(format!("exit {code}"))
            .size(LabelSize::Small)
            .color(Color::Error)
            .into_any_element(),
        (BlockState::Finished, None) => gpui::Empty.into_any_element(),
    };
    let colors = cx.theme().colors();
    let (view, terminal) = (view.clone(), terminal.clone());
    Some(
        h_flex()
            .id(("marley-sticky-header", index))
            .size_full()
            .px_1()
            .gap_2()
            .bg(colors.terminal_background)
            .border_b_1()
            .border_color(colors.border)
            .cursor_pointer()
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(Label::new(command).buffer_font(cx).single_line().truncate()),
            )
            .child(state)
            .child(
                Icon::new(IconName::ArrowUp)
                    .size(IconSize::Small)
                    .color(Color::Muted),
            )
            // The press jumps, as Zed's sticky scroll headers do, and the press and its release
            // are the header's: the terminal under it starts no selection, reports no click to
            // its program and opens no link menu.
            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                cx.stop_propagation();
                crate::blocks::reveal(&terminal, index, cx);
                view.update(cx, |_, cx| cx.notify());
            })
            .on_mouse_up(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .into_any_element(),
    )
}
