//! English at the prompt (#557): a line typed at a shell's prompt that reads as a request in words
//! gets a hint after the cursor, and Ctrl+Shift+Enter hands it to an agent instead of the shell.
//!
//! The reading is `marley_terminal::english`'s local rules, with the words that name commands
//! taken from the search path's programs, the shells' builtins and the terminal's own verified
//! commands; nothing leaves the machine. The line goes to the window's agent terminal (#549's
//! targets), a picker when there are several, or a new Claude Code with the line as its first
//! prompt when there is none. A block that ended with 127 on such a line offers the same through
//! #555's Ask the agent chip.

use std::collections::HashSet;
use std::ffi::OsString;
use std::sync::Arc;

use gpui::{App, Context, Entity, Global, SharedString, Task, Window};
use marley_agent::AgentKind;
use marley_terminal::BlockState;
use marley_terminal::english::{self, BUILTINS, Reading};
use settings::Settings as _;
use terminal::Terminal;
use workspace::Workspace;

use crate::send_selection::{self, OnPick, Pick, Row, TargetPicker};
use crate::{AskAgent, EnglishHint, MarleySettings};

/// What the hint says, after the typed line.
const HINT: &str = "  · ctrl-shift-enter asks the agent";

/// The names of the programs on the search path, read once off the main thread.
#[derive(Default)]
struct Commands {
    path: Option<OsString>,
    names: Option<Arc<HashSet<String>>>,
    reading: Option<Task<()>>,
}

impl Global for Commands {}

/// Reads the search path's commands and installs `marley::AskAgent` on every workspace;
/// [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    cx.set_global(Commands::default());
    read_commands(cx);
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &AskAgent, window, cx| {
            if !ask_typed(workspace, window, cx) {
                // Nothing typed: the key goes on to the program.
                cx.propagate();
            }
        });
    })
    .detach();
}

/// Starts reading the search path's program names, unless they were read for this path, as a
/// test's launcher can change it.
fn read_commands(cx: &mut App) {
    let path = crate::agents::launcher(cx).search_path;
    let commands = cx.global::<Commands>();
    if commands.path == path && (commands.names.is_some() || commands.reading.is_some()) {
        return;
    }
    let scanned = path.clone();
    let reading = cx.spawn(async move |cx| {
        // A blocking walk of every directory on the path, on the blocking pool.
        let names = smol::unblock(move || programs_on(scanned.as_deref())).await;
        cx.update(|cx| {
            let commands = cx.global_mut::<Commands>();
            commands.names = Some(Arc::new(names));
            commands.reading = None;
        });
    });
    let commands = cx.global_mut::<Commands>();
    commands.path = path;
    commands.names = None;
    commands.reading = Some(reading);
}

/// The names of the executable files in each directory of `path`.
fn programs_on(path: Option<&std::ffi::OsStr>) -> HashSet<String> {
    use std::os::unix::fs::PermissionsExt as _;
    let Some(path) = path else {
        return HashSet::new();
    };
    std::env::split_paths(path)
        .filter_map(|directory| std::fs::read_dir(directory).ok())
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter(|entry| {
            entry.metadata().is_ok_and(|metadata| {
                metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
            })
        })
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect()
}

/// Whether `word` names a command in `terminal`: a builtin, a program on the search path, or the
/// first word of one of the terminal's own verified commands, which covers its aliases. Before the
/// search path is read every word does, so no hint shows too early.
fn is_command(word: &str, terminal: &Terminal, cx: &App) -> bool {
    if BUILTINS.contains(&word) {
        return true;
    }
    let Some(names) = cx
        .try_global::<Commands>()
        .and_then(|commands| commands.names.as_ref())
    else {
        return true;
    };
    names.contains(word)
        || terminal.blocks().iter().any(|block| {
            block.command_verified && block.command.split_whitespace().next() == Some(word)
        })
}

/// What `line` reads as in `terminal`.
fn reading(line: &str, terminal: &Terminal, cx: &App) -> Reading {
    english::read_line(line, |word| is_command(word, terminal, cx))
}

/// The hint after a line typed at `terminal`'s prompt that reads as English, while the setting
/// is on; the suggestion hook shows it where no history suggestion does.
pub(crate) fn hint(terminal: &Entity<Terminal>, cx: &App) -> Option<SharedString> {
    if MarleySettings::get_global(cx).english_hint == EnglishHint::Hidden {
        return None;
    }
    let terminal = terminal.read(cx);
    let typed = crate::autosuggest::typed_text(terminal)?;
    (reading(&typed, terminal, cx) == Reading::English).then(|| SharedString::from(HINT))
}

/// Ctrl+Shift+Enter: the line typed at the focused terminal's prompt, when it reads as English,
/// leaves the shell's line and goes to the agent, whatever the hint's setting; false otherwise,
/// so a command's key reaches the shell.
fn ask_typed(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) -> bool {
    let Some(view) = crate::blocks::focused_terminal(workspace, window, cx) else {
        return false;
    };
    let terminal = view.read(cx).terminal().clone();
    let Some(text) = crate::autosuggest::typed_text(terminal.read(cx))
        .map(|text| text.trim().to_string())
        .filter(|text| reading(text, terminal.read(cx), cx) == Reading::English)
    else {
        return false;
    };
    // Ctrl-U: the line is the agent's now, not the shell's.
    terminal.update(cx, |terminal, _| terminal.input(b"\x15".to_vec()));
    ask(workspace, text, window, cx);
    true
}

/// Hands `text` to the window's agent: the one there is, as a paste and a return; the one picked
/// when there are several; or a new Claude Code in the project with `text` as its first prompt.
pub(crate) fn ask(
    workspace: &mut Workspace,
    text: String,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let mut targets = send_selection::agent_targets(workspace, cx);
    let window_handle = window.window_handle();
    match targets.len() {
        0 => {
            let started = crate::agents::start_cli_with_prompt(
                workspace,
                AgentKind::Claude,
                &text,
                None,
                window,
                cx,
            );
            cx.spawn(async move |_, _| {
                if started.await.is_none() {
                    log::warn!("english: Claude Code did not start for the request");
                }
            })
            .detach();
        }
        1 => send_selection::send_text(targets.remove(0), text, true, window_handle, cx),
        _ => {
            let rows = targets
                .into_iter()
                .map(|target| Row {
                    label: target.label(),
                    pick: Pick::Agent(target),
                })
                .collect();
            let on_pick: OnPick = Box::new(move |pick, window, cx| {
                if let Pick::Agent(target) = pick {
                    send_selection::send_text(target, text.clone(), true, window, cx);
                }
            });
            workspace.toggle_modal(window, cx, |window, cx| {
                TargetPicker::new(rows, "Ask which agent…", on_pick, window_handle, window, cx)
            });
        }
    }
}

/// The command #555's chip asks the agent with, for the newest block of `terminal` when it
/// ended with 127, its command is verified (PR-claude-474) and reads as a request, and the
/// setting is on: then the chip shows even with no agent running.
pub(crate) fn asks_on_127(terminal: &Terminal, index: usize, cx: &App) -> Option<String> {
    if MarleySettings::get_global(cx).english_hint == EnglishHint::Hidden {
        return None;
    }
    let anchored = terminal.marley_anchored();
    let block = anchored
        .blocks()
        .last()
        .filter(|block| block.index == index)?;
    let not_found = block.state == BlockState::Finished
        && block.exit_code.0 == Some(127)
        && block.command_verified
        && anchored.at_prompt();
    let command = block.command.trim();
    (not_found && !command.is_empty() && reading(command, terminal, cx) == Reading::English)
        .then(|| command.to_string())
}

/// Asks the agent with `command` from the terminal view's workspace, after the click's own update.
pub(crate) fn ask_later(
    view: &Entity<terminal_view::TerminalView>,
    command: String,
    window: &Window,
    cx: &mut App,
) {
    let Some(workspace) = view.read(cx).marley_workspace().upgrade() else {
        return;
    };
    window.defer(cx, move |window, cx| {
        workspace.update(cx, |workspace, cx| ask(workspace, command, window, cx));
    });
}
