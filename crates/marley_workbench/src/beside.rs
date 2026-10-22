//! An agent beside a terminal (#865).
//!
//! A failed block's Ask Marley opens Marley's own agent in the pane right of the terminal, with the
//! block in its message box, and remembers which terminal the thread came from. When that thread's
//! answer proposes a command in a fenced block, the answer ends with Apply (through #864's
//! `agent_ui::MarleyThreadLook`), which types the command into that terminal and runs it.

use acp_thread::{AcpThread, AgentThreadEntry};
use gpui::{
    AnyElement, AnyWindowHandle, App, Entity, EntityId, Focusable as _, Global, WeakEntity,
};
use terminal_view::TerminalView;
use ui::{Tooltip, prelude::*};
use util::ResultExt as _;

use crate::thread_tab::ThreadTab;

/// Each thread tab Ask Marley opened, with the terminal its block came from and that terminal's
/// window.
#[derive(Default)]
struct Links(
    Vec<(
        WeakEntity<ThreadTab>,
        WeakEntity<TerminalView>,
        AnyWindowHandle,
    )>,
);

impl Global for Links {}

/// Remembers that `tab` was asked about a block of `terminal`, in `window`.
pub fn link(
    tab: &Entity<ThreadTab>,
    terminal: &Entity<TerminalView>,
    window: AnyWindowHandle,
    cx: &mut App,
) {
    let links = &mut cx.default_global::<Links>().0;
    links.retain(|(tab, terminal, _)| tab.upgrade().is_some() && terminal.upgrade().is_some());
    links.push((tab.downgrade(), terminal.downgrade(), window));
}

/// The terminal (and its window) the thread view `view` was asked from, when Ask Marley opened
/// it. The view is being drawn, so it is matched by its id and never read.
fn terminal_of(view: EntityId, cx: &App) -> Option<(Entity<TerminalView>, AnyWindowHandle)> {
    cx.try_global::<Links>()?
        .0
        .iter()
        .find_map(|(tab, terminal, window)| {
            let tab = tab.upgrade()?;
            let root = tab.read(cx).conversation().read(cx).root_thread_view()?;
            if root.entity_id() != view {
                return None;
            }
            Some((terminal.upgrade()?, *window))
        })
}

/// The command the thread's newest answer proposes (#865, D2): its first fenced block of one
/// line.
fn proposed(thread: &AcpThread, cx: &App) -> Option<String> {
    let answer = thread
        .entries()
        .iter()
        .rev()
        .find_map(|entry| match entry {
            AgentThreadEntry::AssistantMessage(message) => Some(message.to_markdown(cx)),
            _ => None,
        })?;
    fenced_command(&answer)
}

/// The first fenced block of `markdown` when it holds one line, without its fences.
fn fenced_command(markdown: &str) -> Option<String> {
    let start = markdown.find("```")?;
    let rest = markdown.get(start + 3..)?;
    let body_start = rest.find('\n')? + 1;
    let body = rest.get(body_start..)?;
    let end = body.find("```")?;
    let mut lines = body
        .get(..end)?
        .lines()
        .filter(|line| !line.trim().is_empty());
    let command = lines.next()?.trim().to_string();
    lines.next().is_none().then_some(command)
}

/// Apply at the end of an answer of a thread Ask Marley opened, when the answer proposes a
/// command: the command typed into the block's terminal with a Return, the terminal shown.
pub fn apply_chip(view: EntityId, thread: &Entity<AcpThread>, cx: &App) -> Option<AnyElement> {
    let (terminal, window) = terminal_of(view, cx)?;
    let command = proposed(thread.read(cx), cx)?;
    Some(
        Button::new("marley-apply-command", format!("Apply: {command}"))
            .style(ButtonStyle::Tinted(ui::TintColor::Accent))
            .label_size(LabelSize::Small)
            .tooltip(Tooltip::text("Run it in the terminal the block came from"))
            .on_click(move |_, _, cx| {
                let line = format!("{command}\r").into_bytes();
                let inner = terminal.read(cx).terminal().clone();
                inner.update(cx, |terminal, _| terminal.input(line));
                let view = terminal.clone();
                window
                    .update(cx, |_, window, cx| {
                        crate::browser::reveal_terminal(&view, window, cx);
                        window.focus(&view.focus_handle(cx), cx);
                    })
                    .log_err();
            })
            .into_any_element(),
    )
}
