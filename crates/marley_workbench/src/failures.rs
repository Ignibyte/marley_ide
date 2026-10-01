//! A failed block's located failures and Jump to First Failure (#620).
//!
//! Each block that finishes non-zero is read once, as it finishes: its output and the rows its
//! lines start on go through `marley_terminal::failures`, and what it found is kept by terminal
//! and block. The block menu's Jump to First Failure, the block's Jump chip and the
//! `marley::JumpToFirstFailure` action scroll the first failure's report to the top of the view,
//! select the block, and open the failing file at its line and column, against the folder the
//! block's command ran in.

use std::path::{Path, PathBuf};

use collections::HashMap;
use editor::Editor;
use gpui::{
    AnyElement, App, Context, Entity, EntityId, Global, InteractiveElement as _, IntoElement as _,
    ParentElement as _, TaskExt as _, Window,
};
use marley_terminal::BlockState;
use marley_terminal::failures::{self as located, Failure};
use terminal::Terminal;
use terminal_view::TerminalView;
use ui::{
    Button, ButtonCommon as _, ButtonStyle, Clickable as _, Color, Icon, IconName, IconSize,
    LabelSize, TintColor, Tooltip, h_flex,
};
use workspace::notifications::NotificationId;
use workspace::{OpenOptions, Toast, Workspace};

use crate::JumpToFirstFailure;

/// The failures of each terminal's failed blocks, read once as each finished.
#[derive(Default)]
struct BlockFailures {
    by_block: HashMap<(EntityId, usize), Vec<Failure>>,
    /// The index of the next block of each terminal to read.
    next: HashMap<EntityId, usize>,
}

impl Global for BlockFailures {}

/// The toasts of a failing file that would not open.
struct FailureFile;

/// Reads each terminal's blocks as they finish, and installs the action on every workspace.
/// [`crate::init`] calls it once, before any window opens.
pub fn init(cx: &mut App) {
    cx.set_global(BlockFailures::default());
    cx.observe_new(
        |view: &mut TerminalView, _, cx: &mut Context<TerminalView>| {
            let terminal = view.terminal().clone();
            let id = terminal.entity_id();
            cx.observe(&terminal, |_, terminal, cx| read_finished(&terminal, cx))
                .detach();
            cx.on_release(move |_, cx| {
                let failures = cx.default_global::<BlockFailures>();
                failures.next.remove(&id);
                failures.by_block.retain(|(terminal, _), _| *terminal != id);
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

/// Reads the blocks of `terminal` that finished since the last read, keeping a failed one's
/// failures. A terminal notifies at every output, so this returns at once when no block finished.
fn read_finished(terminal: &Entity<Terminal>, cx: &mut App) {
    let id = terminal.entity_id();
    let next = cx
        .try_global::<BlockFailures>()
        .and_then(|failures| failures.next.get(&id).copied())
        .unwrap_or(0);
    let (found, read_to) = {
        let terminal = terminal.read(cx);
        let blocks = terminal.blocks();
        let unread = blocks
            .iter()
            .rev()
            .take_while(|block| block.index >= next)
            .collect::<Vec<_>>();
        let mut read_to = next;
        let mut found = Vec::new();
        let mut rows = None;
        for block in unread.into_iter().rev() {
            if block.state != BlockState::Finished {
                break;
            }
            read_to = block.index + 1;
            if block.exit_code.0.is_none_or(|code| code == 0) {
                continue;
            }
            let Some(text) = terminal.block_output(block) else {
                continue;
            };
            let rows = rows.get_or_insert_with(|| terminal.marley_rows());
            let failures = located::failures(&text, block.output_start, &rows.wraps, rows.first);
            if !failures.is_empty() {
                found.push((block.index, failures));
            }
        }
        (found, read_to)
    };
    if read_to == next {
        return;
    }
    let failures = cx.default_global::<BlockFailures>();
    failures.next.insert(id, read_to);
    for (index, block_failures) in found {
        failures.by_block.insert((id, index), block_failures);
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
