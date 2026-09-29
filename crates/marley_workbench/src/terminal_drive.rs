//! An agent reads and types into the program running in a terminal (#525).
//!
//! `terminal_screen` reads a terminal's live screen: its rows, the cursor, the program in the
//! foreground and who controls the terminal. `terminal_type` types text and keys into that
//! program, never at the shell's prompt and never into another agent CLI, behind the
//! `terminal.write` grant and Marley's approval, as `marley.agent_terminal_writes` asks: the first
//! write to each program, every write, or none. A pending write shows as a card under the terminal
//! and a toast, as #571's paused click does; once an agent has typed, a bar there shows what it
//! typed, and Take Over (Ctrl-I) stops its writes until Hand Back. Each terminal's generation
//! advances when its foreground program changes and at each take-over and hand-back, so a write
//! meant for what the agent last read never lands in something else.
//!
//! `terminal_run` (#556) types a command at the shell's prompt instead, as Rerun does, and answers
//! with the block it ran: only at a prompt the terminal's own shell signed, with nothing typed and
//! no program in the foreground. `marley.agent_command_allowlist` runs a command at once,
//! `marley.agent_command_denylist` asks with the same card and toast, and
//! `marley.agent_commands_outside_lists` decides the rest; Enter and Escape answer the card from
//! its terminal. Each block an agent ran carries a mark, and the same take-over stops both tools.

use std::pin::pin;
use std::time::{Duration, Instant};

use collections::{BTreeMap, HashMap};
use futures::channel::oneshot;
use futures::future::{self, Either};
use gpui::{AnyElement, App, Context, Entity, EntityId, Global, Keystroke, SharedString, Task};
use marley_mcp::{AppCall, ToolAnswer};
use marley_terminal::agent_commands::{self, Verdict};
use marley_terminal::{BlockState, PromptShell};
use serde_json::{Value, json};
use settings::{MarleyAgentCommandsOutsideLists, MarleyAgentTerminalWrites, Settings as _};
use terminal::Terminal;
use terminal_view::{MarleyFooterContext, TerminalView};
use ui::{Button, ButtonStyle, Label, LabelSize, Tooltip, prelude::*};
use util::ResultExt as _;
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};

use crate::agent_bar::agent_in;
use crate::click_pause::Who;
use crate::{MarleySettings, RefuseAgentCommand, RunAgentCommand, TakeOverTerminal};

/// How long a write waits for the user's answer: under the transport's 30 seconds, so the agent
/// reads the refusal rather than a timeout.
const WAIT: Duration = Duration::from_secs(25);

/// How long keys and Enter wait after a paste. A program that reads a bracketed paste up to its
/// end marker takes whatever came with it as part of the paste, as Python's REPL does.
pub(crate) const AFTER_PASTE: Duration = Duration::from_millis(200);

/// The most a write types, text, keys and Enter together, the harness's bound.
const MOST_BYTES: usize = 4096;

/// What the card and the bar show of a write, at most.
const SHOWN_CHARACTERS: usize = 120;

/// How long a run waits for its block after typing it, at most, and when the call gives no
/// `wait_seconds`: with the approval, under [`WAIT`].
const RUN_WAIT_SECONDS: u64 = 20;

/// How often a run looks for its block and the block's end.
const RUN_POLL: Duration = Duration::from_millis(50);

/// The control state of each terminal an agent has read or typed into, by its view.
#[derive(Default)]
struct Drives(HashMap<EntityId, Drive>);

impl Global for Drives {}

/// One terminal's control state.
#[derive(Default)]
struct Drive {
    generation: u64,
    /// The foreground process group's leader the generation belongs to.
    program: Option<u32>,
    /// Whether the user allowed writes to this program.
    approved: bool,
    taken_over: bool,
    /// Who typed last, into which program, and what.
    last_write: Option<Written>,
    pending: Option<Pending>,
    /// The blocks an agent's `terminal_run` typed, by index (#556).
    runs: BTreeMap<usize, Ran>,
    /// Whether a run has typed its command and not yet answered.
    run_in_flight: bool,
}

/// A command an agent ran at the terminal's prompt, and who ran it.
#[derive(Clone)]
struct Ran {
    who: String,
    command: String,
}

struct Written {
    who: String,
    program: String,
    text: String,
}

/// A write, or a run, that waits for the user.
struct Pending {
    /// What the card says.
    words: String,
    /// The card's two buttons.
    allow: &'static str,
    deny: &'static str,
    /// Whether it is a run, which Enter and Escape answer.
    run: bool,
    answer: Option<oneshot::Sender<bool>>,
}

/// Registers the take-over on every workspace; [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    cx.set_global(Drives::default());
    // A closed terminal's state goes with it.
    cx.observe_new(|_: &mut TerminalView, _, cx: &mut Context<TerminalView>| {
        let view = cx.entity_id();
        cx.on_release(move |_, cx| {
            cx.default_global::<Drives>().0.remove(&view);
        })
        .detach();
    })
    .detach();
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &TakeOverTerminal, window, cx| {
            let view = crate::blocks::focused_terminal(workspace, window, cx);
            let toggled = view.is_some_and(|view| toggle_control(&view, cx));
            if !toggled {
                // The key goes on to the terminal, which sends it to the program.
                cx.propagate();
            }
        });
        // Enter and Escape answer a run waiting under the focused terminal, and otherwise reach
        // its program (#556).
        workspace.register_action(|workspace, _: &RunAgentCommand, window, cx| {
            if !answer_focused_run(workspace, true, window, cx) {
                cx.propagate();
            }
        });
        workspace.register_action(|workspace, _: &RefuseAgentCommand, window, cx| {
            if !answer_focused_run(workspace, false, window, cx) {
                cx.propagate();
            }
        });
    })
    .detach();
}

/// The terminal's foreground process group's leader, none while its shell waits at its prompt.
fn foreground_program(view: &Entity<TerminalView>, cx: &App) -> Option<u32> {
    program_of(view.read(cx).terminal().read(cx))
}

/// [`foreground_program`] from the terminal itself, for the footer, which renders while its view
/// is being updated and so cannot read it (#595).
fn program_of(terminal: &Terminal) -> Option<u32> {
    let shell = terminal.pid_getter()?.fallback_pid();
    let program = terminal.pid().filter(|pid| *pid != shell)?;
    Some(program.as_u32())
}

/// `view`'s drive with its generation brought up to date: a new foreground program starts a new
/// one, unapproved.
fn drive<'a>(view: &Entity<TerminalView>, cx: &'a mut App) -> &'a mut Drive {
    let program = foreground_program(view, cx);
    let drive = cx
        .default_global::<Drives>()
        .0
        .entry(view.entity_id())
        .or_default();
    if drive.program != program {
        drive.program = program;
        drive.generation += 1;
        drive.approved = false;
        drive.last_write = None;
    }
    drive
}

/// `terminal_screen`: the terminal's rows as the screen shows them, through the agents'
/// redaction, the cursor, the program in the foreground and who controls the terminal.
pub(crate) fn screen(call: &AppCall, cx: &mut App) -> Result<ToolAnswer, String> {
    let (id, view) = crate::mcp::terminal_of(&call.arguments, call.caller(), cx)?;
    let generation;
    let taken_over;
    let approved;
    {
        let drive = drive(&view, cx);
        generation = drive.generation;
        taken_over = drive.taken_over;
        approved = drive.approved;
    }
    let running = foreground_program(&view, cx).is_some();
    let terminal = view.read(cx).terminal().read(cx);
    let content = terminal.last_content();
    let mut rows: BTreeMap<i32, String> = BTreeMap::new();
    for cell in &content.cells {
        let row = rows.entry(cell.point.line).or_default();
        // The cell after a wide character only holds its place.
        if !cell.is_wide_char_spacer() {
            row.push(cell.character());
        }
    }
    let text = rows
        .into_values()
        .map(|row| row.trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n");
    let redacted = crate::mcp::for_agents(&text, crate::mcp::agent_redactor(cx).as_deref());
    let program = running
        .then(|| terminal.foreground_process_command_name())
        .flatten();
    Ok(answer_of(json!({
        "terminal": id,
        "rows": redacted.text.split('\n').collect::<Vec<_>>(),
        "cursor": {
            "row": content.cursor.point.line + i32::try_from(content.display_offset).unwrap_or(0),
            "column": content.cursor.point.column,
        },
        "columns": content.columns,
        "lines": content.screen_lines,
        "alternate_screen": content.mode.contains(terminal::Modes::ALT_SCREEN),
        "scrolled": content.display_offset > 0,
        "program": program,
        "generation": generation,
        "taken_over": taken_over,
        "approval": approval_words(MarleySettings::get_global(cx).agent_terminal_writes),
        "approved": approved,
        "redacted": redacted.count,
    })))
}

const fn approval_words(mode: MarleyAgentTerminalWrites) -> &'static str {
    match mode {
        MarleyAgentTerminalWrites::AskFirstWrite => "ask_first_write",
        MarleyAgentTerminalWrites::AskEveryWrite => "ask_every_write",
        MarleyAgentTerminalWrites::NeverAsk => "never_ask",
    }
}

/// A write, checked: where it goes and what it types.
struct Typing {
    view: Entity<TerminalView>,
    workspace: Option<Entity<Workspace>>,
    generation: u64,
    program: String,
    who: String,
    text: Option<String>,
    keys: Vec<Keystroke>,
    submit: bool,
    bytes: usize,
    ask: bool,
}

/// `terminal_type`: checks the write, asks the user when the setting says to, and types it; the
/// call is answered once the write is typed or refused.
pub(crate) fn type_into(call: AppCall, cx: &mut App) {
    let typing = match check(&call, cx) {
        Ok(typing) => typing,
        Err(refusal) => {
            call.answer(Err(refusal));
            return;
        }
    };
    cx.spawn(async move |cx| {
        let result = if typing.ask {
            ask_then_type(typing, cx).await
        } else {
            Ok(write(&typing, cx).await)
        };
        call.answer(result);
    })
    .detach();
}

/// Pastes `text` into `terminal`, bracketed when its program asked for it, and runs `after` on the
/// terminal once [`AFTER_PASTE`] has passed, so the keys and Enter it sends reach the program as
/// keys (#594).
pub(crate) fn paste_then(
    terminal: &Entity<Terminal>,
    text: &str,
    after: impl FnOnce(&mut Terminal) + 'static,
    cx: &mut App,
) -> Task<()> {
    terminal.update(cx, |terminal, _| terminal.paste(text));
    let terminal = terminal.downgrade();
    cx.spawn(async move |cx| {
        cx.background_executor().timer(AFTER_PASTE).await;
        terminal.update(cx, |terminal, _| after(terminal)).log_err();
    })
}

/// The refusals that need no one's answer, and what the write would type.
fn check(call: &AppCall, cx: &mut App) -> Result<Typing, String> {
    let arguments = &call.arguments;
    let (_, view) = crate::mcp::terminal_of(arguments, call.caller(), cx)?;
    let generation = arguments
        .get("generation")
        .and_then(Value::as_u64)
        .ok_or("give `generation`, from terminal_screen")?;
    let text = arguments
        .get("text")
        .and_then(Value::as_str)
        .map(str::to_string);
    let names: Vec<String> = arguments
        .get("keys")
        .and_then(Value::as_array)
        .map(|keys| {
            keys.iter()
                .filter_map(|key| key.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let submit = arguments
        .get("submit")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let keys = names
        .iter()
        .map(|name| key(name))
        .collect::<Result<Vec<_>, _>>()?;
    let bytes = text.as_ref().map_or(0, String::len) + keys.len() + usize::from(submit);
    if bytes == 0 {
        return Err("give `text`, `keys` or `submit`".to_string());
    }
    if bytes > MOST_BYTES {
        return Err(format!("a write types at most {MOST_BYTES} bytes"));
    }
    if foreground_program(&view, cx).is_none() {
        return Err(
            "no program runs in the terminal's foreground: terminal_type types into a running \
             program, never at the shell's prompt"
                .to_string(),
        );
    }
    if agent_in(view.read(cx).terminal().read(cx)).is_some() {
        return Err(
            "the terminal runs an agent CLI, which terminal_type does not type into".to_string(),
        );
    }
    let program = view
        .read(cx)
        .terminal()
        .read(cx)
        .foreground_process_command_name()
        .unwrap_or_else(|| "the program".to_string());
    let who = Who::of(call, cx).words;
    let mode = MarleySettings::get_global(cx).agent_terminal_writes;
    let drive = drive(&view, cx);
    if drive.generation != generation {
        return Err(format!(
            "the terminal is at generation {}, not {generation}: its program changed or the user \
             took over or handed back; read terminal_screen again",
            drive.generation
        ));
    }
    if drive.taken_over {
        return Err(
            "the user has taken over this terminal; wait until they hand it back".to_string(),
        );
    }
    if drive.pending.is_some() {
        return Err("another write to this terminal waits for the user's answer".to_string());
    }
    let ask = match mode {
        MarleyAgentTerminalWrites::NeverAsk => false,
        MarleyAgentTerminalWrites::AskEveryWrite => true,
        MarleyAgentTerminalWrites::AskFirstWrite => !drive.approved,
    };
    let workspace = crate::mcp::terminals(cx)
        .into_iter()
        .find(|(_, other)| *other == view)
        .map(|(workspace, _)| workspace);
    Ok(Typing {
        view,
        workspace,
        generation,
        program,
        who,
        text,
        keys,
        submit,
        bytes,
        ask,
    })
}

/// A key by Zed's name, such as `escape`, `ctrl-c` or `up`. A lone character with no modifier is
/// text, which goes in `text`.
fn key(name: &str) -> Result<Keystroke, String> {
    let keystroke = Keystroke::parse(name).map_err(|error| format!("key {name:?}: {error}"))?;
    let modified = keystroke.modifiers.control || keystroke.modifiers.alt;
    if keystroke.key.chars().count() == 1 && !modified {
        return Err(format!("key {name:?} is a character: put it in `text`"));
    }
    Ok(keystroke)
}

/// What the card and the bar show of a write.
fn shown(typing: &Typing) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(text) = &typing.text {
        let clean: String = text
            .chars()
            .map(|character| {
                if character.is_control() {
                    ' '
                } else {
                    character
                }
            })
            .take(SHOWN_CHARACTERS)
            .collect();
        parts.push(format!("{clean:?}"));
    }
    parts.extend(typing.keys.iter().map(|key| format!("<{}>", key.unparse())));
    if typing.submit {
        parts.push("<enter>".to_string());
    }
    parts.join(" ")
}

/// Holds the write until the user answers under the terminal, or 25 seconds pass; typed only on
/// Allow and only while the terminal's generation is the one the write was meant for.
async fn ask_then_type(typing: Typing, cx: &gpui::AsyncApp) -> Result<ToolAnswer, String> {
    let (sender, receiver) = oneshot::channel();
    let text = shown(&typing);
    let id = typing.view.entity_id();
    cx.update(|cx| {
        drive(&typing.view, cx).pending = Some(Pending {
            words: format!(
                "{} wants to type into {}: {text}",
                typing.who, typing.program
            ),
            allow: "Allow",
            deny: "Deny",
            run: false,
            answer: Some(sender),
        });
        typing.view.update(cx, |_, cx| cx.notify());
        let message = format!(
            "{} wants to type into {}. Allow it or deny it under the terminal.",
            typing.who, typing.program
        );
        show_toast(typing.workspace.as_ref(), &typing.view, message, cx);
    });
    let timer = pin!(cx.background_executor().timer(WAIT));
    let answer = match future::select(receiver, timer).await {
        Either::Left((Ok(allowed), _)) => Some(allowed),
        Either::Left((Err(_), _)) | Either::Right(_) => None,
    };
    cx.update(|cx| {
        if let Some(drive) = cx.default_global::<Drives>().0.get_mut(&id) {
            drive.pending = None;
        }
        typing.view.update(cx, |_, cx| cx.notify());
        dismiss_toast(typing.workspace.as_ref(), &typing.view, cx);
        match answer {
            Some(true) => {
                let drive = drive(&typing.view, cx);
                if drive.generation != typing.generation || drive.taken_over {
                    return Err(
                        "the terminal's program changed, or the user took over, while Marley \
                         asked; nothing was typed"
                            .to_string(),
                    );
                }
                drive.approved = true;
                Ok(())
            }
            Some(false) => Err(format!(
                "the user refused this write to {}; ask them before you try again",
                typing.program
            )),
            None => Err(format!(
                "the user did not answer within {} seconds; nothing was typed",
                WAIT.as_secs()
            )),
        }
    })?;
    Ok(write(&typing, cx).await)
}

/// Types the write: the text as a paste, bracketed when the program asked for it, then each key
/// and Enter once the paste has landed; answered after the last of them.
async fn write(typing: &Typing, cx: &gpui::AsyncApp) -> ToolAnswer {
    let terminal = cx.update(|cx| typing.view.read(cx).terminal().clone());
    let keys = typing.keys.clone();
    let submit = typing.submit;
    let after = move |terminal: &mut Terminal| {
        for key in &keys {
            terminal.try_keystroke(key, false);
        }
        if submit {
            terminal.input(b"\r".to_vec());
        }
    };
    match &typing.text {
        Some(text) => cx.update(|cx| paste_then(&terminal, text, after, cx)).await,
        None => cx.update(|cx| terminal.update(cx, |terminal, _| after(terminal))),
    }
    cx.update(|cx| {
        let text = shown(typing);
        drive(&typing.view, cx).last_write = Some(Written {
            who: typing.who.clone(),
            program: typing.program.clone(),
            text,
        });
        typing.view.update(cx, |_, cx| cx.notify());
        answer_of(json!({
            "written": typing.bytes,
            "program": typing.program,
            "generation": typing.generation,
        }))
    })
}

/// An answer whose text is its JSON.
const fn answer_of(structured: Value) -> ToolAnswer {
    ToolAnswer {
        structured,
        text: None,
        image: None,
    }
}

/// The toast in the terminal's workspace for a pending write or run, with Show.
fn show_toast(
    workspace: Option<&Entity<Workspace>>,
    view: &Entity<TerminalView>,
    message: String,
    cx: &mut App,
) {
    let Some(workspace) = workspace else {
        return;
    };
    let shown = view.clone();
    let toast = Toast::new(toast_id(view), message).on_click("Show", move |window, cx| {
        crate::browser::reveal_terminal(&shown, window, cx);
    });
    workspace.update(cx, |workspace, cx| workspace.show_toast(toast, cx));
}

fn dismiss_toast(workspace: Option<&Entity<Workspace>>, view: &Entity<TerminalView>, cx: &mut App) {
    if let Some(workspace) = workspace {
        let id = toast_id(view);
        workspace.update(cx, |workspace, cx| workspace.dismiss_toast(&id, cx));
    }
}

fn toast_id(view: &Entity<TerminalView>) -> NotificationId {
    NotificationId::composite::<Drives>(SharedString::from(format!(
        "marley-terminal-write-{}",
        view.entity_id()
    )))
}

/// The user's answer to `view`'s pending write.
fn answer(view: EntityId, allowed: bool, cx: &mut App) {
    let sender = cx
        .default_global::<Drives>()
        .0
        .get_mut(&view)
        .and_then(|drive| drive.pending.as_mut())
        .and_then(|pending| pending.answer.take());
    if let Some(sender) = sender {
        sender.send(allowed).ok();
    }
}

/// Takes over `view` from the agent that typed into its program or ran a command at its prompt, or
/// hands it back; false when no agent has done either, so the key goes on to the program.
fn toggle_control(view: &Entity<TerminalView>, cx: &mut App) -> bool {
    let drive = drive(view, cx);
    if drive.last_write.is_none() && drive.runs.is_empty() && !drive.taken_over {
        return false;
    }
    drive.taken_over = !drive.taken_over;
    drive.generation += 1;
    view.update(cx, |_, cx| cx.notify());
    true
}

/// The card of a pending write, or the bar of an agent's writes, under the terminal; none while
/// no agent has typed into its program.
pub(crate) fn footer(context: &MarleyFooterContext, cx: &App) -> Option<AnyElement> {
    let view = context.view.upgrade()?;
    let id = view.entity_id();
    let drives = cx.try_global::<Drives>()?;
    let drive = drives.0.get(&id)?;
    let colors = cx.theme().colors();
    let bar = h_flex()
        .w_full()
        .gap_2()
        .px_2()
        .py_1()
        .border_t_1()
        .border_color(colors.border_variant)
        .bg(colors.terminal_background);
    if let Some(pending) = &drive.pending {
        // The buttons lead the row: the write's toast stacks at the workspace's bottom right,
        // over the right end of a terminal's footer (#593).
        return Some(
            bar.debug_selector(|| "marley-terminal-write-card".into())
                .child(
                    Button::new(("marley-terminal-write-allow", id.as_u64()), pending.allow)
                        .style(ButtonStyle::Filled)
                        .on_click(move |_, _, cx| {
                            cx.stop_propagation();
                            answer(id, true, cx);
                        }),
                )
                .child(
                    Button::new(("marley-terminal-write-deny", id.as_u64()), pending.deny)
                        .style(ButtonStyle::Subtle)
                        .on_click(move |_, _, cx| {
                            cx.stop_propagation();
                            answer(id, false, cx);
                        }),
                )
                .child(
                    Label::new(pending.words.clone())
                        .size(LabelSize::Small)
                        .color(Color::Warning)
                        .truncate(),
                )
                .into_any_element(),
        );
    }
    let (words, action) = if drive.taken_over {
        ("You have control".to_string(), "Hand Back")
    } else if let Some(ran) = running_run(drive, context.terminal.read(cx)) {
        (format!("{} ran {}", ran.who, ran.command), "Take Over")
    } else {
        // The drive catches up with a new program only when something next reads it, so the
        // bar checks that the program it names still runs (#595).
        let written = drive
            .last_write
            .as_ref()
            .filter(|_| drive.program == program_of(context.terminal.read(cx)))?;
        (
            format!(
                "{} typed into {}: {}",
                written.who, written.program, written.text
            ),
            "Take Over",
        )
    };
    Some(
        bar.debug_selector(|| "marley-terminal-drive-bar".into())
            .child(
                Label::new(words)
                    .size(LabelSize::Small)
                    .color(Color::Muted)
                    .truncate(),
            )
            .child(div().flex_1())
            .child(
                Button::new(("marley-terminal-take-over", id.as_u64()), action)
                    .style(ButtonStyle::Subtle)
                    .on_click(move |_, _, cx| {
                        cx.stop_propagation();
                        toggle_control(&view, cx);
                    }),
            )
            .into_any_element(),
    )
}

/// A run, checked: where it goes, what it types, who asks and whether the user is asked first.
struct Run {
    view: Entity<TerminalView>,
    workspace: Option<Entity<Workspace>>,
    id: u64,
    command: String,
    who: String,
    wait: Duration,
    ask: bool,
}

/// `terminal_run` (#556): checks the command and the terminal's prompt, asks the user when the
/// lists and the setting say to, types the command as Rerun does, and answers with its block once
/// it ends or the wait is over.
pub(crate) fn run_at_prompt(call: AppCall, cx: &mut App) {
    let called = cx.background_executor().now();
    let run = match check_run(&call, cx) {
        Ok(run) => run,
        Err(refusal) => {
            call.answer(Err(refusal));
            return;
        }
    };
    cx.spawn(async move |cx| {
        let result = run_then_wait(run, called, cx).await;
        call.answer(result);
    })
    .detach();
}

/// The refusals that need no one's answer, and whether the user is asked.
fn check_run(call: &AppCall, cx: &mut App) -> Result<Run, String> {
    let arguments = &call.arguments;
    let (id, view) = crate::mcp::terminal_of(arguments, call.caller(), cx)?;
    let command = arguments
        .get("command")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default()
        .to_string();
    if command.is_empty() {
        return Err("give `command`, the line to run".to_string());
    }
    if command.len() > MOST_BYTES {
        return Err(format!("a command is at most {MOST_BYTES} bytes"));
    }
    if command.contains(['\n', '\r']) {
        return Err(
            "`command` is one line: join commands with `&&` or `;`, or run them one by one"
                .to_string(),
        );
    }
    let wait = arguments
        .get("wait_seconds")
        .and_then(Value::as_u64)
        .map_or(RUN_WAIT_SECONDS, |seconds| seconds.min(RUN_WAIT_SECONDS));
    prompt_refusal(&view, cx)?;
    let who = Who::of(call, cx).words;
    let settings = MarleySettings::get_global(cx);
    let ask = match agent_commands::verdict(
        &command,
        &settings.agent_command_allowlist,
        &settings.agent_command_denylist,
    ) {
        Verdict::Allowed => false,
        Verdict::Ask => true,
        Verdict::Outside => {
            settings.agent_commands_outside_lists == MarleyAgentCommandsOutsideLists::Ask
        }
    };
    let drive = drive(&view, cx);
    if drive.taken_over {
        return Err(
            "the user has taken over this terminal; wait until they hand it back".to_string(),
        );
    }
    if drive.pending.is_some() {
        return Err(
            "another command or write waits for the user's answer in this terminal".to_string(),
        );
    }
    if drive.run_in_flight {
        return Err("another command of an agent's is starting in this terminal".to_string());
    }
    let workspace = crate::mcp::terminals(cx)
        .into_iter()
        .find(|(_, other)| *other == view)
        .map(|(workspace, _)| workspace);
    Ok(Run {
        view,
        workspace,
        id,
        command,
        who,
        wait: Duration::from_secs(wait),
        ask,
    })
}

/// Why `view`'s shell cannot take a command now: a program or an agent CLI in the foreground, no
/// prompt the terminal's own shell signed (PR-claude-474), or something typed at it.
fn prompt_refusal(view: &Entity<TerminalView>, cx: &App) -> Result<(), String> {
    let terminal = view.read(cx).terminal().read(cx);
    if agent_in(terminal).is_some() {
        return Err(
            "an agent CLI runs in the terminal; terminal_run types only at a shell's prompt"
                .to_string(),
        );
    }
    if program_of(terminal).is_some() {
        let program = terminal
            .foreground_process_command_name()
            .unwrap_or_else(|| "a program".to_string());
        return Err(format!(
            "{program} runs in the terminal's foreground; terminal_run types only at the shell's \
             prompt: wait for it to end, or type into it with terminal_type"
        ));
    }
    let anchored = terminal.marley_anchored();
    if !anchored.at_prompt() {
        return Err("the terminal's shell is not at its prompt".to_string());
    }
    if anchored.prompt_shell() != Some(PromptShell::Local) {
        return Err(
            "the prompt was not signed by the terminal's own shell (its integration is off, or \
             another host's shell answers); nothing was typed"
                .to_string(),
        );
    }
    // Nothing noted since the prompt is nothing typed; after a key, the line tells.
    let typed = if anchored.input_start().is_none() {
        Some(String::new())
    } else {
        crate::autosuggest::typed_text(terminal)
    };
    match typed {
        Some(text) if text.trim().is_empty() => Ok(()),
        Some(_) => Err(
            "the user has typed at the prompt; nothing was typed, so as not to lose it".to_string(),
        ),
        None => Err(
            "the prompt's line cannot be read (the view is scrolled back, in vi mode, or the \
             cursor left the line); nothing was typed"
                .to_string(),
        ),
    }
}

/// Asks when the run must, types it, and waits for its block: the whole call within [`WAIT`] of
/// `called`, and the block at most the run's wait after the typing.
async fn run_then_wait(
    run: Run,
    called: Instant,
    cx: &gpui::AsyncApp,
) -> Result<ToolAnswer, String> {
    let deadline = called + WAIT;
    if run.ask {
        ask_to_run(&run, deadline, cx).await?;
    }
    let (terminal, before) = cx.update(|cx| type_run(&run, cx))?;
    let typed = cx.background_executor().now();
    let until = deadline.min(typed + run.wait);
    let mut block = None;
    loop {
        let (found, finished) = cx.update(|cx| {
            let blocks = terminal.read(cx).blocks();
            let found = block.or_else(|| {
                blocks
                    .iter()
                    .skip(before)
                    .find(|candidate| {
                        candidate.command_verified && candidate.command.trim() == run.command
                    })
                    .map(|candidate| candidate.index)
            });
            let finished = found
                .and_then(|index| blocks.get(index))
                .is_some_and(|found| found.state == BlockState::Finished);
            (found, finished)
        });
        if block.is_none()
            && let Some(index) = found
        {
            // The block the typing opened is the agent's: its mark, and what `terminal_blocks`
            // says of it (D8).
            cx.update(|cx| {
                drive(&run.view, cx).runs.insert(
                    index,
                    Ran {
                        who: run.who.clone(),
                        command: run.command.clone(),
                    },
                );
                run.view.update(cx, |_, cx| cx.notify());
            });
        }
        block = found;
        if finished || cx.background_executor().now() >= until {
            break;
        }
        cx.background_executor().timer(RUN_POLL).await;
    }
    cx.update(|cx| {
        drive(&run.view, cx).run_in_flight = false;
        run.view.update(cx, |_, cx| cx.notify());
        match block {
            Some(index) => Ok(run_answer(&run, terminal.read(cx), index, cx)),
            None => Err(format!(
                "`{}` was typed at the prompt, but no block for it started within {} seconds; \
                 read terminal_blocks",
                run.command,
                run.wait.as_secs()
            )),
        }
    })
}

/// Types the run at the prompt, checked once more since the user may have answered late: the
/// terminal, and how many blocks it had before.
fn type_run(run: &Run, cx: &mut App) -> Result<(Entity<Terminal>, usize), String> {
    prompt_refusal(&run.view, cx)?;
    let drive = drive(&run.view, cx);
    if drive.taken_over {
        return Err(
            "the user took over this terminal while Marley asked; nothing was typed".to_string(),
        );
    }
    drive.run_in_flight = true;
    let terminal = run.view.read(cx).terminal().clone();
    let before = terminal.read(cx).blocks().len();
    // Ctrl-U, the command and a return, as Rerun sends a block's command.
    let line = format!("\u{15}{}\r", run.command);
    terminal.update(cx, |terminal, _| terminal.input(line.into_bytes()));
    Ok((terminal, before))
}

/// Holds the run until the user answers under the terminal, or `deadline`.
async fn ask_to_run(run: &Run, deadline: Instant, cx: &gpui::AsyncApp) -> Result<(), String> {
    let (sender, receiver) = oneshot::channel();
    let id = run.view.entity_id();
    let shown: String = run.command.chars().take(SHOWN_CHARACTERS).collect();
    cx.update(|cx| {
        drive(&run.view, cx).pending = Some(Pending {
            words: format!("{} wants to run {shown}", run.who),
            allow: "Run",
            deny: "Refuse",
            run: true,
            answer: Some(sender),
        });
        run.view.update(cx, |_, cx| cx.notify());
        let message = format!(
            "{} wants to run {shown} in a terminal. Run it or refuse it under the terminal.",
            run.who
        );
        show_toast(run.workspace.as_ref(), &run.view, message, cx);
    });
    let wait = deadline.saturating_duration_since(cx.background_executor().now());
    let timer = pin!(cx.background_executor().timer(wait));
    let answer = match future::select(receiver, timer).await {
        Either::Left((Ok(allowed), _)) => Some(allowed),
        Either::Left((Err(_), _)) | Either::Right(_) => None,
    };
    cx.update(|cx| {
        if let Some(drive) = cx.default_global::<Drives>().0.get_mut(&id) {
            drive.pending = None;
        }
        run.view.update(cx, |_, cx| cx.notify());
        dismiss_toast(run.workspace.as_ref(), &run.view, cx);
    });
    match answer {
        Some(true) => Ok(()),
        Some(false) => Err(format!(
            "the user refused to run `{}`; ask them before you try again",
            run.command
        )),
        None => Err(format!(
            "the user did not answer within {} seconds; nothing was typed",
            WAIT.as_secs()
        )),
    }
}

/// The run's block as `terminal_read` reads it, through the agents' redaction, with its exit code
/// and how long it took once it ended.
fn run_answer(run: &Run, terminal: &Terminal, index: usize, cx: &App) -> ToolAnswer {
    let Some(block) = terminal.blocks().get(index) else {
        return answer_of(json!({ "terminal": run.id, "block": index }));
    };
    let redactor = crate::mcp::agent_redactor(cx);
    let command = crate::mcp::for_agents(&block.command, redactor.as_deref());
    let output = terminal.block_output(block).unwrap_or_default();
    // Redacted whole before the tail is cut, as terminal_read does.
    let output = crate::mcp::for_agents(&output, redactor.as_deref());
    let (text, truncated) = crate::mcp::tail(&output.text);
    let finished = block.state == BlockState::Finished;
    let duration_ms = terminal
        .marley_anchored()
        .times(index)
        .and_then(|times| times.finished?.duration_since(times.started).ok())
        .and_then(|took| u64::try_from(took.as_millis()).ok());
    ToolAnswer {
        structured: json!({
            "terminal": run.id,
            "block": index,
            "command": command.text,
            "running": !finished,
            "exit_code": finished.then_some(block.exit_code.0).flatten(),
            "duration_ms": duration_ms,
            "output": text,
            "truncated": truncated,
            "redacted": command.count + output.count,
        }),
        text: Some(text),
        image: None,
    }
}

/// Answers the run waiting under the focused terminal; false when none waits, so the key goes on
/// to the terminal's program.
fn answer_focused_run(workspace: &Workspace, allowed: bool, window: &Window, cx: &mut App) -> bool {
    let Some(view) = crate::blocks::focused_terminal(workspace, window, cx) else {
        return false;
    };
    let id = view.entity_id();
    let waiting = cx
        .try_global::<Drives>()
        .and_then(|drives| drives.0.get(&id))
        .and_then(|drive| drive.pending.as_ref())
        .is_some_and(|pending| pending.run);
    if waiting {
        answer(id, allowed, cx);
    }
    waiting
}

/// The agent's run whose block still runs, for the bar.
fn running_run<'a>(drive: &'a Drive, terminal: &Terminal) -> Option<&'a Ran> {
    let (index, ran) = drive.runs.iter().next_back()?;
    terminal
        .blocks()
        .get(*index)
        .is_some_and(|block| block.state != BlockState::Finished)
        .then_some(ran)
}

/// Whether an agent's `terminal_run` typed block `index` of the terminal view `view`.
pub(crate) fn run_by_agent(view: EntityId, index: usize, cx: &App) -> bool {
    cx.try_global::<Drives>()
        .and_then(|drives| drives.0.get(&view))
        .is_some_and(|drive| drive.runs.contains_key(&index))
}

/// The mark before the pill of a block an agent ran, with who ran it as its tooltip.
pub(crate) fn agent_mark(view: EntityId, index: usize, cx: &App) -> Option<AnyElement> {
    let ran = cx.try_global::<Drives>()?.0.get(&view)?.runs.get(&index)?;
    let tooltip = format!("Run by {}", ran.who);
    Some(
        div()
            .id(("marley-agent-run", index))
            .debug_selector(move || format!("marley-agent-run-{index}"))
            .child(
                Icon::new(IconName::Sparkle)
                    .size(IconSize::XSmall)
                    .color(Color::Muted),
            )
            .tooltip(Tooltip::text(tooltip))
            .into_any_element(),
    )
}
