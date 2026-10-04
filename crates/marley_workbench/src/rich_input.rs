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
//!
//! With `marley.prompt_editor` on (the default, #627), the shell's editor docks by itself whenever
//! the shell waits at a prompt off the alternate screen and its terminal holds the focus, and it
//! takes the keys there; when a command starts or a full-screen program shows, it closes and the
//! terminal gets every key raw again. Escape gives the keys back to the shell until its next
//! prompt; Ctrl-C empties the editor.
//!
//! The shell's editor shows #557's hint, or #573's reading, after its text as an inlay, and in
//! `act` colours the words a command would take as arguments when the line is a command followed
//! by English (#573); each edit tells [`crate::typed_line`] the line in front. While its cursor is
//! at the end of its text, the rest of a history command the text starts shows there instead, and
//! → takes it, as #484's suggestion does at the grid's prompt (#637).

use std::collections::HashMap;

use editor::{Editor, EditorEvent, HighlightKey, Inlay, MultiBufferOffset, ToOffset as _};
use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, EntityId, Focusable as _, Global,
    HighlightStyle, InteractiveElement as _, KeyDownEvent, WeakEntity, Window,
};
use marley_agent::AgentKind;
use project::InlayId;
use terminal::Terminal;
use terminal_view::{MarleyFooterContext, TerminalView};
use ui::prelude::*;
use workspace::Workspace;

use crate::agent_bar::agent_in;
use crate::blocks::focused_terminal;
use crate::english::Hint;
use crate::{
    AcceptSuggestion, ClearRichInput, CloseRichInput, MarleySettings, PromptEditor, RichInput,
    SendRichInput,
};
use settings::Settings as _;

/// The id of the shell's editor's hint inlay, far from the ids Zed hands out.
const HINT_INLAY: usize = usize::MAX - 573;

/// A terminal's editor, and whether it shows.
struct Prompt {
    editor: Entity<Editor>,
    open: bool,
    /// Who the text goes to.
    target: Target,
    /// The view's terminal, whose line the shell's editor holds (#573).
    terminal: WeakEntity<Terminal>,
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

/// What the prompt editor by default (#627) knows of each terminal view's shell.
#[derive(Default)]
struct Shells(HashMap<EntityId, ShellState>);

impl Global for Shells {}

#[derive(Clone, Copy, Default)]
struct ShellState {
    /// Whether the shell waited at a prompt at the terminal's last notify.
    at_prompt: bool,
    /// How many blocks the terminal had then: a quick command starts and ends between two
    /// notifies, and only its new block tells that the prompt came again.
    blocks: usize,
    /// Whether the user gave the keys back to the shell with Escape, until its next prompt.
    dismissed: bool,
}

/// Installs `marley::RichInput` on every workspace. [`crate::init`] calls it once, before any
/// window opens.
pub fn init(cx: &mut App) {
    cx.set_global(Prompts::default());
    cx.set_global(Shells::default());
    cx.observe_new(
        |view: &mut TerminalView, window, cx: &mut Context<TerminalView>| {
            let Some(window) = window else {
                return;
            };
            let terminal = view.terminal().clone();
            cx.observe_in(&terminal, window, |view, _, window, cx| {
                follow_prompt(view, window, cx);
            })
            .detach();
            let focus = view.focus_handle(cx);
            cx.on_focus(&focus, window, |view, window, cx| {
                take_focus_at_prompt(view, window, cx);
            })
            .detach();
            let id = cx.entity_id();
            cx.on_release(move |_, cx| {
                cx.global_mut::<Shells>().0.remove(&id);
            })
            .detach();
        },
    )
    .detach();
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
fn target_of(terminal: &Terminal) -> Option<Target> {
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

/// Whether the shell of `view`'s terminal waits at a prompt off the alternate screen, with no agent
/// running: where the prompt editor takes the keys (#627).
fn shell_waits(view: &TerminalView, cx: &App) -> bool {
    let terminal = view.terminal().read(cx);
    matches!(target_of(terminal), Some(Target::Shell))
}

/// Follows `view`'s shell into and out of its prompt (#627): at a prompt the editor docks and
/// takes the focus the terminal held; leaving it, the editor closes and the terminal gets the keys.
/// It runs inside the view's update, so the editor's changes are deferred.
fn follow_prompt(view: &TerminalView, window: &Window, cx: &mut Context<TerminalView>) {
    if MarleySettings::get_global(cx).prompt_editor != PromptEditor::AtEveryPrompt {
        return;
    }
    let waits = shell_waits(view, cx);
    let blocks = view
        .terminal()
        .read(cx)
        .blocks()
        .last()
        .map_or(0, |block| block.index + 1);
    let id = cx.entity_id();
    let state = cx
        .global::<Shells>()
        .0
        .get(&id)
        .copied()
        .unwrap_or_default();
    let arrived = waits && (!state.at_prompt || blocks != state.blocks);
    let left = !waits && state.at_prompt;
    cx.global_mut::<Shells>().0.insert(
        id,
        ShellState {
            at_prompt: waits,
            blocks,
            dismissed: state.dismissed && !arrived,
        },
    );
    if !arrived && !left {
        return;
    }
    let view_entity = cx.entity();
    let editor_focused = cx
        .global::<Prompts>()
        .0
        .get(&id)
        .is_some_and(|prompt| prompt.open && prompt.editor.focus_handle(cx).is_focused(window));
    if waits {
        if view.focus_handle(cx).contains_focused(window, cx) || editor_focused {
            window.defer(cx, move |window, cx| {
                open_for(&view_entity, Target::Shell, window, cx);
            });
        }
    } else if let Some(prompt) = cx.global_mut::<Prompts>().0.get_mut(&id)
        && prompt.open
        && matches!(prompt.target, Target::Shell)
    {
        prompt.open = false;
        // The keys go back to the terminal only when the editor had them.
        if editor_focused {
            window.defer(cx, move |window, cx| {
                window.focus(&view_entity.focus_handle(cx), cx);
            });
        }
        cx.notify();
    }
}

/// The terminal's focus goes on to the shell's editor while the shell waits at a prompt, unless the
/// user gave the keys back with Escape (#627).
fn take_focus_at_prompt(view: &TerminalView, window: &Window, cx: &mut Context<TerminalView>) {
    if MarleySettings::get_global(cx).prompt_editor != PromptEditor::AtEveryPrompt
        || !shell_waits(view, cx)
    {
        return;
    }
    let dismissed = cx
        .global::<Shells>()
        .0
        .get(&cx.entity_id())
        .is_some_and(|state| state.dismissed);
    if dismissed {
        return;
    }
    let view_entity = cx.entity();
    window.defer(cx, move |window, cx| {
        open_for(&view_entity, Target::Shell, window, cx);
    });
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
        // And reads in the shell language's colours (#626).
        if matches!(target, Target::Shell)
            && let Some(language) = crate::prompt_colors::shell_language(cx)
            && let Some(buffer) = editor.buffer().read(cx).as_singleton()
        {
            buffer.update(cx, |buffer, cx| buffer.set_language(Some(language), cx));
        }
        if let Some(typed) = typed {
            editor.set_text(typed, window, cx);
            editor.move_to_end(&editor::actions::MoveToEnd, window, cx);
        }
    });
    window.focus(&editor.focus_handle(cx), cx);
    let terminal = view.read(cx).terminal().clone();
    cx.global_mut::<Prompts>().0.insert(
        id,
        Prompt {
            editor: editor.clone(),
            open: true,
            target,
            terminal: terminal.downgrade(),
        },
    );
    if matches!(target, Target::Shell) {
        // A draft the editor kept is the line in front now (#573).
        let text = editor.read(cx).text(cx);
        crate::typed_line::changed(&terminal, &text, cx);
    }
    paint_hint(&editor, &terminal, matches!(target, Target::Shell), cx);
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
    let view = view.downgrade();
    cx.subscribe(&editor, move |_, event: &EditorEvent, cx| match event {
        EditorEvent::BufferEdited => edited(&view, cx),
        // The history's suggestion shows only while the cursor is at the end of the text (#637).
        EditorEvent::SelectionsChanged { .. } => {
            if let Some((editor, terminal)) = shell_editor(&view, cx) {
                paint_hint(&editor, &terminal, true, cx);
            }
        }
        _ => {}
    })
    .detach();
    editor
}

/// The shell's editor of `view` was edited: its text is the line in front of the terminal, and
/// the hint after it follows (#573).
fn edited(view: &WeakEntity<TerminalView>, cx: &mut App) {
    let Some((editor, terminal)) = shell_editor(view, cx) else {
        return;
    };
    let text = editor.read(cx).text(cx);
    crate::typed_line::changed(&terminal, &text, cx);
    paint_hint(&editor, &terminal, true, cx);
}

/// The shell's editor of `view` while it is open, and the view's terminal.
fn shell_editor(
    view: &WeakEntity<TerminalView>,
    cx: &App,
) -> Option<(Entity<Editor>, Entity<Terminal>)> {
    let view = view.upgrade()?;
    let editor = cx
        .global::<Prompts>()
        .0
        .get(&view.entity_id())
        .filter(|prompt| prompt.open && matches!(prompt.target, Target::Shell))
        .map(|prompt| prompt.editor.clone())?;
    Some((editor, view.read(cx).terminal().clone()))
}

/// Draws the hint after the text of `editor`, the shell's editor of `terminal`, and colours the
/// words the hint warns about; an agent's editor (`shell` false) shows neither.
fn paint_hint(editor: &Entity<Editor>, terminal: &Entity<Terminal>, shell: bool, cx: &mut App) {
    let hint = shell.then(|| shell_hint(editor, terminal, cx)).flatten();
    let warning = cx.theme().status().warning;
    editor.update(cx, |editor, cx| {
        let snapshot = editor.buffer().read(cx).snapshot(cx);
        let end = snapshot.anchor_after(snapshot.len());
        let inlays = hint
            .as_ref()
            .map(|hint| Inlay::edit_prediction(HINT_INLAY, end, hint.text.to_string()))
            .into_iter()
            .collect();
        editor.splice_inlays(&[InlayId::EditPrediction(HINT_INLAY)], inlays, cx);
        match hint.and_then(|hint| hint.warning) {
            Some(range) => editor.highlight_text(
                HighlightKey::Editor,
                vec![
                    snapshot.anchor_before(MultiBufferOffset(range.start))
                        ..snapshot.anchor_after(MultiBufferOffset(range.end)),
                ],
                HighlightStyle {
                    color: Some(warning),
                    ..HighlightStyle::default()
                },
                cx,
            ),
            None => editor.clear_highlights(HighlightKey::Editor, cx),
        }
    });
}

/// What shows after the text of `editor`, the shell's editor of `terminal`: the history's
/// suggestion, else #557's hint or #573's reading, as at the grid's prompt, where the hint shows
/// only without a suggestion (#637).
fn shell_hint(editor: &Entity<Editor>, terminal: &Entity<Terminal>, cx: &App) -> Option<Hint> {
    let text = editor.read(cx).text(cx);
    history_suggestion(editor, &text, terminal, cx).map_or_else(
        || crate::english::hint_for(&text, terminal, cx),
        |rest| {
            Some(Hint {
                text: rest.into(),
                warning: None,
            })
        },
    )
}

/// The rest of the history command `text`, the text of `editor`, starts, while the editor's one
/// cursor sits at the end of it with nothing selected, as #484's suggestion shows only while
/// nothing follows the cursor (#637).
fn history_suggestion(
    editor: &Entity<Editor>,
    text: &str,
    terminal: &Entity<Terminal>,
    cx: &App,
) -> Option<String> {
    let editor = editor.read(cx);
    let snapshot = editor.buffer().read(cx).snapshot(cx);
    let selection = editor.selections.newest_anchor();
    let end = snapshot.len();
    let at_end = editor.selections.count() == 1
        && selection.start.to_offset(&snapshot) == end
        && selection.end.to_offset(&snapshot) == end;
    at_end
        .then(|| crate::autosuggest::suggestion_for(text, terminal, cx))
        .flatten()
}

/// → in the shell's editor of `view`: the history's suggestion shown after its text is added to
/// it; without one the key goes on, to the editor's own cursor move (#637).
fn accept_suggestion(view: &Entity<TerminalView>, window: &mut Window, cx: &mut App) {
    let suggestion = shell_editor(&view.downgrade(), cx).and_then(|(editor, terminal)| {
        let text = editor.read(cx).text(cx);
        let rest = history_suggestion(&editor, &text, &terminal, cx)?;
        Some((editor, rest))
    });
    match suggestion {
        Some((editor, rest)) => editor.update(cx, |editor, cx| editor.insert(&rest, window, cx)),
        None => cx.propagate(),
    }
}

/// Draws the hint of the shell's editor holding `terminal`'s line again, as a reading came
/// (#573).
pub(crate) fn refresh_hint(terminal: EntityId, cx: &mut App) {
    let Some((editor, terminal)) = cx.try_global::<Prompts>().and_then(|prompts| {
        prompts.0.values().find_map(|prompt| {
            let shell = prompt.open && matches!(prompt.target, Target::Shell);
            let held = prompt.terminal.upgrade()?;
            (shell && held.entity_id() == terminal).then(|| (prompt.editor.clone(), held))
        })
    }) else {
        return;
    };
    paint_hint(&editor, &terminal, true, cx);
}

/// Whether a shell's editor holding the line of the terminal `terminal` is open: the line is the
/// editor's then, not the grid's (#573).
pub(crate) fn holds_line_of(terminal: EntityId, cx: &App) -> bool {
    cx.try_global::<Prompts>().is_some_and(|prompts| {
        prompts.0.values().any(|prompt| {
            prompt.open
                && matches!(prompt.target, Target::Shell)
                && prompt.terminal.entity_id() == terminal
        })
    })
}

/// The text of `view`'s shell editor while it is open (#573).
pub(crate) fn shell_text(view: &Entity<TerminalView>, cx: &App) -> Option<String> {
    let prompt = cx.try_global::<Prompts>()?.0.get(&view.entity_id())?;
    (prompt.open && matches!(prompt.target, Target::Shell)).then(|| prompt.editor.read(cx).text(cx))
}

/// Empties `view`'s shell editor, whose line went to an agent (#573).
pub(crate) fn clear_shell(view: &Entity<TerminalView>, window: &mut Window, cx: &mut App) {
    clear(view, window, cx);
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
        // A remote terminal whose link is down sends nothing, so the prompt stays in the editor
        // (#641).
        if terminal.read(cx).marley_inputs_refused().is_some() {
            return;
        }
        // What was typed at the shell's prompt is the editor's now, so the shell's line goes first,
        // unless nothing was typed there: readline rings the bell at Ctrl-U on an empty line, and
        // the bell marks the terminal dirty, so closing it asked to save changes (#635).
        if matches!(target, Target::Shell) {
            crate::typed_line::entering(&terminal, cx);
            let anchored = terminal.read(cx).marley_anchored();
            let empty_line = anchored.at_prompt() && anchored.input_start().is_none();
            if !empty_line {
                terminal.update(cx, |terminal, _| terminal.input(vec![0x15]));
            }
        }
        // Sending is typing: the bell goes, as a key typed in the terminal clears it (#635).
        view.update(cx, TerminalView::clear_bell);
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

/// Closes `view`'s editor, keeping its text, and gives the terminal the focus back. The shell's
/// editor stays away until the next prompt (#627).
fn close(view: &Entity<TerminalView>, window: &mut Window, cx: &mut App) {
    if let Some(prompt) = cx.global_mut::<Prompts>().0.get_mut(&view.entity_id()) {
        prompt.open = false;
        if matches!(prompt.target, Target::Shell)
            && let Some(state) = cx.global_mut::<Shells>().0.get_mut(&view.entity_id())
        {
            state.dismissed = true;
        }
    }
    window.focus(&view.focus_handle(cx), cx);
    view.update(cx, |_, cx| cx.notify());
}

/// Empties `view`'s editor, as Ctrl-C at a shell's prompt drops the line (#627).
fn clear(view: &Entity<TerminalView>, window: &mut Window, cx: &mut App) {
    if let Some(editor) = cx
        .global::<Prompts>()
        .0
        .get(&view.entity_id())
        .map(|prompt| prompt.editor.clone())
    {
        editor.update(cx, |editor, cx| editor.set_text("", window, cx));
    }
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

/// Whether `view` holds the keys: its terminal, or the open editor in its footer.
///
/// The view's focus handle does not count the footer as its own: Zed's terminal element tracks
/// the same handle on the grid, gpui takes the last element to track a handle as its place, and
/// the footer sits beside the grid. So while the shell's prompt editor (#627) or an agent's
/// editor has the keys, the view alone reads as unfocused (#634).
pub(crate) fn holds_focus(view: &TerminalView, window: &Window, cx: &App) -> bool {
    if view.focus_handle(cx).contains_focused(window, cx) {
        return true;
    }
    let terminal = view.terminal().entity_id();
    cx.try_global::<Prompts>().is_some_and(|prompts| {
        prompts.0.values().any(|prompt| {
            prompt.open
                && prompt.terminal.entity_id() == terminal
                && prompt.editor.focus_handle(cx).contains_focused(window, cx)
        })
    })
}

/// The editor of the terminal in `context`, while it is open.
pub fn element(context: &MarleyFooterContext, cx: &App) -> Option<AnyElement> {
    let view = context.view.upgrade()?;
    let prompt = cx.try_global::<Prompts>()?.0.get(&view.entity_id())?;
    if !prompt.open {
        return None;
    }
    let colors = cx.theme().colors();
    let (send_to, close_on, clear_on, accept_on) = (
        context.view.clone(),
        context.view.clone(),
        context.view.clone(),
        context.view.clone(),
    );
    // The shell's editor answers a few keys of its own (#627).
    let mut key_context = gpui::KeyContext::new_with_defaults();
    key_context.add("MarleyRichInput");
    if matches!(prompt.target, Target::Shell) {
        key_context.add("MarleyShellInput");
    }
    Some(
        div()
            .debug_selector(|| "marley-rich-input".into())
            .key_context(key_context)
            .on_action(move |_: &ClearRichInput, window, cx| {
                on_view(&clear_on, window, cx, clear);
            })
            .on_action(move |_: &SendRichInput, window, cx| on_view(&send_to, window, cx, send))
            .on_action(move |_: &CloseRichInput, window, cx| {
                on_view(&close_on, window, cx, close);
            })
            .on_action(
                move |_: &AcceptSuggestion, window, cx| match accept_on.upgrade() {
                    Some(view) => accept_suggestion(&view, window, cx),
                    None => cx.propagate(),
                },
            )
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
