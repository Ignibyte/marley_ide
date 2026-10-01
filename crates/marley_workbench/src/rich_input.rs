//! Rich input (T7e): a Zed editor for a CLI agent's prompt, docked above the agent bar.
//!
//! While an agent runs in a terminal, Ctrl-G (`marley::RichInput`) or the bar's button opens the
//! terminal's editor and gives it the focus. Enter sends the text to the agent as one paste,
//! bracketed when the agent asked for it, and a carriage return; Shift-Enter adds a line; Escape
//! closes the editor and keeps the draft.
//!
//! At a shell's prompt with no agent running, Ctrl-G opens the same editor for the shell (#624),
//! holding the line typed so far: Enter clears the shell's line with Ctrl-U, then sends the text
//! and a carriage return, and Escape leaves the shell's line as it was. Anywhere else, Ctrl-G
//! reaches the program as before.

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
    /// Who the text goes to.
    target: Target,
}

/// Who an editor's text goes to.
#[derive(Clone, Copy)]
enum Target {
    /// The CLI agent running in the terminal.
    Agent(AgentKind),
    /// The shell, waiting at its prompt (#624).
    Shell,
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
            let target_view = focused_terminal(workspace, window, cx).and_then(|view| {
                let target = target_of(view.read(cx).terminal().read(cx))?;
                Some((view, target))
            });
            match target_view {
                Some((view, target)) => {
                    let workspace_entity = cx.entity();
                    crate::shortcut_note::taken(
                        &RichInput,
                        "opened Marley's Rich Input",
                        &view.focus_handle(cx),
                        &workspace_entity,
                        window,
                        cx,
                    );
                    open_for(&view, target, window, cx);
                }
                // The key goes on to the terminal, which sends it to the program.
                None => cx.propagate(),
            }
        });
    })
    .detach();
}

/// Who the editor of `terminal` would write to: its running agent, else its shell while that waits
/// at a prompt off the alternate screen (#624); none, to leave Ctrl-G to the program.
fn target_of(terminal: &terminal::Terminal) -> Option<Target> {
    if let Some(agent) = agent_in(terminal) {
        return Some(Target::Agent(agent));
    }
    let at_prompt = terminal.marley_anchored().at_prompt()
        && !terminal
            .last_content()
            .mode
            .contains(terminal::Modes::ALT_SCREEN);
    at_prompt.then_some(Target::Shell)
}

/// Opens `view`'s editor for `agent`, with the draft it had, and gives it the focus.
pub fn open(view: &Entity<TerminalView>, agent: AgentKind, window: &mut Window, cx: &mut App) {
    open_for(view, Target::Agent(agent), window, cx);
}

/// Opens `view`'s editor for `target` and gives it the focus: an agent's with the draft it had, the
/// shell's with the line typed at its prompt when there is one (#624).
fn open_for(view: &Entity<TerminalView>, target: Target, window: &mut Window, cx: &mut App) {
    let id = view.entity_id();
    let existing = cx
        .global::<Prompts>()
        .0
        .get(&id)
        .map(|prompt| prompt.editor.clone());
    let editor = existing.unwrap_or_else(|| new_editor(view, window, cx));
    let typed = match target {
        Target::Shell => crate::autosuggest::typed_text(view.read(cx).terminal().read(cx))
            .filter(|typed| !typed.trim().is_empty()),
        Target::Agent(_) => None,
    };
    let completions = matches!(target, Target::Shell).then(|| {
        std::rc::Rc::new(crate::shell_completions::ShellCompletions {
            terminal: view.read(cx).terminal().downgrade(),
            workspace: view.read(cx).marley_workspace().clone(),
        }) as std::rc::Rc<dyn editor::CompletionProvider>
    });
    editor.update(cx, |editor, cx| {
        editor.set_placeholder_text(&placeholder(target), window, cx);
        // The shell's command completes paths, history and tasks (#625); an agent's prompt
        // nothing.
        editor.set_completion_provider(completions);
        if let Some(typed) = typed {
            editor.set_text(typed, window, cx);
            editor.move_to_end(&editor::actions::MoveToEnd, window, cx);
        }
    });
    window.focus(&editor.focus_handle(cx), cx);
    cx.global_mut::<Prompts>().0.insert(
        id,
        Prompt {
            editor,
            open: true,
            target,
        },
    );
    view.update(cx, |_, cx| cx.notify());
}

/// What an empty editor says it is for.
fn placeholder(target: Target) -> String {
    match target {
        Target::Agent(agent) => format!("A prompt for {}", agent.display_name()),
        Target::Shell => "A command for the shell".to_string(),
    }
}

/// Whether `view`'s editor is open (#549).
pub fn is_open(view: &Entity<TerminalView>, cx: &App) -> bool {
    cx.try_global::<Prompts>()
        .and_then(|prompts| prompts.0.get(&view.entity_id()))
        .is_some_and(|prompt| prompt.open)
}

/// Inserts `text` at the cursor of `view`'s editor while it is open, and gives the editor the
/// focus; a closed editor is left alone (#549).
pub fn insert(view: &Entity<TerminalView>, text: &str, window: &mut Window, cx: &mut App) {
    let Some(editor) = cx
        .try_global::<Prompts>()
        .and_then(|prompts| prompts.0.get(&view.entity_id()))
        .filter(|prompt| prompt.open)
        .map(|prompt| prompt.editor.clone())
    else {
        return;
    };
    editor.update(cx, |editor, cx| editor.insert(text, window, cx));
    window.focus(&editor.focus_handle(cx), cx);
}

/// A new editor for `view`'s prompt, one to eight lines tall, which goes when the view does.
fn new_editor(view: &Entity<TerminalView>, window: &mut Window, cx: &mut App) -> Entity<Editor> {
    let editor = cx.new(|cx| {
        let mut editor = Editor::auto_height(1, 8, window, cx);
        editor.set_soft_wrap_mode(settings::SoftWrap::EditorWidth, cx);
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
    let Some((editor, target)) = cx
        .global::<Prompts>()
        .0
        .get(&view.entity_id())
        .map(|prompt| (prompt.editor.clone(), prompt.target))
    else {
        return;
    };
    let text = editor.read(cx).text(cx);
    if !text.trim().is_empty() {
        let terminal = view.read(cx).terminal().clone();
        // What was typed at the shell's prompt is the editor's now, so the shell's line goes first.
        if matches!(target, Target::Shell) {
            terminal.update(cx, |terminal, _| terminal.input(vec![0x15]));
        }
        crate::terminal_drive::paste_then(
            &terminal,
            &text,
            |terminal| terminal.input(b"\r".to_vec()),
            cx,
        )
        .detach();
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
