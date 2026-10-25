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
use std::time::SystemTime;

use gpui::{AnyElement, App, Entity, Focusable, MouseButton};
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
/// the buffer font, cut to fit, "is above you" with the block's state, and Jump to it, over the
/// terminal's background (#917). None while the setting is off.
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
    // The block's state in words (#917): its time ticks with the view's repaint while it runs
    // (#916).
    let times = terminal.read(cx).marley_anchored().times(index);
    let state = match (block.state, block.exit_code.0) {
        (BlockState::Pending | BlockState::Running, _) => {
            let so_far = times
                .and_then(|times| SystemTime::now().duration_since(times.started).ok())
                .unwrap_or_default();
            Label::new(format!(
                "running \u{b7} {}",
                marley_terminal::duration_label(so_far)
            ))
            .color(Color::Warning)
        }
        (BlockState::Finished, Some(code)) if code != 0 => {
            Label::new(format!("exit {code}")).color(Color::Error)
        }
        (BlockState::Finished, _) => {
            let took = times
                .and_then(|times| times.finished?.duration_since(times.started).ok())
                .map(marley_terminal::duration_label);
            Label::new(took.map_or_else(|| "done".to_string(), |took| format!("done in {took}")))
                .color(Color::Muted)
        }
    };
    let colors = cx.theme().colors();
    let focus_handle = view.focus_handle(cx);
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
                    .flex_shrink(1.)
                    .min_w_0()
                    .child(Label::new(command).buffer_font(cx).single_line().truncate()),
            )
            .child(
                h_flex()
                    .flex_none()
                    .gap_1()
                    .child(
                        Label::new("is above you \u{b7}")
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                    .child(state.size(LabelSize::Small)),
            )
            .child(div().flex_1())
            // The whole row jumps; the words say so (#917).
            .child(
                Label::new("Jump to it")
                    .size(LabelSize::Small)
                    .color(Color::Accent),
            )
            // The press jumps, as Zed's sticky scroll headers do, and the press and its release
            // are the header's: the terminal under it starts no selection, reports no click to
            // its program and opens no link menu. The press still focuses the terminal, as one on
            // the grid does, or a header in another pane would leave the focus where it was.
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                window.focus(&focus_handle, cx);
                cx.stop_propagation();
                crate::blocks::reveal(&terminal, index, cx);
                view.update(cx, |_, cx| cx.notify());
            })
            .on_mouse_up(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .into_any_element(),
    )
}
