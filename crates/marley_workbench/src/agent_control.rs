//! The agent-control layer's modes (#704): what an agent may do in each area of Marley's tools, and
//! the question a tool that acts asks first.
//!
//! An area's mode is `off` (its tools refuse, reads too), `ask_every`, `ask_first` (a tool that acts
//! asks once per agent session) or `allow`. A session is the caller's Marley terminal, else its
//! client's name, together with its project, for as long as Marley runs; Always for This Project
//! keeps an area allowed in a project across restarts. Reads only pass the mode's `off`: the kill
//! switch and the activity log are #703's.

use std::collections::{BTreeSet, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use db::kvp::KeyValueStore;
use futures::FutureExt as _;
use futures::channel::oneshot;
use gpui::{App, AppContext as _, AsyncApp, Global, Task};
use marley_mcp::{AppCall, Refusal};
use settings::{MarleyAgentControlMode, SettingsStore};
use ui::prelude::*;
use util::ResultExt as _;
use workspace::notifications::simple_message_notification::MessageNotification;
use workspace::notifications::{NotificationId, dismiss_app_notification, show_app_notification};

use crate::asks::Ask;
use crate::prompts::Role;

/// How long the user has to answer, under the 30 seconds the server waits for the app.
const ANSWER_WAIT: Duration = Duration::from_secs(25);

/// The store's scope and key for the projects an area is allowed in for good.
const SCOPE: &str = "marley-agent-control";
const KEY: &str = "always";

/// An area of Marley's tools with a mode of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Area {
    /// Zed's editors (#704).
    Editors,
    /// The Agent Panel's threads (#706).
    Threads,
    /// Zed's palette actions (#707).
    Actions,
    /// Agents started as harness seats (#716).
    Agents,
}

impl Area {
    const fn name(self) -> &'static str {
        match self {
            Self::Editors => "editors",
            Self::Threads => "threads",
            Self::Actions => "actions",
            Self::Agents => "agents",
        }
    }

    const fn words(self) -> &'static str {
        match self {
            Self::Editors => "Zed's editors",
            Self::Threads => "the Agent Panel's threads",
            Self::Actions => "Zed's palette actions",
            Self::Agents => "starting other agents",
        }
    }

    /// The area's mode, `ask_first` when the user set none.
    fn mode(self, cx: &App) -> MarleyAgentControlMode {
        let agent_control = cx
            .global::<SettingsStore>()
            .merged_settings()
            .marley
            .as_ref()
            .and_then(|marley| marley.agent_control.as_ref());
        match self {
            Self::Editors => agent_control
                .and_then(|agent_control| agent_control.editors)
                .unwrap_or_default(),
            Self::Threads => agent_control
                .and_then(|agent_control| agent_control.threads)
                .unwrap_or_default(),
            Self::Actions => agent_control
                .and_then(|agent_control| agent_control.actions)
                .unwrap_or_default(),
            Self::Agents => agent_control
                .and_then(|agent_control| agent_control.agents)
                .unwrap_or_default(),
        }
    }
}

/// What a tool does, for how often it asks (#705).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Level {
    /// It only reads: it passes unless the area is off.
    Read,
    /// It acts: under `ask_first` it asks once per session.
    Act,
    /// It acts past undoing, such as a save: it asks every time unless the area is `allow`.
    Sensitive,
}

/// The user's answer to a tool's question.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Answer {
    Session,
    Project,
    Deny,
}

/// The sessions allowed in this run, by area, caller and project, and the projects an area is
/// allowed in for good, by area and project.
#[derive(Default)]
struct Approvals {
    sessions: HashSet<(&'static str, String, String)>,
    projects: BTreeSet<(String, String)>,
}

impl Global for Approvals {}

/// Reads the projects allowed for good from the store; [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    let projects = KeyValueStore::global(cx)
        .scoped(SCOPE)
        .read(KEY)
        .log_err()
        .flatten()
        .and_then(|text| serde_json::from_str::<BTreeSet<(String, String)>>(&text).log_err())
        .unwrap_or_default();
    cx.set_global(Approvals {
        sessions: HashSet::new(),
        projects,
    });
}

/// The caller's session: its Marley terminal, else its client's name.
fn caller_key(call: &AppCall) -> String {
    let caller = call.caller();
    caller
        .terminal
        .as_ref()
        .map(|terminal| format!("terminal:{terminal}"))
        .or_else(|| {
            caller
                .client
                .as_ref()
                .map(|client| format!("client:{client}"))
        })
        .unwrap_or_else(|| "agent".to_string())
}

/// The caller's project: its terminal's project, else its folder.
fn project_key(call: &AppCall) -> String {
    let caller = call.caller();
    caller
        .project
        .clone()
        .or_else(|| caller.cwd.clone())
        .unwrap_or_default()
}

/// Whether `call` may run in `area`: a read unless the area is off; a tool that acts as the mode
/// and the user's earlier answers say, asking when they don't; a sensitive one asking every time
/// unless the area is `allow`. `what` names what it acts on, for the question.
///
/// # Errors
///
/// `agent_control_off` while the area is off; `agent_control_declined` when the user denies or
/// doesn't answer.
pub(crate) fn admit(
    call: &AppCall,
    area: Area,
    level: Level,
    what: String,
    cx: &App,
) -> Task<Result<(), Refusal>> {
    admit_asking(
        call,
        area,
        level,
        Asked {
            what,
            rows: Vec::new(),
            footnote: None,
        },
        cx,
    )
}

/// What a question shows beyond its headline: a line, the facts as rows, and a footnote.
pub(crate) struct Asked {
    pub(crate) what: String,
    pub(crate) rows: Vec<(&'static str, String)>,
    pub(crate) footnote: Option<&'static str>,
}

/// [`admit`], the question showing `asked`'s rows and footnote too (#871).
///
/// # Errors
///
/// As [`admit`].
pub(crate) fn admit_asking(
    call: &AppCall,
    area: Area,
    level: Level,
    asked: Asked,
    cx: &App,
) -> Task<Result<(), Refusal>> {
    let mode = area.mode(cx);
    if mode == MarleyAgentControlMode::Off {
        return Task::ready(Err(Refusal::new(
            "agent_control_off",
            format!("the user turned agents' use of {} off", area.words()),
        )
        .next(format!(
            "ask the user to set marley.agent_control.{} to ask_first or allow",
            area.name()
        ))));
    }
    let session = (area.name(), caller_key(call), project_key(call));
    let project = (area.name().to_string(), session.2.clone());
    let allowed = cx.try_global::<Approvals>().is_some_and(|approvals| {
        approvals.sessions.contains(&session) || approvals.projects.contains(&project)
    });
    let asks = match (level, mode) {
        (Level::Read, _) | (_, MarleyAgentControlMode::Allow | MarleyAgentControlMode::Off) => {
            false
        }
        (Level::Act, MarleyAgentControlMode::AskFirst) => !allowed,
        (Level::Act, MarleyAgentControlMode::AskEvery) | (Level::Sensitive, _) => true,
    };
    if !asks {
        return Task::ready(Ok(()));
    }
    let who = crate::click_pause::Who::of(call, cx).words;
    let question = Question {
        headline: format!("{who} wants to use {}", call.tool),
        what: asked.what,
        rows: asked.rows,
        footnote: asked.footnote,
        project: session.2.clone(),
    };
    cx.spawn(async move |cx| match ask(question, cx).await {
        Some(Answer::Session) => {
            cx.update(|cx| {
                cx.default_global::<Approvals>().sessions.insert(session);
            });
            Ok(())
        }
        Some(Answer::Project) => {
            cx.update(|cx| {
                let approvals = cx.default_global::<Approvals>();
                approvals.sessions.insert(session);
                approvals.projects.insert(project);
                save(cx);
            });
            Ok(())
        }
        Some(Answer::Deny) => Err(Refusal::new("agent_control_declined", "the user denied it")
            .next("ask the user what they want instead")),
        None => Err(Refusal::new(
            "agent_control_declined",
            format!(
                "the user did not answer within {} seconds; nothing was done",
                ANSWER_WAIT.as_secs()
            ),
        )
        .next("tell the user what you meant to do, and try again when they are there")),
    })
}

/// Writes the projects allowed for good to the store.
fn save(cx: &App) {
    let Some(text) = cx
        .try_global::<Approvals>()
        .and_then(|approvals| serde_json::to_string(&approvals.projects).log_err())
    else {
        return;
    };
    let store = KeyValueStore::global(cx);
    cx.background_spawn(async move { store.scoped(SCOPE).write(KEY.to_string(), text).await })
        .detach_and_log_err(cx);
}

/// What the question shows.
#[derive(Clone)]
struct Question {
    headline: String,
    what: String,
    rows: Vec<(&'static str, String)>,
    footnote: Option<&'static str>,
    project: String,
}

/// The notifications' kind; each question has its own id within it.
struct AgentControlQuestion;

static NEXT_QUESTION: AtomicUsize = AtomicUsize::new(0);

/// The answer, given once from whichever button is clicked in whichever window.
type AnswerSlot = Arc<Mutex<Option<oneshot::Sender<Answer>>>>;

/// Asks `question` in every window and waits for an answer, at most [`ANSWER_WAIT`].
async fn ask(question: Question, cx: &AsyncApp) -> Option<Answer> {
    let id = NotificationId::composite::<AgentControlQuestion>(
        NEXT_QUESTION.fetch_add(1, Ordering::Relaxed),
    );
    let (sender, receiver) = oneshot::channel();
    let slot: AnswerSlot = Arc::new(Mutex::new(Some(sender)));
    cx.update(|cx| show(id.clone(), question, slot, cx));
    let timer = cx.background_executor().timer(ANSWER_WAIT);
    let answered = futures::select_biased! {
        answered = receiver.fuse() => answered.ok(),
        () = timer.fuse() => None,
    };
    cx.update(|cx| dismiss_app_notification(&id, cx));
    answered
}

fn show(id: NotificationId, question: Question, slot: AnswerSlot, cx: &mut App) {
    show_app_notification(id, cx, move |cx| {
        let question = question.clone();
        let answer = |answer: Answer| {
            let slot = Arc::clone(&slot);
            move |_: &mut Window, _: &mut App| give(&slot, answer)
        };
        let mut ask = Ask::new(question.headline);
        if !question.what.is_empty() {
            ask = ask.line(question.what);
        }
        for (label, value) in question.rows {
            ask = ask.row(label, value);
        }
        if !question.project.is_empty() {
            ask = ask.row("Project", question.project);
        }
        if let Some(footnote) = question.footnote {
            ask = ask.footnote(footnote);
        }
        let ask = ask
            .button("Deny", Role::Quiet, answer(Answer::Deny))
            .button(
                "Always for This Project",
                Role::Plain,
                answer(Answer::Project),
            )
            .button(
                "Allow for This Session",
                Role::Primary,
                answer(Answer::Session),
            );
        cx.new(|cx| MessageNotification::new_from_builder(cx, move |_, cx| ask.render(cx)))
    });
}

/// Gives the user's answer, the first one only.
fn give(slot: &AnswerSlot, answer: Answer) {
    let sender = slot.lock().ok().and_then(|mut sender| sender.take());
    if let Some(sender) = sender
        && sender.send(answer).is_err()
    {
        log::debug!("agent control: the answer came after the call stopped waiting");
    }
}
