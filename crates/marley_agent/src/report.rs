//! Agent reports (#652): the state an agent's plugin reports to the Marley terminal it runs in.
//!
//! The contract is rustal-harness's (its TICKET-099 and `AGENT_SEATS.md`), so one Claude Code
//! plugin reports to either host: the same fields, limits and refusal names, which the shared
//! plugin's mod sends through `$MARLEY_BIN report` with the arguments `rh report` takes. Marley
//! checks every rule here, not the program that carries the report, so the two cannot disagree.
//! The wire is one JSON line each way over Marley's agent socket: a [`Request`] in, an
//! [`answer_ok`] or [`answer_refused`] out.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The most parents walked from a reporter to its terminal's shell.
pub const MAX_ANCESTRY: usize = 32;
const MAX_SOURCE: usize = 64;
const MAX_ACTIVITY: usize = 120;
const MAX_SESSION_ID: usize = 512;
const MAX_PROMPT: usize = 2048;
const MAX_OPTIONS: usize = 16;
const MAX_OPTION: usize = 128;
const MAX_RESUME_ARGS: usize = 64;
const MAX_RESUME_ARGV_BYTES: usize = 8192;
const MAX_QUOTA_WINDOWS: usize = 8;
const MAX_QUOTA_KIND: usize = 32;
const MAX_PERCENT_USED: f64 = 10_000.0;

/// The states a report names, the fleet's six.
pub const STATES: [&str; 6] = ["starting", "working", "idle", "waiting", "error", "done"];

/// The label for the report's source, such as `mod:claude-code`.
pub const SOURCE_LABEL: &str = "source";
/// The label that says the seat's state was reported, `reported`.
pub const STATE_SOURCE_LABEL: &str = "state.source";
/// The label for the session id the report named.
pub const SESSION_LABEL: &str = "report.session_id";
/// The label for the report's percent done.
pub const PERCENT_LABEL: &str = "progress.percent";
/// The label for the report's one-line activity.
pub const ACTIVITY_LABEL: &str = "progress.activity";

/// What an agent reports of itself.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    /// Who reports: 1 to 64 of `a-z`, `0-9`, `:`, `.`, `_` and `-`.
    pub source: String,
    /// Higher than the last one accepted from the same source for the same terminal.
    pub seq: u64,
    /// One of [`STATES`].
    pub state: String,
    /// What the agent asks, only while `waiting`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub question: Option<Question>,
    /// How far it is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<Progress>,
    /// The agent's own session id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// The command that resumes the session, under herdr's checks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resume_argv: Option<Vec<String>>,
    /// The tokens the session used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    /// The account's quota windows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota: Option<Vec<QuotaWindow>>,
}

/// A question a waiting agent asks.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Question {
    /// 1 to 2,048 bytes.
    pub prompt: String,
    /// 1 to 16 distinct options of at most 128 characters.
    pub options: Vec<String>,
}

/// How far an agent is.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Progress {
    /// 0 to 100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub percent: Option<u8>,
    /// One line of at most 120 characters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity: Option<String>,
}

/// The tokens a session used.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Usage {
    /// The input tokens not read from the prompt cache, cache writes included.
    #[serde(rename = "input_tokens")]
    pub input: u64,
    /// The output tokens.
    #[serde(rename = "output_tokens")]
    pub output: u64,
    /// The input tokens read from the prompt cache.
    #[serde(rename = "cache_read_tokens")]
    pub cache_read: u64,
}

/// One of an account's quota windows.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct QuotaWindow {
    /// 1 to 32 of `a-z`, `0-9` and `_`, different from the others.
    pub kind: String,
    /// 0 to 10,000, since a spend limit can pass 100.
    pub percent_used: f64,
    /// When the window resets, in milliseconds since the epoch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resets_at_ms: Option<u64>,
}

/// Why Marley refused a report or a release, by the harness's names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The caller is in none of Marley's local terminals, or names another.
    Unknown,
    /// `seq` is not higher than the last one accepted from its source.
    Stale,
    /// Another source or process holds the terminal.
    Authority,
    /// A field is unknown or has the wrong type.
    Shape,
    /// The source breaks its rule.
    Source,
    /// The state is not one of the six.
    State,
    /// The question breaks its rule.
    Question,
    /// The progress breaks its rule.
    Progress,
    /// The session id breaks its rule.
    Session,
    /// The resume argv breaks herdr's checks.
    Argv,
    /// A quota window breaks its rule.
    Quota,
    /// A release from a source or process that is not the authority.
    ReleaseAuthority,
    /// A release for a terminal with no authority.
    ReleaseNone,
}

impl Refusal {
    /// The refusal's name, as the harness names it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Unknown => "agent_unknown",
            Self::Stale => "agent_report_stale",
            Self::Authority => "agent_report_authority",
            Self::Shape => "agent_report_shape",
            Self::Source => "agent_report_source",
            Self::State => "agent_report_state",
            Self::Question => "agent_report_question",
            Self::Progress => "agent_report_progress",
            Self::Session => "agent_report_session",
            Self::Argv => "agent_report_argv",
            Self::Quota => "agent_report_quota",
            Self::ReleaseAuthority => "agent_release_authority",
            Self::ReleaseNone => "agent_release_none",
        }
    }
}

impl Report {
    /// The report's state in the fleet's words.
    #[must_use]
    pub fn fleet_state(&self) -> Option<marley_fleet::State> {
        Some(match self.state.as_str() {
            "starting" => marley_fleet::State::Starting,
            "working" => marley_fleet::State::Working,
            "idle" => marley_fleet::State::Idle,
            "waiting" => marley_fleet::State::Waiting,
            "error" => marley_fleet::State::Error,
            "done" => marley_fleet::State::Done,
            _ => return None,
        })
    }

    /// Checks every field rule of the contract.
    ///
    /// # Errors
    ///
    /// The first rule broken, with its refusal and a reason in words.
    pub fn validate(&self) -> Result<(), (Refusal, String)> {
        let fail = |refusal: Refusal, reason: &str| Err((refusal, reason.to_string()));
        let source_ok = (1..=MAX_SOURCE).contains(&self.source.len())
            && self.source.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b':' | b'.' | b'_' | b'-')
            });
        if !source_ok {
            return fail(
                Refusal::Source,
                "a source is 1 to 64 of a-z, 0-9, ':', '.', '_' and '-'",
            );
        }
        if self.fleet_state().is_none() {
            return fail(Refusal::State, "the state is not one of the six");
        }
        if let Some(question) = &self.question {
            if self.state != "waiting" {
                return fail(Refusal::Question, "a question comes only with waiting");
            }
            question_ok(question).or_else(|reason| fail(Refusal::Question, &reason))?;
        }
        if let Some(progress) = &self.progress {
            if progress.percent.is_some_and(|percent| percent > 100) {
                return fail(Refusal::Progress, "the percent is 0 to 100");
            }
            if progress.activity.as_ref().is_some_and(|activity| {
                activity.chars().count() > MAX_ACTIVITY || activity.chars().any(char::is_control)
            }) {
                return fail(
                    Refusal::Progress,
                    "the activity is one line of at most 120 characters",
                );
            }
        }
        if let Some(session) = &self.session_id
            && (session.is_empty()
                || session.chars().count() > MAX_SESSION_ID
                || session.chars().any(char::is_control))
        {
            return fail(
                Refusal::Session,
                "a session id is 1 to 512 characters, none a control",
            );
        }
        if let Some(argv) = &self.resume_argv {
            validate_resume_argv(argv).or_else(|reason| fail(Refusal::Argv, &reason))?;
        }
        if let Some(quota) = &self.quota {
            quota_ok(quota).or_else(|reason| fail(Refusal::Quota, &reason))?;
        }
        Ok(())
    }

    /// The labels the report writes on its terminal's seat, by the harness's names.
    #[must_use]
    pub fn labels(&self) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();
        let mut set = |key: &str, value: String| {
            let _previous = labels.insert(key.to_string(), value);
        };
        set(SOURCE_LABEL, self.source.clone());
        set(STATE_SOURCE_LABEL, "reported".to_string());
        if let Some(session) = &self.session_id {
            set(SESSION_LABEL, session.clone());
        }
        if let Some(progress) = &self.progress {
            if let Some(percent) = progress.percent {
                set(PERCENT_LABEL, percent.to_string());
            }
            if let Some(activity) = &progress.activity {
                set(ACTIVITY_LABEL, activity.clone());
            }
        }
        if let Some(usage) = &self.usage {
            set("usage.input_tokens", usage.input.to_string());
            set("usage.output_tokens", usage.output.to_string());
            set("usage.cache_read_tokens", usage.cache_read.to_string());
        }
        for window in self.quota.iter().flatten() {
            set(
                &format!("quota.{}.percent_used", window.kind),
                format!("{:.1}", window.percent_used),
            );
            if let Some(resets) = window.resets_at_ms {
                set(
                    &format!("quota.{}.resets_at_ms", window.kind),
                    resets.to_string(),
                );
            }
        }
        labels
    }
}

fn question_ok(question: &Question) -> Result<(), String> {
    if question.prompt.trim().is_empty()
        || question.prompt.len() > MAX_PROMPT
        || question.prompt.contains('\0')
    {
        return Err("a question's prompt is 1 to 2,048 bytes".to_string());
    }
    if question.options.is_empty() || question.options.len() > MAX_OPTIONS {
        return Err("a question has 1 to 16 options".to_string());
    }
    for (index, option) in question.options.iter().enumerate() {
        if option.is_empty()
            || option.chars().count() > MAX_OPTION
            || option.chars().any(char::is_control)
        {
            return Err("an option is 1 to 128 characters, none a control".to_string());
        }
        if question.options[..index].contains(option) {
            return Err("a question's options are distinct".to_string());
        }
    }
    Ok(())
}

fn quota_ok(quota: &[QuotaWindow]) -> Result<(), String> {
    if quota.len() > MAX_QUOTA_WINDOWS {
        return Err("at most eight quota windows".to_string());
    }
    for (index, window) in quota.iter().enumerate() {
        let kind_ok = (1..=MAX_QUOTA_KIND).contains(&window.kind.len())
            && window
                .kind
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
        if !kind_ok {
            return Err("a quota's kind is 1 to 32 of a-z, 0-9 and '_'".to_string());
        }
        if quota[..index].iter().any(|other| other.kind == window.kind) {
            return Err("each quota window has a kind of its own".to_string());
        }
        if !(0.0..=MAX_PERCENT_USED).contains(&window.percent_used) {
            return Err("a quota's percent used is 0 to 10,000".to_string());
        }
    }
    Ok(())
}

/// herdr's checks on a resume argv: 1 to 64 elements, at most 8,192 bytes, no control character
/// and no apostrophe, and a plain command name first that does not start with `-`.
///
/// # Errors
///
/// The rule broken, in words.
pub fn validate_resume_argv(argv: &[String]) -> Result<(), String> {
    if argv.is_empty() || argv.len() > MAX_RESUME_ARGS {
        return Err("a resume argv has 1 to 64 elements".to_string());
    }
    if argv.iter().map(String::len).sum::<usize>() > MAX_RESUME_ARGV_BYTES {
        return Err("a resume argv is at most 8,192 bytes".to_string());
    }
    if argv
        .iter()
        .any(|argument| argument.chars().any(char::is_control))
    {
        return Err("a resume argv holds no control character".to_string());
    }
    if argv.iter().any(|argument| argument.contains('\'')) {
        return Err("a resume argv holds no apostrophe".to_string());
    }
    let first = argv.first().map(String::as_str).unwrap_or_default();
    let plain = !first.is_empty()
        && !first.starts_with('-')
        && first
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'));
    if !plain {
        return Err("a resume argv starts with a plain command name".to_string());
    }
    Ok(())
}

/// What a program sends over the agent socket.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "verb", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    /// A report for the terminal named.
    Report {
        /// The `MARLEY_TERMINAL_ID` the program runs under.
        terminal: String,
        /// The report.
        report: Box<Report>,
    },
    /// A release of the terminal's authority.
    Release {
        /// The `MARLEY_TERMINAL_ID` the program runs under.
        terminal: String,
        /// The source releasing.
        source: String,
    },
}

/// The answer to an accepted request.
#[must_use]
pub fn answer_ok() -> String {
    serde_json::json!({ "ok": true }).to_string()
}

/// The answer to a refused request: its name and the reason.
#[must_use]
pub fn answer_refused(refusal: Refusal, reason: &str) -> String {
    serde_json::json!({ "refused": refusal.name(), "reason": reason }).to_string()
}

/// A process's parent and start time, from the text of its `/proc/PID/stat`: the fields after the
/// last `)`, which ends the command name, the parent at index 1 and the start time at index 19.
#[must_use]
pub fn stat_fields(stat: &str) -> Option<(u32, u64)> {
    let (_, after) = stat.rsplit_once(')')?;
    let fields: Vec<&str> = after.split_whitespace().collect();
    let parent = fields.get(1)?.parse().ok()?;
    let started = fields.get(19)?.parse().ok()?;
    Some((parent, started))
}
