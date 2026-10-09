//! Zed's open editors for agents over Marley's MCP server (#704): `editor_list`, `editor_read`
//! and `editor_open`, through the editors area's mode (`agent_control`) and #703's log and switch.
//!
//! An editor is a center tab showing one file of an open project. Its id is the editor's entity
//! id. A read gives the buffer as it is now, unsaved changes included, a page of lines at a time,
//! masked by the agents' redaction; a file whose name matches `marley.agent_control.secret_globs`
//! is refused. An open lands only inside an open project's folders.

use std::path::{Path, PathBuf};

use editor::{Editor, ToPoint as _};
use gpui::{AnyWindowHandle, App, Entity};
use marley_mcp::{AppCall, Refusal, ToolAnswer};
use serde_json::{Value, json};
use settings::SettingsStore;
use util::ResultExt as _;
use util::paths::{PathMatcher, PathStyle};
use workspace::{MultiWorkspace, OpenOptions, OpenVisible, Workspace};

use crate::agent_control::{Area, admit};

/// The most editors `editor_list` names.
const LISTED: usize = 200;

/// The secret globs while the user set none, the defaults in `default.json`.
const SECRET_GLOBS: [&str; 8] = [
    ".env*",
    "*.pem",
    "*.key",
    "id_rsa*",
    "id_ed25519*",
    "*credentials*",
    ".netrc",
    "*.p12",
];

/// Answers `editor_list`, `editor_read` and `editor_open`.
pub(crate) fn answer(mut call: AppCall, cx: &App) {
    let tool = call.tool.clone();
    match tool.as_str() {
        "editor_list" => after_admit(call, false, String::new(), list, cx),
        "editor_read" => {
            // A read reveals a file's text, so it is listed in Agent Activity (#703).
            crate::agent_activity::log(&mut call, cx);
            after_admit(call, false, String::new(), read, cx);
        }
        "editor_open" => {
            let what = open_words(&call.arguments);
            after_admit(call, true, what, open, cx);
        }
        other => call.answer(Err(format!("Marley answers no tool named {other}"))),
    }
}

/// Runs `then` once the editors area admits `call`, else answers the refusal.
fn after_admit(call: AppCall, acts: bool, what: String, then: fn(AppCall, &mut App), cx: &App) {
    let admitted = admit(&call, Area::Editors, acts, what, cx);
    cx.spawn(async move |cx| match admitted.await {
        Ok(()) => cx.update(|cx| then(call, cx)),
        Err(refusal) => call.answer(Err(refusal)),
    })
    .detach();
}

/// An open editor of one file.
struct OpenEditor {
    editor: Entity<Editor>,
    path: PathBuf,
    project: String,
    active: bool,
}

/// Every open editor of one file in Marley's windows, each once.
fn open_editors(cx: &App) -> Vec<OpenEditor> {
    let workspaces: Vec<Entity<Workspace>> = cx
        .windows()
        .into_iter()
        .filter_map(|window| window.downcast::<MultiWorkspace>()?.read(cx).ok())
        .flat_map(|multi_workspace| multi_workspace.workspaces().cloned().collect::<Vec<_>>())
        .collect();
    let mut found: Vec<OpenEditor> = Vec::new();
    for workspace in workspaces {
        for pane in workspace.read(cx).panes() {
            let active = pane.read(cx).active_item().map(|item| item.item_id());
            for editor in pane.read(cx).items_of_type::<Editor>() {
                if found.iter().any(|open| open.editor == editor) {
                    continue;
                }
                let Some(path) = file_path(&editor, cx) else {
                    continue;
                };
                found.push(OpenEditor {
                    project: project_of(&workspace, &path, cx),
                    active: active == Some(editor.entity_id()),
                    path,
                    editor,
                });
            }
        }
    }
    found
}

/// The absolute path of the one local file `editor` shows.
fn file_path(editor: &Entity<Editor>, cx: &App) -> Option<PathBuf> {
    let buffer = editor.read(cx).buffer().read(cx).as_singleton()?;
    let file = buffer.read(cx).file()?;
    Some(file.as_local()?.abs_path(cx))
}

/// The name of the project folder of `workspace` holding `path`, if one does.
fn project_of(workspace: &Entity<Workspace>, path: &Path, cx: &App) -> String {
    workspace
        .read(cx)
        .project()
        .read(cx)
        .visible_worktrees(cx)
        .find(|worktree| path.starts_with(worktree.read(cx).abs_path()))
        .map(|worktree| worktree.read(cx).root_name().as_unix_str().to_string())
        .unwrap_or_default()
}

/// `editor_list`.
fn list(call: AppCall, cx: &mut App) {
    let editors: Vec<Value> = open_editors(cx)
        .into_iter()
        .take(LISTED)
        .filter_map(|open| {
            let editor = open.editor.read(cx);
            let buffer = editor.buffer().read(cx).as_singleton()?;
            let buffer = buffer.read(cx);
            let snapshot = editor.buffer().read(cx).snapshot(cx);
            let selections: Vec<Value> = editor
                .selections
                .disjoint_anchors()
                .iter()
                .map(|selection| {
                    let start = selection.start.to_point(&snapshot);
                    let end = selection.end.to_point(&snapshot);
                    json!({
                        "start": { "line": start.row + 1, "column": start.column + 1 },
                        "end": { "line": end.row + 1, "column": end.column + 1 },
                    })
                })
                .collect();
            Some(json!({
                "id": open.editor.entity_id().as_u64(),
                "path": open.path.to_string_lossy(),
                "project": open.project,
                "dirty": buffer.is_dirty(),
                "language": buffer.language().map(|language| language.name().to_string()),
                "active": open.active,
                "selections": selections,
            }))
        })
        .collect();
    let text = format!("{} open editors", editors.len());
    call.answer::<Refusal>(Ok(ToolAnswer {
        structured: json!({ "editors": editors }),
        text: Some(text),
        image: None,
    }));
}

/// The patterns a file's name may not match to be read: the user's, else Marley's defaults.
fn secret_globs(cx: &App) -> PathMatcher {
    let globs: Vec<String> = cx
        .global::<SettingsStore>()
        .merged_settings()
        .marley
        .as_ref()
        .and_then(|marley| marley.agent_control.as_ref())
        .and_then(|agent_control| agent_control.secret_globs.clone())
        .unwrap_or_else(|| SECRET_GLOBS.iter().map(ToString::to_string).collect());
    PathMatcher::new_lenient(globs, PathStyle::local(), |error| {
        log::warn!("marley.agent_control.secret_globs: {error}");
    })
}

/// `editor_read`.
fn read(call: AppCall, cx: &mut App) {
    let open = match editor_named(&call.arguments, cx) {
        Ok(open) => open,
        Err(refusal) => {
            call.answer(Err(refusal));
            return;
        }
    };
    if let Some(name) = open.path.file_name()
        && secret_globs(cx).is_match_std_path(name)
    {
        call.answer(Err(Refusal::new(
            "secret_file",
            format!(
                "{} matches the user's secret patterns (marley.agent_control.secret_globs), so \
                 it is not read",
                open.path.display()
            ),
        )
        .next("ask the user for what you need from it")));
        return;
    }
    let Some(buffer) = open.editor.read(cx).buffer().read(cx).as_singleton() else {
        call.answer(Err("the editor no longer shows one file".to_string()));
        return;
    };
    let (text, dirty) = {
        let buffer = buffer.read(cx);
        (buffer.text(), buffer.is_dirty())
    };
    let redacted = crate::mcp::for_agents(&text, crate::mcp::agent_redactor(cx).as_deref());
    let after = call
        .arguments
        .get("start_line")
        .and_then(Value::as_u64)
        .and_then(|line| usize::try_from(line).ok())
        .filter(|line| *line > 1)
        .map(|line| line - 1);
    let page = match crate::mcp::page_from(&redacted.text, after) {
        Ok(page) => page,
        Err(refusal) => {
            call.answer(Err(refusal));
            return;
        }
    };
    let mut structured = json!({
        "id": open.editor.entity_id().as_u64(),
        "path": open.path.to_string_lossy(),
        "dirty": dirty,
        "redacted": redacted.count,
    });
    page.fill_forward(&mut structured);
    let next = structured.get("next").and_then(Value::as_u64);
    if let Some(fields) = structured.as_object_mut() {
        fields.insert(
            "next_line".into(),
            next.map_or(Value::Null, |next| Value::from(next + 1)),
        );
    }
    let text = structured
        .get("text")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let text = match next {
        Some(next) => format!(
            "[{}: read on with editor_read start_line={}]\n{text}",
            open.path.display(),
            next + 1
        ),
        None => text,
    };
    call.answer::<Refusal>(Ok(ToolAnswer {
        structured,
        text: Some(text),
        image: None,
    }));
}

/// The open editor `arguments` name by `id`, else by `path`.
fn editor_named(arguments: &Value, cx: &App) -> Result<OpenEditor, Refusal> {
    let id = arguments.get("id").and_then(Value::as_u64);
    let path = arguments
        .get("path")
        .and_then(Value::as_str)
        .map(PathBuf::from);
    if id.is_none() && path.is_none() {
        return Err(
            Refusal::new("bad_argument", "give an editor's id or its file's path")
                .next("take an id from editor_list"),
        );
    }
    open_editors(cx)
        .into_iter()
        .find(|open| match (id, &path) {
            (Some(id), _) => open.editor.entity_id().as_u64() == id,
            (None, Some(path)) => open.path == *path,
            (None, None) => false,
        })
        .ok_or_else(|| {
            Refusal::new("no_editor", "no open editor has that id or path")
                .next("list the open editors with editor_list, or open the file with editor_open")
        })
}

/// What `editor_open` would do, in words, for the question.
fn open_words(arguments: &Value) -> String {
    let path = arguments
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or_default();
    arguments.get("line").and_then(Value::as_u64).map_or_else(
        || format!("open {path}"),
        |line| format!("open {path} at line {line}"),
    )
}

/// `editor_open`.
fn open(call: AppCall, cx: &mut App) {
    let path = call
        .arguments
        .get("path")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute());
    let Some(path) = path else {
        call.answer(Err(Refusal::new(
            "bad_argument",
            "give the file's absolute path",
        )));
        return;
    };
    let position = |name: &str| {
        call.arguments
            .get(name)
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .filter(|value| *value >= 1)
            .unwrap_or(1)
    };
    let (line, column) = (position("line"), position("column"));
    let Some((multi_workspace, workspace)) = project_holding(&path, cx) else {
        call.answer(Err(Refusal::new(
            "outside_projects",
            format!(
                "{} is in none of the open projects' folders",
                path.display()
            ),
        )
        .next("open a file inside an open project; list them with editor_list")));
        return;
    };
    let options = OpenOptions {
        visible: Some(OpenVisible::None),
        focus: Some(true),
        ..OpenOptions::default()
    };
    let opening = AnyWindowHandle::from(multi_workspace).update(cx, |_, window, cx| {
        if let Some(root) = window.root::<MultiWorkspace>().flatten() {
            root.update(cx, |multi_workspace, cx| {
                multi_workspace.activate(workspace.clone(), None, window, cx);
            });
        }
        window.activate_window();
        workspace.update(cx, |workspace, cx| {
            workspace.open_abs_path(path.clone(), options, window, cx)
        })
    });
    let Ok(opening) = opening else {
        call.answer(Err("the project's window is gone".to_string()));
        return;
    };
    cx.spawn(async move |cx| {
        let item = match opening.await {
            Ok(item) => item,
            Err(error) => {
                call.answer(Err(format!(
                    "Marley could not open {}: {error:#}",
                    path.display()
                )));
                return;
            }
        };
        let Some(editor) = cx.update(|cx| item.act_as::<Editor>(cx)) else {
            call.answer(Err(format!("{} did not open in an editor", path.display())));
            return;
        };
        AnyWindowHandle::from(multi_workspace)
            .update(cx, |_, window, cx| {
                editor.update(cx, |editor, cx| {
                    editor.go_to_singleton_buffer_point(
                        language::Point::new(line - 1, column - 1),
                        window,
                        cx,
                    );
                });
            })
            .log_err();
        call.answer::<Refusal>(Ok(ToolAnswer {
            structured: json!({
                "id": editor.entity_id().as_u64(),
                "path": path.to_string_lossy(),
                "line": line,
                "column": column,
            }),
            text: Some(format!("{} is open at line {line}", path.display())),
            image: None,
        }));
    })
    .detach();
}

/// The window and the workspace whose project's folders hold `path`.
fn project_holding(
    path: &Path,
    cx: &App,
) -> Option<(gpui::WindowHandle<MultiWorkspace>, Entity<Workspace>)> {
    cx.windows().into_iter().find_map(|window| {
        let handle = window.downcast::<MultiWorkspace>()?;
        let workspace = handle
            .read(cx)
            .ok()?
            .workspaces()
            .find(|workspace| {
                workspace
                    .read(cx)
                    .project()
                    .read(cx)
                    .visible_worktrees(cx)
                    .any(|worktree| path.starts_with(worktree.read(cx).abs_path()))
            })?
            .clone();
        Some((handle, workspace))
    })
}
