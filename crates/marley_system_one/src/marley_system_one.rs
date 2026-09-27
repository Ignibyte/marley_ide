//! The System One layer's pure core (#565).
//!
//! A use of the layer asks a compiled-in [`QuestionSet`] about a [`state::State`] that Marley
//! builds from facts computed in code and from text masked before it is added. The answer never
//! approves anything: the use may display, rank, route on or refuse with the
//! [`reading::Reading`] it gets back. With the layer off, a provider down or a budget spent, that
//! reading says so, and nothing reaches the use as an error. [`request`] holds the `/v1/systemone`
//! request and its answer, [`policy`] decides whether a call may go out, and [`files`] keeps each
//! call. There is no gpui, no HTTP and no clock here: `marley_workbench::system_one` sends the
//! request, holds the key, keeps the time and names the folder the files go in.

// gate:21 runs Zed's dylint lints (`tooling/lints`) with these as errors in the Marley crates;
// Zed's crates keep them at warn (CONSTITUTION §0).
#![cfg_attr(
    dylint_lib = "lints",
    deny(
        async_block_without_await,
        blocking_io_on_foreground,
        entity_update_in_render,
        map_lookup_then_insert,
        notify_in_render,
        owned_string_into_shared,
        shared_string_from_str_literal
    )
)]

pub mod files;
pub mod policy;
pub mod reading;
pub mod request;
pub mod state;

use std::time::Duration;

/// The model asked when the settings name none: Jev, pinned to the version the compiled-in
/// thresholds were set for.
pub const DEFAULT_MODEL: &str = "jev-1.13.0";

/// A question set, compiled in, named and versioned.
///
/// A set is never edited: a change to one of its questions makes a new version, so a logged call
/// always names exactly what it asked, and a threshold belongs to one version and one model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuestionSet {
    /// The set's name and version, such as `check/1`.
    pub id: &'static str,
    /// The model its thresholds were set for.
    pub model: &'static str,
    /// Its questions.
    pub questions: &'static [Question],
}

/// One typed question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Question {
    /// A yes or a no, answered as the probability of yes.
    Noul {
        /// The key the answer comes back under.
        key: &'static str,
        /// What is asked.
        instructions: &'static str,
        /// What yes means.
        when_true: &'static str,
        /// What no means.
        when_false: &'static str,
    },
    /// One of a fixed list of options, at most 255. Wherever the state may not settle the question,
    /// one option is `cannot_tell`, since a model cannot say "I don't know" without one.
    Choice {
        /// The key the answer comes back under.
        key: &'static str,
        /// What is asked.
        instructions: &'static str,
        /// Each option's name and what it means.
        options: &'static [(&'static str, &'static str)],
    },
    /// A level on an ordered scale of 2 to 10 levels.
    Score {
        /// The key the answer comes back under.
        key: &'static str,
        /// What is asked.
        instructions: &'static str,
        /// What each level means, from the lowest.
        levels: &'static [&'static str],
    },
}

impl Question {
    /// The key the answer comes back under.
    #[must_use]
    pub const fn key(&self) -> &'static str {
        match self {
            Self::Noul { key, .. } | Self::Choice { key, .. } | Self::Score { key, .. } => key,
        }
    }
}

/// A use of the layer: the name its mode is set under, the set it asks, and how long a call may
/// take before the use goes on without an answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UseSpec {
    /// The use's name, the key of its mode in `marley.system_one.uses`.
    pub name: &'static str,
    /// The set it asks.
    pub set: &'static QuestionSet,
    /// How long a call may take.
    pub deadline: Duration,
}

/// The check's questions: whether the last command of a terminal failed.
pub const CHECK_SET: QuestionSet = QuestionSet {
    id: "check/1",
    model: DEFAULT_MODEL,
    questions: &[Question::Noul {
        key: "command_failed",
        instructions: "Did the last command in this terminal fail?",
        when_true: "The command failed: it reported an error, exited with a nonzero code, or did \
                    not do what it was asked.",
        when_false: "The command did what it was asked.",
    }],
};

/// The check (`marley: system one check`), which asks [`CHECK_SET`] about the terminal used last.
/// It runs only by hand, so one request and one answer can be seen without turning a use on.
pub const CHECK: UseSpec = UseSpec {
    name: "check",
    set: &CHECK_SET,
    deadline: Duration::from_secs(2),
};
