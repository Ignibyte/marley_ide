//! Runnable commands in the Markdown preview (#530).
//!
//! A shell code block in Zed's Markdown preview, one whose fence names `sh`, `shell`, `bash`,
//! `zsh` or `fish` or no language, shows Insert in Terminal beside Copy. A click puts the block's
//! text at the prompt of the terminal the focus entered last, without running it, and brings that
//! terminal forward with the focus, so Enter runs it. Nothing is typed while the terminal's shell
//! is not at its prompt, where a program would take the text, nor several lines without the
//! shell's bracketed paste, which would run them one by one; a toast in the preview's workspace
//! says why.

use std::sync::Arc;

use gpui::{AnyElement, App, Focusable as _, WeakEntity, Window};
use markdown::parser::CodeBlockKind;
use markdown_preview::markdown_preview_view::MarleyCodeBlockAction;
use marley_terminal::BlockState;
use terminal::Modes;
use ui::{IconButton, IconName, IconSize, Tooltip, prelude::*};
use util::ResultExt as _;
use workspace::Workspace;

/// The fence languages whose blocks are shell commands, Warp's list less its own tag.
const SHELL_LANGUAGES: [&str; 5] = ["sh", "shell", "bash", "zsh", "fish"];

/// Sets the preview's code-block hook; [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    cx.set_global(MarleyCodeBlockAction(Arc::new(button)));
}

/// Whether a code block of `kind` holds shell commands: a fence naming a shell, by the info
/// string's first word, or naming nothing.
fn is_shell(kind: &CodeBlockKind) -> bool {
    match kind {
        CodeBlockKind::Fenced => true,
        CodeBlockKind::FencedLang(info) => info
            .split_whitespace()
            .next()
            .is_some_and(|name| SHELL_LANGUAGES.contains(&name)),
        CodeBlockKind::Indented | CodeBlockKind::FencedSrc(_) => false,
    }
}

/// The preview's hook: Insert in Terminal on a shell block with something in it.
fn button(
    workspace: &WeakEntity<Workspace>,
    kind: &CodeBlockKind,
    code: &str,
    _: &App,
) -> Option<AnyElement> {
    if !is_shell(kind) || code.trim().is_empty() {
        return None;
    }
    let (workspace, code) = (workspace.clone(), code.to_string());
    Some(
        IconButton::new("marley-insert-in-terminal", IconName::Terminal)
            .icon_size(IconSize::Small)
            .tooltip(Tooltip::text("Insert in Terminal"))
            .on_click(move |_, window, cx| insert(&workspace, &code, window, cx))
            .into_any_element(),
    )
}

/// Puts `code` at the prompt of the terminal the focus entered last, unrun, after Ctrl-U clears
/// the line, and brings that terminal forward with the focus; or says in a toast why not.
fn insert(workspace: &WeakEntity<Workspace>, code: &str, window: &Window, cx: &mut App) {
    let Some((view, terminal_window)) = crate::browser::last_terminal_with_window(cx) else {
        toast(
            workspace,
            "No terminal to insert into: click in one, then try again.",
            cx,
        );
        return;
    };
    let text = code.trim_end_matches('\n').to_string();
    let refusal = {
        let terminal = view.read(cx).terminal().read(cx);
        if !terminal.marley_anchored().at_prompt() {
            let running = terminal
                .blocks()
                .last()
                .filter(|block| block.state != BlockState::Finished)
                .map(|block| block.command.trim().to_string())
                .or_else(|| terminal.marley_foreground_argv().map(|argv| argv.join(" ")));
            Some(running.map_or_else(
                || "That terminal is not at its shell's prompt: nothing was typed.".to_string(),
                |command| format!("That terminal is running {command}: nothing was typed."),
            ))
        } else if text.contains('\n')
            && !terminal
                .last_content()
                .mode
                .contains(Modes::BRACKETED_PASTE)
        {
            Some(
                "That shell has bracketed paste off, so these lines would run one by one: nothing \
                 was typed."
                    .to_string(),
            )
        } else {
            None
        }
    };
    if let Some(refusal) = refusal {
        toast(workspace, refusal, cx);
        return;
    }
    let preview_window = window.window_handle();
    // The terminal's pane may be the preview's, which bringing the terminal forward updates: so
    // after this update, as a sent pick does.
    cx.defer(move |cx| {
        terminal_window
            .update(cx, |_, window, cx| {
                if terminal_window != preview_window {
                    window.activate_window();
                }
                crate::browser::reveal_terminal(&view, window, cx);
                window.focus(&view.focus_handle(cx), cx);
                let terminal = view.read(cx).terminal().clone();
                terminal.update(cx, |terminal, _| {
                    terminal.input(b"\x15".to_vec());
                    terminal.paste(&text);
                });
            })
            .log_err();
    });
}

/// Shows `message` in `workspace`'s toast, while it is open.
fn toast(workspace: &WeakEntity<Workspace>, message: impl Into<String>, cx: &mut App) {
    let message = message.into();
    workspace
        .update(cx, |workspace, cx| {
            crate::send_selection::show_toast(workspace, message, cx);
        })
        .log_err();
}
