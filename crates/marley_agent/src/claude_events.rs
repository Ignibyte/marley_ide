//! Claude Code's hook events, as Marley's plugin sends them (#519).
//!
//! The plugin's `hooks/event.py` answers each hook with an OSC 777 notify titled
//! `marley-event`, whose body is the base64 of a short JSON summary of the event. [`decode`]
//! reads that body, and [`fold`] turns one event into the fleet events that move the
//! terminal's seat, starting from the seat as it was:
//! - a prompt or a tool makes the seat working, and shows the prompt and the tool in flight;
//! - a permission request makes it waiting, until the tool it asked for finishes (tools run in
//!   parallel, so another tool finishing does not end the wait) or the turn ends;
//! - `Stop` makes it idle with the last message, and `StopFailure` failed with its error;
//! - a subagent's events move a count and leave the lead's state, except that a subagent's
//!   permission request makes the seat wait too;
//! - `SessionEnd` ends the seat, and a new session in the same terminal starts it over;
//! - what the user's request has done so far, [`TurnFacts`], rides on the seat as its `turn`
//!   label for the stop kind (#566), which every turn's start and end clears.
//!
//! A prompt a harness injects (a task notification, a system reminder, a slash command's
//! envelope) is not the user's, so the seat keeps the user's prompt; the continuation after a
//! compaction changes nothing. The tags and prefixes are the ones Orca observed, from
//! `src/shared/harness-injected-user-turns.ts` in stablyai/orca (MIT). Claude Code does not
//! document them, so on a version they were not checked on a [`PromptReading::AllTyped`] reads
//! every prompt as the user's (#648); [`fold`] reads them with the tags.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use base64::Engine as _;
use marley_fleet::{Question, Session, SessionEvent, State, Transport};
use serde::{Deserialize, Serialize};

use crate::stall::{self, FlagShown};
use crate::stop_kind::{self, StopKindShown};
use crate::{AgentStatus, TurnEvent};

/// The largest summary a frame carries, decoded; the plugin keeps its summaries under it.
const MAX_SUMMARY: usize = 3_000;

/// How many of the request's tool lines [`TurnFacts`] keeps, for the loop rule (#569).
const MAX_TOOL_LINES: usize = 12;

/// The most characters of a tool line [`TurnFacts`] keeps.
const MAX_TOOL_LINE: usize = 120;

/// The label that names the seat's agent, `claude-code`.
pub const AGENT_LABEL: &str = "agent";
/// The label for the user's latest prompt, on one line.
pub const PROMPT_LABEL: &str = "prompt";
/// The label for the lead's tool in flight and what it acts on, such as `Bash: ls -la`.
pub const TOOL_LABEL: &str = "tool";
/// The label for the turn's last message, on one line.
pub const MESSAGE_LABEL: &str = "message";
/// The label for a failed turn's error type, such as `rate_limit`.
pub const ERROR_LABEL: &str = "error";
/// The label for how many subagents run in the turn.
pub const SUBAGENTS_LABEL: &str = "subagents";
/// The label for Claude Code's session id.
pub const SESSION_LABEL: &str = "session_id";
/// The label for the session's working directory, where Claude Code started (#535).
pub const CWD_LABEL: &str = "cwd";
/// The label for the user's prompt the lead's latest event belongs to.
pub const PROMPT_ID_LABEL: &str = "prompt_id";
/// The label for the session's permission mode, such as `default` or `bypassPermissions`.
pub const PERMISSION_MODE_LABEL: &str = "permission_mode";
/// The label for the user's request's [`TurnFacts`], as JSON (#566).
pub const TURN_LABEL: &str = "turn";
/// A lead's tool in flight, by its tool call's id.
const LEAD_TOOL_PREFIX: &str = "lead_tool:";
/// A subagent's tool in flight, by its tool call's id.
const SUBAGENT_TOOL_PREFIX: &str = "subagent_tool:";
/// The tool call a wait ends with; empty when the request named no call Marley saw start.
const WAITING_ON_LABEL: &str = "waiting_on";

/// One hook event, as the plugin summarized it. Fields the plugin adds later are ignored.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct HookEvent {
    /// The summary's format, 1.
    pub v: u32,
    /// The hook: `UserPromptSubmit`, `PreToolUse`, `Stop` and the rest.
    pub event: String,
    /// Claude Code's session.
    pub session_id: Option<String>,
    /// The subagent the event comes from; none for the lead.
    pub agent_id: Option<String>,
    /// The user's prompt the event belongs to.
    pub prompt_id: Option<String>,
    /// The session's permission mode, such as `default` or `plan`.
    pub permission_mode: Option<String>,
    /// The session's working directory.
    pub cwd: Option<String>,
    /// The session's transcript.
    pub transcript_path: Option<String>,
    /// The prompt, for `UserPromptSubmit`.
    pub prompt: Option<String>,
    /// The tool, for the tool events and `PermissionRequest`.
    pub tool: Option<String>,
    /// What the tool acts on: a file, a command, a pattern.
    pub preview: Option<String>,
    /// The tool call's id, for the tool events.
    pub tool_use_id: Option<String>,
    /// The last message, for `Stop`, `StopFailure` and `SubagentStop`.
    pub message: Option<String>,
    /// The error type, for `StopFailure`.
    pub error: Option<String>,
    /// Why the session started, for `SessionStart`.
    pub source: Option<String>,
    /// Why the session ended, for `SessionEnd`: `clear`, `resume`, `logout`, `prompt_input_exit`
    /// or `other` (#540).
    pub reason: Option<String>,
    /// `manual` or `auto`, for `PostCompact`.
    pub trigger: Option<String>,
    /// Whether the user interrupted the tool, for `PostToolUseFailure`.
    pub is_interrupt: Option<bool>,
    /// The first question's option labels, for an `AskUserQuestion` (#570).
    pub options: Option<Vec<String>>,
}

/// What the user's request has done so far (#566), for the stop kind. The fold starts from the
/// seat at each event, so the facts ride on it as its `turn` label.
///
/// A prompt the user types starts them over. A prompt a harness injects continues the request,
/// so it keeps the tools and the checks and starts only the turn's own `interrupted` and
/// `pending` over.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TurnFacts {
    /// When the user's prompt came, in the fleet's epoch milliseconds.
    pub started_ms: u64,
    /// The lead's tools by name, with how many times each started.
    pub tools: BTreeMap<String, u32>,
    /// How many tools the subagents started.
    pub subagent_tools: u32,
    /// How many tools failed, the lead's and the subagents', interrupts aside.
    pub failures: u32,
    /// Whether the user interrupted the turn.
    pub interrupted: bool,
    /// How many permissions were asked for.
    pub permissions_asked: u32,
    /// The tool calls a permission was asked for that have not finished.
    pub pending: BTreeSet<String>,
    /// Whether a Write, Edit, `MultiEdit` or `NotebookEdit` finished.
    pub edited: bool,
    /// Whether a Bash finished without failing after the last edit, or with no edit, at all.
    pub checked_after_edit: bool,
    /// How many times the request has stopped, which tells one stop from the next.
    pub stops: u32,
    /// The lead's newest tool lines as they ended, oldest first, for the loop rule (#569).
    pub tool_lines: Vec<ToolLine>,
}

/// A lead tool's line as it ended (#569).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolLine {
    /// The tool and what it acted on, such as `Bash: cargo test`.
    pub line: String,
    /// Whether it ended in a failure.
    pub failed: bool,
}

impl TurnFacts {
    /// The facts a seat's labels hold. None yet, or a label this Marley does not read, is none.
    #[must_use]
    pub fn of(labels: &BTreeMap<String, String>) -> Self {
        labels
            .get(TURN_LABEL)
            .and_then(|facts| serde_json::from_str(facts).ok())
            .unwrap_or_default()
    }
}

/// Why a frame's body is not an event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// The body is not base64.
    Base64,
    /// The summary is longer than a frame carries.
    TooLong(usize),
    /// The summary is not the plugin's JSON.
    Json(String),
    /// A summary of a format this Marley does not read.
    Version(u32),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Base64 => write!(formatter, "the body is not base64"),
            Self::TooLong(length) => write!(formatter, "the summary is {length} bytes"),
            Self::Json(error) => write!(formatter, "the summary is not an event: {error}"),
            Self::Version(version) => write!(formatter, "a summary of format {version}"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Reads a `marley-event` notification's body.
///
/// # Errors
///
/// [`DecodeError`] when the body is not the plugin's base64 summary of format 1.
pub fn decode(body: &str) -> Result<HookEvent, DecodeError> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(body.trim())
        .map_err(|_| DecodeError::Base64)?;
    if bytes.len() > MAX_SUMMARY {
        return Err(DecodeError::TooLong(bytes.len()));
    }
    let event: HookEvent =
        serde_json::from_slice(&bytes).map_err(|error| DecodeError::Json(error.to_string()))?;
    if event.v != 1 {
        return Err(DecodeError::Version(event.v));
    }
    Ok(event)
}

/// The fleet events that move seat `seat` for `event`, from the seat as it was (`previous`), at
/// `now_ms`. An event that changes nothing gives none.
#[must_use]
pub fn fold(
    seat: &str,
    previous: Option<&Session>,
    event: &HookEvent,
    now_ms: u64,
) -> Vec<SessionEvent> {
    fold_with(seat, previous, event, now_ms, PromptReading::Recognized)
}

/// [`fold`], its prompts read as `reading` says (#648).
#[must_use]
pub fn fold_with(
    seat: &str,
    previous: Option<&Session>,
    event: &HookEvent,
    now_ms: u64,
    reading: PromptReading,
) -> Vec<SessionEvent> {
    if event.event == "SessionEnd" {
        return vec![SessionEvent::Ended {
            id: seat.to_string(),
            ts_ms: now_ms,
        }];
    }
    let mut moving = Moving::from_seat(previous);
    moving.note_session(event);
    if !moving.take(event, now_ms, reading) {
        return Vec::new();
    }
    // Any event is the agent moving again, which ends a stall's flag; a loop's stays while the
    // loop goes on (#569).
    let still_looping = moving
        .labels
        .get(stall::FLAG_LABEL)
        .is_some_and(|flag| flag == "looping")
        && moving.state == State::Working
        && stall::repeats(&moving.facts.tool_lines).is_some();
    if !still_looping {
        moving.clear(&stall::FLAG_LABELS);
    }
    moving.into_events(seat, now_ms)
}

/// How many subagents a seat's labels count.
#[must_use]
pub fn subagents(labels: &BTreeMap<String, String>) -> u32 {
    labels
        .get(SUBAGENTS_LABEL)
        .and_then(|count| count.parse().ok())
        .unwrap_or(0)
}

/// The status Claude Code's events give an agent row.
#[must_use]
pub const fn seat_status(state: State) -> AgentStatus {
    match state {
        State::Starting | State::Working => AgentStatus::Working,
        State::Waiting => AgentStatus::Waiting,
        State::Idle | State::Done => AgentStatus::Idle,
        State::Error => AgentStatus::Failed,
    }
}

/// An agent row's second line from its seat, such as `working · 1 subagent · Add a README`.
///
/// It gives what the agent is doing, the subagents running and the user's prompt. The row's icon
/// names the agent, so the line starts with the state. A working seat whose last event is at
/// least `no_update_after_ms` old at `now_ms` reads `no update in N m` in place of `working`
/// (#547): an Escape fires no hook, so the seat cannot know the turn stopped. A threshold of 0
/// never marks it. An idle seat with a stop kind reads it as `shown` says (#566), and a working
/// seat with a stall or loop flag reads `looping?` or `stalled?` as `flag` says, before `no update`
/// (#569).
#[must_use]
pub fn seat_line(
    seat: &Session,
    now_ms: u64,
    no_update_after_ms: u64,
    shown: StopKindShown,
    flag: FlagShown,
) -> String {
    let flagged = stall::row_word(&seat.labels, flag).filter(|_| seat.state == State::Working);
    let state = flagged.unwrap_or_else(|| {
        if seat.state == State::Working
            && no_update_after_ms > 0
            && marley_fleet::is_stale(seat, now_ms, no_update_after_ms)
        {
            let minutes = now_ms.saturating_sub(seat.last_event_ms) / 60_000;
            format!("no update in {minutes} m")
        } else if let Some(word) =
            stop_kind::row_word(&seat.labels, shown).filter(|_| seat.state == State::Idle)
        {
            word
        } else {
            seat_status(seat.state).label().to_string()
        }
    });
    let subagents = match subagents(&seat.labels) {
        0 => None,
        1 => Some("1 subagent".to_string()),
        count => Some(format!("{count} subagents")),
    };
    std::iter::once(state)
        .chain(subagents)
        .chain(seat.labels.get(PROMPT_LABEL).cloned())
        .collect::<Vec<_>>()
        .join(" · ")
}

/// An agent row's third line: what the agent waits on, its error, the tool in flight, or the
/// turn's last message, after the prompt's parts the stop kind found it does not cover, as
/// `shown` says (#566).
#[must_use]
pub fn seat_activity(seat: &Session, shown: StopKindShown) -> Option<String> {
    match seat.state {
        State::Waiting => seat
            .question
            .as_ref()
            .map(|question| question.prompt.clone()),
        State::Error => seat.labels.get(ERROR_LABEL).cloned(),
        State::Idle => {
            let not_covered = stop_kind::not_covered(&seat.labels, shown);
            let message = seat.labels.get(MESSAGE_LABEL).cloned();
            match (not_covered, message) {
                (Some(not_covered), Some(message)) => Some(format!("{not_covered} · {message}")),
                (not_covered, message) => not_covered.or(message),
            }
        }
        State::Done => seat.labels.get(MESSAGE_LABEL).cloned(),
        State::Starting | State::Working => seat.labels.get(TOOL_LABEL).cloned(),
    }
}

/// A seat while an event moves it.
struct Moving {
    labels: BTreeMap<String, String>,
    state: State,
    question: Option<Question>,
    facts: TurnFacts,
}

impl Moving {
    fn from_seat(previous: Option<&Session>) -> Self {
        previous.map_or_else(
            || Self {
                labels: BTreeMap::new(),
                state: State::Starting,
                question: None,
                facts: TurnFacts::default(),
            },
            |session| Self {
                labels: session.labels.clone(),
                state: session.state,
                question: session.question.clone(),
                facts: TurnFacts::of(&session.labels),
            },
        )
    }

    /// Starts the seat over for another session in the same terminal, and keeps the lead's
    /// session details.
    fn note_session(&mut self, event: &HookEvent) {
        if let Some(session_id) = &event.session_id {
            if self
                .labels
                .get(SESSION_LABEL)
                .is_some_and(|known| known != session_id)
            {
                *self = Self::from_seat(None);
            }
            self.set(SESSION_LABEL, session_id);
        }
        self.set(AGENT_LABEL, "claude-code");
        if event.agent_id.is_none() {
            for (key, value) in [
                (PROMPT_ID_LABEL, &event.prompt_id),
                (PERMISSION_MODE_LABEL, &event.permission_mode),
                (CWD_LABEL, &event.cwd),
                ("transcript_path", &event.transcript_path),
            ] {
                if let Some(value) = value {
                    self.set(key, value);
                }
            }
        }
    }

    /// Moves the seat for `event` at `now_ms`; false when the event changes nothing.
    fn take(&mut self, event: &HookEvent, now_ms: u64, reading: PromptReading) -> bool {
        let lead = event.agent_id.is_none();
        match event.event.as_str() {
            "SessionStart" => {
                if matches!(
                    event.source.as_deref(),
                    Some("startup" | "resume" | "clear" | "fork")
                ) {
                    self.end_turn(State::Idle);
                    self.clear(&[PROMPT_LABEL, MESSAGE_LABEL, ERROR_LABEL]);
                    self.facts = TurnFacts::default();
                }
            }
            "UserPromptSubmit" if lead => {
                let prompt = event.prompt.as_deref().unwrap_or_default();
                if reading.is_continuation(prompt) {
                    return false;
                }
                self.end_turn(State::Working);
                self.clear(&[MESSAGE_LABEL, ERROR_LABEL]);
                if reading.is_injected(prompt) {
                    self.facts.interrupted = false;
                    self.facts.pending.clear();
                } else {
                    self.set(PROMPT_LABEL, &one_line(prompt));
                    self.facts = TurnFacts {
                        started_ms: now_ms,
                        ..TurnFacts::default()
                    };
                }
            }
            "PreToolUse" => self.tool_starts(event, lead),
            "PostToolUse" | "PostToolUseFailure" => {
                self.note_tool_end(event, lead);
                self.tool_ends(event, lead);
            }
            "PermissionRequest" => {
                let asked = tool_line(event);
                let waiting_on = self
                    .labels
                    .iter()
                    .filter(|(_, shown)| **shown == asked)
                    .find_map(|(key, _)| tool_call(key))
                    .unwrap_or_default()
                    .to_string();
                self.facts.permissions_asked = self.facts.permissions_asked.saturating_add(1);
                // A request that names no call Marley saw start is not pending: nothing would
                // ever finish it, and the stop would read `blocked` after the call ran.
                let call = event
                    .tool_use_id
                    .clone()
                    .unwrap_or_else(|| waiting_on.clone());
                if !call.is_empty() {
                    let _added = self.facts.pending.insert(call);
                }
                self.wait(format!("{PERMISSION_FOR}{asked}"), Vec::new(), &waiting_on);
            }
            "Stop" if lead => {
                self.end_turn(State::Idle);
                self.facts.stops = self.facts.stops.saturating_add(1);
                if let Some(message) = &event.message {
                    self.set(MESSAGE_LABEL, &one_line(message));
                }
            }
            "StopFailure" if lead => {
                self.end_turn(State::Error);
                self.set(
                    ERROR_LABEL,
                    &one_line(event.error.as_deref().unwrap_or("unknown")),
                );
            }
            "PostCompact" if event.trigger.as_deref() == Some("manual") => {
                self.end_turn(State::Idle);
            }
            "SubagentStart" => {
                let count = subagents(&self.labels).saturating_add(1);
                self.set(SUBAGENTS_LABEL, &count.to_string());
            }
            "SubagentStop" => match subagents(&self.labels).saturating_sub(1) {
                0 => self.clear(&[SUBAGENTS_LABEL]),
                count => self.set(SUBAGENTS_LABEL, &count.to_string()),
            },
            _ => return false,
        }
        true
    }

    fn tool_starts(&mut self, event: &HookEvent, lead: bool) {
        if lead {
            if let Some(tool) = &event.tool {
                let count = self.facts.tools.entry(tool.clone()).or_default();
                *count = count.saturating_add(1);
            }
        } else {
            self.facts.subagent_tools = self.facts.subagent_tools.saturating_add(1);
        }
        let shown = tool_line(event);
        if let Some(id) = &event.tool_use_id {
            let prefix = if lead {
                LEAD_TOOL_PREFIX
            } else {
                SUBAGENT_TOOL_PREFIX
            };
            self.set(&format!("{prefix}{id}"), &shown);
        }
        if event.tool.as_deref() == Some("AskUserQuestion") {
            let question = event
                .preview
                .as_deref()
                .map_or_else(|| "A question".to_string(), one_line);
            let options = event.options.clone().unwrap_or_default();
            self.wait(
                question,
                options,
                event.tool_use_id.as_deref().unwrap_or_default(),
            );
        } else if lead {
            self.set(TOOL_LABEL, &shown);
            if self.state != State::Waiting {
                self.state = State::Working;
            }
        }
    }

    fn tool_ends(&mut self, event: &HookEvent, lead: bool) {
        if let Some(id) = &event.tool_use_id {
            self.clear(&[
                &format!("{LEAD_TOOL_PREFIX}{id}"),
                &format!("{SUBAGENT_TOOL_PREFIX}{id}"),
            ]);
            if self.labels.get(WAITING_ON_LABEL) == Some(id) {
                self.clear(&[WAITING_ON_LABEL]);
                self.question = None;
                self.state = State::Working;
            }
        }
        if !lead {
            return;
        }
        // Another of the lead's tools may still run: it becomes the one shown.
        let running = self.lead_tools().next().map(str::to_string);
        match running {
            Some(running) => self.set(TOOL_LABEL, &running),
            None => self.clear(&[TOOL_LABEL]),
        }
        if event.event == "PostToolUseFailure" && event.is_interrupt == Some(true) {
            self.end_turn(State::Idle);
        } else if self.state != State::Waiting {
            self.state = State::Working;
        }
    }

    /// Counts what a finished tool tells the stop kind: a failure, an interrupt, an edit, or a
    /// command after the last edit. An interrupt of the lead's tool ends the turn, which counts
    /// as a stop.
    fn note_tool_end(&mut self, event: &HookEvent, lead: bool) {
        if let Some(call) = &event.tool_use_id {
            let _was_pending = self.facts.pending.remove(call);
        }
        let failed = event.event == "PostToolUseFailure";
        if lead && event.is_interrupt != Some(true) {
            let line = tool_line(event).chars().take(MAX_TOOL_LINE).collect();
            self.facts.tool_lines.push(ToolLine { line, failed });
            let excess = self.facts.tool_lines.len().saturating_sub(MAX_TOOL_LINES);
            self.facts.tool_lines.drain(..excess).for_each(drop);
        }
        if failed {
            if event.is_interrupt == Some(true) {
                // Tools in parallel each report the one interrupt.
                if lead && !self.facts.interrupted {
                    self.facts.interrupted = true;
                    self.facts.stops = self.facts.stops.saturating_add(1);
                }
            } else {
                self.facts.failures = self.facts.failures.saturating_add(1);
            }
            return;
        }
        match event.tool.as_deref() {
            Some("Write" | "Edit" | "MultiEdit" | "NotebookEdit") => {
                self.facts.edited = true;
                self.facts.checked_after_edit = false;
            }
            Some("Bash") => self.facts.checked_after_edit = true,
            _ => {}
        }
    }

    /// The lead's tools in flight, each as `Tool: preview`.
    fn lead_tools(&self) -> impl Iterator<Item = &str> {
        self.labels
            .iter()
            .filter(|(key, _)| key.starts_with(LEAD_TOOL_PREFIX))
            .map(|(_, shown)| shown.as_str())
    }

    fn wait(&mut self, prompt: String, options: Vec<String>, waiting_on: &str) {
        self.set(WAITING_ON_LABEL, waiting_on);
        self.state = State::Waiting;
        self.question = Some(Question {
            prompt,
            options,
            context_refs: Vec::new(),
        });
    }

    /// Forgets the turn's tools, its wait, its subagents and the last stop's kind, and moves the
    /// seat to `state`. The stop kind of the turn that ends now lands after the fold, when it
    /// is asked for.
    fn end_turn(&mut self, state: State) {
        self.labels.retain(|key, _| {
            !key.starts_with(LEAD_TOOL_PREFIX)
                && !key.starts_with(SUBAGENT_TOOL_PREFIX)
                && ![WAITING_ON_LABEL, TOOL_LABEL, SUBAGENTS_LABEL].contains(&key.as_str())
                && !stop_kind::STOP_LABELS.contains(&key.as_str())
        });
        self.question = None;
        self.state = state;
    }

    fn set(&mut self, key: &str, value: &str) {
        let _previous = self.labels.insert(key.to_string(), value.to_string());
    }

    fn clear(&mut self, keys: &[&str]) {
        self.labels.retain(|key, _| !keys.contains(&key.as_str()));
    }

    fn into_events(mut self, seat: &str, now_ms: u64) -> Vec<SessionEvent> {
        // A map of strings and numbers always serializes: the label is the facts, or none.
        match serde_json::to_string(&self.facts) {
            Ok(facts) => self.set(TURN_LABEL, &facts),
            Err(_) => self.clear(&[TURN_LABEL]),
        }
        let waiting = self.state == State::Waiting;
        let mut events = vec![SessionEvent::Upsert {
            id: seat.to_string(),
            ts_ms: now_ms,
            title: "Claude Code".to_string(),
            state: self.state,
            labels: self.labels,
            transport: Some(Transport::Local),
        }];
        // An upsert keeps a waiting seat's question, so a new question is raised on its own.
        if let Some(question) = self.question.filter(|_| waiting) {
            events.push(SessionEvent::QuestionRaised {
                id: seat.to_string(),
                ts_ms: now_ms,
                question,
            });
        }
        events
    }
}

/// The tool call a tool-in-flight label names.
fn tool_call(key: &str) -> Option<&str> {
    key.strip_prefix(LEAD_TOOL_PREFIX)
        .or_else(|| key.strip_prefix(SUBAGENT_TOOL_PREFIX))
}

/// How a permission request's question starts, before the tool and what it acts on.
const PERMISSION_FOR: &str = "Permission for ";

/// The most characters a banner's body holds, Orca's limit, which fits two or three lines.
const BANNER_BODY_MAX: usize = 180;

/// What a desktop banner says about `event` on `seat` (#538).
///
/// A finish gives the turn's last message; a permission request `Using <tool>: <input>`; a
/// question its text; a failure its kind in words, such as `rate limit`. The text is on one line
/// and cut by [`notification_text`].
#[must_use]
pub fn banner_body(event: TurnEvent, seat: &Session) -> String {
    let text = match event {
        TurnEvent::Finished => seat.labels.get(MESSAGE_LABEL).cloned(),
        TurnEvent::NeedsInput => seat.question.as_ref().map(|question| {
            question
                .prompt
                .strip_prefix(PERMISSION_FOR)
                .map_or_else(|| question.prompt.clone(), |asked| format!("Using {asked}"))
        }),
        TurnEvent::Failed => seat
            .labels
            .get(ERROR_LABEL)
            .map(|error| error.replace('_', " ")),
    };
    notification_text(&text.unwrap_or_default())
}

/// `text` on one line, its runs of whitespace as single spaces, and past 180 characters cut to
/// 179 on a character boundary and ended with `…`, as Orca's banners are.
#[must_use]
pub fn notification_text(text: &str) -> String {
    let text = one_line(text);
    if text.chars().count() <= BANNER_BODY_MAX {
        return text;
    }
    let mut cut: String = text.chars().take(BANNER_BODY_MAX - 1).collect();
    cut.push('…');
    cut
}

/// A tool and what it acts on, such as `Bash: ls -la`, or the tool alone.
fn tool_line(event: &HookEvent) -> String {
    let tool = event.tool.as_deref().unwrap_or("a tool");
    event.preview.as_deref().map_or_else(
        || tool.to_string(),
        |preview| format!("{tool}: {}", one_line(preview)),
    )
}

/// `text` on one line: its runs of whitespace, line breaks included, as single spaces.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The tags a harness puts at the head of a prompt it injects, as Orca observed them.
const HARNESS_TAGS: [&str; 19] = [
    "agent-message",
    "bash-input",
    "bash-stderr",
    "bash-stdout",
    "command-args",
    "command-message",
    "command-name",
    "cross-session-message",
    "fork-boilerplate",
    "local-command-caveat",
    "local-command-stderr",
    "local-command-stdout",
    "mcp-polling-update",
    "mcp-resource-update",
    "system-reminder",
    "task-notification",
    "teammate-message",
    "user-memory-input",
    "user-prompt-submit-hook",
];

/// The opening of the prompt Claude Code sends to continue after a compaction.
const COMPACT_CONTINUATION: &str = "this session is being continued from a previous conversation";

/// The openings of the other prompts a harness injects, as Orca observed them.
const HARNESS_PREFIXES: [&str; 7] = [
    "<channel source=",
    "[request interrupted",
    "a message arrived from ",
    "another claude session sent a message",
    "no response requested.",
    "caveat: the messages below were generated by the user while running local commands",
    COMPACT_CONTINUATION,
];

/// A prompt's opening, lowercased, for the checks below, so a long paste stays cheap.
fn opening(prompt: &str) -> String {
    prompt
        .trim_start()
        .chars()
        .take(256)
        .collect::<String>()
        .to_lowercase()
}

/// Whether a harness injected `prompt` (a known tag or opening), rather than the user.
#[must_use]
pub fn is_harness_injected(prompt: &str) -> bool {
    let opening = opening(prompt);
    let tag = opening.strip_prefix('<').and_then(|rest| {
        rest.split(|character: char| character.is_whitespace() || character == '>')
            .next()
    });
    tag.is_some_and(|tag| HARNESS_TAGS.contains(&tag))
        || HARNESS_PREFIXES
            .iter()
            .any(|prefix| opening.starts_with(prefix))
}

/// Whether `prompt` is the one Claude Code sends to continue after a compaction.
#[must_use]
pub fn is_compact_continuation(prompt: &str) -> bool {
    opening(prompt).starts_with(COMPACT_CONTINUATION)
}

/// How a session's prompts are read (#648): with the tags and openings that mark a prompt a
/// harness injects, or every prompt as the user's, on a Claude Code version they were not
/// checked on.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PromptReading {
    /// A known tag or opening marks a prompt as injected.
    #[default]
    Recognized,
    /// Every prompt is the user's, as `UserPromptSubmit` means in Claude Code's hooks reference.
    AllTyped,
}

impl PromptReading {
    /// Whether a harness injected `prompt`, as this reading tells it.
    #[must_use]
    pub fn is_injected(self, prompt: &str) -> bool {
        self == Self::Recognized && is_harness_injected(prompt)
    }

    /// Whether `prompt` is the continuation after a compaction, as this reading tells it.
    #[must_use]
    pub fn is_continuation(self, prompt: &str) -> bool {
        self == Self::Recognized && is_compact_continuation(prompt)
    }

    /// Where `prompt` came from, as this reading tells it.
    #[must_use]
    pub fn origin(self, prompt: &str) -> PromptOrigin {
        match self {
            Self::Recognized => prompt_origin(prompt),
            Self::AllTyped => PromptOrigin::User,
        }
    }
}

/// Where a prompt came from, as per-turn diffs title its turn (#509).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptOrigin {
    /// The user typed it.
    User,
    /// The user ran a slash command, whose envelope names it, such as `/review`.
    SlashCommand(String),
    /// A harness injected it: by the tag at its head, or by an opening with no tag.
    Injected(Option<&'static str>),
    /// Claude Code's own continuation after a compaction, which starts no turn.
    Continuation,
}

/// The tags of a slash command's envelope, which the user's own command carries.
const COMMAND_TAGS: [&str; 3] = ["command-name", "command-message", "command-args"];

/// Where `prompt` came from (#509): the compaction's continuation, a slash command the user ran,
/// a harness's injection with its tag, or the user.
#[must_use]
pub fn prompt_origin(prompt: &str) -> PromptOrigin {
    if is_compact_continuation(prompt) {
        return PromptOrigin::Continuation;
    }
    let opening = opening(prompt);
    let tag = opening.strip_prefix('<').and_then(|rest| {
        rest.split(|character: char| character.is_whitespace() || character == '>')
            .next()
    });
    if let Some(tag) = tag.and_then(|tag| HARNESS_TAGS.iter().copied().find(|known| *known == tag))
    {
        if COMMAND_TAGS.contains(&tag) {
            return PromptOrigin::SlashCommand(command_name(prompt).unwrap_or_default());
        }
        return PromptOrigin::Injected(Some(tag));
    }
    if HARNESS_PREFIXES
        .iter()
        .any(|prefix| opening.starts_with(prefix))
    {
        return PromptOrigin::Injected(None);
    }
    PromptOrigin::User
}

/// The command a slash command's envelope names, between `<command-name>` and its close.
fn command_name(prompt: &str) -> Option<String> {
    let (_, rest) = prompt.split_once("<command-name>")?;
    let (name, _) = rest.split_once("</command-name>")?;
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_string())
}
