//! `show` from a Marley terminal (#805): a file or a command's stdin shown in a tab beside the
//! terminal, as a table, a component tree, markdown or an image.
//!
//! The terminal's `marley-agent show` sends a path over the agent socket, having copied stdin to a
//! private file beside the socket first. This module reads the file off the main thread, checks it
//! against its kind, answers the program, and opens the tab: a file Zed's table preview reads in
//! that preview, any other table parsed into a `BlockTable`, a tree in #800's component tab,
//! markdown in Zed's markdown preview, and an image in Zed's image viewer. A table or a component
//! tab of the same title is reused.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use editor::Editor;
use gpui::{AnyWindowHandle, App, AppContext as _, Entity, TaskExt as _, Window};
use markdown_preview::markdown_preview_view::MarkdownPreviewView;
use marley_agent::report::{self, Refusal, ShowKind};
use marley_components::{Templates, Tree, parse_str_with};
use marley_mcp::agent_socket::AgentRequest;
use marley_terminal::tables::{self, Table};
use terminal_view::TerminalView;
use ui::SharedString;
use util::ResultExt as _;
use workspace::{OpenOptions, OpenVisible, Workspace};

/// The largest file `show` reads.
const MOST_BYTES: u64 = 16 * 1024 * 1024;

/// The extensions of the files Zed's table preview reads whole.
const PREVIEWED: [&str; 5] = ["csv", "tsv", "psv", "ssv", "jsonl"];

/// What the program asked to show.
pub(crate) struct Show {
    pub(crate) path: PathBuf,
    pub(crate) kind: ShowKind,
    pub(crate) title: Option<String>,
    pub(crate) stdin: bool,
}

/// A file read and checked, ready for its tab.
enum Shown {
    /// A file Zed's table preview reads.
    TableFile(PathBuf),
    Table(Table),
    Component(Tree),
    Markdown(PathBuf),
    Image(PathBuf),
}

/// Reads and checks `show`, answers the program, and opens its tab beside `view`.
pub(crate) fn arrived(
    request: AgentRequest,
    workspace: Entity<Workspace>,
    view: Entity<TerminalView>,
    show: Show,
    cx: &App,
) {
    let templates = crate::component_library::templates(cx);
    let title = SharedString::from(show.title.clone().unwrap_or_else(|| {
        show.path
            .file_name()
            .map_or_else(
                || show.path.to_string_lossy(),
                |name| name.to_string_lossy(),
            )
            .into_owned()
    }));
    cx.spawn(async move |cx| {
        let read = smol::unblock(move || read_as(&show, &templates)).await;
        match read {
            Err(reason) => request.answer(report::answer_refused(Refusal::Show, &reason)),
            Ok(shown) => {
                request.answer(report::answer_ok());
                cx.update(|cx| open(shown, title, &workspace, &view, cx));
            }
        }
    })
    .detach();
}

/// `show`'s file read as its kind, or why it cannot be. A stdin copy read into a table or a tree
/// is deleted; one a tab reads from stays.
fn read_as(show: &Show, templates: &Arc<Templates>) -> Result<Shown, String> {
    let path = &show.path;
    let metadata =
        std::fs::metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("{} is not a file", path.display()));
    }
    if metadata.len() > MOST_BYTES {
        return Err(format!("{} is larger than 16 MB", path.display()));
    }
    let text =
        || std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()));
    let as_table = |text: &str| {
        tables::parse(text).ok_or_else(|| {
            "the input does not read as a table: no consistent columns, JSON objects or markdown \
             table"
                .to_string()
        })
    };
    let shown = match show.kind {
        ShowKind::Image => Ok(Shown::Image(path.clone())),
        ShowKind::Markdown => text().map(|_| Shown::Markdown(path.clone())),
        ShowKind::Table if !show.stdin && previewed(path) => Ok(Shown::TableFile(path.clone())),
        ShowKind::Table => as_table(&text()?).map(Shown::Table),
        ShowKind::Component => parse_str_with(&text()?, templates)
            .map(Shown::Component)
            .map_err(|error| error.to_string()),
        ShowKind::Json => {
            let text = text()?;
            match parse_str_with(&text, templates) {
                Ok(tree) => Ok(Shown::Component(tree)),
                Err(error) => tables::parse(&text)
                    .map(Shown::Table)
                    .ok_or_else(|| error.to_string()),
            }
        }
    };
    if show.stdin && matches!(shown, Ok(Shown::Table(_) | Shown::Component(_))) {
        std::fs::remove_file(path).log_err();
    }
    shown
}

/// Whether Zed's table preview reads `path` by its extension.
fn previewed(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| PREVIEWED.contains(&extension.to_ascii_lowercase().as_str()))
}

/// Opens `shown`'s tab beside `view`, in its window.
fn open(
    shown: Shown,
    title: SharedString,
    workspace: &Entity<Workspace>,
    view: &Entity<TerminalView>,
    cx: &mut App,
) {
    let Some(window) = crate::browser::window_of(workspace, cx) else {
        return;
    };
    AnyWindowHandle::from(window)
        .update(cx, |_, window, cx| match shown {
            Shown::TableFile(path) => {
                crate::block_tables::open_file(workspace, view, None, path, window, cx);
            }
            Shown::Table(table) => {
                crate::block_tables::show_table(workspace, view, title, table, window, cx);
            }
            Shown::Component(tree) => workspace.update(cx, |workspace, cx| {
                crate::component_tools::open_tab(
                    workspace,
                    view,
                    title.to_string(),
                    tree,
                    window,
                    cx,
                );
            }),
            Shown::Markdown(path) => open_markdown(workspace, view, &path, window, cx),
            Shown::Image(path) => {
                let options = OpenOptions {
                    visible: Some(OpenVisible::None),
                    focus: Some(true),
                    ..OpenOptions::default()
                };
                workspace
                    .update(cx, |workspace, cx| {
                        workspace.open_abs_path(path, options, window, cx)
                    })
                    .detach_and_log_err(cx);
            }
        })
        .log_err();
}

/// Opens Zed's markdown preview of `path` in the terminal's pane, over an editor no pane shows.
fn open_markdown(
    workspace: &Entity<Workspace>,
    view: &Entity<TerminalView>,
    path: &Path,
    window: &Window,
    cx: &mut App,
) {
    let project = workspace.read(cx).project().clone();
    let buffer = project.update(cx, |project, cx| project.open_local_buffer(path, cx));
    let workspace = workspace.downgrade();
    let view = view.downgrade();
    window
        .spawn(cx, async move |cx| {
            let buffer = buffer.await?;
            workspace.update_in(cx, |workspace, window, cx| {
                let editor = cx.new(|cx| Editor::for_buffer(buffer, Some(project), window, cx));
                let pane = view
                    .upgrade()
                    .and_then(|view| workspace.pane_for(&view))
                    .unwrap_or_else(|| workspace.active_pane().clone());
                MarkdownPreviewView::open_preview_in_pane(workspace, editor, pane, window, cx);
            })
        })
        .detach_and_log_err(cx);
}
