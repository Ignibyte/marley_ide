//! Autosuggestions (T3a), as Warp and fish show them.
//!
//! While the shell waits at its prompt, the rest of the newest command in history that starts
//! with what was typed shows dimmed after the cursor, and → takes it. The history is the
//! terminal's own commands, newest first, then its shell's history file, which the shell
//! integration names and which is read once per file off the main thread.

use std::collections::HashMap;
use std::io::ErrorKind;
use std::sync::Arc;

use gpui::{App, AppContext as _, Context, Entity, Global};
use terminal::{Modes, Terminal};
use terminal_view::MarleyTerminalSuggestion;
use workspace::Workspace;

use crate::AcceptSuggestion;
use crate::blocks::focused_terminal;

/// The history files by the path the shell named them with, each file's commands oldest first,
/// and `None` while it is read.
#[derive(Default)]
struct HistoryFiles(HashMap<String, Option<Arc<Vec<String>>>>);

impl Global for HistoryFiles {}

/// Draws each terminal's suggestion and installs `marley::AcceptSuggestion` on every workspace.
/// [`crate::init`] calls it once, before any window opens.
pub fn init(cx: &mut App) {
    cx.set_global(HistoryFiles::default());
    cx.set_global(MarleyTerminalSuggestion(Arc::new(|terminal, cx| {
        read_history_once_drawn(terminal, cx);
        suggestion(terminal.read(cx), cx).map(Into::into)
    })));
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &AcceptSuggestion, window, cx| {
            let accepted = focused_terminal(workspace, window, cx).and_then(|view| {
                let terminal = view.read(cx).terminal().clone();
                let rest = suggestion(terminal.read(cx), cx)?;
                Some((terminal, rest))
            });
            match accepted {
                Some((terminal, rest)) => {
                    terminal.update(cx, |terminal, _| terminal.input(rest.into_bytes()));
                }
                // The key goes on to the terminal, which sends it to the program.
                None => cx.propagate(),
            }
        });
    })
    .detach();
}

/// The rest of the suggestion for what was typed at `terminal`'s prompt: its own commands the
/// shell's hook reported, newest first, then its history file's.
fn suggestion(terminal: &Terminal, cx: &App) -> Option<String> {
    let typed = typed_text(terminal)?;
    let anchored = terminal.marley_anchored();
    // The shell at the prompt's own commands, the local one's or an ssh host's, and the history
    // file only for the local shell, since a host's names a file on the host (#526).
    let host = anchored.prompt_shell()?.host();
    let own = anchored
        .blocks()
        .iter()
        .rev()
        .filter(|block| block.command_verified && anchored.block_host(block.index) == host)
        .map(|block| block.command.as_str());
    let file = host
        .is_none()
        .then(|| anchored.history_file())
        .flatten()
        .and_then(|path| cx.try_global::<HistoryFiles>()?.0.get(path)?.clone());
    let from_file = file
        .iter()
        .flat_map(|commands| commands.iter().rev().map(String::as_str));
    marley_terminal::suggestion(&typed, own.chain(from_file)).map(str::to_string)
}

/// What was typed at `terminal`'s prompt: its cells from where the first key after the prompt
/// was typed up to the cursor, while the cursor is on that line with nothing after it.
fn typed_text(terminal: &Terminal) -> Option<String> {
    let anchored = terminal.marley_anchored();
    if !anchored.at_prompt() || terminal.vi_mode_enabled() {
        return None;
    }
    let (start_line, start_column) = anchored.input_start()?;
    let content = terminal.last_content();
    if content.display_offset != 0 || content.mode.contains(Modes::ALT_SCREEN) {
        return None;
    }
    let cursor = content.cursor.point;
    let cursor_line = content.marley_screen_top + u64::try_from(cursor.line).ok()?;
    if cursor_line != start_line || cursor.column < start_column {
        return None;
    }
    let mut typed = String::new();
    for cell in content
        .cells
        .iter()
        .filter(|cell| cell.point.line == cursor.line && !cell.is_wide_char_spacer())
    {
        let column = cell.point.column;
        if (start_column..cursor.column).contains(&column) {
            typed.push(cell.character());
        } else if column >= cursor.column && cell.character() != ' ' {
            return None;
        }
    }
    Some(typed)
}

/// Reads `terminal`'s history file once the frame is drawn, the first time a terminal names it.
fn read_history_once_drawn(terminal: &Entity<Terminal>, cx: &mut App) {
    let Some(path) = terminal.read(cx).marley_anchored().history_file() else {
        return;
    };
    if !cx.global::<HistoryFiles>().0.contains_key(path) {
        let path = path.to_string();
        cx.defer(move |cx| read_history(path, cx));
    }
}

/// Reads the history file at `path` off the main thread, unless it is read already; a missing
/// file holds no commands.
fn read_history(path: String, cx: &mut App) {
    let files = cx.global_mut::<HistoryFiles>();
    if files.0.contains_key(&path) {
        return;
    }
    files.0.insert(path.clone(), None);
    cx.spawn(async move |cx| {
        let read_path = path.clone();
        let commands = cx
            .background_spawn(futures::future::lazy(
                move |_| match std::fs::read_to_string(&read_path) {
                    Ok(text) => marley_terminal::parse_history(&text),
                    Err(error) => {
                        if error.kind() != ErrorKind::NotFound {
                            log::warn!("reading the shell history {read_path}: {error}");
                        }
                        Vec::new()
                    }
                },
            ))
            .await;
        cx.update(|cx| {
            cx.global_mut::<HistoryFiles>()
                .0
                .insert(path, Some(Arc::new(commands)));
            cx.refresh_windows();
        });
    })
    .detach();
}
