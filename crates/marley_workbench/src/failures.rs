//! A failed block's located failures and Jump to First Failure (#620).
//!
//! Each block that finishes non-zero is read once, as it finishes: its output and the rows its
//! lines start on go through `marley_terminal::failures`, and what it found is kept by terminal
//! and block. The block menu's Jump to First Failure, the block's Jump chip and the
//! `marley::JumpToFirstFailure` action scroll the first failure's report to the top of the view,
//! select the block, and open the failing file at its line and column, against the folder the
//! block's command ran in.
//!
//! A failed block's failures inside the project are also its project's diagnostics (#623), under
//! a language server id no server takes, so the Diagnostics view and the status bar's counts show
//! them; the next run of the same command in the same folder replaces them, and a run that
//! succeeds clears them.

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use collections::HashMap;
use editor::Editor;
use gpui::{
    AnyElement, App, Context, Entity, EntityId, Global, InteractiveElement as _, IntoElement as _,
    ParentElement as _, Subscription, TaskExt as _, WeakEntity, Window,
};
use language::DiagnosticSourceKind;
use lsp::LanguageServerId;
use marley_terminal::BlockState;
use marley_terminal::failures::{self as located, Failure, Severity};
use project::lsp_store::DocumentDiagnosticsUpdate;
use terminal::Terminal;
use terminal_view::TerminalView;
use ui::{
    Button, ButtonCommon as _, ButtonStyle, Clickable as _, Color, Icon, IconName, IconSize,
    LabelSize, TintColor, Tooltip, h_flex,
};
use util::ResultExt as _;
use workspace::notifications::NotificationId;
use workspace::{OpenOptions, Toast, Workspace};

use crate::JumpToFirstFailure;

/// The failures of each terminal's failed blocks, read once as each finished.
#[derive(Default)]
struct BlockFailures {
    by_block: HashMap<(EntityId, usize), Vec<Failure>>,
    /// The index of the next block of each terminal to read.
    next: HashMap<EntityId, usize>,
    /// The files each command's last failed run published diagnostics for, by its command and
    /// folder (#623).
    published: HashMap<Command, Vec<PathBuf>>,
    /// The terminal each view shows and the watch on it: Zed's task rerun gives a view a new
    /// terminal (#623).
    watched: HashMap<EntityId, (EntityId, Subscription)>,
}

/// A command and the folder it ran in: its runs replace each other's diagnostics (#623).
type Command = (String, Option<String>);

/// The language server id blocks' diagnostics are published under (#623): servers count up from
/// zero, so none takes it.
const BLOCK_DIAGNOSTICS: LanguageServerId = LanguageServerId(usize::MAX - 623);

impl Global for BlockFailures {}

/// The toasts of a failing file that would not open.
struct FailureFile;

/// Reads each terminal's blocks as they finish, and installs the action on every workspace.
/// [`crate::init`] calls it once, before any window opens.
pub fn init(cx: &mut App) {
    cx.set_global(BlockFailures::default());
    cx.observe_new(
        |view: &mut TerminalView, _, cx: &mut Context<TerminalView>| {
            watch(view, cx);
            cx.observe_self(|view, cx| {
                let watched = cx
                    .try_global::<BlockFailures>()
                    .and_then(|failures| failures.watched.get(&cx.entity_id()))
                    .map(|(terminal, _)| *terminal);
                if watched != Some(view.terminal().entity_id()) {
                    watch(view, cx);
                }
            })
            .detach();
            let view_id = cx.entity_id();
            cx.on_release(move |_, cx| {
                let failures = cx.default_global::<BlockFailures>();
                if let Some((terminal, _)) = failures.watched.remove(&view_id) {
                    failures.forget(terminal);
                }
            })
            .detach();
        },
    )
    .detach();
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action_renderer(|div, _, _, cx| {
            div.on_action(
                cx.listener(|workspace, _: &JumpToFirstFailure, window, cx| {
                    jump_focused(workspace, window, cx);
                }),
            )
        });
    })
    .detach();
}

impl BlockFailures {
    /// Lets go of what a terminal no view shows any more had.
    fn forget(&mut self, terminal: EntityId) {
        self.next.remove(&terminal);
        self.by_block.retain(|(id, _), _| *id != terminal);
    }
}

/// Watches the terminal `view` shows, its blocks read as each finishes, in place of the one it
/// showed before.
fn watch(view: &TerminalView, cx: &mut Context<TerminalView>) {
    let terminal = view.terminal().clone();
    let subscription = cx.observe(&terminal, |view, terminal, cx| {
        let workspace = view.marley_workspace().clone();
        read_finished(&terminal, &workspace, cx);
    });
    let view_id = cx.entity_id();
    let failures = cx.default_global::<BlockFailures>();
    if let Some((earlier, _)) = failures
        .watched
        .insert(view_id, (terminal.entity_id(), subscription))
    {
        failures.forget(earlier);
    }
    let workspace = view.marley_workspace().clone();
    read_finished(&terminal, &workspace, cx);
}

/// Reads the blocks of `terminal` that finished since the last read, keeping a failed one's
/// failures and publishing them to `workspace`'s project (#623). A terminal notifies at every
/// output, so this returns at once when no block finished.
fn read_finished(terminal: &Entity<Terminal>, workspace: &WeakEntity<Workspace>, cx: &mut App) {
    let id = terminal.entity_id();
    let next = cx
        .try_global::<BlockFailures>()
        .and_then(|failures| failures.next.get(&id).copied())
        .unwrap_or(0);
    let (found, runs, read_to) = {
        let terminal = terminal.read(cx);
        let blocks = terminal.blocks();
        let unread = blocks
            .iter()
            .rev()
            .take_while(|block| block.index >= next)
            .collect::<Vec<_>>();
        let mut read_to = next;
        let mut found = Vec::new();
        // Each finished run's command, with its failures; none for a run that succeeded.
        let mut runs: Vec<(Command, Vec<Failure>)> = Vec::new();
        let mut rows = None;
        for block in unread.into_iter().rev() {
            if block.state != BlockState::Finished {
                break;
            }
            read_to = block.index + 1;
            let command = (block.command.clone(), block.prompt.pwd.clone());
            if block.exit_code.0.is_none_or(|code| code == 0) {
                runs.push((command, Vec::new()));
                continue;
            }
            let failures = terminal
                .block_output(block)
                .map(|text| {
                    let rows = rows.get_or_insert_with(|| terminal.marley_rows());
                    located::failures(&text, block.output_start, &rows.wraps, rows.first)
                })
                .unwrap_or_default();
            runs.push((command, failures.clone()));
            if !failures.is_empty() {
                found.push((block.index, failures));
            }
        }
        (found, runs, read_to)
    };
    if read_to == next {
        return;
    }
    let failures = cx.default_global::<BlockFailures>();
    failures.next.insert(id, read_to);
    for (index, block_failures) in found {
        failures.by_block.insert((id, index), block_failures);
    }
    publish(workspace, runs, cx);
}

/// Publishes each run's failures as diagnostics of `workspace`'s project, in place of what the
/// same command's last run published; a run with none clears them (#623). Only a local project
/// takes them, and only for files in its worktrees.
fn publish(workspace: &WeakEntity<Workspace>, runs: Vec<(Command, Vec<Failure>)>, cx: &mut App) {
    let Some(workspace) = workspace.upgrade() else {
        return;
    };
    let project = workspace.read(cx).project().clone();
    if !project.read(cx).is_local() {
        return;
    }
    let lsp_store = project.read(cx).lsp_store();
    for (command, failures) in runs {
        let mut by_file: HashMap<PathBuf, Vec<lsp::Diagnostic>> = HashMap::default();
        for failure in &failures {
            by_file
                .entry(place_of(&failure.path, command.1.as_deref()))
                .or_default()
                .push(diagnostic(failure));
        }
        let named: Vec<PathBuf> = by_file.keys().cloned().collect();
        let state = cx.default_global::<BlockFailures>();
        let earlier = state.published.remove(&command).unwrap_or_default();
        if !named.is_empty() {
            state.published.insert(command, named.clone());
        }
        // The earlier run's files this one names nothing in are cleared.
        let cleared = earlier
            .into_iter()
            .filter(|file| !named.contains(file))
            .map(|file| (file, Vec::new()));
        let updates: Vec<_> = cleared
            .chain(by_file)
            .filter_map(|(file, diagnostics)| {
                let uri = lsp::Uri::from_file_path(&file).ok()?;
                Some(DocumentDiagnosticsUpdate {
                    diagnostics: lsp::PublishDiagnosticsParams {
                        uri,
                        diagnostics,
                        version: None,
                    },
                    result_id: None,
                    registration_id: None,
                    server_id: BLOCK_DIAGNOSTICS,
                    disk_based_sources: Cow::Borrowed(&[]),
                })
            })
            .collect();
        if updates.is_empty() {
            continue;
        }
        lsp_store.update(cx, |store, cx| {
            store
                .merge_lsp_diagnostics(DiagnosticSourceKind::Pushed, updates, |_, _, _| false, cx)
                .log_err();
        });
    }
}

/// `failure` as a language server's diagnostic: at its line and column, an error or a warning.
fn diagnostic(failure: &Failure) -> lsp::Diagnostic {
    let start = lsp::Position::new(
        failure.line.saturating_sub(1),
        failure.column.unwrap_or(1).saturating_sub(1),
    );
    let end = lsp::Position::new(start.line, start.character.saturating_add(1));
    let message = if failure.message.is_empty() {
        format!("{}:{}", failure.path, failure.line)
    } else {
        failure.message.clone()
    };
    lsp::Diagnostic {
        range: lsp::Range::new(start, end),
        severity: Some(match failure.severity {
            Severity::Error => lsp::DiagnosticSeverity::ERROR,
            Severity::Warning => lsp::DiagnosticSeverity::WARNING,
        }),
        source: Some("marley".to_string()),
        message: lsp::DiagnosticMessage::from(message),
        ..lsp::Diagnostic::default()
    }
}

/// The first failure of block `index` of `terminal`, when it failed and one was found.
#[must_use]
pub fn first(terminal: &Entity<Terminal>, index: usize, cx: &App) -> Option<Failure> {
    let failures = cx
        .try_global::<BlockFailures>()?
        .by_block
        .get(&(terminal.entity_id(), index))?;
    located::first_failure(failures).cloned()
}

/// Jumps to the first failure of block `index` of `view`'s terminal: selects the block, scrolls
/// the failure's report to the view's second row, under the pinned command, and opens the failing file at its place, against
/// the folder the block's command ran in.
pub(crate) fn jump(view: &Entity<TerminalView>, index: usize, window: &mut Window, cx: &mut App) {
    let terminal = view.read(cx).terminal().clone();
    let Some(failure) = first(&terminal, index, cx) else {
        return;
    };
    let folder = terminal
        .read(cx)
        .blocks()
        .iter()
        .find(|block| block.index == index)
        .and_then(|block| block.prompt.pwd.clone());
    crate::blocks::select(&terminal, index, cx);
    // A block scrolled back pins its command over the top row (#529), so the report goes one row
    // below it.
    crate::blocks::reveal_line(&terminal, failure.row.saturating_sub(1), cx);
    view.update(cx, |_, cx| cx.notify());
    let Some(workspace) = view.read(cx).marley_workspace().upgrade() else {
        return;
    };
    let path = place_of(&failure.path, folder.as_deref());
    let point = language::Point::new(
        failure.line.saturating_sub(1),
        failure.column.unwrap_or(1).saturating_sub(1),
    );
    let opening = workspace.update(cx, |workspace, cx| {
        workspace.open_abs_path(path.clone(), OpenOptions::default(), window, cx)
    });
    let workspace = workspace.downgrade();
    window
        .spawn(cx, async move |cx| {
            match opening.await {
                Ok(item) => {
                    if let Some(editor) = cx.update(|_, cx| item.act_as::<Editor>(cx))? {
                        editor.update_in(cx, |editor, window, cx| {
                            editor.go_to_singleton_buffer_point(point, window, cx);
                        })?;
                    }
                }
                Err(error) => {
                    let message = format!("Could not open {}: {error:#}", path.display());
                    workspace.update(cx, |workspace, cx| {
                        workspace.show_toast(
                            Toast::new(NotificationId::unique::<FailureFile>(), message),
                            cx,
                        );
                    })?;
                }
            }
            anyhow::Ok(())
        })
        .detach_and_log_err(cx);
}

/// The file `path` names, against `folder` when it is relative.
fn place_of(path: &str, folder: Option<&str>) -> PathBuf {
    let path = Path::new(path);
    match folder {
        Some(folder) if path.is_relative() => Path::new(folder).join(path),
        _ => path.to_path_buf(),
    }
}

/// `marley::JumpToFirstFailure`: the focused terminal's newest failed block with a failure.
fn jump_focused(workspace: &Workspace, window: &Window, cx: &mut Context<Workspace>) {
    let Some(view) = crate::blocks::focused_terminal(workspace, window, cx) else {
        cx.propagate();
        return;
    };
    let terminal = view.read(cx).terminal().clone();
    let newest = terminal
        .read(cx)
        .blocks()
        .iter()
        .rev()
        .map(|block| block.index)
        .find(|index| first(&terminal, *index, cx).is_some());
    // The jump opens a file in this workspace, which this action's update holds.
    if let Some(index) = newest {
        window.defer(cx, move |window, cx| jump(&view, index, window, cx));
    }
}

/// The Jump chip of block `index`, for a failed block with a failure: its click jumps.
pub(crate) fn chip(
    view: &Entity<TerminalView>,
    terminal: &Entity<Terminal>,
    index: usize,
    cx: &App,
) -> Option<AnyElement> {
    let failure = first(terminal, index, cx)?;
    let view = view.clone();
    let place = format!("{}:{}", failure.path, failure.line);
    Some(
        h_flex()
            .child(
                Button::new(("marley-jump-to-failure", index), "Jump to Failure")
                    .style(ButtonStyle::Tinted(TintColor::Error))
                    .label_size(LabelSize::XSmall)
                    .end_icon(
                        Icon::new(IconName::ArrowUpRight)
                            .size(IconSize::XSmall)
                            .color(Color::Error),
                    )
                    .tooltip(Tooltip::text(format!("Jump to {place}")))
                    .on_click(move |_, window, cx| {
                        cx.stop_propagation();
                        jump(&view, index, window, cx);
                    }),
            )
            .into_any_element(),
    )
}
