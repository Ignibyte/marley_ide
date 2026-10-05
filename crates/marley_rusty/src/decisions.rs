//! Rusty's decisions as `brain_due` answers them: the follow-ups due (#655), and every decision
//! with its status and dates for the Decisions tab (#659).
//!
//! `brain_due` answers `{ due, all }`: the decisions whose follow-up falls within its horizon, the
//! follow-up day first, and every decision, the newest decided first. The tab draws both as Rusty
//! serves them and computes no date: Rusty flags an overdue follow-up itself.

use serde::Deserialize;

/// Rusty's tool for the decisions whose follow-up is due, and every decision.
pub const BRAIN_DUE: &str = "brain_due";

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
