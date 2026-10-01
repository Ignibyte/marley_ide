//! Native block headers (#628): a block's prompt rows drawn as Marley's header, the command in the
//! buffer font, in place of the prompt the shell drew.
//!
//! The terminal's element finds each verified block's prompt rows on screen
//! (`marley_terminal::prompt_rows`) and asks the `MarleyBlockHeader` hook this module sets for the
//! header; when it gets one, it leaves those rows' cells out and lays the header over the same
//! rows, so no row moves, and the block's pill and hover actions draw over it.
//! `marley.block_headers` turns it on.

use std::sync::Arc;

use gpui::{AnyElement, App, Entity, MouseButton, Pixels};
use settings::Settings as _;
use terminal::Terminal;
use terminal_view::{MarleyBlockHeader, TerminalView};
use ui::prelude::*;

use crate::{BlockHeaders, MarleySettings};

/// Sets the header's hook; [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    cx.set_global(MarleyBlockHeader(Arc::new(header)));
}

/// The header over the prompt rows of the block at `index`: the command's first line on the first
/// row, `line_height` tall, the rows under it empty. None while the setting is off.
fn header(
    _view: &Entity<TerminalView>,
    terminal: &Entity<Terminal>,
    index: usize,
    line_height: Pixels,
    cx: &App,
) -> Option<AnyElement> {
    if MarleySettings::get_global(cx).block_headers != BlockHeaders::Native {
        return None;
    }
    let block = terminal.read(cx).blocks().get(index)?;
    let mut lines = block.command.trim().lines();
    let first = lines.next().unwrap_or_default();
    let command = if lines.next().is_some() {
        format!("{first} …")
    } else {
        first.to_string()
    };
    Some(
        v_flex()
            .id(("marley-block-header", index))
            .size_full()
            .child(
                h_flex()
                    .h(line_height)
                    .w_full()
                    .min_w_0()
                    .child(Label::new(command).buffer_font(cx).single_line().truncate()),
            )
            // The header stands for the prompt's text, which is not there to select: a press on
            // it and its release are the header's, so the terminal under it starts no selection
            // and reports no click to its program.
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_mouse_up(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .into_any_element(),
    )
}
