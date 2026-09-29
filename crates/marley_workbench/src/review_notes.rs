//! Review notes to the agent (#522).
//!
//! Zed's project and branch diffs take review notes on changed lines and draw Send Review to
//! Agent, whose action upstream left with no handler. Marley handles it: the picker lists the
//! agents whose folder holds every noted file, each with whether it can take the notes now, and
//! Copy notes. A pick of an idle Claude Code pastes the notes as one prompt, in the form Orca's
//! report describes, and presses Enter; the notes stay in the diff, marked Sent. An agent at work,
//! one that waits on a permission or a question, and one Marley cannot read are refused.

use std::path::Path;

use editor::actions::SendReviewToAgent;
use editor::{Editor, ReviewNote};
use gpui::{AnyWindowHandle, App, ClipboardItem, Context, Focusable as _, WeakEntity, Window};
use marley_agent::ReviewLine;
use util::ResultExt as _;
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};

use crate::send_selection::{self, OnPick, Pick, Row, Target, TargetPicker};

/// The toasts' id.
struct ReviewNotes;

/// Registers the handler on every workspace; [`crate::init`] calls it once.
pub fn init(cx: &App) {
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
        .map(|target| Row {
            label: format!(
                "{} · {} · {}",
                target.kind.display_name(),
                target.project,
                readiness(&target)
            ),
            pick: Pick::Agent(target),
        })
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

/// Whether `target`'s agent works in a folder that holds every noted file, so the notes' paths
/// mean the same files to it.
fn holds_every_file(target: &Target, notes: &[ReviewNote]) -> bool {
    target
        .cwd
        .as_deref()
        .is_some_and(|cwd| notes.iter().all(|note| note.path.starts_with(cwd)))
}

/// Whether the agent can take the notes now, in the picker's words.
const fn readiness(target: &Target) -> &'static str {
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
            cx.write_to_clipboard(ClipboardItem::new_string(prompt(notes, None)));
            return;
        }
        Pick::Agent(target) => target,
    };
    if !target.ready {
        let message = format!(
            "{} in {} is {}; the notes were not sent.",
            target.kind.display_name(),
            target.project,
            readiness(&target)
        );
        if let Some(workspace) = target.view.read(cx).marley_workspace().upgrade() {
            workspace.update(cx, |workspace, cx| show_toast(workspace, message, cx));
        }
        return;
    }
    let text = prompt(notes, target.cwd.as_deref());
    let ids: Vec<usize> = notes.iter().map(|note| note.id).collect();
    let editor = editor.clone();
    // After the picker's update, as a selection's send runs.
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
                editor
                    .update(cx, |editor, cx| editor.mark_review_notes_sent(&ids, cx))
                    .log_err();
            })
            .log_err();
    });
}

/// The prompt for `notes`, each file relative to `cwd`, the agent's folder, when it lies under
/// it, else absolute.
fn prompt(notes: &[ReviewNote], cwd: Option<&Path>) -> String {
    marley_agent::review_prompt(
        notes
            .iter()
            .map(|note| ReviewLine {
                path: cwd
                    .and_then(|cwd| note.path.strip_prefix(cwd).ok())
                    .filter(|relative| !relative.as_os_str().is_empty())
                    .unwrap_or(&note.path)
                    .to_string_lossy()
                    .into_owned(),
                first: note.first_line,
                last: note.last_line,
                comment: note.comment.clone(),
            })
            .collect(),
    )
}

fn show_toast(workspace: &mut Workspace, message: impl Into<String>, cx: &mut Context<Workspace>) {
    workspace.show_toast(
        Toast::new(NotificationId::unique::<ReviewNotes>(), message.into()),
        cx,
    );
}
