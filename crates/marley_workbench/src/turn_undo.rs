//! Keep and Undo This Turn… on a rail turn row (#714).
//!
//! Undo takes one turn's changes out of the working tree with a three-way merge of trees: base the
//! turn's commit, ours a fresh checkpoint, theirs the turn's start (`turn_git::undo_merge_in`), so
//! later turns and the user's own edits stay. It refuses, saying why, while an agent works in the
//! turn's checkout, a folder inside it or one around it; on a conflict; and over a file open with
//! unsaved edits. It asks first, naming each file it restores or removes, and writes only the
//! merge's paths through a temporary index (`turn_git::write_merged_in`), which refuses a file
//! changed while the question was open. The user's index, `HEAD`, branches and the turn refs stay;
//! the undone turn's ref keeps its diff. Keep marks a turn reviewed.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use futures::channel::oneshot;
use git::repository::GitRepositoryCheckpoint;
use gpui::{App, AppContext as _, AsyncWindowContext, Global, PromptLevel, Window};
use marley_agent::claude_events::CWD_LABEL;

use crate::launch::verbatim;
use crate::turn_git::{self, PathChange, UndoMerge, UndoWrite};
use crate::turns::{Turn, Turns};
use marley_rail::TurnMark;

/// The work directories an undo runs in, so each takes one at a time.
#[derive(Default)]
struct Undoing(HashSet<PathBuf>);

impl Global for Undoing {}

/// Keep: marks the turn `sha` of the terminal `view` reviewed.
pub(crate) fn keep(view: u64, sha: &str, cx: &mut App) {
    Turns::mark(view, sha, TurnMark::Kept, cx);
}

/// Undo This Turn… on the turn `sha` of the terminal `view`.
pub(crate) fn undo(view: u64, sha: &str, window: &mut Window, cx: &mut App) {
    let Some(turn) = Turns::of(view, cx)
        .iter()
        .find(|turn| turn.sha == sha)
        .cloned()
    else {
        return;
    };
    if turn.mark == Some(TurnMark::Undone) {
        return;
    }
    let Some(repository) = turn.repository.upgrade() else {
        refuse("The turn's repository is no longer open.", None, window, cx);
        return;
    };
    let work_dir = repository.read(cx).work_directory_abs_path.to_path_buf();
    if let Some((agent, folder)) = agent_at_work(&work_dir, window, cx) {
        refuse_busy(&agent, &folder, window, cx);
        return;
    }
    if !cx.default_global::<Undoing>().0.insert(work_dir.clone()) {
        refuse(
            "An undo is already running in this repository.",
            None,
            window,
            cx,
        );
        return;
    }
    let checkpoint = repository.update(cx, |repository, _| repository.checkpoint());
    window
        .spawn(cx, async move |cx| {
            let outcome = run(view, &turn, &work_dir, checkpoint, cx).await;
            let shown = cx.update(|window, cx| {
                let _was_running = cx.default_global::<Undoing>().0.remove(&work_dir);
                if let Err(error) = &outcome {
                    refuse(
                        "The turn could not be undone.",
                        Some(&verbatim(&format!("{error:#}"))),
                        window,
                        cx,
                    );
                }
            });
            if shown.is_err() {
                log::warn!("turn undo: the window closed before the undo ended");
            }
        })
        .detach();
}

/// The undo after the first agent check: the merge, the unsaved check, the question, the agent
/// check again, and the write.
async fn run(
    view: u64,
    turn: &Turn,
    work_dir: &Path,
    checkpoint: oneshot::Receiver<anyhow::Result<GitRepositoryCheckpoint>>,
    cx: &mut AsyncWindowContext,
) -> anyhow::Result<()> {
    let now = checkpoint
        .await
        .context("the repository dropped the checkpoint")??
        .commit_sha
        .to_string();
    let merged = match turn_git::undo_merge_in(work_dir, &turn.sha, &now).await? {
        UndoMerge::Merged(tree) => tree,
        UndoMerge::Conflict(files) => {
            return tell(
                "The turn cannot be undone: a later change meets its changes in these files.",
                Some(&verbatim(&files.join("\n"))),
                cx,
            )
            .await;
        }
    };
    let changes = turn_git::changed_paths_in(work_dir, &now, &merged).await?;
    if changes.is_empty() {
        return tell(
            "Nothing to undo: the files already match the turn's start.",
            None,
            cx,
        )
        .await;
    }
    let unsaved = cx.update(|window, cx| unsaved_among(work_dir, &changes, window, cx))?;
    if !unsaved.is_empty() {
        return tell(
            "The turn cannot be undone while these files have unsaved edits. Save or revert them first.",
            Some(&verbatim(&unsaved.join("\n"))),
            cx,
        )
        .await;
    }
    if !confirmed(turn, &changes, cx).await? {
        return Ok(());
    }
    if let Some((agent, folder)) = cx.update(|window, cx| agent_at_work(work_dir, window, cx))? {
        return tell(&busy_message(&agent, &folder), None, cx).await;
    }
    match turn_git::write_merged_in(work_dir, &now, &merged).await? {
        UndoWrite::Written => {
            cx.update(|_, cx| Turns::mark(view, &turn.sha, TurnMark::Undone, cx))?;
            Ok(())
        }
        UndoWrite::Changed(why) => tell(
            "A file changed after the question opened, so nothing was written. Try the undo again.",
            Some(&verbatim(&why)),
            cx,
        )
        .await,
    }
}

/// The question before the write: each file restored or removed, and that the agent's
/// conversation is not changed.
async fn confirmed(
    turn: &Turn,
    changes: &[PathChange],
    cx: &mut AsyncWindowContext,
) -> anyhow::Result<bool> {
    let files = changes
        .iter()
        .map(|change| match change {
            PathChange::Restored(path) => format!("restore {path}"),
            PathChange::Removed(path) => format!("remove  {path}"),
        })
        .collect::<Vec<_>>()
        .join("\n");
    let title: String = turn.title.chars().take(80).collect();
    let message = format!("Undo the turn “{title}”?");
    let detail = format!(
        "{}\n\nThe agent's conversation is not changed: tell the agent what you undid.",
        verbatim(&files)
    );
    let answer = cx.update(|window, cx| {
        window.prompt(
            PromptLevel::Warning,
            &message,
            Some(&detail),
            &["Undo Turn", "Cancel"],
            cx,
        )
    })?;
    Ok(answer.await == Ok(0))
}

/// Says why the undo stopped, with one button, and waits for it.
async fn tell(
    message: &str,
    detail: Option<&str>,
    cx: &mut AsyncWindowContext,
) -> anyhow::Result<()> {
    let answer =
        cx.update(|window, cx| window.prompt(PromptLevel::Info, message, detail, &["OK"], cx))?;
    if answer.await.is_err() {
        log::debug!("turn undo: the window closed over its answer");
    }
    Ok(())
}

/// Says why the undo stopped before it began, with one button.
fn refuse(message: &str, detail: Option<&str>, window: &mut Window, cx: &mut App) {
    let answer = window.prompt(PromptLevel::Info, message, detail, &["OK"], cx);
    cx.background_spawn(async move {
        if answer.await.is_err() {
            log::debug!("turn undo: the window closed over its answer");
        }
    })
    .detach();
}

fn refuse_busy(agent: &str, folder: &Path, window: &mut Window, cx: &mut App) {
    refuse(&busy_message(agent, folder), None, window, cx);
}

fn busy_message(agent: &str, folder: &Path) -> String {
    format!(
        "The turn cannot be undone while {agent} works in {}. Wait for it, or stop it, then try again.",
        folder.display()
    )
}

/// The first agent at work in `checkout`, a folder inside it or a folder around it, with its
/// folder: a terminal's agent starting, working or waiting (#550's rule), or a thread that is
/// generating. The turn's own agent counts: its open turn would take the undo as its change.
fn agent_at_work(checkout: &Path, window: &Window, cx: &App) -> Option<(String, PathBuf)> {
    let overlaps = |folder: &Path| folder.starts_with(checkout) || checkout.starts_with(folder);
    let workspaces = crate::mcp::window_workspaces(window, cx);
    for (_, view) in crate::mcp::terminals_of(workspaces.clone(), cx) {
        let terminal = view.read(cx).terminal().read(cx);
        let Some(kind) = crate::agent_bar::agent_in(terminal) else {
            continue;
        };
        if crate::close_guard::working_status(&view, kind, cx).is_none() {
            continue;
        }
        let folder = crate::agent_events::seat_of(view.entity_id(), kind, cx)
            .and_then(|seat| seat.labels.get(CWD_LABEL))
            .map(PathBuf::from)
            .or_else(|| terminal.working_directory());
        if let Some(folder) = folder.filter(|folder| overlaps(folder)) {
            return Some((kind.display_name().to_string(), folder));
        }
    }
    for workspace in workspaces {
        for thread in crate::rail::live_threads(workspace.read(cx), cx) {
            let thread = thread.read(cx);
            if thread.status() != acp_thread::ThreadStatus::Generating {
                continue;
            }
            let folders: Vec<PathBuf> = thread.work_dirs().map_or_else(
                || {
                    workspace
                        .read(cx)
                        .root_paths(cx)
                        .iter()
                        .map(|root| root.to_path_buf())
                        .collect()
                },
                |dirs| dirs.paths().to_vec(),
            );
            if let Some(folder) = folders.into_iter().find(|folder| overlaps(folder)) {
                return Some(("an agent thread".to_string(), folder));
            }
        }
    }
    None
}

/// The paths among `changes` open with unsaved edits in any of Marley's windows.
fn unsaved_among(
    work_dir: &Path,
    changes: &[PathChange],
    window: &Window,
    cx: &App,
) -> Vec<String> {
    let projects: Vec<_> = crate::mcp::window_workspaces(window, cx)
        .iter()
        .map(|workspace| workspace.read(cx).project().clone())
        .collect();
    let mut unsaved = Vec::new();
    for change in changes {
        let (PathChange::Restored(path) | PathChange::Removed(path)) = change;
        let absolute = work_dir.join(path);
        let dirty = projects.iter().any(|project| {
            let project = project.read(cx);
            project
                .find_project_path(&absolute, cx)
                .and_then(|project_path| project.buffer_store().read(cx).get_by_path(&project_path))
                .is_some_and(|buffer| buffer.read(cx).is_dirty())
        });
        if dirty && !unsaved.contains(path) {
            unsaved.push(path.clone());
        }
    }
    unsaved
}
