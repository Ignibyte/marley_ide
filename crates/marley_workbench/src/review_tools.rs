//! Findings into the project diff (#730).
//!
//! `review_notes_add` puts a tool's or a reviewing agent's findings on the changed side of the
//! project diff, each under its lines as a proposal that names its source and offers Keep and
//! Drop. Only kept notes count on Send Review to Agent and go to an agent (#522). The call is
//! admitted as the editor tools are, and a note whose lines the diff does not show is left out by
//! name.

use std::ops::Range;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use editor::{Anchor, Editor};
use git_ui::project_diff::ProjectDiff;
use gpui::{AnyWindowHandle, App, AsyncApp, Entity, SharedString, WindowHandle};
use language::Point;
use marley_mcp::{AppCall, Refusal, ToolAnswer};
use serde_json::{Value, json};
use workspace::{MultiWorkspace, Workspace};

use crate::agent_control::{Area, Level, admit};

/// The most notes one call places.
const NOTES_MAX: usize = 50;
/// The most characters a note's comment keeps.
const COMMENT_MAX: usize = 4000;
/// The most characters a source name keeps.
const SOURCE_MAX: usize = 60;
/// How long a call waits for the diff to load the notes' files. The user's answer to the question
/// may already have taken most of an app call's 30 seconds.
const LOAD_WAIT: Duration = Duration::from_secs(3);

/// One finding as the call gave it.
#[derive(Clone)]
struct Finding {
    path: PathBuf,
    first_line: u32,
    last_line: u32,
    comment: String,
}

/// Answers the `review` family's tools.
pub(crate) fn answer(call: AppCall, cx: &App) {
    let tool = call.tool.clone();
    match tool.as_str() {
        "review_notes_add" => notes_add(call, cx),
        other => call.answer(Err(format!("Marley answers no tool named {other}"))),
    }
}

/// `review_notes_add`: the findings checked, the user asked, then the notes placed.
fn notes_add(call: AppCall, cx: &App) {
    let findings = match findings_of(&call.arguments) {
        Ok(findings) => findings,
        Err(refusal) => {
            call.answer(Err(refusal));
            return;
        }
    };
    let source = source_of(&call, cx);
    let Some((multi_workspace, workspace)) = findings
        .first()
        .and_then(|first| crate::editor_tools::project_holding(&first.path, cx))
    else {
        call.answer(Err(Refusal::new(
            "outside_projects",
            "the first note's file is in none of the open projects' folders",
        )
        .next("give files inside an open project; list them with editor_list")));
        return;
    };
    let what = format!(
        "put {} review notes from {source} into the project diff",
        findings.len()
    );
    let admitted = admit(&call, Area::Editors, Level::Act, what, cx);
    cx.spawn(async move |cx| {
        if let Err(refusal) = admitted.await {
            call.answer(Err(refusal));
            return;
        }
        let answer = place(&findings, &source, multi_workspace, &workspace, cx).await;
        call.answer(answer);
    })
    .detach();
}

/// The call's notes: absolute paths, lines from 1, words, at most fifty.
fn findings_of(arguments: &Value) -> Result<Vec<Finding>, Refusal> {
    let notes = arguments
        .get("notes")
        .and_then(Value::as_array)
        .filter(|notes| !notes.is_empty())
        .ok_or_else(|| Refusal::new("bad_argument", "give the findings in `notes`"))?;
    if notes.len() > NOTES_MAX {
        return Err(Refusal::new(
            "bad_argument",
            format!("at most {NOTES_MAX} notes in one call"),
        )
        .next("send the rest in another call"));
    }
    notes
        .iter()
        .enumerate()
        .map(|(index, note)| {
            let bad =
                |what: &str| Refusal::new("bad_argument", format!("note {}: {what}", index + 1));
            let path = note
                .get("path")
                .and_then(Value::as_str)
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
                .ok_or_else(|| bad("give the file's absolute `path`"))?;
            let line = |name: &str| {
                note.get(name)
                    .and_then(Value::as_u64)
                    .and_then(|line| u32::try_from(line).ok())
                    .filter(|line| *line >= 1)
            };
            let first_line = line("first_line").ok_or_else(|| bad("`first_line` counts from 1"))?;
            let last_line = line("last_line").unwrap_or(first_line).max(first_line);
            let comment: String = note
                .get("comment")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|comment| !comment.is_empty())
                .ok_or_else(|| bad("give the note's `comment`"))?
                .chars()
                .take(COMMENT_MAX)
                .collect();
            Ok(Finding {
                path,
                first_line,
                last_line,
                comment,
            })
        })
        .collect()
}

/// The findings' source: the call's `source`, else the caller's name; one line, no control
/// characters.
fn source_of(call: &AppCall, cx: &App) -> String {
    let given = call
        .arguments
        .get("source")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|source| !source.is_empty())
        .map(str::to_string);
    given
        .unwrap_or_else(|| crate::click_pause::Who::of(call, cx).words)
        .chars()
        .filter(|character| !character.is_control())
        .take(SOURCE_MAX)
        .collect()
}

/// Shows the project diff in the project's window, waits for the notes' files to load in it, and
/// places each note it can; the answer lists the placed and the refused.
async fn place(
    findings: &[Finding],
    source: &str,
    multi_workspace: WindowHandle<MultiWorkspace>,
    workspace: &Entity<Workspace>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, Refusal> {
    let window = AnyWindowHandle::from(multi_workspace);
    let first = findings
        .first()
        .ok_or_else(|| Refusal::new("bad_argument", "give the findings in `notes`"))?;
    let project_path = cx
        .update(|cx| {
            workspace
                .read(cx)
                .project()
                .read(cx)
                .find_project_path(&first.path, cx)
        })
        .ok_or_else(|| Refusal::new("outside_projects", "the file is in no project's folder"))?;
    window
        .update(cx, |_, window, cx| {
            if let Some(root) = window.root::<MultiWorkspace>().flatten() {
                root.update(cx, |multi_workspace, cx| {
                    multi_workspace.activate(workspace.clone(), None, window, cx);
                });
            }
            workspace.update(cx, |workspace, cx| {
                ProjectDiff::deploy_at_project_path(workspace, project_path, window, cx);
            });
        })
        .map_err(|_| Refusal::new("no_window", "the project's window is gone"))?;
    let editor = cx
        .update(|cx| {
            let diff = workspace.read(cx).items_of_type::<ProjectDiff>(cx).next()?;
            Some(diff.read(cx).editor(cx).read(cx).rhs_editor().clone())
        })
        .ok_or_else(|| Refusal::new("no_diff", "the project's diff did not open"))?;

    // The diff loads its files a few at a time; a file with no change never loads.
    let started = Instant::now();
    let ranges = loop {
        let ranges: Vec<Result<Range<Anchor>, &'static str>> = cx.update(|cx| {
            findings
                .iter()
                .map(|finding| range_in_diff(&editor, workspace, finding, cx))
                .collect()
        });
        let waiting = ranges.iter().any(|range| matches!(range, Err(NOT_LOADED)));
        if !waiting || started.elapsed() >= LOAD_WAIT {
            break ranges;
        }
        cx.background_executor()
            .timer(Duration::from_millis(100))
            .await;
    };

    let mut placed = Vec::new();
    let mut refused = Vec::new();
    for (finding, range) in findings.iter().zip(ranges) {
        let note = json!({
            "path": finding.path.display().to_string(),
            "first_line": finding.first_line,
            "last_line": finding.last_line,
        });
        let range = match range {
            Ok(range) => range,
            Err(reason) => {
                refused.push(json!({
                    "path": finding.path.display().to_string(),
                    "first_line": finding.first_line,
                    "reason": reason,
                }));
                continue;
            }
        };
        let comment = finding.comment.clone();
        let source = SharedString::from(source.to_string());
        let id = window
            .update(cx, |_, window, cx| {
                editor.update(cx, |editor, cx| {
                    editor.add_review_note(range, comment, source, window, cx)
                })
            })
            .ok()
            .flatten();
        if id.is_some() {
            placed.push(note);
        } else {
            refused.push(json!({
                "path": finding.path.display().to_string(),
                "first_line": finding.first_line,
                "reason": "the diff could not take the note",
            }));
        }
    }
    let text = format!(
        "{} notes from {source} wait in the project diff for the user to keep or drop; {} left out.",
        placed.len(),
        refused.len()
    );
    Ok(ToolAnswer {
        structured: json!({ "source": source, "placed": placed, "refused": refused }),
        text: Some(text),
        image: None,
    })
}

/// Why a note has no place yet: its file has not loaded in the diff, which a file with no change
/// never does.
const NOT_LOADED: &str = "the file has no changes in the project diff";

/// The anchors of `finding`'s lines in the diff, or why they have none.
fn range_in_diff(
    editor: &Entity<Editor>,
    workspace: &Entity<Workspace>,
    finding: &Finding,
    cx: &App,
) -> Result<Range<Anchor>, &'static str> {
    if !in_project(workspace, &finding.path, cx) {
        return Err("the file is not in the first note's project");
    }
    let multibuffer = editor.read(cx).buffer().read(cx);
    let buffer = multibuffer
        .all_buffers()
        .into_iter()
        .find(|buffer| {
            buffer
                .read(cx)
                .file()
                .and_then(|file| file.as_local())
                .is_some_and(|file| file.abs_path(cx) == finding.path)
        })
        .ok_or(NOT_LOADED)?;
    let text = buffer.read(cx).snapshot();
    let last_row = finding.last_line - 1;
    if last_row > text.max_point().row {
        return Err("the lines run past the file's end");
    }
    let start = text.anchor_before(Point::new(finding.first_line - 1, 0));
    let end = text.anchor_after(Point::new(last_row, text.line_len(last_row)));
    let snapshot = multibuffer.snapshot(cx);
    let start = snapshot
        .anchor_in_excerpt(start)
        .ok_or("the lines are not shown in the project diff")?;
    let end = snapshot
        .anchor_in_excerpt(end)
        .ok_or("the lines are not shown in the project diff")?;
    Ok(start..end)
}

/// Whether `path` is inside one of `workspace`'s visible folders.
fn in_project(workspace: &Entity<Workspace>, path: &Path, cx: &App) -> bool {
    workspace
        .read(cx)
        .project()
        .read(cx)
        .visible_worktrees(cx)
        .any(|worktree| path.starts_with(worktree.read(cx).abs_path()))
}
