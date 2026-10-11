//! Zed's language servers for agents over Marley's MCP server (#712): `editor_diagnostics`,
//! `editor_definition`, `editor_references` and `editor_hover`, in the editors area at read level.
//!
//! A position is a file's absolute path inside an open project's folders, a 1-based line, and a
//! symbol's name on that line or a 1-based column counted as `editor_list` counts it (bytes within
//! the line). The file is opened in its project without a tab and registered with its language
//! servers for the call; the call waits a short while for a server to run for it, then a short
//! while for the server's answer, inside the MCP server's own wait. Answers carry 1-based
//! positions and each location's line, masked by the agents' redaction, and a file matching the
//! secret globs keeps its path and line but not its text. Lists come a page at a time. Marley's
//! own failed-block diagnostics (#623) are left out: they are not a language server's.

use std::path::{Path, PathBuf};
use std::time::Duration;

use futures::future::Either;
use gpui::{App, AppContext as _, AsyncApp, Entity, Task};
use language::{Anchor, Buffer, BufferSnapshot, Point, ToPoint as _};
use lsp::{DiagnosticSeverity, NumberOrString};
use marley_mcp::redact::Redactor;
use marley_mcp::{AppCall, Caller, Refusal, ToolAnswer};
use project::lsp_store::OpenLspBufferHandle;
use project::{Location, Project, ProjectPath};
use serde_json::{Value, json};
use workspace::Workspace;

use crate::agent_control::{Area, Level, admit};

/// How long a call waits for a language server to run for its file.
const SERVER_WAIT: Duration = Duration::from_secs(15);
/// How long a call waits for the server's answer.
const ANSWER_WAIT: Duration = Duration::from_secs(10);
/// How long a file opened for the call waits for its first diagnostics.
const FIRST_DIAGNOSTICS_WAIT: Duration = Duration::from_secs(3);
/// How often a wait looks again.
const POLL: Duration = Duration::from_millis(200);
/// The most entries a page holds.
const PAGE_ENTRIES: usize = 200;
/// The most files a project listing reads.
const LISTED_FILES: usize = 200;
/// The most characters of a location's line an answer carries.
const LINE_CHARS: usize = 200;
/// What a location in a secret file shows in place of its line.
const SECRET_LINE: &str = "[secret file]";

/// Answers the four language server tools.
pub(crate) fn answer(mut call: AppCall, cx: &App) {
    // Each answer hands an agent text from the project's files (#703).
    crate::agent_activity::log(&mut call, cx);
    let admitted = admit(&call, Area::Editors, Level::Read, String::new(), cx);
    cx.spawn(async move |cx| {
        let answer = match admitted.await {
            Ok(()) => respond(&call, cx).await,
            Err(refusal) => Err(refusal),
        };
        call.answer(answer);
    })
    .detach();
}

async fn respond(call: &AppCall, cx: &AsyncApp) -> Result<ToolAnswer, Refusal> {
    let arguments = &call.arguments;
    match call.tool.as_str() {
        "editor_diagnostics" if arguments.get("path").is_some() => {
            file_diagnostics(arguments, cx).await
        }
        "editor_diagnostics" => {
            let caller = call.caller().clone();
            project_diagnostics(arguments, &caller, cx).await
        }
        "editor_definition" => definition(arguments, cx).await,
        "editor_references" => references(arguments, cx).await,
        "editor_hover" => hover(arguments, cx).await,
        other => Err(Refusal::from(format!(
            "Marley answers no tool named {other}"
        ))),
    }
}

/// A file opened for a call, registered with its language servers until the call ends.
struct Opened {
    project: Entity<Project>,
    buffer: Entity<Buffer>,
    path: PathBuf,
    /// Whether a server had the file open before this call.
    was_open: bool,
    _registered: OpenLspBufferHandle,
}

/// `editor_definition`.
async fn definition(arguments: &Value, cx: &AsyncApp) -> Result<ToolAnswer, Refusal> {
    let (opened, point) = at_position(arguments, cx).await?;
    let asked = cx.update(|cx| {
        opened.project.update(cx, |project, cx| {
            project.definitions(&opened.buffer, point, cx)
        })
    });
    let links = within(asked, cx)
        .await
        .ok_or_else(answer_timeout)?
        .map_err(|error| Refusal::new("lsp_error", format!("{error:#}")))?
        .unwrap_or_default();
    let (definitions, redacted) = cx.update(|cx| {
        let redactor = crate::mcp::agent_redactor(cx);
        let mut redacted = 0;
        let definitions: Vec<Value> = links
            .iter()
            .map(|link| {
                let (location, count) = location_json(&link.target, redactor.as_deref(), cx);
                redacted += count;
                location
            })
            .collect();
        (definitions, redacted)
    });
    let text = listing("definition", &definitions);
    Ok(ToolAnswer {
        structured: json!({
            "path": opened.path.to_string_lossy(),
            "definitions": definitions,
            "redacted": redacted,
        }),
        text: Some(text),
        image: None,
    })
}

/// `editor_references`.
async fn references(arguments: &Value, cx: &AsyncApp) -> Result<ToolAnswer, Refusal> {
    let start = start_of(arguments);
    let (opened, point) = at_position(arguments, cx).await?;
    let asked = cx.update(|cx| {
        opened.project.update(cx, |project, cx| {
            project.references(&opened.buffer, point, cx)
        })
    });
    let locations = within(asked, cx)
        .await
        .ok_or_else(answer_timeout)?
        .map_err(|error| Refusal::new("lsp_error", format!("{error:#}")))?
        .unwrap_or_default();
    let (references, redacted) = cx.update(|cx| {
        let redactor = crate::mcp::agent_redactor(cx);
        let mut redacted = 0;
        let references: Vec<Value> = locations
            .iter()
            .map(|location| {
                let (location, count) = location_json(location, redactor.as_deref(), cx);
                redacted += count;
                location
            })
            .collect();
        (references, redacted)
    });
    let page = page_of(references, start)?;
    let text = listing("reference", &page.entries);
    Ok(ToolAnswer {
        structured: json!({
            "path": opened.path.to_string_lossy(),
            "references": page.entries,
            "total": page.total,
            "start": start,
            "next": page.next,
            "redacted": redacted,
        }),
        text: Some(text),
        image: None,
    })
}

/// `editor_hover`.
async fn hover(arguments: &Value, cx: &AsyncApp) -> Result<ToolAnswer, Refusal> {
    let (opened, point) = at_position(arguments, cx).await?;
    let asked = cx.update(|cx| {
        opened
            .project
            .update(cx, |project, cx| project.hover(&opened.buffer, point, cx))
    });
    let hovers = within(asked, cx)
        .await
        .ok_or_else(answer_timeout)?
        .unwrap_or_default();
    let (text, range) = cx.update(|cx| {
        let snapshot = opened.buffer.read(cx).snapshot();
        let text = hovers
            .iter()
            .flat_map(|hover| &hover.contents)
            .map(|block| block.text.trim())
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");
        let range = hovers
            .iter()
            .find_map(|hover| hover.range.clone())
            .map(|range| range_json(&range, &snapshot));
        (text, range)
    });
    let redacted =
        cx.update(|cx| crate::mcp::for_agents(&text, crate::mcp::agent_redactor(cx).as_deref()));
    let (shown, cut) = cut_to(&redacted.text, crate::mcp::PAGE_BYTES);
    Ok(ToolAnswer {
        structured: json!({
            "path": opened.path.to_string_lossy(),
            "text": shown,
            "range": range.unwrap_or(Value::Null),
            "cut": cut,
            "redacted": redacted.count,
        }),
        text: Some(if shown.is_empty() {
            "No hover text here.".to_string()
        } else {
            shown.to_string()
        }),
        image: None,
    })
}

/// `editor_diagnostics` for one file: pulled afresh where its server pulls, else as it last
/// published them; a file this call opened gets a moment for its first ones.
async fn file_diagnostics(arguments: &Value, cx: &AsyncApp) -> Result<ToolAnswer, Refusal> {
    let start = start_of(arguments);
    let path = path_of(arguments)?;
    let opened = open(&path, cx).await?;
    let pulled = cx.update(|cx| {
        let lsp_store = opened.project.read(cx).lsp_store();
        lsp_store.update(cx, |store, cx| {
            store.pull_diagnostics_for_buffer(opened.buffer.clone(), cx)
        })
    });
    if let Some(Err(error)) = within(pulled, cx).await {
        log::debug!("lsp tools: pulling diagnostics: {error:#}");
    }
    if !opened.was_open {
        let waited = std::time::Instant::now();
        while waited.elapsed() < FIRST_DIAGNOSTICS_WAIT
            && cx.update(|cx| entries_of(&opened.buffer, None, cx).is_empty())
        {
            cx.background_executor().timer(POLL).await;
        }
    }
    let (entries, redacted) = cx.update(|cx| {
        let redactor = crate::mcp::agent_redactor(cx);
        let mut redacted = 0;
        let entries: Vec<Value> = entries_of(&opened.buffer, redactor.as_deref(), cx)
            .into_iter()
            .map(|(entry, count)| {
                redacted += count;
                entry
            })
            .collect();
        (entries, redacted)
    });
    diagnostics_answer(entries, start, redacted, 0)
}

/// `editor_diagnostics` for a project: every file its servers counted errors or warnings in, as
/// they last published them, a page at a time.
async fn project_diagnostics(
    arguments: &Value,
    caller: &Caller,
    cx: &AsyncApp,
) -> Result<ToolAnswer, Refusal> {
    let start = start_of(arguments);
    let named = arguments.get("project").and_then(Value::as_str);
    let (projects, paths, hidden) = cx.update(|cx| {
        let projects = projects_for(named, caller, cx);
        let secrets = crate::editor_tools::secret_globs(cx);
        let mut paths: Vec<(Entity<Project>, ProjectPath)> = Vec::new();
        let mut hidden = 0;
        for project in &projects {
            for (path, _, summary) in project.read(cx).diagnostic_summaries(false, cx) {
                if summary.error_count + summary.warning_count == 0
                    || paths.iter().any(|(_, listed)| *listed == path)
                {
                    continue;
                }
                let secret = path
                    .path
                    .file_name()
                    .is_some_and(|name| secrets.is_match_std_path(Path::new(name)));
                if secret {
                    hidden += 1;
                } else {
                    paths.push((project.clone(), path));
                }
            }
        }
        paths.truncate(LISTED_FILES);
        (projects, paths, hidden)
    });
    if projects.is_empty() {
        return Err(Refusal::new("no_project", "no open project matches")
            .next("name an open project, or give a file's path"));
    }
    let mut entries = Vec::new();
    let mut redacted = 0;
    for (project, path) in paths {
        let opening =
            cx.update(|cx| project.update(cx, |project, cx| project.open_buffer(path, cx)));
        let buffer = match opening.await {
            Ok(buffer) => buffer,
            Err(error) => {
                log::debug!("lsp tools: opening a file for its diagnostics: {error:#}");
                continue;
            }
        };
        cx.update(|cx| {
            let redactor = crate::mcp::agent_redactor(cx);
            for (entry, count) in entries_of(&buffer, redactor.as_deref(), cx) {
                redacted += count;
                entries.push(entry);
            }
        });
    }
    diagnostics_answer(entries, start, redacted, hidden)
}

/// The projects a listing reads: the one `named` names (its name or a folder of it), else the
/// caller's, else every open local one.
fn projects_for(named: Option<&str>, caller: &Caller, cx: &App) -> Vec<Entity<Project>> {
    let local: Vec<Entity<Workspace>> = crate::mcp::all_workspaces(cx)
        .into_iter()
        .filter(|workspace| workspace.read(cx).project().read(cx).is_local())
        .collect();
    let chosen: Vec<Entity<Workspace>> = match named {
        Some(named) => local
            .into_iter()
            .filter(|workspace| {
                workspace.read(cx).root_paths(cx).iter().any(|root| {
                    root.file_name().is_some_and(|name| name == named)
                        || Path::new(named).starts_with(root.as_ref())
                })
            })
            .collect(),
        None => crate::browser_tools::caller_workspace(caller, cx)
            .map_or(local, |workspace| vec![workspace]),
    };
    let mut projects: Vec<Entity<Project>> = Vec::new();
    for workspace in chosen {
        let project = workspace.read(cx).project().clone();
        if !projects.contains(&project) {
            projects.push(project);
        }
    }
    projects
}

/// The diagnostics answer: a page of `entries` from `start`.
fn diagnostics_answer(
    entries: Vec<Value>,
    start: usize,
    redacted: usize,
    hidden: usize,
) -> Result<ToolAnswer, Refusal> {
    let page = page_of(entries, start)?;
    let text = if page.entries.is_empty() {
        "No diagnostics.".to_string()
    } else {
        page.entries
            .iter()
            .map(|entry| {
                format!(
                    "{}:{}:{}: {}: {}",
                    entry["path"].as_str().unwrap_or_default(),
                    entry["line"],
                    entry["column"],
                    entry["severity"].as_str().unwrap_or_default(),
                    entry["message"].as_str().unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    Ok(ToolAnswer {
        structured: json!({
            "diagnostics": page.entries,
            "total": page.total,
            "start": start,
            "next": page.next,
            "redacted": redacted,
            "hidden_secret_files": hidden,
        }),
        text: Some(text),
        image: None,
    })
}

/// `buffer`'s diagnostics, the primary entry of each group, without Marley's own (#623), each
/// with how many secrets its message lost.
fn entries_of(
    buffer: &Entity<Buffer>,
    redactor: Option<&Redactor>,
    cx: &App,
) -> Vec<(Value, usize)> {
    let buffer = buffer.read(cx);
    let Some(path) = buffer
        .file()
        .and_then(|file| file.as_local())
        .map(|file| file.abs_path(cx))
    else {
        return Vec::new();
    };
    let snapshot = buffer.snapshot();
    snapshot
        .diagnostic_groups(None)
        .into_iter()
        .filter_map(|(_, group)| {
            let entry = group.entries.get(group.primary_ix)?;
            let diagnostic = &entry.diagnostic;
            if diagnostic.source.as_deref() == Some("marley") {
                return None;
            }
            let severity = match diagnostic.severity {
                DiagnosticSeverity::ERROR => "error",
                DiagnosticSeverity::WARNING => "warning",
                DiagnosticSeverity::INFORMATION => "information",
                _ => "hint",
            };
            let message = crate::mcp::for_agents(diagnostic.message.as_str(), redactor);
            let mut value = range_json(&entry.range, &snapshot);
            if let Some(fields) = value.as_object_mut() {
                fields.insert("path".into(), json!(path.to_string_lossy()));
                fields.insert("severity".into(), json!(severity));
                fields.insert("message".into(), json!(message.text));
                fields.insert("source".into(), json!(diagnostic.source));
                fields.insert(
                    "code".into(),
                    json!(diagnostic.code.as_ref().map(|code| match code {
                        NumberOrString::Number(number) => number.to_string(),
                        NumberOrString::String(text) => text.clone(),
                    })),
                );
            }
            Some((value, message.count))
        })
        .collect()
}

/// The file and point `arguments` name, opened and registered with its language servers, once a
/// server runs for it.
async fn at_position(arguments: &Value, cx: &AsyncApp) -> Result<(Opened, Point), Refusal> {
    let path = path_of(arguments)?;
    let line = arguments
        .get("line")
        .and_then(Value::as_u64)
        .and_then(|line| u32::try_from(line).ok())
        .filter(|line| *line >= 1)
        .ok_or_else(|| {
            Refusal::new("bad_argument", "give the 1-based line").next("add \"line\": 1 or more")
        })?;
    let symbol = arguments
        .get("symbol")
        .and_then(Value::as_str)
        .map(str::to_string);
    let column = arguments
        .get("column")
        .and_then(Value::as_u64)
        .and_then(|column| u32::try_from(column).ok())
        .filter(|column| *column >= 1);
    if symbol.is_none() && column.is_none() {
        return Err(Refusal::new(
            "bad_argument",
            "give a symbol on the line, or a 1-based column",
        )
        .next("add \"symbol\": the name as it is written on that line"));
    }
    let opened = open(&path, cx).await?;
    let point = cx.update(|cx| {
        let snapshot = opened.buffer.read(cx).snapshot();
        point_in(&snapshot, line - 1, symbol.as_deref(), column)
    })?;
    Ok((opened, point))
}

/// The point on `row` where `symbol` first stands, else at `column`, clipped to the line.
fn point_in(
    snapshot: &BufferSnapshot,
    row: u32,
    symbol: Option<&str>,
    column: Option<u32>,
) -> Result<Point, Refusal> {
    let last = snapshot.max_point().row;
    if row > last {
        return Err(
            Refusal::new("bad_argument", format!("the file has {} lines", last + 1))
                .next("give a line within the file"),
        );
    }
    let line_len = snapshot.line_len(row);
    let text: String = snapshot
        .text_for_range(Point::new(row, 0)..Point::new(row, line_len))
        .collect();
    match symbol {
        Some(symbol) => {
            let offset = text.find(symbol).ok_or_else(|| {
                Refusal::new(
                    "no_symbol",
                    format!(
                        "{symbol} is not on line {}, which reads: {}",
                        row + 1,
                        text.trim()
                    ),
                )
                .next("give the name as it is written on that line, or a column")
            })?;
            Ok(Point::new(row, u32::try_from(offset).unwrap_or(line_len)))
        }
        None => Ok(Point::new(
            row,
            column.map_or(0, |column| (column - 1).min(line_len)),
        )),
    }
}

/// The absolute path `arguments` name.
fn path_of(arguments: &Value) -> Result<PathBuf, Refusal> {
    arguments
        .get("path")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .ok_or_else(|| {
            Refusal::new("bad_argument", "give the file's absolute path")
                .next("take the path from editor_list, or join the project's folder and the file")
        })
}

/// `path` opened in the project whose folders hold it, without a tab, and registered with its
/// language servers; refused for a secret file, a file outside the projects or no file, and when
/// no server runs for it within [`SERVER_WAIT`].
async fn open(path: &Path, cx: &AsyncApp) -> Result<Opened, Refusal> {
    let checked = cx.update(|cx| {
        crate::editor_tools::not_secret(path, cx)?;
        let outside = || {
            Refusal::new(
                "outside_projects",
                format!("{} is in no open project's folders", path.display()),
            )
            .next("give a file inside an open project")
        };
        let (_, workspace) = crate::editor_tools::project_holding(path, cx).ok_or_else(outside)?;
        let project = workspace.read(cx).project().clone();
        let project_path = project
            .read(cx)
            .find_project_path(path, cx)
            .ok_or_else(outside)?;
        Ok::<_, Refusal>((project, project_path))
    })?;
    let (project, project_path) = checked;
    let there = cx
        .background_spawn({
            let path = path.to_path_buf();
            async move {
                smol::fs::metadata(&path)
                    .await
                    .is_ok_and(|metadata| metadata.is_file())
            }
        })
        .await;
    if !there {
        return Err(
            Refusal::new("no_file", format!("{} is not a file", path.display()))
                .next("check the path"),
        );
    }
    let opening =
        cx.update(|cx| project.update(cx, |project, cx| project.open_buffer(project_path, cx)));
    let buffer = opening.await.map_err(|error| {
        Refusal::new(
            "no_file",
            format!("{} could not be opened: {error:#}", path.display()),
        )
        .next("check the path")
    })?;
    let was_open = cx.update(|cx| server_has(&project, &buffer, cx));
    let registered = cx.update(|cx| {
        project.update(cx, |project, cx| {
            project.register_buffer_with_language_servers(&buffer, cx)
        })
    });
    let waited = std::time::Instant::now();
    while !cx.update(|cx| server_has(&project, &buffer, cx)) {
        if waited.elapsed() >= SERVER_WAIT {
            return Err(Refusal::new(
                "no_language_server",
                format!(
                    "no language server runs for {} yet",
                    path.display()
                ),
            )
            .next("call again in a few seconds: Zed starts a file's language server when it first opens it")
            .next("check the language's server in the user's settings when it never starts"));
        }
        cx.background_executor().timer(POLL).await;
    }
    Ok(Opened {
        project,
        buffer,
        path: path.to_path_buf(),
        was_open,
        _registered: registered,
    })
}

/// Whether a language server has `buffer` open.
fn server_has(project: &Entity<Project>, buffer: &Entity<Buffer>, cx: &App) -> bool {
    let id = buffer.read(cx).remote_id();
    project
        .read(cx)
        .lsp_store()
        .read(cx)
        .language_server_ids_for_opened_buffer(id)
        .is_some_and(|servers| !servers.is_empty())
}

/// `task`'s value, unless [`ANSWER_WAIT`] passes first.
async fn within<T: 'static>(task: Task<T>, cx: &AsyncApp) -> Option<T> {
    let timer = cx.background_executor().timer(ANSWER_WAIT);
    match futures::future::select(Box::pin(task), timer).await {
        Either::Left((value, _)) => Some(value),
        Either::Right(_) => None,
    }
}

fn answer_timeout() -> Refusal {
    Refusal::new(
        "lsp_timeout",
        format!(
            "the language server did not answer within {} s",
            ANSWER_WAIT.as_secs()
        ),
    )
    .next("call again: a server still indexing answers later")
}

/// A location as an answer gives it: its path, its 1-based start and end, and its line, masked,
/// or `[secret file]`; with how many secrets the line lost.
fn location_json(location: &Location, redactor: Option<&Redactor>, cx: &App) -> (Value, usize) {
    let buffer = location.buffer.read(cx);
    let snapshot = buffer.snapshot();
    let path = buffer
        .file()
        .and_then(|file| file.as_local())
        .map(|file| file.abs_path(cx));
    let secret = path.as_ref().is_some_and(|path| {
        path.file_name()
            .is_some_and(|name| crate::editor_tools::secret_globs(cx).is_match_std_path(name))
    });
    let start = location.range.start.to_point(&snapshot);
    let (text, count) = if secret {
        (SECRET_LINE.to_string(), 0)
    } else {
        let line: String = snapshot
            .text_for_range(
                Point::new(start.row, 0)..Point::new(start.row, snapshot.line_len(start.row)),
            )
            .collect();
        let line: String = line.trim().chars().take(LINE_CHARS).collect();
        let redacted = crate::mcp::for_agents(&line, redactor);
        (redacted.text, redacted.count)
    };
    let mut value = range_json(&location.range, &snapshot);
    if let Some(fields) = value.as_object_mut() {
        fields.insert(
            "path".into(),
            json!(path.map(|path| path.to_string_lossy().into_owned())),
        );
        fields.insert("text".into(), json!(text));
    }
    (value, count)
}

/// A range's 1-based start and end, columns counted as `editor_list` counts them.
fn range_json(range: &std::ops::Range<Anchor>, snapshot: &BufferSnapshot) -> Value {
    let start = range.start.to_point(snapshot);
    let end = range.end.to_point(snapshot);
    json!({
        "line": start.row + 1,
        "column": start.column + 1,
        "end_line": end.row + 1,
        "end_column": end.column + 1,
    })
}

/// A page of entries.
struct Page {
    entries: Vec<Value>,
    total: usize,
    next: Option<usize>,
}

/// `entries` from `start`, at most [`PAGE_ENTRIES`] of them and [`crate::mcp::PAGE_BYTES`] of
/// JSON; `next` where the next page starts.
fn page_of(entries: Vec<Value>, start: usize) -> Result<Page, Refusal> {
    let total = entries.len();
    if start > total {
        return Err(Refusal::new(
            "bad_argument",
            format!("start {start} is past the {total} entries"),
        )
        .next("start from the last answer's next, or 0"));
    }
    let mut page = Vec::new();
    let mut bytes = 0;
    for entry in entries.into_iter().skip(start) {
        let size = entry.to_string().len();
        if page.len() == PAGE_ENTRIES || (!page.is_empty() && bytes + size > crate::mcp::PAGE_BYTES)
        {
            break;
        }
        bytes += size;
        page.push(entry);
    }
    let end = start + page.len();
    Ok(Page {
        entries: page,
        total,
        next: (end < total).then_some(end),
    })
}

/// The `start` argument; 0 when left out.
fn start_of(arguments: &Value) -> usize {
    arguments
        .get("start")
        .and_then(Value::as_u64)
        .and_then(|start| usize::try_from(start).ok())
        .unwrap_or_default()
}

/// `text` cut to at most `limit` bytes at a character's edge, and whether it was cut.
fn cut_to(text: &str, limit: usize) -> (&str, bool) {
    if text.len() <= limit {
        return (text, false);
    }
    let mut end = limit;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    (text.get(..end).unwrap_or_default(), true)
}

/// Locations as lines of text: `path:line:column: text`.
fn listing(what: &str, locations: &[Value]) -> String {
    if locations.is_empty() {
        return format!("No {what} found.");
    }
    locations
        .iter()
        .map(|location| {
            format!(
                "{}:{}:{}: {}",
                location["path"].as_str().unwrap_or_default(),
                location["line"],
                location["column"],
                location["text"].as_str().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
