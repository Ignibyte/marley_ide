//! Rich input (T7e): a Zed editor for a CLI agent's prompt, docked above the agent bar.
//!
//! While an agent runs in a terminal, Ctrl-G (`marley::RichInput`) or the bar's button opens the
//! terminal's editor and gives it the focus. Enter sends the text to the agent as one paste,
//! bracketed when the agent asked for it, and a carriage return; Shift-Enter adds a line; Escape
//! closes the editor and keeps the draft. Without an agent, Ctrl-G reaches the program as before.

use std::collections::HashMap;

use editor::Editor;
use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, EntityId, Focusable as _, Global,
    InteractiveElement as _, KeyDownEvent, WeakEntity, Window,
};
use marley_agent::AgentKind;
use terminal_view::{MarleyFooterContext, TerminalView};
use ui::prelude::*;
use workspace::Workspace;

use crate::agent_bar::agent_in;
use crate::blocks::focused_terminal;
use crate::{CloseRichInput, RichInput, SendRichInput};

/// A terminal's editor, and whether it shows.
struct Prompt {
    editor: Entity<Editor>,
    open: bool,
}

/// Each terminal view's editor, made the first time it opens and dropped with the view.
#[derive(Default)]
struct Prompts(HashMap<EntityId, Prompt>);

impl Global for Prompts {}

/// Installs `marley::RichInput` on every workspace. [`crate::init`] calls it once, before any
/// window opens.
pub fn init(cx: &mut App) {
    cx.set_global(Prompts::default());
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &RichInput, window, cx| {
            let agent_view = focused_terminal(workspace, window, cx).and_then(|view| {
                let agent = agent_in(view.read(cx).terminal().read(cx))?;
                Some((view, agent))
            });
            match agent_view {
                Some((view, agent)) => open(&view, agent, window, cx),
                // The key goes on to the terminal, which sends it to the program.
                None => cx.propagate(),
            }
        });
    })
    .detach();
}

/// Opens `view`'s editor for `agent`, with the draft it had, and gives it the focus.
pub fn open(view: &Entity<TerminalView>, agent: AgentKind, window: &mut Window, cx: &mut App) {
    let id = view.entity_id();
    let existing = cx
        .global::<Prompts>()
        .0
        .get(&id)
        .map(|prompt| prompt.editor.clone());
    let editor = existing.unwrap_or_else(|| new_editor(view, agent, window, cx));
    window.focus(&editor.focus_handle(cx), cx);
    cx.global_mut::<Prompts>()
        .0
        .insert(id, Prompt { editor, open: true });
    view.update(cx, |_, cx| cx.notify());
}

/// A new editor for `view`'s prompt to `agent`, one to eight lines tall, which goes when the view
/// does.
fn new_editor(
    view: &Entity<TerminalView>,
    agent: AgentKind,
    window: &mut Window,
    cx: &mut App,
) -> Entity<Editor> {
    let placeholder = format!("A prompt for {}", agent.display_name());
    let editor = cx.new(|cx| {
        let mut editor = Editor::auto_height(1, 8, window, cx);
        editor.set_soft_wrap_mode(settings::SoftWrap::EditorWidth, cx);
        editor.set_placeholder_text(&placeholder, window, cx);
        editor
    });
    let id = view.entity_id();
    cx.observe_release(view, move |_, cx| {
        cx.global_mut::<Prompts>().0.remove(&id);
    })
    .detach();
    editor
}

/// Sends the editor's text to the agent as one paste and a carriage return, then closes it
/// empty; an empty editor only closes.
fn send(view: &Entity<TerminalView>, window: &mut Window, cx: &mut App) {
    let Some(editor) = cx
        .global::<Prompts>()
        .0
        .get(&view.entity_id())
        .map(|prompt| prompt.editor.clone())
    else {
        return;
    };
    let text = editor.read(cx).text(cx);
    if !text.trim().is_empty() {
        let terminal = view.read(cx).terminal().clone();
        terminal.update(cx, |terminal, _| {
            terminal.paste(&text);
            terminal.input(b"\r".to_vec());
        });
        editor.update(cx, |editor, cx| editor.set_text("", window, cx));
    }
    close(view, window, cx);
}

/// Closes `view`'s editor, keeping its text, and gives the terminal the focus back.
fn close(view: &Entity<TerminalView>, window: &mut Window, cx: &mut App) {
    if let Some(prompt) = cx.global_mut::<Prompts>().0.get_mut(&view.entity_id()) {
        prompt.open = false;
    }
    window.focus(&view.focus_handle(cx), cx);
    view.update(cx, |_, cx| cx.notify());
}

/// Runs `action` on the terminal view behind `view`, when it is still there.
fn on_view(
    view: &WeakEntity<TerminalView>,
    window: &mut Window,
    cx: &mut App,
    action: fn(&Entity<TerminalView>, &mut Window, &mut App),
) {
    if let Some(view) = view.upgrade() {
        action(&view, window, cx);
    }
}

/// The editor of the terminal in `context`, while it is open.
pub fn element(context: &MarleyFooterContext, cx: &App) -> Option<AnyElement> {
    let view = context.view.upgrade()?;
    let prompt = cx.try_global::<Prompts>()?.0.get(&view.entity_id())?;
    if !prompt.open {
        return None;
    }
    let colors = cx.theme().colors();
    let (send_to, close_on) = (context.view.clone(), context.view.clone());
    Some(
        div()
            .debug_selector(|| "marley-rich-input".into())
            .key_context("MarleyRichInput")
            .on_action(move |_: &SendRichInput, window, cx| on_view(&send_to, window, cx, send))
            .on_action(move |_: &CloseRichInput, window, cx| {
                on_view(&close_on, window, cx, close);
            })
            // The terminal view sends its program the keys it maps: special keys and chords. The
            // editor's own keys have run by now, and a key that types text goes on to the
            // editor's text input, which the terminal leaves alone.
            .on_key_down(|event: &KeyDownEvent, _, cx| {
                let modifiers = &event.keystroke.modifiers;
                let types_text = event.keystroke.key_char.is_some()
                    && !(modifiers.control || modifiers.alt || modifiers.platform);
                if !types_text {
                    cx.stop_propagation();
                }
            })
            .flex_none()
            .w_full()
            .px_2()
            .py_1p5()
            .border_t_1()
            .border_color(colors.border_variant)
            .bg(colors.editor_background)
            .child(prompt.editor.clone())
            .into_any_element(),
    )
}
