//! English at the prompt (#557): a line typed at a shell's prompt that reads as a request in words
//! gets a hint after the cursor, and Ctrl+Shift+Enter hands it to an agent instead of the shell.
//!
//! The reading is `marley_terminal::english`'s local rules, with the words that name commands
//! taken from the search path's programs, the shells' builtins and the terminal's own verified
//! commands; nothing leaves the machine. The line goes to the window's agent terminal (#549's
//! targets), a picker when there are several, or a new Claude Code with the line as its first
//! prompt when there is none. A block that ended with 127 on such a line offers the same through
//! #555's Ask the agent chip.
//!
//! The line is the shell's prompt editor's while it is open (#627), and the hint shows there too.
//! Where #573's model has read a line the rules leave open, its reading takes the hint's place.

use std::collections::HashSet;
use std::ffi::OsString;
use std::ops::Range;
use std::sync::Arc;

use gpui::{App, AppContext as _, Context, Entity, Global, SharedString, Task, Window};
use marley_agent::AgentKind;
use marley_terminal::BlockState;
use marley_terminal::english::{self, BUILTINS, Reading};
use settings::{Settings as _, SystemOneMode};
use terminal::Terminal;
use workspace::Workspace;

use crate::send_selection::{self, OnPick, Pick, Row, TargetPicker};
use crate::typed_line::{self, Kind};
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
        // A blocking walk of every directory on the path, off the main thread.
        let names = cx
            .background_spawn(futures::future::lazy(move |_| {
                programs_on(scanned.as_deref())
            }))
            .await;
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

/// Where `word`, a command, is known from, for #573's facts.
pub(crate) fn command_source(word: &str, cx: &App) -> &'static str {
    if BUILTINS.contains(&word) {
        "a shell builtin"
    } else if cx
        .try_global::<Commands>()
        .and_then(|commands| commands.names.as_ref())
        .is_some_and(|names| names.contains(word))
    {
        "a program on the search path"
    } else {
        "a command this terminal ran"
    }
}

/// What `line` reads as in `terminal`.
fn reading(line: &str, terminal: &Terminal, cx: &App) -> Reading {
    english::read_line(line, |word| is_command(word, terminal, cx))
}

/// Whether `line` reads as English in `terminal` by the rules.
pub(crate) fn reads_as_english(line: &str, terminal: &Terminal, cx: &App) -> bool {
    reading(line, terminal, cx) == Reading::English
}

/// Whether #573's model is asked about `line` in `terminal`.
pub(crate) fn open_case(line: &str, terminal: &Terminal, cx: &App) -> bool {
    english::open_case(line, |word| is_command(word, terminal, cx))
}

/// What shows after a line at a shell's prompt, and the part of the line coloured as a warning.
pub(crate) struct Hint {
    pub(crate) text: SharedString,
    /// The byte range of the words a command would take as its arguments (#573).
    pub(crate) warning: Option<Range<usize>>,
}

/// The hint after `line` at `terminal`'s prompt while the setting is on: #573's reading of the
/// line where there is one, else #557's when the line reads as English.
pub(crate) fn hint_for(line: &str, terminal: &Entity<Terminal>, cx: &App) -> Option<Hint> {
    if MarleySettings::get_global(cx).english_hint == EnglishHint::Hidden || line.trim().is_empty()
    {
        return None;
    }
    let rules = reads_as_english(line, terminal.read(cx), cx).then(|| Hint {
        text: SharedString::from(HINT),
        warning: None,
    });
    let Some((kind, mode)) = typed_line::shown(terminal.entity_id(), line, cx) else {
        return rules;
    };
    // `suggest` asks; `act` says.
    let mark = if mode == SystemOneMode::Suggest {
        "?"
    } else {
        ","
    };
    let first = line.split_whitespace().next().unwrap_or_default();
    let text = match kind {
        Kind::Command if mode == SystemOneMode::Act => return None,
        Kind::Command => return rules,
        Kind::Request => format!("  · a request{mark} ctrl-shift-enter asks the agent"),
        Kind::Comment => format!("  · a comment{}", mark.trim_end_matches(',')),
        Kind::CommandThenEnglish => {
            format!("  · English after {first}{mark} ctrl-shift-enter asks the agent")
        }
    };
    let warning = (kind == Kind::CommandThenEnglish && mode == SystemOneMode::Act)
        .then(|| arguments(line))
        .flatten();
    Some(Hint {
        text: text.into(),
        warning,
    })
}

/// The byte range of `line`'s words after its first, on its first line.
fn arguments(line: &str) -> Option<Range<usize>> {
    let start = line.len() - line.trim_start().len();
    let first_end = start + line[start..].find(char::is_whitespace)?;
    let rest = &line[first_end..];
    let arguments_start = first_end + (rest.len() - rest.trim_start().len());
    let line_end = line.find('\n').unwrap_or(line.len());
    let words = line.get(arguments_start..line_end)?.trim_end();
    (!words.is_empty()).then(|| arguments_start..arguments_start + words.len())
}

/// The hint after the line typed at `terminal`'s grid prompt; the suggestion hook shows it where
/// no history suggestion does.
pub(crate) fn hint(terminal: &Entity<Terminal>, cx: &App) -> Option<SharedString> {
    let typed = crate::autosuggest::typed_text(terminal.read(cx))?;
    hint_for(&typed, terminal, cx).map(|hint| hint.text)
}

/// Whether Ctrl+Shift+Enter hands `line` to an agent: it reads as English, or the reading shown
/// after it offers the agent (#573).
fn offers_agent(line: &str, terminal: &Entity<Terminal>, cx: &App) -> bool {
    reads_as_english(line, terminal.read(cx), cx)
        || typed_line::shown(terminal.entity_id(), line, cx)
            .is_some_and(|(kind, _)| matches!(kind, Kind::Request | Kind::CommandThenEnglish))
}

/// Ctrl+Shift+Enter: the line at the focused terminal's prompt, in the shell's editor while it is
/// open, else typed at the prompt, when it offers the agent, leaves the shell's line and goes to
/// the agent, whatever the hint's setting; false otherwise, so a command's key reaches the shell.
fn ask_typed(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) -> bool {
    let Some(view) = crate::blocks::focused_terminal(workspace, window, cx) else {
        return false;
    };
    let terminal = view.read(cx).terminal().clone();
    let in_editor = crate::rich_input::shell_text(&view, cx);
    let typed = in_editor
        .clone()
        .or_else(|| crate::autosuggest::typed_text(terminal.read(cx)));
    let Some(text) = typed.filter(|text| offers_agent(text, &terminal, cx)) else {
        return false;
    };
    typed_line::asked_agent(&terminal, cx);
    if in_editor.is_some() {
        crate::rich_input::clear_shell(&view, window, cx);
    }
    // Ctrl-U: the line is the agent's now, not the shell's.
    terminal.update(cx, |terminal, _| terminal.input(b"\x15".to_vec()));
    ask(workspace, text.trim().to_string(), window, cx);
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
