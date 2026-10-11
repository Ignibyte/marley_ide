//! A harness seat in one step (#691).
//!
//! `marley: new harness seat` asks for a name, the agent, a folder and a role, then runs the
//! harness's `seat add` and `seat start` (its TICKET-109) through the command Marley follows the
//! harness with, and opens the new session's tab.
//!
//! New Agent's **On `<harness>`…** (#741) does it without the form: `start_agent_on` names the seat
//! for the agent and the folder, starts it, and types its attach view into a new terminal, through
//! `ssh -t` for a harness on another host.
//!
//! `seat_add` on Marley's MCP server (#692) does the same for an agent: it asks the user as
//! `settings_change` does, runs `seat add` on Apply and answers, then runs `seat start`, which can
//! take a minute for Claude Code, after the answer; a start that fails is a notification.
//! `seat_stop` and `seat_remove` (#710) ask the same way and run the harness's `seat stop` or
//! `seat remove` (its TICKET-114).
//!
//! The command is `harness::Harness::seat_command`'s: a `marley.harness` command up to its `mcp`,
//! so a harness reached over SSH is set up over SSH, or the embedded runtime's `rh --state
//! <root>`. Each answers one JSON object on stdout, or `rh: CODE: reason` on stderr, which the form
//! shows. The palette lists the action only while the harness's write verbs are on.

use std::path::{Path, PathBuf};

use editor::Editor;
use gpui::{
    App, AppContext as _, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable,
    SharedString, Task, WeakEntity, Window, actions,
};
use marley_agent::AgentKind;
use marley_mcp::{AppCall, Refusal, ToolAnswer};
use serde_json::{Value, json};
use ui::{ButtonStyle, Headline, HeadlineSize, prelude::*};
use util::ResultExt as _;
use workspace::notifications::simple_message_notification::MessageNotification;
use workspace::notifications::{NotificationId, show_app_notification};
use workspace::{ModalView, Toast, Workspace};

use crate::harness::{Harness, Slot, harness_writes, seat_command_of, seats_of, writes_on};

actions!(
    marley,
    [
        /// Adds and starts a harness seat: a Claude Code or Codex session from a new profile, with
        /// a role, opened in its tab.
        #[derive(Eq)]
        NewHarnessSeat
    ]
);

/// The agent a seat runs.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SeatAgent {
    Claude,
    Codex,
}

impl SeatAgent {
    /// The seat agent an agent CLI runs as, for the two the harness runs (#741).
    pub(crate) const fn of(kind: AgentKind) -> Option<Self> {
        match kind {
            AgentKind::Claude => Some(Self::Claude),
            AgentKind::Codex => Some(Self::Codex),
            AgentKind::Gemini | AgentKind::OpenCode => None,
        }
    }

    /// The agent as `seat add --agent` takes it.
    const fn argument(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
        }
    }

    /// The agent as the user reads it.
    const fn shown(self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "Codex",
        }
    }
}

/// The seat a form or an agent asks for.
struct Seat {
    name: String,
    agent: SeatAgent,
    cwd: String,
    role: String,
}

impl Seat {
    /// `seat add` and `seat start` for the seat, after the harness's own `base` arguments.
    ///
    /// When `remote`, the words reach `rh` through the host's shell, as ssh joins them, so each is
    /// quoted for it (#740).
    fn commands(&self, base: Vec<String>, remote: bool) -> (Vec<String>, Vec<String>) {
        let word = |word: &str| {
            if remote {
                crate::harness_hosts::shell_word(word)
            } else {
                word.to_string()
            }
        };
        let mut add = base.clone();
        add.extend([
            "seat".to_string(),
            "add".to_string(),
            word(&self.name),
            "--agent".to_string(),
            self.agent.argument().to_string(),
            "--cwd".to_string(),
            word(&self.cwd),
        ]);
        if !self.role.is_empty() {
            add.extend(["--role".to_string(), word(&self.role)]);
        }
        let mut start = base;
        start.extend(["seat".to_string(), "start".to_string(), word(&self.name)]);
        (add, start)
    }
}

/// The notification for a seat that was added and did not start.
struct SeatStartFailed;

/// Answers `seat_add` (#692): checks the seat, asks the user, and on Apply adds it, answers, and
/// starts it.
pub(crate) fn answer_seat_add(call: AppCall, cx: &App) {
    if !writes_on(cx) {
        call.answer(Err(Refusal::new(
            "tool_off",
            "Marley's harness writes are off (`marley.harness_writes`), or it follows no harness",
        )
        .next(
            "ask the user to turn on marley.harness_writes and name their harness",
        )));
        return;
    }
    let seat = match seat_of(&call.arguments) {
        Ok(seat) => seat,
        Err(refusal) => {
            call.answer(Err(refusal));
            return;
        }
    };
    let Some((program, base)) = Harness::seat_command(cx) else {
        call.answer(Err(Refusal::new(
            "unavailable",
            "Marley follows no harness it can reach",
        )));
        return;
    };
    let asker = call
        .caller()
        .client
        .clone()
        .unwrap_or_else(|| "An agent".to_string());
    let question = crate::settings_change::Question {
        headline: format!("{asker} wants to set up a harness seat"),
        change: format!(
            "{} · {} · in {}{}",
            seat.name,
            seat.agent.shown(),
            seat.cwd,
            if seat.role.is_empty() {
                String::new()
            } else {
                format!(" · role {}", seat.role)
            }
        ),
        file: format!("through {} {}", program.display(), base.join(" ")),
    };
    let (add, start) = seat.commands(base, false);
    cx.spawn(async move |cx| {
        if let Err(refusal) = crate::settings_change::ask_user(question, cx).await {
            call.answer(Err(refusal));
            return;
        }
        let added = match run_seat(&program, &add).await {
            Ok(added) => added,
            Err(said) => {
                call.answer(Err(refusal_of(&said)));
                return;
            }
        };
        call.answer::<Refusal>(Ok(ToolAnswer {
            structured: json!({
                "result": "starting",
                "seat": seat.name,
                "profile": added.get("profile"),
                "kind": added.get("kind"),
            }),
            text: None,
            image: None,
        }));
        if let Err(said) = run_seat(&program, &start).await {
            let message = format!("The harness seat {} did not start: {said}", seat.name);
            let message = SharedString::from(message);
            cx.update(move |cx| {
                show_app_notification(NotificationId::unique::<SeatStartFailed>(), cx, move |cx| {
                    cx.new(|cx| MessageNotification::new(message.clone(), cx))
                });
            });
        }
    })
    .detach();
}

/// Answers `seat_stop` and `seat_remove` (#710): asks the user, and on Apply runs the harness's
/// `seat stop` or `seat remove` (its TICKET-114) and answers its object with `result`.
pub(crate) fn answer_seat_end(call: AppCall, cx: &App) {
    let removes = call.tool == "seat_remove";
    if !writes_on(cx) {
        call.answer(Err(Refusal::new(
            "tool_off",
            "Marley's harness writes are off (`marley.harness_writes`), or it follows no harness",
        )
        .next(
            "ask the user to turn on marley.harness_writes and name their harness",
        )));
        return;
    }
    let name = call
        .arguments
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default()
        .to_string();
    if name.is_empty() {
        call.answer(Err(Refusal::new("bad_argument", "give the seat's `name`")));
        return;
    }
    let Some((program, base)) = Harness::seat_command(cx) else {
        call.answer(Err(Refusal::new(
            "unavailable",
            "Marley follows no harness it can reach",
        )));
        return;
    };
    let asker = call
        .caller()
        .client
        .clone()
        .unwrap_or_else(|| "An agent".to_string());
    let (verb, change) = if removes {
        (
            "remove",
            format!("{name} · stop its sessions and delete its profile"),
        )
    } else {
        (
            "stop",
            format!("{name} · stop its sessions; the profile stays, so it can start again"),
        )
    };
    let question = crate::settings_change::Question {
        headline: format!("{asker} wants to {verb} the harness seat {name}"),
        change,
        file: format!("through {} {}", program.display(), base.join(" ")),
    };
    let mut command = base;
    command.extend(["seat".to_string(), verb.to_string(), name.clone()]);
    cx.spawn(async move |cx| {
        if let Err(refusal) = crate::settings_change::ask_user(question, cx).await {
            call.answer(Err(refusal));
            return;
        }
        match run_seat(&program, &command).await {
            Ok(mut answer) => {
                if let Some(fields) = answer.as_object_mut() {
                    fields.insert(
                        "result".into(),
                        Value::from(if removes { "removed" } else { "stopped" }),
                    );
                    fields
                        .entry("seat")
                        .or_insert_with(|| Value::from(name.clone()));
                }
                call.answer::<Refusal>(Ok(ToolAnswer {
                    structured: answer,
                    text: None,
                    image: None,
                }));
            }
            Err(said) => call.answer(Err(refusal_of(&said))),
        }
    })
    .detach();
}

/// The seat `seat_add`'s arguments name.
///
/// # Errors
///
/// `bad_argument` for an agent other than `claude` or `codex`, or no name or folder.
fn seat_of(arguments: &Value) -> Result<Seat, Refusal> {
    let text = |name: &str| {
        arguments
            .get(name)
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or_default()
            .to_string()
    };
    let agent = match text("agent").as_str() {
        "claude" => SeatAgent::Claude,
        "codex" => SeatAgent::Codex,
        _ => {
            return Err(Refusal::new(
                "bad_argument",
                "`agent` is `claude` or `codex`",
            ));
        }
    };
    let seat = Seat {
        name: text("name"),
        agent,
        cwd: text("cwd"),
        role: text("role"),
    };
    if seat.name.is_empty() || seat.cwd.is_empty() {
        return Err(Refusal::new(
            "bad_argument",
            "give the seat's `name` and the folder it works in, `cwd`",
        ));
    }
    Ok(seat)
}

/// The harness's `CODE: reason` as a refusal by that code, as its message of 2026-10-07 asks: the
/// token before the first `:` is the code when it is lowercase letters and `_`, and the rest is
/// shown as given. A plain `text` is `refused` with all of it (#693).
fn refusal_of(said: &str) -> Refusal {
    match said.split_once(": ") {
        Some((code, reason))
            if !code.is_empty() && code.chars().all(|c| c.is_ascii_lowercase() || c == '_') =>
        {
            Refusal::new(code.to_string(), reason)
        }
        _ => Refusal::new(Refusal::REFUSED, said),
    }
}

/// Opens the form in `workspace`, with the active project's folder as the seat's.
pub(crate) fn open_form(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let slots = seat_slots(cx);
    let Some(slot) = slots.first().cloned() else {
        return;
    };
    let folder = terminal_view::default_working_directory(workspace, cx);
    let handle = workspace.weak_handle();
    workspace.toggle_modal(window, cx, |window, cx| {
        NewSeatModal::new(handle, folder, slots, slot, window, cx)
    });
}

/// The harnesses a seat can be made on: the primary one while its write verbs are on, then each
/// `marley.harnesses` names while `marley.harness_writes` is (#740).
pub(crate) fn seat_slots(cx: &App) -> Vec<Slot> {
    let mut slots = Vec::new();
    if writes_on(cx) {
        slots.push(Slot::Primary);
    }
    if harness_writes(cx) {
        slots.extend(
            crate::harness_hosts::Hosts::names(cx)
                .into_iter()
                .map(Slot::Host),
        );
    }
    slots
}

/// The toast while an agent starts on a harness, and the notification when it does not (#741).
struct AgentOnHarness;

/// Starts `kind` on `slot`'s harness in `folder`, a path on its host (#741): a seat named for the
/// agent and the folder, then a new terminal of `workspace` attached to it. A refusal, or a seat
/// the harness gives no view of, is a notification, and no terminal opens.
pub(crate) fn start_agent_on(
    workspace: &mut Workspace,
    slot: Slot,
    kind: AgentKind,
    folder: String,
    window: &Window,
    cx: &mut Context<Workspace>,
) {
    let Some(agent) = SeatAgent::of(kind) else {
        return;
    };
    let place = slot
        .name()
        .map_or_else(|| "the harness".to_string(), ToString::to_string);
    let Some((program, base, remote)) = seat_command_of(&slot, cx) else {
        not_started(agent, &place, "Marley follows no harness it can reach", cx);
        return;
    };
    let taken: Vec<String> = seats_of(&slot, cx)
        .seats()
        .iter()
        .map(|seat| seat.title.clone())
        .collect();
    let stem = seat_stem(agent, &folder);
    workspace.show_toast(
        Toast::new(
            NotificationId::unique::<AgentOnHarness>(),
            format!(
                "Starting {} on {place} in {folder}: up to a minute",
                agent.shown()
            ),
        ),
        cx,
    );
    cx.spawn_in(window, async move |workspace, cx| {
        let started = add_and_start(&program, &base, remote, agent, &folder, &stem, &taken).await;
        let started = match started {
            Ok(started) => started,
            Err(reason) => {
                workspace
                    .update(cx, |workspace, cx| {
                        workspace.dismiss_toast(&NotificationId::unique::<AgentOnHarness>(), cx);
                        not_started(agent, &place, &reason, cx);
                    })
                    .log_err();
                return;
            }
        };
        workspace
            .update_in(cx, |workspace, window, cx| {
                workspace.dismiss_toast(&NotificationId::unique::<AgentOnHarness>(), cx);
                let Some(line) = attach_line(&slot, &started, cx) else {
                    let reason = "the seat started, but the harness gave no view of it";
                    not_started(agent, &place, reason, cx);
                    return;
                };
                let input = format!("{line}\n").into_bytes();
                // The link's local end works in the project's folder, as a new terminal would.
                let folder = terminal_view::default_working_directory(workspace, cx);
                crate::agents::start_in_terminal(
                    workspace,
                    folder,
                    None,
                    Some(input),
                    None,
                    window,
                    cx,
                )
                .detach_and_log_err(cx);
            })
            .log_err();
    })
    .detach();
}

/// Adds the seat under the first name the root takes, `stem`, then `stem-2` … `stem-9`, skipping
/// the names in `taken`, and starts it: `seat start`'s answer.
async fn add_and_start(
    program: &Path,
    base: &[String],
    remote: bool,
    agent: SeatAgent,
    folder: &str,
    stem: &str,
    taken: &[String],
) -> Result<Value, SharedString> {
    let names = std::iter::once(stem.to_string())
        .chain((2..=9).map(|number| format!("{stem}-{number}")))
        .filter(|name| !taken.contains(name));
    for name in names {
        let seat = Seat {
            name,
            agent,
            cwd: folder.to_string(),
            role: String::new(),
        };
        let (add, start) = seat.commands(base.to_vec(), remote);
        match run_seat(program, &add).await {
            Ok(_) => return run_seat(program, &start).await,
            // A seat of that name the rail does not show, such as a stopped one.
            Err(said) if said.starts_with("seat_exists") => {}
            Err(said) => return Err(said),
        }
    }
    Err(format!("the names {stem} to {stem}-9 are all taken").into())
}

/// A seat's name from the agent and the folder's last part, as `claude-other` for `/srv/other`:
/// lowercase letters, digits and dashes.
fn seat_stem(agent: SeatAgent, folder: &str) -> String {
    let last = folder
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or_default();
    let word: String = last
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let word = word.trim_matches('-');
    if word.is_empty() {
        agent.argument().to_string()
    } else {
        format!("{}-{word}", agent.argument())
    }
}

/// The line that attaches a terminal to a started seat: its `tmux` view (`rh attach`), else its
/// `native` one (`rh view`), from `seat start`'s answer.
fn attach_line(slot: &Slot, started: &Value, cx: &App) -> Option<SharedString> {
    let views: Vec<(SharedString, Vec<String>)> = started
        .get("views")?
        .as_array()?
        .iter()
        .filter_map(crate::harness::view_argv)
        .collect();
    let (_, argv) = ["tmux", "native"]
        .iter()
        .find_map(|kind| views.iter().find(|(shown, _)| shown.as_ref() == *kind))?;
    Some(crate::harness::view_command(slot, argv, cx))
}

/// Says that the agent did not start on `place`, and why.
fn not_started(agent: SeatAgent, place: &str, reason: &str, cx: &mut App) {
    let message = SharedString::from(format!(
        "Could not start {} on {place}: {reason}",
        agent.shown()
    ));
    show_app_notification(NotificationId::unique::<AgentOnHarness>(), cx, move |cx| {
        cx.new(|cx| MessageNotification::new(message.clone(), cx))
    });
}

/// The form: the seat's name, agent, folder and role, and how the last run went.
struct NewSeatModal {
    workspace: WeakEntity<Workspace>,
    /// The harnesses the seat can go on, and the one chosen (#740).
    slots: Vec<Slot>,
    slot: Slot,
    name: Entity<Editor>,
    folder: Entity<Editor>,
    role: Entity<Editor>,
    agent: SeatAgent,
    status: Option<(SharedString, Color)>,
    running: Option<Task<()>>,
}

impl NewSeatModal {
    fn new(
        workspace: WeakEntity<Workspace>,
        folder: Option<PathBuf>,
        slots: Vec<Slot>,
        slot: Slot,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let field = |placeholder: &str, text: Option<String>, window: &mut Window, cx: &mut App| {
            cx.new(|cx| {
                let mut editor = Editor::single_line(window, cx);
                editor.set_placeholder_text(placeholder, window, cx);
                if let Some(text) = text {
                    editor.set_text(text, window, cx);
                }
                editor
            })
        };
        let name = field("A name, such as manager or builder-1", None, window, cx);
        let folder = field(
            "The folder it works in",
            folder.map(|folder| folder.display().to_string()),
            window,
            cx,
        );
        let role = field("manager, or any label (optional)", None, window, cx);
        Self {
            workspace,
            slots,
            slot,
            name,
            folder,
            role,
            agent: SeatAgent::Claude,
            status: None,
            running: None,
        }
    }

    /// Runs `seat add`, then `seat start`, and opens the session's tab; a refusal stays in the
    /// form.
    fn create(&mut self, window: &Window, cx: &mut Context<Self>) {
        if self.running.is_some() {
            return;
        }
        let seat = Seat {
            name: self.name.read(cx).text(cx).trim().to_string(),
            agent: self.agent,
            cwd: self.folder.read(cx).text(cx).trim().to_string(),
            role: self.role.read(cx).text(cx).trim().to_string(),
        };
        if seat.name.is_empty() || seat.cwd.is_empty() {
            self.status = Some((
                SharedString::new_static("A seat needs a name and a folder"),
                Color::Error,
            ));
            cx.notify();
            return;
        }
        let Some((program, base, remote)) = seat_command_of(&self.slot, cx) else {
            self.status = Some((
                SharedString::new_static("Marley follows no harness it can reach"),
                Color::Error,
            ));
            cx.notify();
            return;
        };
        let (add, start) = seat.commands(base, remote);
        let slot = self.slot.clone();
        let waiting = if self.agent == SeatAgent::Claude {
            "Starting the seat: Claude Code reports once it is up, within a minute"
        } else {
            "Starting the seat"
        };
        self.status = Some((SharedString::new_static(waiting), Color::Muted));
        cx.notify();
        let workspace = self.workspace.clone();
        self.running = Some(cx.spawn_in(window, async move |modal, cx| {
            let started = async {
                run_seat(&program, &add).await?;
                run_seat(&program, &start).await
            }
            .await;
            match started {
                Ok(value) => {
                    let id = value
                        .get("id")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    if !id.is_empty() {
                        workspace
                            .update_in(cx, |workspace, window, cx| {
                                crate::harness::open_in_home(
                                    workspace.weak_handle(),
                                    slot,
                                    id,
                                    window,
                                    cx,
                                );
                            })
                            .log_err();
                    }
                    modal.update(cx, |_, cx| cx.emit(DismissEvent)).log_err();
                }
                Err(reason) => {
                    modal
                        .update(cx, |modal, cx| {
                            modal.running = None;
                            modal.status = Some((reason, Color::Error));
                            cx.notify();
                        })
                        .log_err();
                }
            }
        }));
    }

    /// The Harness choice, while more than one harness can take the seat (#740).
    fn render_slots(&self, cx: &Context<Self>) -> AnyElement {
        v_flex()
            .gap_0p5()
            .child(
                Label::new("Harness")
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .children(self.slots.iter().enumerate().map(|(index, slot)| {
                        let label = slot
                            .name()
                            .map_or_else(|| SharedString::new_static("Default"), Clone::clone);
                        let chosen = slot.clone();
                        Button::new(("harness-seat-slot", index), label)
                            .style(ButtonStyle::Filled)
                            .toggle_state(self.slot == *slot)
                            .on_click(cx.listener(move |modal, _, _, cx| {
                                modal.slot = chosen.clone();
                                cx.notify();
                            }))
                    })),
            )
            .into_any_element()
    }

    fn render_field(label: &'static str, editor: &Entity<Editor>, cx: &App) -> AnyElement {
        let colors = cx.theme().colors();
        v_flex()
            .gap_0p5()
            .child(Label::new(label).size(LabelSize::Small).color(Color::Muted))
            .child(
                div()
                    .px_2()
                    .py_0p5()
                    .rounded_md()
                    .border_1()
                    .border_color(colors.border)
                    .bg(colors.editor_background)
                    .child(editor.clone()),
            )
            .into_any_element()
    }
}

/// Runs one harness command: its JSON answer, or the harness's `CODE: reason`.
async fn run_seat(program: &Path, arguments: &[String]) -> Result<Value, SharedString> {
    let output = crate::process::output(program, arguments, None, &[])
        .await
        .map_err(|error| {
            SharedString::from(format!("{} did not start: {error}", program.display()))
        })?;
    if output.status.success() {
        return serde_json::from_slice::<Value>(&output.stdout).map_err(|error| {
            SharedString::from(format!("the harness's answer did not parse: {error}"))
        });
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Exit 2 is the argument parser's: Marley called the harness wrongly (#693).
    if output.status.code() == Some(2) {
        let first = stderr.lines().next().unwrap_or_default();
        return Err(format!("Marley called the harness wrongly (a usage error): {first}").into());
    }
    let said = stderr
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("the harness refused, and said nothing");
    Err(said.strip_prefix("rh: ").unwrap_or(said).to_string().into())
}

impl ModalView for NewSeatModal {}

impl EventEmitter<DismissEvent> for NewSeatModal {}

impl Focusable for NewSeatModal {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.name.focus_handle(cx)
    }
}

impl Render for NewSeatModal {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let agent_button = |id: &'static str, label: &'static str, agent: SeatAgent| {
            Button::new(id, label)
                .style(ButtonStyle::Filled)
                .toggle_state(self.agent == agent)
                .on_click(cx.listener(move |modal, _, _, cx| {
                    modal.agent = agent;
                    cx.notify();
                }))
        };
        v_flex()
            .key_context("NewHarnessSeat")
            .on_action(cx.listener(|_, _: &menu::Cancel, _, cx| cx.emit(DismissEvent)))
            .on_action(cx.listener(|modal, _: &menu::Confirm, window, cx| modal.create(window, cx)))
            .w(rems(34.))
            .elevation_3(cx)
            .p_3()
            .gap_3()
            .child(
                v_flex()
                    .gap_1()
                    .child(Headline::new("New Harness Seat").size(HeadlineSize::Small))
                    .child(
                        Label::new(
                            "The harness adds a profile for the seat and starts it under its \
                             supervision. A seat whose role is manager becomes the root's manager.",
                        )
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                    ),
            )
            .when(self.slots.len() > 1, |form| {
                form.child(self.render_slots(cx))
            })
            .child(Self::render_field("Name", &self.name, cx))
            .child(
                v_flex()
                    .gap_0p5()
                    .child(
                        Label::new("Agent")
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(agent_button(
                                "harness-seat-claude",
                                "Claude Code",
                                SeatAgent::Claude,
                            ))
                            .child(agent_button(
                                "harness-seat-codex",
                                "Codex",
                                SeatAgent::Codex,
                            )),
                    ),
            )
            .child(Self::render_field("Folder", &self.folder, cx))
            .child(Self::render_field("Role", &self.role, cx))
            .when_some(self.status.clone(), |this, (status, color)| {
                this.child(Label::new(status).size(LabelSize::Small).color(color))
            })
            .child(
                h_flex().justify_end().child(
                    Button::new("harness-seat-create", "Create")
                        .style(ButtonStyle::Filled)
                        .disabled(self.running.is_some())
                        .on_click(cx.listener(|modal, _, window, cx| modal.create(window, cx))),
                ),
            )
    }
}
