//! Rusty's decisions as `brain_due` answers them, and a follow-up as `brain_follow_up` takes it.
//!
//! The follow-ups due came first (#655), every decision with its status and dates for the
//! Decisions tab next (#659), and the follow-up recorded from that tab last (#660).
//!
//! `brain_due` answers `{ due, all }`: the decisions whose follow-up falls within its horizon, the
//! follow-up day first, and every decision, the newest decided first. The tab draws both as Rusty
//! serves them and computes no date: Rusty flags an overdue follow-up itself.

use serde::Deserialize;
use serde_json::{Map, Value};

/// Rusty's tool for the decisions whose follow-up is due, and every decision.
pub const BRAIN_DUE: &str = "brain_due";

/// Rusty's tool that records how a decision went.
pub const BRAIN_FOLLOW_UP: &str = "brain_follow_up";

/// One decision as Rusty sums it up.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DecisionSummary {
    /// The decision page's slug.
    pub slug: String,
    /// Its title.
    #[serde(default)]
    pub title: String,
    /// Where it stands: `decided`, `kept`, `revised` or `superseded`; empty reads `decided`.
    #[serde(default)]
    pub status: String,
    /// The day it was decided, `YYYY-MM-DD`.
    #[serde(default)]
    pub decided: String,
    /// The day its follow-up is due, `YYYY-MM-DD`; Rusty sends an empty one for none.
    #[serde(default)]
    pub follow_up_by: Option<String>,
    /// Whether that day has passed.
    #[serde(default)]
    pub overdue: bool,
    /// The day of the last follow-up, `YYYY-MM-DD`; empty before the first.
    #[serde(default)]
    pub followed_up: String,
    /// The slug of the decision that replaced this one; empty unless it was superseded.
    #[serde(default)]
    pub superseded_by: String,
}

impl DecisionSummary {
    /// Where it stands.
    #[must_use]
    pub fn status(&self) -> DecisionStatus {
        DecisionStatus::parse(&self.status)
    }

    /// The follow-up day, when one is set.
    #[must_use]
    pub fn follow_up(&self) -> Option<&str> {
        self.follow_up_by.as_deref().filter(|day| !day.is_empty())
    }

    /// "decided DAY", when Rusty has the day.
    #[must_use]
    pub fn decided_line(&self) -> Option<String> {
        (!self.decided.is_empty()).then(|| format!("decided {}", self.decided))
    }

    /// "follow up by DAY", with "· overdue" when Rusty flags it; none without a follow-up day.
    #[must_use]
    pub fn follow_up_line(&self) -> Option<String> {
        let day = self.follow_up()?;
        Some(if self.overdue {
            format!("follow up by {day} · overdue")
        } else {
            format!("follow up by {day}")
        })
    }

    /// "followed up DAY", when Rusty has recorded a follow-up.
    #[must_use]
    pub fn followed_up_line(&self) -> Option<String> {
        (!self.followed_up.is_empty()).then(|| format!("followed up {}", self.followed_up))
    }

    /// The slug of the decision that replaced this one, when Rusty names one.
    #[must_use]
    pub fn successor(&self) -> Option<&str> {
        Some(self.superseded_by.as_str()).filter(|slug| !slug.is_empty())
    }
}

/// Where a decision stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionStatus {
    /// Decided, its follow-up not yet recorded.
    Decided,
    /// Kept at its follow-up.
    Kept,
    /// Revised at its follow-up, with a new day.
    Revised,
    /// Replaced by another decision.
    Superseded,
    /// A status Rusty may add later, as written.
    Other(String),
}

impl DecisionStatus {
    /// The status as Rusty writes it; empty reads `Decided`, as Rusty's summary does.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        match text.trim() {
            "" | "decided" => Self::Decided,
            "kept" => Self::Kept,
            "revised" => Self::Revised,
            "superseded" => Self::Superseded,
            other => Self::Other(other.to_string()),
        }
    }

    /// Its word on the row.
    #[must_use]
    pub fn word(&self) -> &str {
        match self {
            Self::Decided => "decided",
            Self::Kept => "kept",
            Self::Revised => "revised",
            Self::Superseded => "superseded",
            Self::Other(word) => word,
        }
    }
}

/// `brain_due`'s answer.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Due {
    /// The follow-ups due within the horizon, the follow-up day first.
    #[serde(default)]
    pub due: Vec<DecisionSummary>,
    /// Every decision, the newest decided first.
    #[serde(default)]
    pub all: Vec<DecisionSummary>,
}

/// `brain_due`'s answer, whole.
///
/// # Errors
///
/// When the answer is not `brain_due`'s.
pub fn parse_due(text: &str) -> Result<Due, serde_json::Error> {
    serde_json::from_str(text)
}

/// The follow-ups due, in Rusty's order, from `brain_due`'s answer.
///
/// # Errors
///
/// When the answer is not `brain_due`'s.
pub fn due_from_answer(text: &str) -> Result<Vec<DecisionSummary>, serde_json::Error> {
    parse_due(text).map(|due| due.due)
}

/// A section of the Decisions tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// The follow-ups due.
    Due,
    /// Every decision.
    All,
}

/// One line of the Decisions tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entry {
    /// The "Due" header, only when something is due.
    DueHeader,
    /// The count of every decision.
    AllHeader {
        /// How many decisions there are.
        count: usize,
    },
    /// A decision, by its section and its place in that section's list.
    Row {
        /// The section it is in.
        section: Section,
        /// Its place in the section's list.
        index: usize,
    },
}

/// The tab's lines: the Due header and rows when any are due, then the count and every decision.
#[must_use]
pub fn entries(due: &Due) -> Vec<Entry> {
    let mut entries = Vec::with_capacity(due.due.len() + due.all.len() + 2);
    if !due.due.is_empty() {
        entries.push(Entry::DueHeader);
        entries.extend((0..due.due.len()).map(|index| Entry::Row {
            section: Section::Due,
            index,
        }));
    }
    entries.push(Entry::AllHeader {
        count: due.all.len(),
    });
    entries.extend((0..due.all.len()).map(|index| Entry::Row {
        section: Section::All,
        index,
    }));
    entries
}

/// "1 decision", "7 decisions".
#[must_use]
pub fn count_line(count: usize) -> String {
    if count == 1 {
        "1 decision".to_string()
    } else {
        format!("{count} decisions")
    }
}

/// How a follow-up says a decision went: the three statuses `brain_follow_up` takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FollowUpStatus {
    /// It held.
    Kept,
    /// It changed, with a new day to look again or none.
    Revised,
    /// Another decision replaced it.
    Superseded,
}

impl FollowUpStatus {
    /// The three, in the order the form shows them.
    pub const ALL: [Self; 3] = [Self::Kept, Self::Revised, Self::Superseded];

    /// The status as Rusty takes it.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Kept => "kept",
            Self::Revised => "revised",
            Self::Superseded => "superseded",
        }
    }

    /// Its button's label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Kept => "Kept",
            Self::Revised => "Revised",
            Self::Superseded => "Superseded",
        }
    }
}

/// What a follow-up still needs before it can be recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Missing {
    /// No status chosen.
    Status,
    /// No words on how it went.
    Outcome,
    /// Superseded with no successor picked.
    Successor,
}

impl Missing {
    /// The form's hint for it.
    #[must_use]
    pub const fn hint(self) -> &'static str {
        match self {
            Self::Status => "Choose kept, revised or superseded.",
            Self::Outcome => "Write how it went.",
            Self::Successor => "Choose the decision that replaced it.",
        }
    }
}

/// A follow-up as the form holds it, before it is sent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FollowUpDraft {
    /// The status chosen, if any.
    pub status: Option<FollowUpStatus>,
    /// How it went, as typed.
    pub outcome: String,
    /// The next follow-up day as typed; sent only when revised.
    pub day: String,
    /// The successor's slug; sent only when superseded.
    pub successor: Option<String>,
}

impl FollowUpDraft {
    /// The first thing the draft lacks, or none when it can be recorded.
    #[must_use]
    pub fn missing(&self) -> Option<Missing> {
        let Some(status) = self.status else {
            return Some(Missing::Status);
        };
        if self.outcome.trim().is_empty() {
            return Some(Missing::Outcome);
        }
        (status == FollowUpStatus::Superseded && self.successor.is_none())
            .then_some(Missing::Successor)
    }

    /// `brain_follow_up`'s arguments for the decision `slug`: the outcome trimmed, the day only
    /// when revised and given, the successor only when superseded. None while something is
    /// missing. The day goes as typed: Rusty says whether it is one.
    #[must_use]
    pub fn arguments(&self, slug: &str) -> Option<Value> {
        if self.missing().is_some() {
            return None;
        }
        let status = self.status?;
        let mut fields = vec![
            ("slug", Value::from(slug)),
            ("status", Value::from(status.word())),
            ("outcome", Value::from(self.outcome.trim())),
        ];
        let day = self.day.trim();
        if status == FollowUpStatus::Revised && !day.is_empty() {
            fields.push(("follow_up_by", Value::from(day)));
        }
        if status == FollowUpStatus::Superseded
            && let Some(successor) = &self.successor
        {
            fields.push(("successor", Value::from(successor.as_str())));
        }
        let arguments: Map<String, Value> = fields
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect();
        Some(Value::Object(arguments))
    }
}
