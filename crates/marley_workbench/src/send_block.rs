//! A terminal's block sent to a CLI agent (#555): Send to Agent in the Block menu,
//! `ctrl-shift-enter` on a selected block, and Ask the agent on the newest failed block.
//!
//! A block with long output goes as a reference the agent reads with Marley's `terminal_read`
//! tool; one with short output, or output no longer in the scrollback, goes inline as its
//! Markdown. Both pass the agents' redactor. The target follows a selection's send (#549): the one
//! agent terminal, the picker for several, a toast for none, and never the block's own terminal.

use std::time::SystemTime;

use gpui::{AnyElement, App, Entity, Window};
use marley_terminal::BlockState;
use terminal::Terminal;
use terminal_view::TerminalView;
use ui::{Button, ButtonStyle, LabelSize, TintColor, prelude::*};

use crate::agent_bar::agent_in;
use crate::send_selection::{self, OnPick, Pick, Row, Target, TargetPicker};

/// The most output lines that still go inline.
const INLINE_LINES: usize = 32;

/// The most output bytes that still go inline.
const INLINE_BYTES: usize = 4096;

/// What an agent gets for the block at `index` of `view`'s terminal, redacted: its Markdown when
/// the output is short or gone, else a reference to `terminal_read` with the id `terminal_list`
/// gives the terminal.
fn text_for(view: &Entity<TerminalView>, index: usize, cx: &App) -> Option<String> {
    let terminal = view.read(cx).terminal().read(cx);
    let block = terminal.blocks().get(index)?;
    let redactor = crate::mcp::agent_redactor(cx);
    let redact = |text: &str| {
        redactor
            .as_ref()
            .map_or_else(|| text.to_string(), |redactor| redactor.redact(text).text)
    };
    // Redacted before the size is judged, so the size is that of what leaves.
    let output = terminal.block_output(block).map(|output| redact(&output));
    let short = output.as_ref().is_none_or(|output| {
        output.lines().count() <= INLINE_LINES && output.len() <= INLINE_BYTES
    });
    if short {
        let took = terminal.marley_anchored().times(index).and_then(|times| {
            times
                .finished
                .unwrap_or_else(SystemTime::now)
                .duration_since(times.started)
                .ok()
        });
        return Some(redact(&block.markdown(output.as_deref(), took)));
    }
    let id = view.entity_id().as_u64();
    let outcome = match (block.state, block.exit_code.0) {
        (BlockState::Finished, Some(code)) => format!("exit {code}"),
        (BlockState::Finished, None) => "finished".to_string(),
        (BlockState::Running | BlockState::Pending, _) => "running".to_string(),
    };
    Some(format!(
        "[terminal {id} block {index}: {}, {outcome}; terminal_read terminal={id} block={index}] ",
        redact(&block.command)
    ))
}

/// The agents a block of `view` can go to: every agent terminal of its window but its own.
fn targets(view: &Entity<TerminalView>, cx: &App) -> Vec<Target> {
    let Some(workspace) = view.read(cx).marley_workspace().upgrade() else {
        return Vec::new();
    };
    let own = view.entity_id();
    send_selection::agent_targets(workspace.read(cx), cx)
        .into_iter()
        .filter(|target| target.view.entity_id() != own)
        .collect()
}

/// Sends the block at `index` of `view`'s terminal to an agent: the one there is, the one picked
/// when there are several, or a toast when there is none. Deferred, since the key's action runs
/// while the workspace this reads and updates is being updated.
pub(crate) fn send(view: &Entity<TerminalView>, index: usize, window: &Window, cx: &mut App) {
    let view = view.clone();
    window.defer(cx, move |window, cx| send_now(&view, index, window, cx));
}

fn send_now(view: &Entity<TerminalView>, index: usize, window: &mut Window, cx: &mut App) {
    let Some(workspace) = view.read(cx).marley_workspace().upgrade() else {
        return;
    };
    let Some(text) = text_for(view, index, cx) else {
        return;
    };
    let mut targets = targets(view, cx);
    let window_handle = window.window_handle();
    match targets.len() {
        0 => workspace.update(cx, |workspace, cx| {
            send_selection::show_toast(
                workspace,
                "No agent runs in another terminal of this window.",
                cx,
            );
        }),
        1 => send_selection::send_text(targets.remove(0), text, window_handle, cx),
        _ => {
            let rows = targets
                .into_iter()
                .map(|target| Row {
                    label: target.label(),
                    pick: Pick::Agent(target),
                })
                .collect();
            let on_pick: OnPick = Box::new(move |pick, window, cx| {
                if let Pick::Agent(target) = pick {
                    send_selection::send_text(target, text.clone(), window, cx);
                }
            });
            workspace.update(cx, |workspace, cx| {
                workspace.toggle_modal(window, cx, |window, cx| {
                    TargetPicker::new(
                        rows,
                        "Send the block to…",
                        on_pick,
                        window_handle,
                        window,
                        cx,
                    )
                });
            });
        }
    }
}

/// The chip for each block starting on screen, part of the element's `MarleyBlockChip` hook
/// (`bookmarks`): Ask the agent for the newest block when it failed, the shell waits at its prompt
/// with no agent in front, and another terminal of the window runs an agent.
pub(crate) fn chip(
    view: &Entity<TerminalView>,
    terminal: &Entity<Terminal>,
    index: usize,
    cx: &App,
) -> Option<AnyElement> {
    let terminal = terminal.read(cx);
    let anchored = terminal.marley_anchored();
    let block = anchored
        .blocks()
        .last()
        .filter(|block| block.index == index)?;
    let failed =
        block.state == BlockState::Finished && block.exit_code.0.is_some_and(|code| code != 0);
    if !failed
        || !anchored.at_prompt()
        || agent_in(terminal).is_some()
        || targets(view, cx).is_empty()
    {
        return None;
    }
    let view = view.clone();
    Some(
        Button::new(("marley-ask-agent", index), "Ask the agent")
            .style(ButtonStyle::Tinted(TintColor::Accent))
            .label_size(LabelSize::XSmall)
            .on_click(move |_, window, cx| {
                cx.stop_propagation();
                send(&view, index, window, cx);
            })
            .into_any_element(),
    )
}
