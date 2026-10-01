//! Native block headers (#628): a block's prompt rows drawn as Marley's header.
//!
//! The header shows the command in the buffer font, in place of the prompt the shell drew, with
//! the folder the command ran in and its git branch (#630).
//!
//! The terminal's element finds each verified block's prompt rows on screen
//! (`marley_terminal::prompt_rows`) and asks the `MarleyBlockHeader` hook this module sets for the
//! header; when it gets one, it leaves those rows' cells out and lays the header over the same
//! rows, so no row moves, and the block's pill and hover actions draw over it. Over two rows or
//! more the folder and branch take the first row and the command the second; over one, they follow
//! the command. The branch is the one the folder was on when the block started, recorded once.
//! `marley.block_headers` turns it off.

use std::collections::HashMap;
use std::path::{Component, Path};
use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, EntityId, Global, MouseButton, Pixels};
use marley_terminal::BlockState;
use settings::Settings as _;
use terminal::Terminal;
use terminal_view::{MarleyBlockHeader, TerminalView};
use ui::prelude::*;

use crate::{BlockHeaders, MarleySettings};

/// The branch each block's folder was on when the block started, by its terminal's entity id and
/// its index; `None` for a folder on no branch.
#[derive(Default)]
struct Branches(HashMap<(EntityId, usize), Option<SharedString>>);

impl Global for Branches {}

/// Sets the header's hook and records the blocks' branches; [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    cx.set_global(MarleyBlockHeader(Arc::new(header)));
    cx.set_global(Branches::default());
    cx.observe_new(
        |view: &mut TerminalView, _, cx: &mut Context<TerminalView>| {
            let terminal = view.terminal().clone();
            // A block's start notifies the terminal.
            cx.observe(&terminal, |view, terminal, cx| {
                record_branches(view, &terminal, cx);
            })
            .detach();
            let terminal_id = terminal.entity_id();
            cx.on_release(move |_, cx| {
                if cx.has_global::<Branches>() {
                    cx.global_mut::<Branches>()
                        .0
                        .retain(|(terminal, _), _| *terminal != terminal_id);
                }
            })
            .detach();
        },
    )
    .detach();
}

/// Records the branch of each started block of `terminal` that has none recorded yet, from the
/// repository of the view's project that holds the block's folder.
fn record_branches(view: &TerminalView, terminal: &Entity<Terminal>, cx: &mut App) {
    if MarleySettings::get_global(cx).block_headers != BlockHeaders::Native {
        return;
    }
    let id = terminal.entity_id();
    let started: Vec<(usize, String)> = terminal
        .read(cx)
        .blocks()
        .iter()
        .filter(|block| block.state != BlockState::Pending)
        .filter(|block| !cx.global::<Branches>().0.contains_key(&(id, block.index)))
        .filter_map(|block| Some((block.index, block.prompt.pwd.clone()?)))
        .collect();
    if started.is_empty() {
        return;
    }
    let Some(project) = view
        .marley_workspace()
        .upgrade()
        .map(|workspace| workspace.read(cx).project().clone())
    else {
        return;
    };
    for (index, folder) in started {
        let branch = crate::agent_bar::branch_of(&project, Path::new(&folder), cx);
        cx.global_mut::<Branches>().0.insert((id, index), branch);
    }
}

/// The header over the `rows` prompt rows of the block at `index`, each `line_height` tall: the
/// folder and branch over the command, or after it on one row. None while the setting is off.
fn header(
    _view: &Entity<TerminalView>,
    terminal: &Entity<Terminal>,
    index: usize,
    line_height: Pixels,
    rows: usize,
    cx: &App,
) -> Option<AnyElement> {
    if MarleySettings::get_global(cx).block_headers != BlockHeaders::Native {
        return None;
    }
    let block = terminal.read(cx).blocks().get(index)?;
    let mut lines = block.command.trim().lines();
    let first = lines.next().unwrap_or_default();
    let command = if lines.next().is_some() {
        format!("{first} …")
    } else {
        first.to_string()
    };
    let branch = cx
        .try_global::<Branches>()
        .and_then(|branches| branches.0.get(&(terminal.entity_id(), index)).cloned())
        .flatten();
    let place = [
        block.prompt.pwd.as_deref().map(folder_label),
        branch.map(String::from),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" · ");
    let row = || h_flex().h(line_height).w_full().min_w_0().gap_2();
    // The command keeps its width; the folder and branch give way to it.
    let command = div()
        .flex_none()
        .child(Label::new(command).buffer_font(cx).single_line());
    let place = (!place.is_empty()).then(|| {
        div().flex_1().min_w_0().child(
            Label::new(place)
                .buffer_font(cx)
                .color(Color::Muted)
                .single_line()
                .truncate(),
        )
    });
    let header = if rows >= 2 {
        v_flex()
            .child(row().children(place))
            .child(row().child(command))
    } else {
        v_flex().child(row().child(command).children(place))
    };
    Some(
        header
            .id(("marley-block-header", index))
            .size_full()
            // The header stands for the prompt's text, which is not there to select: a press on
            // it and its release are the header's, so the terminal under it starts no selection
            // and reports no click to its program.
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_mouse_up(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .into_any_element(),
    )
}

/// `folder` as the header shows it, short enough to leave the branch room: the home folder as `~`,
/// and past two folders only the last two, after `…`.
fn folder_label(folder: &str) -> String {
    let path = Path::new(folder);
    let (root, rest) = path
        .strip_prefix(paths::home_dir())
        .map_or(("", path), |rest| ("~", rest));
    let names: Vec<&str> = rest
        .components()
        .filter_map(|component| match component {
            Component::Normal(name) => name.to_str(),
            _ => None,
        })
        .collect();
    match names.as_slice() {
        [] if root.is_empty() => "/".to_string(),
        [] => root.to_string(),
        [.., parent, name] if names.len() > 2 => format!("…/{parent}/{name}"),
        _ => format!("{root}/{}", names.join("/")),
    }
}
