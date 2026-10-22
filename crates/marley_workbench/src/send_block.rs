//! A terminal's block sent to a CLI agent (#555): Send to Agent in the Block menu,
//! `ctrl-shift-enter` on a selected block, and Ask the agent on the newest failed block.
//!
//! A block with long output goes as a reference the agent reads with Marley's `terminal_read`
//! tool; one with short output, or output no longer in the scrollback, goes inline as its
//! Markdown. Both pass the agents' redactor. The target follows a selection's send (#549): the one
//! agent terminal, the picker for several, a toast for none, and never the block's own terminal.

use std::time::SystemTime;

use gpui::{AnyElement, App, Entity, Window};
use marley_agent::AgentKind;
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

/// What a block goes to an agent with (#826).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Request {
    /// The block alone, as the key and the Block menu send it.
    Bare,
    /// Fix what failed.
    Fix,
    /// Say why it failed, changing nothing.
    Explain,
}

impl Request {
    const fn words(self) -> Option<&'static str> {
        match self {
            Self::Bare => None,
            Self::Fix => Some("This command failed. Fix it:"),
            Self::Explain => Some("Explain why this command failed. Change nothing:"),
        }
    }
}

/// Sends the block at `index` of `view`'s terminal to an agent with `request`: the one there is,
/// the one picked when there are several, and with none a toast for a bare block, or a new Claude
/// Code for a request (#826). Deferred, since the key's action runs while the workspace this reads
/// and updates is being updated.
pub(crate) fn send(
    view: &Entity<TerminalView>,
    index: usize,
    request: Request,
    window: &Window,
    cx: &mut App,
) {
    let view = view.clone();
    window.defer(cx, move |window, cx| {
        send_now(&view, index, request, window, cx);
    });
}

fn send_now(
    view: &Entity<TerminalView>,
    index: usize,
    request: Request,
    window: &mut Window,
    cx: &mut App,
) {
    let Some(workspace) = view.read(cx).marley_workspace().upgrade() else {
        return;
    };
    let Some(block) = text_for(view, index, cx) else {
        return;
    };
    let text = request
        .words()
        .map_or_else(|| block.clone(), |words| format!("{words}\n\n{block}"));
    let mut targets = targets(view, cx);
    let window_handle = window.window_handle();
    match targets.len() {
        0 if request == Request::Bare => workspace.update(cx, |workspace, cx| {
            send_selection::show_toast(
                workspace,
                "No agent runs in another terminal of this window.",
                cx,
            );
        }),
        0 => workspace.update(cx, |workspace, cx| {
            let started = crate::agents::start_cli_with_prompt(
                workspace,
                AgentKind::Claude,
                &text,
                None,
                window,
                cx,
            );
            cx.spawn(async move |_, _| {
                if started.await.is_none() {
                    log::warn!("send block: Claude Code did not start for the block");
                }
            })
            .detach();
        }),
        1 => send_selection::send_text(targets.remove(0), text, false, window_handle, cx),
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
                    send_selection::send_text(target, text.clone(), false, window, cx);
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
/// with no agent in front, and another terminal of the window runs an agent; or, for a request
/// the shell could not run (127), with its words, whatever runs (#557).
pub(crate) fn chip(
    view: &Entity<TerminalView>,
    terminal: &Entity<Terminal>,
    index: usize,
    cx: &App,
) -> Option<AnyElement> {
    let terminal = terminal.read(cx);
    // A request the shell could not run (127) asks the agent with its own words, even with no
    // agent running (#557).
    if agent_in(terminal).is_none()
        && let Some(command) = crate::english::asks_on_127(terminal, index, cx)
    {
        let view = view.clone();
        return Some(
            Button::new(("marley-ask-agent", index), "Ask the agent")
                .style(ButtonStyle::Tinted(TintColor::Accent))
                .label_size(LabelSize::XSmall)
                .on_click(move |_, window, cx| {
                    cx.stop_propagation();
                    crate::english::ask_later(&view, command.clone(), window, cx);
                })
                .into_any_element(),
        );
    }
    let anchored = terminal.marley_anchored();
    // A block from before a restart is history, which no agent is asked about (#764).
    let block = anchored
        .live()
        .last()
        .filter(|block| block.index == index)?;
    let failed =
        block.state == BlockState::Finished && block.exit_code.0.is_some_and(|code| code != 0);
    if !failed || !anchored.at_prompt() || agent_in(terminal).is_some() {
        return None;
    }
    // Ask an agent to fix it, or to explain it; with no agent running, Claude Code starts with it
    // (#826). Or ask Marley's own agent beside the terminal (#865).
    let (fix_view, explain_view, beside_view) = (view.clone(), view.clone(), view.clone());
    Some(
        h_flex()
            .gap_1()
            .child(
                Button::new(("marley-ask-agent", index), "Ask an agent")
                    .style(ButtonStyle::Tinted(TintColor::Accent))
                    .label_size(LabelSize::XSmall)
                    .on_click(move |_, window, cx| {
                        cx.stop_propagation();
                        send(&fix_view, index, Request::Fix, window, cx);
                    }),
            )
            .child(
                Button::new(("marley-explain-block", index), "Explain")
                    .style(ButtonStyle::Subtle)
                    .label_size(LabelSize::XSmall)
                    .on_click(move |_, window, cx| {
                        cx.stop_propagation();
                        send(&explain_view, index, Request::Explain, window, cx);
                    }),
            )
            .child(
                Button::new(("marley-ask-beside", index), "Ask Marley")
                    .style(ButtonStyle::Subtle)
                    .label_size(LabelSize::XSmall)
                    .tooltip(ui::Tooltip::text(
                        "Open Marley's agent beside this terminal with the block",
                    ))
                    .on_click(move |_, window, cx| {
                        cx.stop_propagation();
                        ask_beside(&beside_view, index, window, cx);
                    }),
            )
            .into_any_element(),
    )
}

/// Opens Marley's own agent beside `view` with the block at `index` and "Fix it" in its message box,
/// working in the block's folder, and links the thread to the terminal for Apply (#865).
/// Deferred, as `send` is: the click runs while the workspace is being updated.
fn ask_beside(view: &Entity<TerminalView>, index: usize, window: &Window, cx: &mut App) {
    let view = view.clone();
    window.defer(cx, move |window, cx| {
        let Some(workspace) = view.read(cx).marley_workspace().upgrade() else {
            return;
        };
        let Some(block) = text_for(&view, index, cx) else {
            return;
        };
        let folder = {
            let terminal = view.read(cx).terminal().read(cx);
            terminal
                .blocks()
                .get(index)
                .and_then(|block| block.prompt.pwd.as_deref().map(std::path::PathBuf::from))
                .or_else(|| terminal.working_directory())
        };
        let Some(folder) = folder else {
            return;
        };
        let text = format!(
            "This command failed. Fix it, and give the command to run in a code block:\n\n{block}"
        );
        let window_handle = window.window_handle();
        let tab = workspace.update(cx, |workspace, cx| {
            crate::assistant::ask_beside(workspace, folder, text, window, cx)
        });
        if let Some(tab) = tab {
            crate::beside::link(&tab, &view, window_handle, cx);
        }
    });
}

/// How long the block at `index` took, once it finished (#826).
pub(crate) fn duration(terminal: &Entity<Terminal>, index: usize, cx: &App) -> Option<AnyElement> {
    let anchored = terminal.read(cx).marley_anchored();
    let block = anchored
        .live()
        .iter()
        .find(|block| block.index == index)
        .filter(|block| block.state == BlockState::Finished)?;
    let times = anchored.times(block.index)?;
    let took = times.finished?.duration_since(times.started).ok()?;
    Some(
        Label::new(marley_terminal::duration_label(took))
            .size(LabelSize::XSmall)
            .color(Color::Muted)
            .into_any_element(),
    )
}
