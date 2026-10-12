//! Agents that start agents (#716).
//!
//! An agent in a Marley terminal calls `agent_start`; after the user allows it, Marley adds and
//! starts a seat on the first harness, labelled with its caller (`started_by = marley:<terminal
//! id>`) and its delegation (`delegation = <request id>`), sends the first prompt once with the
//! request id as its delivery, and opens the seat's view beside the caller, or in a new worktree's
//! window. The harness is the only record (rustal-harness MREQ-010): when a delegated seat's last
//! delivery reads `ended` and its `delegation.told` label is unset, Marley types one line into the
//! caller once the caller is at its prompt, then sets `delegation.told` with `session_label`. A
//! pointer missed while Marley was closed is typed once the restored caller is ready. A caller
//! that no open terminal holds gets a Needs you entry instead, which opens the seat and counts as
//! told. `agent_status` reads a delegation's state, and reading an ended one marks it told.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use gpui::{
    App, AppContext as _, AsyncApp, Entity, Global, SharedString, TaskExt as _, WeakEntity, Window,
};
use marley_fleet::Session;
use marley_mcp::{AppCall, Refusal, ToolAnswer};
use serde_json::{Value, json};
use terminal_view::TerminalView;
use util::ResultExt as _;
use workspace::item::Item as _;
use workspace::notifications::simple_message_notification::MessageNotification;
use workspace::notifications::{NotificationId, show_app_notification};
use workspace::{SplitDirection, Workspace};

use crate::agent_control::{Area, Level};
use crate::harness::Slot;
use crate::harness_seat::SeatAgent;

/// The label naming the terminal that started a seat, as `marley:<terminal id>`.
const STARTED_BY: &str = "started_by";
/// The label holding the delegation's id, the caller's `request_id`.
const DELEGATION: &str = "delegation";
/// The label set once the caller was told the delegation ended.
const TOLD: &str = "delegation.told";
/// The harness's labels for a session's last send (MREQ-010).
const DELIVERY_STATE: &str = "delivery.state";
const DELIVERY_OUTCOME: &str = "delivery.outcome";
/// What `started_by` begins with for a seat a Marley terminal started.
const CALLER_PREFIX: &str = "marley:";
/// The harness's limit on a prompt.
const MAX_PROMPT: usize = 8192;
/// How many lines `agent_status` reads from the seat's output.
const STATUS_LINES: u64 = 40;
/// How long an ended delegation waits for its caller before Needs you lists it: a restored
/// terminal comes back after the harness's first sessions do.
const CALLER_GRACE: Duration = Duration::from_secs(10);
/// How often a pending pointer looks again at its caller, and a refused first prompt is sent again.
const RECHECK: Duration = Duration::from_secs(2);
/// How long the first prompt is sent again while the seat refuses it: a Claude Code seat takes up
/// to a minute to start.
const SEND_FOR: Duration = Duration::from_secs(120);

/// What Marley is doing about ended delegations, between the harness's passes.
#[derive(Default)]
pub(crate) struct Delegations {
    /// Delegations whose pointer is being typed or whose `told` label is being set.
    in_flight: HashSet<String>,
    /// Delegations whose `told` label was set, until the harness publishes it.
    told: HashSet<String>,
    /// When each ended delegation was first seen without its caller.
    missing_since: HashMap<String, Instant>,
    /// The ended delegations whose caller no open terminal holds, for Needs you.
    orphans: Vec<Orphan>,
    /// Each delegated seat's caller's title, by the seat's session id, for the rail: worked out
    /// here, since the rail draws inside its window's update, which hides that window's
    /// terminals (PR-claude-596).
    callers: HashMap<String, String>,
    /// Whether the recheck loop runs.
    watching: bool,
}

impl Global for Delegations {}

/// An ended delegation whose caller is closed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Orphan {
    /// The seat's session id.
    pub(crate) seat: String,
    /// The seat's title.
    pub(crate) title: String,
    /// The delegation's id.
    pub(crate) delegation: String,
}

/// The ended delegations Needs you lists.
pub(crate) fn orphans(cx: &App) -> Vec<Orphan> {
    cx.try_global::<Delegations>()
        .map(|delegations| delegations.orphans.clone())
        .unwrap_or_default()
}

/// The terminal id a seat's `started_by` names, when a Marley terminal started it.
pub(crate) fn caller_of(seat: &Session) -> Option<&str> {
    seat.labels.get(STARTED_BY)?.strip_prefix(CALLER_PREFIX)
}

/// Where the started agent works.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Place {
    Here,
    Worktree,
}

/// How the started agent shows.
#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    Terminal,
    Thread,
}

/// An `agent_start` call's arguments, checked.
struct Request {
    agent: SeatAgent,
    prompt: String,
    place: Place,
    view: View,
    id: String,
}

impl Request {
    fn of(arguments: &Value) -> Result<Self, Refusal> {
        let text = |key: &str| {
            arguments
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
        };
        let bad = |reason: &str| {
            Refusal::new("bad_argument", reason).next("agent_start's schema lists the arguments")
        };
        let agent = match text("agent") {
            "claude" => SeatAgent::Claude,
            "codex" => SeatAgent::Codex,
            _ => return Err(bad("`agent` is `claude` or `codex`")),
        };
        let prompt = text("prompt").trim().to_string();
        if prompt.is_empty() || prompt.len() > MAX_PROMPT {
            return Err(bad("`prompt` holds 1 to 8192 bytes"));
        }
        let place = match text("place") {
            "here" => Place::Here,
            "worktree" => Place::Worktree,
            _ => return Err(bad("`place` is `here` or `worktree`")),
        };
        let view = match (text("view"), agent) {
            ("terminal", _) => View::Terminal,
            ("thread", SeatAgent::Claude) => View::Thread,
            ("thread", SeatAgent::Codex) => {
                return Err(bad(
                    "a thread view is for Claude Code; use `terminal` for Codex",
                ));
            }
            _ => return Err(bad("`view` is `terminal` or `thread`")),
        };
        let id = uuid::Uuid::parse_str(text("request_id"))
            .map_err(|_| bad("`request_id` is a UUID you keep"))?
            .to_string();
        Ok(Self {
            agent,
            prompt,
            place,
            view,
            id,
        })
    }

    /// The prompt's first line, for the question.
    fn first_line(&self) -> &str {
        self.prompt.lines().next().unwrap_or_default()
    }
}

/// What a checked `agent_start` starts from.
struct Start {
    request: Request,
    terminal_id: String,
    workspace: Entity<Workspace>,
    caller: WeakEntity<TerminalView>,
    folder: std::path::PathBuf,
    window: gpui::WindowHandle<workspace::MultiWorkspace>,
    /// The program and the arguments that run the first harness's commands, and whether they
    /// pass through a remote shell.
    command: (std::path::PathBuf, Vec<String>, bool),
}

/// Answers `agent_start`: the caller and the gates checked, the user asked, then the seat added
/// and started with its labels, the prompt sent once, and the view opened.
pub(crate) fn answer_start(call: AppCall, cx: &App) {
    let start = match checked(&call, cx) {
        Ok(start) => start,
        Err(refusal) => {
            call.answer(Err(refusal));
            return;
        }
    };
    let place = match start.request.place {
        Place::Here => format!("in {}", start.folder.display()),
        Place::Worktree => "in a new worktree".to_string(),
    };
    let what = format!(
        "{} {place}, with: {}",
        start.request.agent.shown(),
        start.request.first_line()
    );
    let admitted = crate::agent_control::admit(&call, Area::Agents, Level::Sensitive, what, cx);
    cx.spawn(async move |cx| {
        if let Err(refusal) = admitted.await {
            call.answer(Err(refusal));
            return;
        }
        // The server waits 30 seconds for the answer, and a seat takes up to a minute to start,
        // so the answer comes now and the start goes on, as `seat_add`'s does.
        call.answer::<Refusal>(Ok(ToolAnswer {
            structured: json!({ "delegation": start.request.id, "result": "starting" }),
            text: None,
            image: None,
        }));
        let delegation = start.request.id.clone();
        if let Err(refusal) = run(start, cx).await {
            let message = SharedString::from(format!(
                "The agent for delegation {delegation} did not start: {}",
                refusal.reason
            ));
            cx.update(|cx| {
                show_app_notification(NotificationId::unique::<StartFailed>(), cx, move |cx| {
                    cx.new(|cx| MessageNotification::new(message.clone(), cx))
                });
            });
        }
    })
    .detach();
}

/// The notification for a delegation that did not start.
struct StartFailed;

/// The request, its caller and the gates, or the refusal.
fn checked(call: &AppCall, cx: &App) -> Result<Start, Refusal> {
    let request = Request::of(&call.arguments)?;
    let terminal_id = call.caller().terminal.clone().ok_or_else(caller_unknown)?;
    let (workspace, caller) =
        crate::mcp::caller_terminal(call.caller(), cx).ok_or_else(caller_unknown)?;
    if !crate::harness::writes_on(cx) {
        return Err(Refusal::new(
            "tool_off",
            "the user has not let Marley write to the harness",
        )
        .next("ask the user to turn on marley.harness_writes"));
    }
    let command = crate::harness::seat_command_of(&Slot::Primary, cx).ok_or_else(|| {
        Refusal::new("no_harness", "Marley follows no harness it can reach")
            .next("ask the user to set marley.harness")
    })?;
    let folder = caller
        .read(cx)
        .terminal()
        .read(cx)
        .working_directory()
        .ok_or_else(|| {
            Refusal::new(
                "no_folder",
                "Marley does not know the folder your terminal works in",
            )
        })?;
    let window = crate::browser::window_of(&workspace, cx).ok_or_else(caller_unknown)?;
    Ok(Start {
        request,
        terminal_id,
        workspace,
        caller: caller.downgrade(),
        folder,
        window,
        command,
    })
}

/// The allowed start: the worktree when asked, the seat with its labels, the prompt, the view.
async fn run(start: Start, cx: &mut AsyncApp) -> Result<(), Refusal> {
    let Start {
        request,
        terminal_id,
        workspace,
        caller,
        folder,
        window,
        command: (program, base, remote),
    } = start;
    let (folder, home) = match request.place {
        Place::Here => (folder, workspace.downgrade()),
        Place::Worktree => {
            let making = window.update(cx, |_, window, cx| {
                workspace.update(cx, |_, cx| {
                    cx.spawn_in(window, async move |workspace, cx| {
                        crate::worktree_agents::make_for_seat(workspace, cx).await
                    })
                })
            });
            let made = match making {
                Ok(making) => making.await,
                Err(error) => Err(format!("{error:#}").into()),
            };
            let (path, worktree) =
                made.map_err(|reason| Refusal::new("worktree_failed", reason.to_string()))?;
            (path, worktree.downgrade())
        }
    };
    let folder_text = folder.to_string_lossy().into_owned();
    let taken: Vec<String> = cx.update(|cx| {
        crate::harness::seats_of(&Slot::Primary, cx)
            .seats()
            .iter()
            .map(|seat| seat.title.clone())
            .collect()
    });
    let stem = crate::harness_seat::seat_stem(request.agent, &folder_text);
    let labels = [
        (
            STARTED_BY.to_string(),
            format!("{CALLER_PREFIX}{terminal_id}"),
        ),
        (DELEGATION.to_string(), request.id.clone()),
    ];
    let (started, seat) = crate::harness_seat::add_and_start(
        &program,
        &base,
        remote,
        request.agent,
        &folder_text,
        (&stem, &taken),
        &labels,
    )
    .await
    .map_err(|said| crate::harness_seat::refusal_of(&said))?;
    let session = started.get("id").and_then(Value::as_str).ok_or_else(|| {
        Refusal::new(
            "seat_start_failed",
            format!("the seat {seat} started, but the harness did not name its session"),
        )
    })?;
    window
        .update(cx, |_, window, cx| {
            open_view(&home, &caller, request.view, &started, &folder, window, cx);
        })
        .log_err();
    send_prompt(session, &request, cx).await.map_err(|reason| {
        Refusal::new(
            "send_failed",
            format!("the seat {seat} started, but its prompt was not sent: {reason}"),
        )
    })
}

/// Sends the first prompt with the request's id as its delivery, so a repeat delivers nothing
/// twice: again each [`RECHECK`] while the seat refuses it, as a seat still starting does, for
/// [`SEND_FOR`] at most.
async fn send_prompt(session: &str, request: &Request, cx: &AsyncApp) -> Result<(), SharedString> {
    let arguments = json!({ "id": session, "text": request.prompt, "delivery": request.id });
    let until = Instant::now() + SEND_FOR;
    loop {
        match crate::harness::write_tool(&Slot::Primary, "session_send", arguments.clone(), cx)
            .await
        {
            Ok(_) => return Ok(()),
            Err(reason) if Instant::now() >= until => return Err(reason),
            Err(_) => cx.background_executor().timer(RECHECK).await,
        }
    }
}

fn caller_unknown() -> Refusal {
    Refusal::new(
        "caller_unknown",
        "agent_start starts an agent for a Marley terminal, and this call names none",
    )
    .next("call it from an agent running in a Marley terminal")
}

/// Opens the started seat in `home`: a thread tab, or a terminal attached to it, beside `caller`
/// when it is in `home` (the `here` place), else in `home`'s active pane (a worktree's window).
fn open_view(
    home: &WeakEntity<Workspace>,
    caller: &WeakEntity<TerminalView>,
    view: View,
    started: &Value,
    folder: &std::path::Path,
    window: &mut Window,
    cx: &mut App,
) {
    let Some(home) = home.upgrade() else {
        return;
    };
    if view == View::Thread {
        let Some(profile) = started.get("profile").and_then(Value::as_str) else {
            log::warn!("delegation: the seat started, but the harness did not name its profile");
            return;
        };
        let folder = Some(folder.to_path_buf());
        home.update(cx, |workspace, cx| {
            crate::harness_seat::open_seat_thread(
                workspace,
                &Slot::Primary,
                profile,
                folder,
                window,
                cx,
            );
        });
        return;
    }
    let Some(line) = crate::harness_seat::attach_line(&Slot::Primary, started, cx) else {
        log::warn!("delegation: the seat started, but the harness gave no view of it");
        return;
    };
    let input = format!("{line}\n").into_bytes();
    let directory = Some(folder.to_path_buf());
    let opening = home.update(cx, |workspace, cx| {
        crate::agents::start_in_terminal(workspace, directory, None, Some(input), None, window, cx)
    });
    let home_weak = home.downgrade();
    let caller = caller.clone();
    window
        .spawn(cx, async move |cx| {
            let terminal = opening.await?;
            home_weak.update_in(cx, |workspace, window, cx| {
                beside_caller(workspace, &terminal, &caller, window, cx);
            })
        })
        .detach_and_log_err(cx);
}

/// Moves the terminal view of `terminal` into a pane split right of the pane that holds `caller`,
/// when `workspace` holds both.
fn beside_caller(
    workspace: &mut Workspace,
    terminal: &WeakEntity<terminal::Terminal>,
    caller: &WeakEntity<TerminalView>,
    window: &mut Window,
    cx: &mut gpui::Context<Workspace>,
) {
    let (Some(terminal), Some(caller)) = (terminal.upgrade(), caller.upgrade()) else {
        return;
    };
    let holding = |item_id: gpui::EntityId, cx: &App| {
        workspace
            .panes()
            .iter()
            .find(|pane| pane.read(cx).items().any(|item| item.item_id() == item_id))
            .cloned()
    };
    let Some(view) = workspace
        .items_of_type::<TerminalView>(cx)
        .find(|view| view.read(cx).terminal() == &terminal)
    else {
        return;
    };
    let (Some(source), Some(beside)) = (
        holding(view.entity_id(), cx),
        holding(caller.entity_id(), cx),
    ) else {
        return;
    };
    let split = workspace.split_pane(beside, SplitDirection::Right, window, cx);
    workspace::move_item(&source, &split, view.entity_id(), 0, true, window, cx);
}

/// Answers `agent_status`: the delegated seat's state, its delivery's, its last lines redacted,
/// and whether its caller was told; reading an ended delegation marks it told.
pub(crate) fn answer_status(call: AppCall, cx: &mut App) {
    let delegation = call
        .arguments
        .get("delegation")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let Some(seat) = crate::harness::seats_of(&Slot::Primary, cx)
        .seats()
        .iter()
        .find(|seat| {
            caller_of(seat).is_some()
                && seat.labels.get(DELEGATION).map(String::as_str) == Some(delegation.as_str())
        })
        .cloned()
    else {
        call.answer(Err(Refusal::new(
            "no_delegation",
            format!("no seat of the harness carries the delegation {delegation}"),
        )
        .next("pass the request_id agent_start took")));
        return;
    };
    let ended = delivery_ended(&seat);
    let told = was_told(&seat, cx);
    if ended && !told {
        mark_told(&seat.id, &delegation, cx);
    }
    let redactor = crate::mcp::agent_redactor(cx);
    let arguments = json!({ "id": seat.id, "range": { "mode": "tail", "lines": STATUS_LINES } });
    cx.spawn(async move |cx| {
        let output: Vec<String> =
            match crate::harness::read_tool(&Slot::Primary, "session_read", arguments, cx).await {
                Ok(value) => crate::harness::read_lines(&value)
                    .log_err()
                    .unwrap_or_default()
                    .iter()
                    .map(|line| {
                        redactor
                            .as_ref()
                            .map_or_else(|| line.to_string(), |redactor| redactor.redact(line).text)
                    })
                    .collect(),
                Err(reason) => {
                    log::warn!("agent_status: {reason}");
                    Vec::new()
                }
            };
        call.answer::<Refusal>(Ok(ToolAnswer {
            structured: json!({
                "delegation": delegation,
                "seat": seat.title,
                "state": format!("{:?}", seat.state).to_lowercase(),
                "delivery": seat.labels.get(DELIVERY_STATE),
                "outcome": seat.labels.get(DELIVERY_OUTCOME),
                "output": output,
                "told": told || ended,
            }),
            text: None,
            image: None,
        }));
    })
    .detach();
}

/// Whether the seat's last delivery ended.
fn delivery_ended(seat: &Session) -> bool {
    seat.labels.get(DELIVERY_STATE).map(String::as_str) == Some("ended")
}

/// Whether the seat's caller was told: its label, or Marley set it and the harness has yet to
/// publish it.
fn was_told(seat: &Session, cx: &App) -> bool {
    seat.labels.contains_key(TOLD)
        || seat.labels.get(DELEGATION).is_some_and(|delegation| {
            cx.try_global::<Delegations>()
                .is_some_and(|delegations| delegations.told.contains(delegation))
        })
}

/// Sets the seat's `delegation.told` label through the harness, once.
pub(crate) fn mark_told(seat: &str, delegation: &str, cx: &mut App) {
    let delegations = cx.default_global::<Delegations>();
    if !delegations.in_flight.insert(delegation.to_string()) {
        return;
    }
    let (seat, delegation) = (seat.to_string(), delegation.to_string());
    cx.spawn(async move |cx| {
        let arguments = json!({
            "id": seat,
            "key": TOLD,
            "value": "1",
            "request": uuid::Uuid::new_v4().to_string(),
        });
        let labelled =
            crate::harness::write_tool(&Slot::Primary, "session_label", arguments, cx).await;
        cx.update(|cx| {
            let delegations = cx.default_global::<Delegations>();
            delegations.in_flight.remove(&delegation);
            match labelled {
                Ok(_) => {
                    delegations.told.insert(delegation);
                }
                Err(reason) => log::warn!("delegation {delegation}: told not set: {reason}"),
            }
        });
    })
    .detach();
}

/// Follows the delegations after `slot`'s sessions changed: a pass, and the watch while a pointer
/// waits.
pub(crate) fn follow(slot: &Slot, cx: &mut App) {
    if *slot == Slot::Primary && pass(cx) {
        watch(cx);
    }
}

/// One pass over the first harness's seats: each ended delegation whose caller was not told gets
/// its pointer typed into a ready caller, waits for a busy one, and goes to Needs you while no
/// open terminal holds its caller. Whether anything waits.
fn pass(cx: &mut App) -> bool {
    let slot = &Slot::Primary;
    let pending: Vec<Session> = crate::harness::seats_of(slot, cx)
        .seats()
        .iter()
        .filter(|seat| caller_of(seat).is_some() && seat.labels.contains_key(DELEGATION))
        .filter(|seat| delivery_ended(seat) && !was_told(seat, cx))
        .cloned()
        .collect();
    let terminals = crate::mcp::terminals(cx);
    let callers: HashMap<String, String> = crate::harness::seats_of(slot, cx)
        .seats()
        .iter()
        .filter_map(|seat| {
            let caller = caller_of(seat)?;
            let (_, view) = terminals.iter().find(|(_, view)| {
                view.read(cx).terminal().read(cx).marley_terminal_id() == Some(caller)
            })?;
            Some((
                seat.id.clone(),
                view.read(cx).tab_content_text(0, cx).to_string(),
            ))
        })
        .collect();
    if cx.default_global::<Delegations>().callers != callers {
        cx.default_global::<Delegations>().callers = callers;
    }
    let mut waiting = false;
    let mut orphans = Vec::new();
    let mut missing = Vec::new();
    for seat in &pending {
        let (Some(caller), Some(delegation)) = (caller_of(seat), seat.labels.get(DELEGATION))
        else {
            continue;
        };
        if cx
            .try_global::<Delegations>()
            .is_some_and(|delegations| delegations.in_flight.contains(delegation))
        {
            continue;
        }
        let found = terminals.iter().find(|(_, view)| {
            view.read(cx).terminal().read(cx).marley_terminal_id() == Some(caller)
        });
        let Some((workspace, view)) = found else {
            missing.push(delegation.clone());
            let since = *cx
                .default_global::<Delegations>()
                .missing_since
                .entry(delegation.clone())
                .or_insert_with(Instant::now);
            if since.elapsed() >= CALLER_GRACE {
                orphans.push(Orphan {
                    seat: seat.id.clone(),
                    title: seat.title.clone(),
                    delegation: delegation.clone(),
                });
            } else {
                waiting = true;
            }
            continue;
        };
        let project = workspace
            .read(cx)
            .project()
            .read(cx)
            .visible_worktrees(cx)
            .next()
            .map(|worktree| worktree.read(cx).root_name_str().to_string())
            .unwrap_or_default();
        let ready = crate::send_selection::target_of(project, view.clone(), cx)
            .is_some_and(|target| target.ready);
        if ready {
            point(seat, delegation, view, cx);
        } else {
            waiting = true;
        }
    }
    let delegations = cx.default_global::<Delegations>();
    delegations
        .missing_since
        .retain(|delegation, _| missing.contains(delegation));
    if delegations.orphans != orphans {
        delegations.orphans = orphans;
    }
    // A still-missing caller keeps the watch, so a restored one is told once it is ready.
    waiting || !delegations.orphans.is_empty()
}

/// Types the pointer into the caller and Enter, then sets `told`.
fn point(seat: &Session, delegation: &str, caller: &Entity<TerminalView>, cx: &mut App) {
    let outcome = seat
        .labels
        .get(DELIVERY_OUTCOME)
        .map_or("ended", String::as_str);
    // Ids only: the child's words never reach the caller this way (D4).
    let line = format!(
        "Agent {} ended: {outcome}. Read the result: agent_status {{\"delegation\": \"{delegation}\"}}.",
        seat.title
    );
    let terminal = caller.read(cx).terminal().clone();
    crate::terminal_drive::paste_then(
        &terminal,
        &line,
        |terminal| terminal.input(b"\r".to_vec()),
        cx,
    )
    .detach();
    mark_told(&seat.id, delegation, cx);
}

/// Looks again every [`RECHECK`] while a pointer waits for its caller, since the caller's own
/// state changes outside the harness's passes.
fn watch(cx: &mut App) {
    if std::mem::replace(&mut cx.default_global::<Delegations>().watching, true) {
        return;
    }
    cx.spawn(async move |cx: &mut AsyncApp| {
        loop {
            cx.background_executor().timer(RECHECK).await;
            if !cx.update(pass) {
                cx.update(|cx| cx.default_global::<Delegations>().watching = false);
                break;
            }
        }
    })
    .detach();
}

/// The title of the terminal that started a delegated seat, for the rail's line under it.
pub(crate) fn caller_title(seat: &Session, cx: &App) -> Option<String> {
    cx.try_global::<Delegations>()?
        .callers
        .get(&seat.id)
        .cloned()
}
