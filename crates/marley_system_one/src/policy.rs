//! Whether a call may go out.
//!
//! [`may_send`] decides what a project may send from the two lists the user keeps, and a [`Gate`]
//! holds back a call that would spend past the day's budget, reach a provider that keeps failing,
//! pass the rate the provider allows, or ask about a state already answered. Time is the host's: every
//! method takes it in milliseconds.

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;

use crate::state::Detail;

/// Failures in a row that open the breaker.
pub const BREAKER_FAILURES: u32 = 5;

/// How long an open breaker holds calls back, in milliseconds.
pub const BREAKER_OPEN_MS: u64 = 120_000;

/// The most calls in a minute, under the 1,200 `api.typesafe.ai` takes.
pub const CALLS_PER_MINUTE: usize = 1_000;

/// A minute, in milliseconds.
const MINUTE_MS: u64 = 60_000;

/// Why a call did not go out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// What the user allowed does not cover it.
    Refused(String),
    /// Held back for the budget, the provider or the rate.
    Unavailable(String),
}

/// What a project may send, from its folders and the two lists.
///
/// A remote project sends nothing. A folder on the metadata-only list that holds one of the
/// project's folders lets it send facts alone, and one on the projects list lets it send facts and
/// text; the metadata-only list wins when both hold it.
///
/// # Errors
///
/// A [`Refusal::Refused`] when the project may send nothing.
pub fn may_send(
    folders: &[PathBuf],
    local: bool,
    projects: &[PathBuf],
    metadata_only: &[PathBuf],
) -> Result<Detail, Refusal> {
    if !local {
        return Err(Refusal::Refused(
            "a remote project sends nothing".to_string(),
        ));
    }
    let listed = |list: &[PathBuf]| {
        folders
            .iter()
            .any(|folder| list.iter().any(|listed| folder.starts_with(listed)))
    };
    if listed(metadata_only) {
        Ok(Detail::Facts)
    } else if listed(projects) {
        Ok(Detail::Full)
    } else {
        Err(Refusal::Refused("project not listed".to_string()))
    }
}

/// A call's cost in billionths of a cent: its input tokens at `price`, in thousandths of a cent
/// per million tokens.
#[must_use]
pub const fn cost(input_tokens: u64, price: u64) -> u64 {
    input_tokens.saturating_mul(price)
}

/// A budget in cents, in the billionths of a cent [`cost`] counts.
#[must_use]
pub const fn budget(cents: u64) -> u64 {
    cents.saturating_mul(1_000_000_000)
}

/// The gate every call to a provider passes, and what it learned from the calls before.
#[derive(Debug, Default)]
pub struct Gate {
    /// The day the spend is counted for.
    day: String,
    /// What the day's calls cost, in billionths of a cent.
    spent: u64,
    /// Failures since the last call that answered.
    failures: u32,
    /// When an open breaker lets calls through again.
    open_until: Option<u64>,
    /// When the last minute's calls went out.
    sent: VecDeque<u64>,
    /// The state each subject was last answered about, by its hash.
    answered: HashMap<String, String>,
}

impl Gate {
    /// Whether a call about `subject` in the state `state_hash` may go out at `now` on `day`, with
    /// `budget` billionths of a cent to spend in a day.
    ///
    /// # Errors
    ///
    /// A [`Refusal`] that says why the call is held back.
    pub fn admit(
        &mut self,
        subject: &str,
        state_hash: &str,
        day: &str,
        now: u64,
        budget: u64,
    ) -> Result<(), Refusal> {
        if self.day != day {
            day.clone_into(&mut self.day);
            self.spent = 0;
        }
        if self.spent >= budget {
            return Err(Refusal::Unavailable("over the daily budget".to_string()));
        }
        if let Some(until) = self.open_until {
            if now < until {
                return Err(Refusal::Unavailable("breaker open".to_string()));
            }
            self.open_until = None;
            self.failures = 0;
        }
        self.sent
            .retain(|&sent| now.saturating_sub(sent) < MINUTE_MS);
        if self.sent.len() >= CALLS_PER_MINUTE {
            return Err(Refusal::Unavailable(format!(
                "over {CALLS_PER_MINUTE} calls a minute"
            )));
        }
        if self
            .answered
            .get(subject)
            .is_some_and(|answered| answered == state_hash)
        {
            return Err(Refusal::Refused(
                "unchanged since the last answer".to_string(),
            ));
        }
        self.sent.push_back(now);
        Ok(())
    }

    /// Notes a call that got an answer about `subject` in the state `state_hash`, costing `cost`.
    pub fn answered(&mut self, subject: &str, state_hash: &str, cost: u64) {
        self.spent = self.spent.saturating_add(cost);
        self.failures = 0;
        let _last = self
            .answered
            .insert(subject.to_string(), state_hash.to_string());
    }

    /// Notes a call the provider did not answer at `now`; the fifth in a row opens the breaker.
    pub const fn failed(&mut self, now: u64) {
        self.failures = self.failures.saturating_add(1);
        if self.failures >= BREAKER_FAILURES {
            self.open_until = Some(now.saturating_add(BREAKER_OPEN_MS));
        }
    }

    /// What `day`'s calls cost, in billionths of a cent.
    #[must_use]
    pub fn spent_on(&self, day: &str) -> u64 {
        if self.day == day { self.spent } else { 0 }
    }

    /// Whether the breaker holds calls back at `now`.
    #[must_use]
    pub fn breaker_open(&self, now: u64) -> bool {
        self.open_until.is_some_and(|until| now < until)
    }
}
