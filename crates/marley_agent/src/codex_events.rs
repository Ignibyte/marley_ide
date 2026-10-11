//! Codex's state from its own App Server (#650).
//!
//! Marley joins the App Server a Codex terminal's TUI runs against as a second client and reads
//! the lead thread's messages: [`Notification::decode`] reads the ones Marley follows, and
//! [`fold`] turns each into the fleet events that move the terminal's seat, starting from the seat
//! as it was:
//! - `thread/status/changed` decides the state: `active` is working, or waiting with
//!   `waitingOnApproval` or `waitingOnUserInput`; `idle` is idle; `systemError` is failed;
//!   `notLoaded` ends the seat;
//! - a turn that completes as `failed` makes the seat failed with the turn's error, through the
//!   `idle` that follows, until the next turn starts;
//! - token use and the thread's approval and sandbox policies ride on the seat as labels;
//! - the account's rate-limit windows ride on the seat as `quota.KIND.` labels, and a turn that
//!   fails at the usage limit makes the seat limited until the next turn starts (#715).
//!
//! The shapes are the ones Codex 0.155.1, 0.158.0 and 0.162.0 generate as JSON Schema; a field
//! Marley does not read is ignored, and a message that does not parse leaves the seat as it was.

use std::collections::BTreeMap;

use marley_fleet::{Question, Session, SessionEvent, State, Transport};
use serde::Deserialize;
use serde_json::Value;

use crate::claude_events::{AGENT_LABEL, ERROR_LABEL};
use crate::usage::{LIMIT_RESETS_AT_LABEL, LIMITED_LABEL, QUOTA_PREFIX, limit, limit_reset};
use crate::{AgentStatus, claude_events};

/// The agent label's value on a Codex seat.
pub const AGENT: &str = "codex";
/// The label for the lead thread's id.
pub const THREAD_LABEL: &str = "thread";
/// The label for what a waiting seat waits on: [`WAITS_ON_APPROVAL`] or [`WAITS_ON_INPUT`].
pub const WAIT_LABEL: &str = "wait";
/// The label for the thread's total token use.
pub const TOKENS_LABEL: &str = "tokens";
/// The label for the model's context window, in tokens.
pub const CONTEXT_WINDOW_LABEL: &str = "context_window";
/// The label for the thread's sandbox policy, such as `workspaceWrite`.
pub const SANDBOX_LABEL: &str = "sandbox";
/// The label for the thread's approval policy, such as `on-request`.
pub const APPROVAL_LABEL: &str = "approval";
/// Marks a seat whose last turn failed, so the `idle` after it keeps it failed.
const TURN_FAILED_LABEL: &str = "turn_failed";
/// The notification that carries the account's rate-limit windows (#715).
pub const RATE_LIMITS_UPDATED: &str = "account/rateLimits/updated";
/// The `codexErrorInfo` codes of a turn stopped by the account's limits, as T3 Code classes them.
const LIMIT_CODES: [&str; 2] = ["usageLimitExceeded", "rateLimitExceeded"];

/// [`WAIT_LABEL`]'s value while Codex waits on an approval.
pub const WAITS_ON_APPROVAL: &str = "approval";
/// [`WAIT_LABEL`]'s value while Codex waits on the user's answer.
pub const WAITS_ON_INPUT: &str = "input";

/// A thread, as `thread/started`, `thread/read` and `thread/resume` give it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    /// Its id.
    pub id: String,
    /// The thread that spawned it, for a subagent's.
    #[serde(default)]
    pub parent_thread_id: Option<String>,
    /// Whether it is a side thread the TUI makes and drops, for a title or a recap.
    #[serde(default)]
    pub ephemeral: bool,
    /// Who started it: `user`, or Codex itself (`system`, `thread_title`).
    #[serde(default, rename = "threadSource")]
    pub started_by: Option<String>,
    /// When it was made, in seconds since the epoch.
    #[serde(default)]
    pub created_at: i64,
    /// The folder it works in, which names its project in a push (#752).
    #[serde(default)]
    pub cwd: Option<String>,
    /// Its status.
    pub status: ThreadStatus,
}

impl Thread {
    /// Whether this is a thread the user works in: not a side thread, no subagent's, and started
    /// by the user or by an older Codex that does not say.
    #[must_use]
    pub fn is_lead(&self) -> bool {
        !self.ephemeral
            && self.parent_thread_id.is_none()
            && self
                .started_by
                .as_deref()
                .is_none_or(|source| source == "user")
    }
}

/// A thread's status.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ThreadStatus {
    /// No longer loaded in the server.
    NotLoaded,
    /// Loaded, with no turn running.
    Idle,
    /// Stopped on an error of the server's.
    SystemError,
    /// A turn runs.
    Active {
        /// What the turn waits on: `waitingOnApproval`, `waitingOnUserInput`.
        #[serde(default, rename = "activeFlags")]
        active_flags: Vec<String>,
    },
    /// A status this Marley does not know.
    #[serde(other)]
    Unknown,
}

/// A turn, as `turn/completed` gives it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Turn {
    /// Its id, which its `turn/started` named (#753).
    #[serde(default)]
    pub id: String,
    /// `completed`, `interrupted`, `failed` or `inProgress`.
    pub status: String,
    /// What went wrong, for a failed turn.
    #[serde(default)]
    pub error: Option<TurnError>,
}

/// A failed turn's error.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TurnError {
    /// Its message.
    pub message: String,
    /// Codex's code for it (`codexErrorInfo`): a word such as `usageLimitExceeded`, or for an
    /// HTTP failure the name of its kind, such as `httpConnectionFailed` (#715).
    #[serde(default, rename = "codexErrorInfo", deserialize_with = "error_code")]
    pub code: Option<String>,
}

impl TurnError {
    /// Whether the account's usage or rate limit stopped the turn (#715).
    #[must_use]
    pub fn is_limit(&self) -> bool {
        self.code
            .as_deref()
            .is_some_and(|code| LIMIT_CODES.contains(&code))
    }
}

/// `codexErrorInfo`'s word, or the one key of its object form.
fn error_code<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(match Value::deserialize(deserializer)? {
        Value::String(code) => Some(code),
        Value::Object(object) if object.len() == 1 => object.keys().next().cloned(),
        _ => None,
    })
}

/// One window of the account's `codex` rate-limit bucket, as Codex sends it (#715).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitWindow {
    /// How much of it is used, in percent; 100 or more is full.
    pub used_percent: i64,
    /// How long it lasts, in minutes, when Codex says.
    #[serde(default)]
    pub window_duration_mins: Option<i64>,
    /// When it resets, in seconds since the epoch, when Codex says.
    #[serde(default)]
    pub resets_at: Option<i64>,
}

/// The account's `codex` bucket: its primary and secondary windows (#715).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
pub struct RateLimits {
    /// The shorter window, usually five hours.
    #[serde(default)]
    pub primary: Option<RateLimitWindow>,
    /// The longer window, usually a week.
    #[serde(default)]
    pub secondary: Option<RateLimitWindow>,
}

impl RateLimits {
    /// The `codex` bucket's windows an `account/rateLimits/updated` carries; `None` for another
    /// bucket, such as a model's own, which Marley passes over.
    ///
    /// # Errors
    ///
    /// When the parameters do not have the shape Codex generates.
    pub fn decode(params: &Value) -> Result<Option<Self>, String> {
        let snapshot = params
            .get("rateLimits")
            .ok_or_else(|| format!("{RATE_LIMITS_UPDATED} has no rateLimits"))?;
        match snapshot.get("limitId") {
            None | Some(Value::Null) => {}
            Some(Value::String(id)) if id == "codex" => {}
            Some(Value::String(_)) => return Ok(None),
            Some(_) => return Err(format!("{RATE_LIMITS_UPDATED}: limitId is not a string")),
        }
        serde_json::from_value(snapshot.clone())
            .map(Some)
            .map_err(|error| format!("{RATE_LIMITS_UPDATED}: {error}"))
    }

    /// Takes a sparse update in: each window it carries replaces the last, keeping a duration or a
    /// reset the update leaves out.
    pub fn merge(&mut self, update: &Self) {
        for (slot, window) in [
            (&mut self.primary, update.primary),
            (&mut self.secondary, update.secondary),
        ] {
            if let Some(mut window) = window {
                if let Some(last) = slot {
                    window.window_duration_mins =
                        window.window_duration_mins.or(last.window_duration_mins);
                    window.resets_at = window.resets_at.or(last.resets_at);
                }
                *slot = Some(window);
            }
        }
    }

    /// Each window with its kind: `five_hour` or `seven_day` by its length, as Claude Code names
    /// them, otherwise `primary` or `secondary`, as rustal-harness names the same windows.
    fn windows(&self) -> impl Iterator<Item = (&'static str, RateLimitWindow)> {
        [("primary", self.primary), ("secondary", self.secondary)]
            .into_iter()
            .filter_map(|(slot, window)| {
                let window = window?;
                let kind = match window.window_duration_mins {
                    Some(300) => "five_hour",
                    Some(10_080) => "seven_day",
                    _ => slot,
                };
                Some((kind, window))
            })
    }
}

/// A thread's approval and sandbox policies, in Codex's words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policies {
    /// `untrusted`, `on-request`, `never` or `granular`.
    pub approval: String,
    /// `dangerFullAccess`, `workspaceWrite`, `readOnly` or `externalSandbox`.
    pub sandbox: String,
}

impl Policies {
    /// The policies from an approval policy (a word, or a `granular` object) and a sandbox policy
    /// (an object tagged by `type`).
    fn of(approval: &Value, sandbox: &Value) -> Option<Self> {
        let approval = match approval {
            Value::String(word) => word.clone(),
            Value::Object(object) if object.contains_key("granular") => "granular".to_string(),
            _ => return None,
        };
        let sandbox = sandbox.get("type")?.as_str()?.to_string();
        Some(Self { approval, sandbox })
    }
}

/// A message from the App Server that Marley follows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notification {
    /// `thread/started`: a thread was made.
    ThreadStarted(Thread),
    /// `thread/status/changed`.
    StatusChanged {
        /// The thread.
        thread: String,
        /// Its status now.
        status: ThreadStatus,
    },
    /// `turn/started`.
    TurnStarted {
        /// The thread.
        thread: String,
        /// The turn's id, which its `turn/completed` names again (#753).
        turn: String,
    },
    /// `turn/completed`.
    TurnCompleted {
        /// The thread.
        thread: String,
        /// The turn as it ended.
        turn: Turn,
    },
    /// `thread/tokenUsage/updated`.
    TokenUsage {
        /// The thread.
        thread: String,
        /// The thread's total.
        total: u64,
        /// The model's context window, when known.
        context_window: Option<u64>,
    },
    /// `thread/settings/updated`, which Codex sends to clients of its experimental API.
    SettingsUpdated {
        /// The thread.
        thread: String,
        /// Its policies now.
        policies: Policies,
    },
    /// `thread/closed`.
    ThreadClosed {
        /// The thread.
        thread: String,
    },
}

impl Notification {
    /// The notification `method` carries with `params`; `None` for a method Marley does not follow.
    ///
    /// # Errors
    ///
    /// When a followed method's parameters do not have the shape Codex generates.
    pub fn decode(method: &str, params: &Value) -> Result<Option<Self>, String> {
        let thread = || {
            params
                .get("threadId")
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| format!("{method} names no threadId"))
        };
        let parsed = |key: &str| {
            params
                .get(key)
                .cloned()
                .ok_or_else(|| format!("{method} has no {key}"))
        };
        let notification = match method {
            "thread/started" => Self::ThreadStarted(
                serde_json::from_value(parsed("thread")?)
                    .map_err(|error| format!("{method}: {error}"))?,
            ),
            "thread/status/changed" => Self::StatusChanged {
                thread: thread()?,
                status: serde_json::from_value(parsed("status")?)
                    .map_err(|error| format!("{method}: {error}"))?,
            },
            "turn/started" => Self::TurnStarted {
                thread: thread()?,
                turn: parsed("turn")?
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            },
            "turn/completed" => Self::TurnCompleted {
                thread: thread()?,
                turn: serde_json::from_value(parsed("turn")?)
                    .map_err(|error| format!("{method}: {error}"))?,
            },
            "thread/tokenUsage/updated" => {
                let usage = parsed("tokenUsage")?;
                let total = usage
                    .pointer("/total/totalTokens")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| format!("{method} has no total.totalTokens"))?;
                Self::TokenUsage {
                    thread: thread()?,
                    total,
                    context_window: usage.get("modelContextWindow").and_then(Value::as_u64),
                }
            }
            "thread/settings/updated" => {
                let settings = parsed("threadSettings")?;
                let policies = Policies::of(
                    settings.get("approvalPolicy").unwrap_or(&Value::Null),
                    settings.get("sandboxPolicy").unwrap_or(&Value::Null),
                )
                .ok_or_else(|| format!("{method} has no approval or sandbox policy"))?;
                Self::SettingsUpdated {
                    thread: thread()?,
                    policies,
                }
            }
            "thread/closed" => Self::ThreadClosed { thread: thread()? },
            _ => return Ok(None),
        };
        Ok(Some(notification))
    }

    /// The thread the notification is about.
    #[must_use]
    pub fn thread(&self) -> &str {
        match self {
            Self::ThreadStarted(thread) => &thread.id,
            Self::StatusChanged { thread, .. }
            | Self::TurnStarted { thread, .. }
            | Self::TurnCompleted { thread, .. }
            | Self::TokenUsage { thread, .. }
            | Self::SettingsUpdated { thread, .. }
            | Self::ThreadClosed { thread } => thread,
        }
    }
}

/// The thread a `thread/read` answer gives.
///
/// # Errors
///
/// When the answer holds no thread in Codex's shape.
pub fn read_answer(result: &Value) -> Result<Thread, String> {
    thread_in(result, "thread/read")
}

/// The thread in `method`'s answer.
fn thread_in(result: &Value, method: &str) -> Result<Thread, String> {
    let thread = result
        .get("thread")
        .cloned()
        .ok_or_else(|| format!("{method} gave no thread"))?;
    serde_json::from_value(thread).map_err(|error| format!("{method}: {error}"))
}

/// The thread and its policies a `thread/resume` answer gives.
///
/// # Errors
///
/// When the answer holds no thread, or no approval or sandbox policy, in Codex's shape.
pub fn resume_answer(result: &Value) -> Result<(Thread, Policies), String> {
    let thread = thread_in(result, "thread/resume")?;
    let policies = Policies::of(
        result.get("approvalPolicy").unwrap_or(&Value::Null),
        result.get("sandbox").unwrap_or(&Value::Null),
    )
    .ok_or("thread/resume gave no approval or sandbox policy")?;
    Ok((thread, policies))
}

/// What moves a Codex seat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input<'a> {
    /// Marley follows this thread now, in the status it has.
    Lead(&'a Thread),
    /// A notification about the lead.
    Notification(&'a Notification),
    /// The lead's policies, from Marley's `thread/resume`.
    Policies(&'a Policies),
    /// The account's rate-limit windows, merged from every update so far (#715).
    RateLimits(&'a RateLimits),
    /// The App Server stopped, with its last error line when it printed one.
    ServerStopped(Option<&'a str>),
}

/// The fleet events that move Codex seat `seat` for `input`, from the seat as it was
/// (`previous`), at `now_ms`. An input that changes nothing gives none.
#[must_use]
pub fn fold(
    seat: &str,
    previous: Option<&Session>,
    input: &Input<'_>,
    now_ms: u64,
) -> Vec<SessionEvent> {
    let mut moving = Moving::from_seat(previous);
    let changed = match input {
        Input::Lead(thread) => {
            if moving.labels.get(THREAD_LABEL) != Some(&thread.id) {
                moving = Moving::from_seat(None);
                moving.set(THREAD_LABEL, &thread.id);
            }
            if let Some(cwd) = &thread.cwd {
                moving.set(claude_events::CWD_LABEL, cwd);
            }
            moving.take_status(&thread.status)
        }
        Input::Notification(notification) => match notification {
            Notification::StatusChanged { status, .. } => moving.take_status(status),
            Notification::TurnStarted { .. } => {
                moving.clear(&[
                    TURN_FAILED_LABEL,
                    ERROR_LABEL,
                    LIMITED_LABEL,
                    LIMIT_RESETS_AT_LABEL,
                ]);
                moving.state = State::Working;
                Step::Moved
            }
            Notification::TurnCompleted { turn, .. } if turn.status == "failed" => {
                let message = turn
                    .error
                    .as_ref()
                    .map_or("the turn failed", |error| error.message.as_str());
                moving.set(ERROR_LABEL, &one_line(message));
                moving.set(TURN_FAILED_LABEL, "true");
                if turn.error.as_ref().is_some_and(TurnError::is_limit) {
                    moving.set(LIMITED_LABEL, &turn.id);
                    match limit_reset(&moving.labels, now_ms) {
                        Some(reset) => moving.set(LIMIT_RESETS_AT_LABEL, &reset.to_string()),
                        None => moving.clear(&[LIMIT_RESETS_AT_LABEL]),
                    }
                }
                moving.state = State::Error;
                moving.clear(&[WAIT_LABEL]);
                Step::Moved
            }
            Notification::TurnCompleted { .. } | Notification::ThreadStarted(_) => Step::Still,
            Notification::TokenUsage {
                total,
                context_window,
                ..
            } => {
                moving.set(TOKENS_LABEL, &total.to_string());
                match context_window {
                    Some(window) => moving.set(CONTEXT_WINDOW_LABEL, &window.to_string()),
                    None => moving.clear(&[CONTEXT_WINDOW_LABEL]),
                }
                Step::Moved
            }
            Notification::SettingsUpdated { policies, .. } => moving.take_policies(policies),
            Notification::ThreadClosed { .. } => Step::Ended,
        },
        Input::Policies(policies) => moving.take_policies(policies),
        Input::RateLimits(limits) => moving.take_rate_limits(limits),
        Input::ServerStopped(last) => {
            let error = last.map_or_else(
                || "Codex's App Server stopped".to_string(),
                |last| format!("Codex's App Server stopped: {}", one_line(last)),
            );
            moving.set(ERROR_LABEL, &error);
            moving.state = State::Error;
            moving.clear(&[WAIT_LABEL]);
            Step::Moved
        }
    };
    match changed {
        Step::Still => Vec::new(),
        Step::Ended => vec![SessionEvent::Ended {
            id: seat.to_string(),
            ts_ms: now_ms,
        }],
        Step::Moved => moving.into_events(seat, now_ms),
    }
}

/// The words a Codex row's line starts with.
///
/// `working`, `waiting on approval`, `waiting on input`, `idle`, `failed`, `limited` for a turn
/// the usage limit stopped (#715), or `no update in N m` for a working seat whose last event is
/// `no_update_after_ms` old or more at `now_ms` (#547; 0 never marks it).
#[must_use]
pub fn seat_words(seat: &Session, now_ms: u64, no_update_after_ms: u64) -> String {
    if limit(seat).is_some() {
        return "limited".to_string();
    }
    match seat.state {
        State::Working
            if no_update_after_ms > 0
                && marley_fleet::is_stale(seat, now_ms, no_update_after_ms) =>
        {
            let minutes = now_ms.saturating_sub(seat.last_event_ms) / 60_000;
            format!("no update in {minutes} m")
        }
        State::Waiting => match seat.labels.get(WAIT_LABEL).map(String::as_str) {
            Some(WAITS_ON_APPROVAL) => "waiting on approval".to_string(),
            Some(WAITS_ON_INPUT) => "waiting on input".to_string(),
            _ => AgentStatus::Waiting.label().to_string(),
        },
        state => claude_events::seat_status(state).label().to_string(),
    }
}

/// The thread's total token use, once Codex reported it.
#[must_use]
pub fn tokens(seat: &Session) -> Option<u64> {
    seat.labels.get(TOKENS_LABEL)?.parse().ok()
}

/// A Codex row's third line: what the seat waits on, or the error it failed with.
#[must_use]
pub fn seat_activity(seat: &Session) -> Option<String> {
    match seat.state {
        State::Waiting => seat
            .question
            .as_ref()
            .map(|question| question.prompt.clone()),
        State::Error => seat.labels.get(ERROR_LABEL).cloned(),
        State::Starting | State::Working | State::Idle | State::Done => None,
    }
}

/// Whether `seat` is a Codex seat.
#[must_use]
pub fn is_codex(seat: &Session) -> bool {
    seat.labels
        .get(AGENT_LABEL)
        .is_some_and(|agent| agent == AGENT)
}

/// What an input did to the seat.
enum Step {
    Still,
    Moved,
    Ended,
}

/// A seat while an input moves it.
struct Moving {
    labels: BTreeMap<String, String>,
    state: State,
}

impl Moving {
    fn from_seat(previous: Option<&Session>) -> Self {
        let mut moving = previous.map_or_else(
            || Self {
                labels: BTreeMap::new(),
                state: State::Starting,
            },
            |session| Self {
                labels: session.labels.clone(),
                state: session.state,
            },
        );
        moving.set(AGENT_LABEL, AGENT);
        moving
    }

    fn set(&mut self, key: &str, value: &str) {
        let _previous = self.labels.insert(key.to_string(), value.to_string());
    }

    fn clear(&mut self, keys: &[&str]) {
        for key in keys {
            let _previous = self.labels.remove(*key);
        }
    }

    fn take_status(&mut self, status: &ThreadStatus) -> Step {
        let (state, wait) = match status {
            ThreadStatus::NotLoaded => return Step::Ended,
            ThreadStatus::Unknown => return Step::Still,
            // A failed turn stays failed through the `idle` after it, until the next turn.
            ThreadStatus::Idle if self.labels.contains_key(TURN_FAILED_LABEL) => {
                (State::Error, None)
            }
            ThreadStatus::Idle => (State::Idle, None),
            ThreadStatus::SystemError => {
                self.set(
                    ERROR_LABEL,
                    "Codex's thread stopped on an error of its App Server",
                );
                (State::Error, None)
            }
            ThreadStatus::Active { active_flags } => {
                let flag = |name: &str| active_flags.iter().any(|flag| flag == name);
                if flag("waitingOnApproval") {
                    (State::Waiting, Some(WAITS_ON_APPROVAL))
                } else if flag("waitingOnUserInput") {
                    (State::Waiting, Some(WAITS_ON_INPUT))
                } else {
                    (State::Working, None)
                }
            }
        };
        self.state = state;
        match wait {
            Some(wait) => self.set(WAIT_LABEL, wait),
            None => self.clear(&[WAIT_LABEL]),
        }
        Step::Moved
    }

    /// The seat's `quota.` labels made the windows of `limits`; still when nothing changed.
    fn take_rate_limits(&mut self, limits: &RateLimits) -> Step {
        let before = self.labels.clone();
        self.labels.retain(|key, _| !key.starts_with(QUOTA_PREFIX));
        for (kind, window) in limits.windows() {
            self.set(
                &format!("{QUOTA_PREFIX}{kind}.percent_used"),
                &window.used_percent.max(0).to_string(),
            );
            let reset = window
                .resets_at
                .and_then(|seconds| u64::try_from(seconds).ok())
                .map(|seconds| seconds.saturating_mul(1000));
            if let Some(reset) = reset {
                self.set(
                    &format!("{QUOTA_PREFIX}{kind}.resets_at_ms"),
                    &reset.to_string(),
                );
            }
        }
        if self.labels == before {
            Step::Still
        } else {
            Step::Moved
        }
    }

    fn take_policies(&mut self, policies: &Policies) -> Step {
        self.set(APPROVAL_LABEL, &policies.approval);
        self.set(SANDBOX_LABEL, &policies.sandbox);
        Step::Moved
    }

    fn into_events(self, seat: &str, now_ms: u64) -> Vec<SessionEvent> {
        let question = (self.state == State::Waiting).then(|| Question {
            prompt: match self.labels.get(WAIT_LABEL).map(String::as_str) {
                Some(WAITS_ON_APPROVAL) => "Waits on an approval".to_string(),
                Some(WAITS_ON_INPUT) => "Waits on an answer".to_string(),
                _ => "Waits for you".to_string(),
            },
            options: Vec::new(),
            context_refs: Vec::new(),
        });
        let mut events = vec![SessionEvent::Upsert {
            id: seat.to_string(),
            ts_ms: now_ms,
            title: "Codex".to_string(),
            state: self.state,
            labels: self.labels,
            transport: Some(Transport::Local),
        }];
        // An upsert keeps a waiting seat's question, so a new question is raised on its own.
        if let Some(question) = question {
            events.push(SessionEvent::QuestionRaised {
                id: seat.to_string(),
                ts_ms: now_ms,
                question,
            });
        }
        events
    }
}

/// `text` on one line, its runs of whitespace one space each.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A decision on a request Codex's App Server asks its clients to approve (#651), in the
/// protocol's words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// `accept`: allowed once, or for the turn.
    Accept,
    /// `acceptForSession`: allowed, and the same for the rest of the session.
    AcceptForSession,
    /// `decline`: denied; the turn goes on.
    Decline,
    /// `cancel`: denied and the turn interrupted, or the request cancelled.
    Cancel,
    /// An MCP server's approval accepted for good, `_meta.persist` `always` (#752).
    AcceptAlways,
    /// The option at this index of a question's options (#752).
    Option(u8),
}

impl Decision {
    /// The protocol's word for it: the decision's, or `accept` for an approval kept for good and
    /// `answer` for a question's option, which their responses spell out.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Accept | Self::AcceptAlways => "accept",
            Self::AcceptForSession => "acceptForSession",
            Self::Decline => "decline",
            Self::Cancel => "cancel",
            Self::Option(_) => "answer",
        }
    }

    fn of_word(word: &str) -> Option<Self> {
        [
            Self::Accept,
            Self::AcceptForSession,
            Self::Decline,
            Self::Cancel,
        ]
        .into_iter()
        .find(|decision| decision.word() == word)
    }
}

/// One question of `item/tool/requestUserInput` (#752).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asked {
    /// The id its answer is keyed by.
    pub id: String,
    /// Its short heading.
    pub header: String,
    /// The question.
    pub question: String,
    /// Whether its answer is a secret, which Codex's TUI hides as it is typed.
    pub secret: bool,
    /// Its options' labels, in order.
    pub options: Vec<String>,
}

/// How long an accepted MCP server approval lasts, as `_meta.persist` offers it (#752).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Persist {
    /// For the rest of the session.
    Session,
    /// For good, in Codex's own configuration.
    Always,
}

impl Persist {
    /// The protocol's word for it.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::Always => "always",
        }
    }
}

/// An MCP server's request that asks only for an approval: a form with nothing to fill in, as
/// Codex's TUI reads one (#752).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Approval {
    /// Whether it approves a tool call (`_meta.codex_approval_kind` `mcp_tool_call`).
    pub tool: bool,
    /// How long an accept may last beyond this once, as `_meta.persist` offers.
    pub persist: Vec<Persist>,
}

/// What a request asks, as the inbox reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestKind {
    /// `item/commandExecution/requestApproval`.
    Command {
        /// The command, when the server sent it.
        command: Option<String>,
        /// Where it runs.
        cwd: Option<String>,
        /// The simple decisions of the request's `availableDecisions`, in its order; `None` when
        /// it sent no list.
        offered: Option<Vec<Decision>>,
        /// The extra permissions it asks for, in words.
        extra: Option<String>,
    },
    /// `item/fileChange/requestApproval`: its item, whose `item/started` names the files.
    FileChange {
        /// The `fileChange` item.
        item: String,
        /// Why, when the server says.
        reason: Option<String>,
        /// Whether it asks for writes under a whole folder for the session.
        grant_root: bool,
    },
    /// `item/permissions/requestApproval`.
    Permissions {
        /// What it asks for, in words: `write /tmp/out, network`; empty when Marley reads none of
        /// it.
        asks: String,
        /// The paths it names.
        paths: Vec<String>,
        /// Why, when the server says.
        reason: Option<String>,
    },
    /// `mcpServer/elicitation/request`.
    Elicitation {
        /// The MCP server.
        server: String,
        /// What it asks.
        message: String,
        /// The approval it asks for, when it asks for nothing more (#752); a form with fields, a
        /// URL or a user's verification has none.
        approval: Option<Approval>,
    },
    /// `item/tool/requestUserInput`: Codex's own questions (#752).
    Question {
        /// The questions, in order.
        questions: Vec<Asked>,
    },
}

/// A request Codex's App Server asks its clients to approve (#651), as it came.
///
/// Its JSON-RPC id (a number or a string), its thread, what it asks, and its params, which a
/// click must still match when its answer goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// The JSON-RPC id the answer carries back.
    pub id: Value,
    /// Its thread.
    pub thread: String,
    /// What it asks.
    pub kind: RequestKind,
    /// The params as sent.
    pub params: Value,
}

impl Request {
    /// The request `method` with `id` and `params` carries; `None` for a method the inbox does not
    /// list.
    ///
    /// # Errors
    ///
    /// When a listed method's params do not have the shape Codex generates.
    pub fn decode(id: &Value, method: &str, params: &Value) -> Result<Option<Self>, String> {
        let text = |key: &str| params.get(key).and_then(Value::as_str).map(str::to_string);
        let thread = text("threadId").ok_or_else(|| format!("{method} names no threadId"))?;
        let kind = match method {
            "item/commandExecution/requestApproval" => RequestKind::Command {
                command: text("command").map(|command| one_line(&command)),
                cwd: text("cwd"),
                offered: params
                    .get("availableDecisions")
                    .and_then(Value::as_array)
                    .map(|decisions| {
                        decisions
                            .iter()
                            .filter_map(Value::as_str)
                            .filter_map(Decision::of_word)
                            .collect()
                    }),
                extra: params
                    .get("additionalPermissions")
                    .map(permission_words)
                    .map(|(words, _)| words)
                    .filter(|words| !words.is_empty()),
            },
            "item/fileChange/requestApproval" => RequestKind::FileChange {
                item: text("itemId").ok_or_else(|| format!("{method} names no itemId"))?,
                reason: text("reason"),
                grant_root: params.get("grantRoot").is_some_and(|root| !root.is_null()),
            },
            "item/permissions/requestApproval" => {
                let (asks, paths) = params
                    .get("permissions")
                    .map(permission_words)
                    .unwrap_or_default();
                RequestKind::Permissions {
                    asks,
                    paths,
                    reason: text("reason"),
                }
            }
            "mcpServer/elicitation/request" => RequestKind::Elicitation {
                server: text("serverName").unwrap_or_else(|| "An MCP server".to_string()),
                message: text("message")
                    .map(|message| one_line(&message))
                    .unwrap_or_default(),
                approval: approval_of(params),
            },
            "item/tool/requestUserInput" => RequestKind::Question {
                questions: params
                    .get("questions")
                    .and_then(Value::as_array)
                    .map(|questions| questions.iter().filter_map(asked_of).collect())
                    .unwrap_or_default(),
            },
            _ => return Ok(None),
        };
        Ok(Some(Self {
            id: id.clone(),
            thread,
            kind,
            params: params.clone(),
        }))
    }

    /// The decisions the inbox offers in place, in order: only those the request offers, and an
    /// allow only beside what it allows (`files`: a file change's paths from its item, when
    /// known).
    #[must_use]
    pub fn decisions(&self, files: Option<&[String]>) -> Vec<Decision> {
        match &self.kind {
            RequestKind::Command {
                command, offered, ..
            } => offered
                .iter()
                .flatten()
                .copied()
                .filter(|decision| {
                    command.is_some()
                        || !matches!(decision, Decision::Accept | Decision::AcceptForSession)
                })
                .collect(),
            RequestKind::FileChange { grant_root, .. } => {
                let known = files.is_some_and(|files| !files.is_empty());
                let mut decisions = Vec::new();
                if known {
                    decisions.push(Decision::Accept);
                    if !grant_root {
                        decisions.push(Decision::AcceptForSession);
                    }
                }
                decisions.extend([Decision::Decline, Decision::Cancel]);
                decisions
            }
            RequestKind::Permissions { asks, .. } if asks.is_empty() => vec![Decision::Decline],
            RequestKind::Permissions { .. } => vec![
                Decision::Accept,
                Decision::AcceptForSession,
                Decision::Decline,
            ],
            RequestKind::Elicitation {
                approval: Some(approval),
                ..
            } => {
                let mut decisions = vec![Decision::Accept];
                if approval.persist.contains(&Persist::Session) {
                    decisions.push(Decision::AcceptForSession);
                }
                if approval.persist.contains(&Persist::Always) {
                    decisions.push(Decision::AcceptAlways);
                }
                if !approval.tool {
                    decisions.push(Decision::Decline);
                }
                decisions.push(Decision::Cancel);
                decisions
            }
            RequestKind::Elicitation { approval: None, .. } => {
                vec![Decision::Decline, Decision::Cancel]
            }
            // One question with options, not secret, is answered by an option's label, as the TUI
            // answers it; any other waits in the terminal.
            RequestKind::Question { questions } => match questions.as_slice() {
                [asked] if !asked.secret => (0..asked.options.len())
                    .filter_map(|index| u8::try_from(index).ok())
                    .map(Decision::Option)
                    .collect(),
                _ => Vec::new(),
            },
        }
    }

    /// The label of the option `decision` picks, for a question's button and its `Sent:` line.
    #[must_use]
    pub fn option_label(&self, decision: Decision) -> Option<&str> {
        let (RequestKind::Question { questions }, Decision::Option(index)) = (&self.kind, decision)
        else {
            return None;
        };
        questions
            .first()?
            .options
            .get(usize::from(index))
            .map(String::as_str)
    }

    /// The response that carries `decision` for this request; `None` for a decision it does not
    /// take. Permissions are granted as the request asked them, for the turn or the session, and
    /// denied with an empty grant, as Codex's TUI does.
    #[must_use]
    pub fn response(&self, decision: Decision) -> Option<Value> {
        match &self.kind {
            RequestKind::Command { .. } | RequestKind::FileChange { .. } => match decision {
                Decision::AcceptAlways | Decision::Option(_) => None,
                _ => Some(serde_json::json!({ "decision": decision.word() })),
            },
            RequestKind::Permissions { .. } => {
                let asked = self
                    .params
                    .get("permissions")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({}));
                match decision {
                    Decision::Accept => {
                        Some(serde_json::json!({ "permissions": asked, "scope": "turn" }))
                    }
                    Decision::AcceptForSession => {
                        Some(serde_json::json!({ "permissions": asked, "scope": "session" }))
                    }
                    Decision::Decline => {
                        Some(serde_json::json!({ "permissions": {}, "scope": "turn" }))
                    }
                    Decision::Cancel | Decision::AcceptAlways | Decision::Option(_) => None,
                }
            }
            RequestKind::Elicitation { approval, .. } => {
                let persist = |kept: Persist| {
                    approval
                        .as_ref()
                        .filter(|approval| approval.persist.contains(&kept))
                        .map(|_| {
                            serde_json::json!({
                                "action": "accept",
                                "_meta": { "persist": kept.word() },
                            })
                        })
                };
                match decision {
                    Decision::Decline | Decision::Cancel => {
                        Some(serde_json::json!({ "action": decision.word() }))
                    }
                    Decision::Accept => approval
                        .as_ref()
                        .map(|_| serde_json::json!({ "action": "accept" })),
                    Decision::AcceptForSession => persist(Persist::Session),
                    Decision::AcceptAlways => persist(Persist::Always),
                    Decision::Option(_) => None,
                }
            }
            RequestKind::Question { questions } => {
                let asked = questions.first()?;
                let label = self.option_label(decision)?;
                let answers: serde_json::Map<String, Value> =
                    std::iter::once((asked.id.clone(), serde_json::json!({ "answers": [label] })))
                        .collect();
                Some(serde_json::json!({ "answers": answers }))
            }
        }
    }

    /// What the request asks, on one line, for the inbox: the command and where, the files to
    /// edit (`files`, when known), the permissions, or the MCP server's message.
    #[must_use]
    pub fn ask(&self, files: Option<&[String]>) -> String {
        let with_reason = |text: String, reason: &Option<String>| match reason {
            Some(reason) if !reason.trim().is_empty() => format!("{text} · {}", one_line(reason)),
            _ => text,
        };
        match &self.kind {
            RequestKind::Command {
                command,
                cwd,
                extra,
                ..
            } => {
                let mut text = command.clone().unwrap_or_else(|| "A command".to_string());
                if let Some(cwd) = cwd {
                    text.push_str(" · in ");
                    text.push_str(cwd);
                }
                if let Some(extra) = extra {
                    text.push_str(" · asks ");
                    text.push_str(extra);
                }
                text
            }
            RequestKind::FileChange { reason, .. } => {
                let text = match files.filter(|files| !files.is_empty()) {
                    Some([file]) => format!("Edit 1 file: {file}"),
                    Some(files) => format!("Edit {} files: {}", files.len(), files.join(", ")),
                    None => "Edit files".to_string(),
                };
                with_reason(text, reason)
            }
            RequestKind::Permissions { asks, reason, .. } => {
                let asks = if asks.is_empty() {
                    "more access"
                } else {
                    asks.as_str()
                };
                with_reason(format!("Permissions: {asks}"), reason)
            }
            RequestKind::Elicitation {
                server, message, ..
            } => format!("{server}: {message}"),
            RequestKind::Question { questions } => {
                let Some(first) = questions.first() else {
                    return "A question".to_string();
                };
                let text = if first.header.trim().is_empty() {
                    one_line(&first.question)
                } else {
                    format!("{}: {}", one_line(&first.header), one_line(&first.question))
                };
                match questions.len() {
                    1 => text,
                    count => format!("{text} · and {} more", count - 1),
                }
            }
        }
    }
}

/// The approval an elicitation asks for, when it is a form with nothing to fill in (#752).
fn approval_of(params: &Value) -> Option<Approval> {
    let mode = params.get("mode").and_then(Value::as_str).unwrap_or("form");
    let schema = params.get("requestedSchema");
    let fields = schema
        .and_then(|schema| schema.get("properties"))
        .and_then(Value::as_object)
        .is_some_and(|properties| !properties.is_empty());
    if mode != "form" || fields {
        return None;
    }
    let meta = params.get("_meta");
    let tool = meta
        .and_then(|meta| meta.get("codex_approval_kind"))
        .and_then(Value::as_str)
        == Some("mcp_tool_call");
    let offered: Vec<&str> = match meta.and_then(|meta| meta.get("persist")) {
        Some(Value::String(word)) => vec![word.as_str()],
        Some(Value::Array(words)) => words.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    };
    let persist = [Persist::Session, Persist::Always]
        .into_iter()
        .filter(|kept| offered.contains(&kept.word()))
        .collect();
    Some(Approval { tool, persist })
}

/// One question of `item/tool/requestUserInput`, when it names its id.
fn asked_of(question: &Value) -> Option<Asked> {
    let text = |key: &str| {
        question
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    Some(Asked {
        id: question.get("id").and_then(Value::as_str)?.to_string(),
        header: text("header"),
        question: text("question"),
        secret: question
            .get("isSecret")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        options: question
            .get("options")
            .and_then(Value::as_array)
            .map(|options| {
                options
                    .iter()
                    .filter_map(|option| option.get("label").and_then(Value::as_str))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
    })
}

/// A permission profile in words, `read a, write b, network`, and the paths it names.
fn permission_words(profile: &Value) -> (String, Vec<String>) {
    let mut words = Vec::new();
    let mut paths = Vec::new();
    let file_system = profile.get("fileSystem");
    for access in ["read", "write"] {
        let named: Vec<String> = file_system
            .and_then(|file_system| file_system.get(access))
            .and_then(Value::as_array)
            .map(|list| {
                list.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        if !named.is_empty() {
            words.push(format!("{access} {}", named.join(", ")));
            paths.extend(named);
        }
    }
    let entries = file_system
        .and_then(|file_system| file_system.get("entries"))
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    if entries > 0 {
        words.push(format!("{entries} file system entries"));
    }
    if profile
        .pointer("/network/enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        words.push("network".to_string());
    }
    (words.join(", "), paths)
}

/// The thread and the request a `serverRequest/resolved` names.
#[must_use]
pub fn resolved(params: &Value) -> Option<(String, Value)> {
    let thread = params.get("threadId")?.as_str()?.to_string();
    let request = params.get("requestId")?.clone();
    Some((thread, request))
}

/// The thread, the turn and the text of an `item/started` whose item is the user's message, the
/// prompt a turn answers (#753): its text inputs, on one line.
#[must_use]
pub fn user_message_started(params: &Value) -> Option<(String, String, String)> {
    let item = params.get("item")?;
    if item.get("type")?.as_str()? != "userMessage" {
        return None;
    }
    let text = item
        .get("content")?
        .as_array()?
        .iter()
        .filter(|input| input.get("type").and_then(Value::as_str) == Some("text"))
        .filter_map(|input| input.get("text")?.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    Some((
        params.get("threadId")?.as_str()?.to_string(),
        params
            .get("turnId")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        one_line(&text),
    ))
}

/// The thread, the item and the paths of an `item/started` whose item is a file change.
#[must_use]
pub fn file_change_started(params: &Value) -> Option<(String, String, Vec<String>)> {
    let item = params.get("item")?;
    if item.get("type")?.as_str()? != "fileChange" {
        return None;
    }
    let paths = item
        .get("changes")?
        .as_array()?
        .iter()
        .filter_map(|change| change.get("path")?.as_str().map(str::to_string))
        .collect();
    Some((
        params.get("threadId")?.as_str()?.to_string(),
        item.get("id")?.as_str()?.to_string(),
        paths,
    ))
}
