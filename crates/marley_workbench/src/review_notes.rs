//! Review notes to the agent (#522).
//!
//! Zed's project and branch diffs take review notes on changed lines and draw Send Review to
//! Agent, whose action upstream left with no handler. Marley handles it: the picker lists the
//! agents whose folder holds every noted file, each with whether it can take the notes now, and
//! Copy notes. A pick of an idle Claude Code pastes the notes as one prompt, in the form Orca's
//! report describes, and presses Enter; the notes stay in the diff, marked Sent. An agent at work,
//! one that waits on a permission or a question, and one Marley cannot read are refused. Notes on a
//! Markdown file in the review tab go through the same picker (#729).

use std::path::Path;
use std::rc::Rc;

use editor::actions::SendReviewToAgent;
use editor::{Editor, ReviewNote};
use gpui::{
    AnyWindowHandle, App, ClipboardItem, Context, Entity, Focusable as _, WeakEntity, Window,
};
use marley_agent::ReviewLine;
use settings::Settings as _;
use util::ResultExt as _;
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};

use crate::send_selection::{self, OnPick, Pick, Row, Target, TargetPicker};

/// The toasts' id.
struct ReviewNotes;

/// Registers the handler on every workspace; [`crate::init`] calls it once.
pub fn init(cx: &App) {
    // The project diff's Viewed ticks, another reviewing aid (#731).
    crate::viewed_marks::init(cx);
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &SendReviewToAgent, window, cx| {
            if let Err(message) = open_picker(workspace, window, cx) {
                show_toast(workspace, message, cx);
            }
        });
    })
    .detach();
}

/// Opens the picker for the active diff's unsent notes; the message says why it did not.
fn open_picker(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> Result<(), &'static str> {
    let editor = workspace
        .active_item(cx)
        .and_then(|item| item.act_as::<Editor>(cx))
        .ok_or("Send Review to Agent sends the notes of a project diff or a branch diff.")?;
    let notes = editor.read(cx).unsent_review_notes(cx);
    if notes.is_empty() {
        return Err("This diff has no review notes that were not sent.");
    }
    let mut rows: Vec<Row> = send_selection::agent_targets(workspace, cx)
        .into_iter()
        .filter(|target| holds_every_file(target, &notes))
        .map(row_of)
        .collect();
    let placeholder = if rows.is_empty() {
        "No agent works on these files: Copy notes…"
    } else {
        "Send the review notes to…"
    };
    rows.push(Row {
        label: "Copy notes".to_string(),
        pick: Pick::Copy,
    });
    let window_handle = window.window_handle();
    let editor = editor.downgrade();
    let on_pick: OnPick =
        Box::new(move |pick, window, cx| deliver(pick, &editor, &notes, window, cx));
    workspace.toggle_modal(window, cx, |window, cx| {
        TargetPicker::new(rows, placeholder, on_pick, window_handle, window, cx)
    });
    Ok(())
}

/// Puts the unsent notes of `editor`'s diff on the clipboard as an agent would get them, as the
/// picker's Copy notes does (#893).
pub(crate) fn copy_notes(editor: &Entity<Editor>, cx: &App) {
    let notes = editor.read(cx).unsent_review_notes(cx);
    cx.write_to_clipboard(ClipboardItem::new_string(prompt(&notes, None, cx)));
}

/// Sends the unsent notes of `editor`'s diff to `target`, as the picker's row for it does (#893);
/// the message says why nothing went.
pub(crate) fn send_notes_to(
    target: Target,
    editor: &Entity<Editor>,
    window: AnyWindowHandle,
    cx: &mut App,
) -> Result<(), &'static str> {
    let notes = editor.read(cx).unsent_review_notes(cx);
    if notes.is_empty() {
        return Err("This diff has no review notes that were not sent.");
    }
    deliver(Pick::Agent(target), &editor.downgrade(), &notes, window, cx);
    Ok(())
}

/// Opens the picker for notes on the Markdown file at `path` (#729): the agents whose folder holds
/// it, each with its readiness, and Copy notes. `lines` name the file by its absolute path;
/// `on_sent` runs once an agent took them.
pub(crate) fn send_file_notes(
    workspace: &mut Workspace,
    path: &Path,
    lines: Vec<ReviewLine>,
    on_sent: Rc<dyn Fn(&mut App)>,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let mut rows: Vec<Row> = send_selection::agent_targets(workspace, cx)
        .into_iter()
        .filter(|target| {
            target
                .cwd
                .as_deref()
                .is_some_and(|cwd| path.starts_with(cwd))
        })
        .map(row_of)
        .collect();
    let placeholder = if rows.is_empty() {
        "No agent works on this file: Copy notes…"
    } else {
        "Send the notes on this file to…"
    };
    rows.push(Row {
        label: "Copy notes".to_string(),
        pick: Pick::Copy,
    });
    let window_handle = window.window_handle();
    let on_pick: OnPick = Box::new(move |pick, window, cx| {
        let target = match pick {
            Pick::Copy => {
                let text = message_of(lines.clone(), None, cx);
                cx.write_to_clipboard(ClipboardItem::new_string(text));
                return;
            }
            Pick::Agent(target) => target,
        };
        if refused(&target, cx) {
            return;
        }
        let text = message_of(lines.clone(), target.cwd.as_deref(), cx);
        let on_sent = Rc::clone(&on_sent);
        paste_and_submit(target, text, window, move |cx| on_sent(cx), cx);
    });
    workspace.toggle_modal(window, cx, |window, cx| {
        TargetPicker::new(rows, placeholder, on_pick, window_handle, window, cx)
    });
}

/// A picker's row for `target`: its agent, its project and whether it can take notes now.
fn row_of(target: Target) -> Row {
    Row {
        label: format!(
            "{} · {} · {}",
            target.kind.display_name(),
            target.project,
            readiness(&target)
        ),
        pick: Pick::Agent(target),
    }
}

/// Whether `target`'s agent works in a folder that holds every noted file, so the notes' paths
/// mean the same files to it.
fn holds_every_file(target: &Target, notes: &[ReviewNote]) -> bool {
    target
        .cwd
        .as_deref()
        .is_some_and(|cwd| notes.iter().all(|note| note.path.starts_with(cwd)))
}

/// Whether the agent can take the notes now, in the picker's words.
pub(crate) const fn readiness(target: &Target) -> &'static str {
    if target.ready {
        "ready"
    } else if target.waiting {
        "asking for permission"
    } else if target.status.is_some() {
        "working"
    } else {
        "no idle signal"
    }
}

/// What the pick does: Copy puts the prompt on the clipboard and leaves the notes unsent; an
/// agent that can take them gets the prompt and Enter, its terminal is shown, and the notes are
/// marked sent; any other agent is left alone, with a toast.
fn deliver(
    pick: Pick,
    editor: &WeakEntity<Editor>,
    notes: &[ReviewNote],
    window: AnyWindowHandle,
    cx: &mut App,
) {
    let target = match pick {
        Pick::Copy => {
            cx.write_to_clipboard(ClipboardItem::new_string(prompt(notes, None, cx)));
            return;
        }
        Pick::Agent(target) => target,
    };
    if refused(&target, cx) {
        return;
    }
    let text = prompt(notes, target.cwd.as_deref(), cx);
    let ids: Vec<usize> = notes.iter().map(|note| note.id).collect();
    let editor = editor.clone();
    paste_and_submit(
        target,
        text,
        window,
        move |cx| {
            editor
                .update(cx, |editor, cx| editor.mark_review_notes_sent(&ids, cx))
                .log_err();
        },
        cx,
    );
}

/// Whether `target` cannot take notes now; a toast then says why.
fn refused(target: &Target, cx: &mut App) -> bool {
    if target.ready {
        return false;
    }
    let message = format!(
        "{} in {} is {}; the notes were not sent.",
        target.kind.display_name(),
        target.project,
        readiness(target)
    );
    if let Some(workspace) = target.view.read(cx).marley_workspace().upgrade() {
        workspace.update(cx, |workspace, cx| show_toast(workspace, message, cx));
    }
    true
}

/// Pastes `text` into `target`'s terminal and presses Enter, its window activated and its tab shown
/// and focused, after the picker's update as a selection's send runs; `after` runs then.
fn paste_and_submit(
    target: Target,
    text: String,
    window: AnyWindowHandle,
    after: impl FnOnce(&mut App) + 'static,
    cx: &mut App,
) {
    cx.defer(move |cx| {
        window
            .update(cx, |_, window, cx| {
                window.activate_window();
                crate::browser::reveal_terminal(&target.view, window, cx);
                window.focus(&target.view.focus_handle(cx), cx);
                let terminal = target.view.read(cx).terminal().clone();
                crate::terminal_drive::paste_then(
                    &terminal,
                    &text,
                    |terminal| terminal.input(b"\r".to_vec()),
                    cx,
                )
                .detach();
                after(cx);
            })
            .log_err();
    });
}

/// The prompt for `notes`, each file relative to `cwd`, the agent's folder, when it lies under
/// it, else absolute.
fn prompt(notes: &[ReviewNote], cwd: Option<&Path>, cx: &App) -> String {
    let lines = notes
        .iter()
        .map(|note| ReviewLine {
            path: note.path.to_string_lossy().into_owned(),
            first: note.first_line,
            last: note.last_line,
            comment: note.comment.clone(),
            source: note.source.clone(),
            quote: None,
        })
        .collect();
    message_of(lines, cwd, cx)
}

/// The message for `lines`, whose paths are absolute, each made relative to `cwd` when it lies
/// under it.
fn message_of(lines: Vec<ReviewLine>, cwd: Option<&Path>, cx: &App) -> String {
    let lines = marley_agent::review_prompt(
        lines
            .into_iter()
            .map(|line| ReviewLine {
                path: cwd
                    .and_then(|cwd| Path::new(&line.path).strip_prefix(cwd).ok())
                    .filter(|relative| !relative.as_os_str().is_empty())
                    .map_or_else(
                        || line.path.clone(),
                        |relative| relative.to_string_lossy().into_owned(),
                    ),
                ..line
            })
            .collect(),
    );
    // In the user's template, or Marley's own (#732); Copy carries the same message.
    let template = crate::MarleySettings::get_global(cx)
        .review_notes_template
        .clone();
    marley_agent::review_message(template.as_deref(), &lines)
}

/// The message for notes on an agent's last reply (#729), in the same template.
pub(crate) fn reply_message(notes: &str, cx: &App) -> String {
    let template = crate::MarleySettings::get_global(cx)
        .review_notes_template
        .clone();
    marley_agent::review_message(template.as_deref(), notes)
}

fn show_toast(workspace: &mut Workspace, message: impl Into<String>, cx: &mut Context<Workspace>) {
    workspace.show_toast(
        Toast::new(NotificationId::unique::<ReviewNotes>(), message.into()),
        cx,
    );
}
