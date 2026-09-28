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

/// The most parts of a prompt the stop kind asks about (#566).
pub const MAX_PARTS: usize = 6;

/// What a stopped turn needs, asked of the agent's last message.
const STOP_KIND_QUESTION: Question = Question::Choice {
    key: "kind",
    instructions: "The state is a coding agent's turn that has just stopped: the user's prompt, the \
                   agent's last message, and facts about the tools it ran. What does the last \
                   message say about the user's request?",
    options: &[
        (
            "done_checked",
            "The message reports the work done and names a check that ran: tests, a build, or a \
             command whose result it cites.",
        ),
        (
            "done_claimed",
            "The message reports the work done and names no check that ran.",
        ),
        (
            "asks_you",
            "The message asks the user something, or offers choices, and waits for the answer.",
        ),
        (
            "blocked",
            "The message says the agent cannot go on: it needs something, a permission, or a fix \
             for a failure it could not make itself.",
        ),
        (
            "still_going",
            "The message describes work under way, or a next step the agent will take itself.",
        ),
        (
            "cannot_tell",
            "The message does not settle which of these it is.",
        ),
    ],
};

/// A noul on whether the last message covers one part of the prompt, which the state labels
/// `part N`: the parts are the state's text, so they are masked like any text.
const fn part(key: &'static str, instructions: &'static str) -> Question {
    Question::Noul {
        key,
        instructions,
        when_true: "The message states or directly implies that this part of the request is done.",
        when_false: "The message does not mention this part of the request, or leaves it for \
                     later.",
    }
}

const PART_1: Question = part(
    "part_1_done",
    "The state's `part 1` line is one part of the user's request. Does the agent's last message \
     say that part is done?",
);
const PART_2: Question = part(
    "part_2_done",
    "The state's `part 2` line is one part of the user's request. Does the agent's last message \
     say that part is done?",
);
const PART_3: Question = part(
    "part_3_done",
    "The state's `part 3` line is one part of the user's request. Does the agent's last message \
     say that part is done?",
);
const PART_4: Question = part(
    "part_4_done",
    "The state's `part 4` line is one part of the user's request. Does the agent's last message \
     say that part is done?",
);
const PART_5: Question = part(
    "part_5_done",
    "The state's `part 5` line is one part of the user's request. Does the agent's last message \
     say that part is done?",
);
const PART_6: Question = part(
    "part_6_done",
    "The state's `part 6` line is one part of the user's request. Does the agent's last message \
     say that part is done?",
);

/// The stop kind's questions by how many parts the prompt has.
///
/// The count runs from none (a metadata-only project, whose state has no text) to
/// [`MAX_PARTS`]. Questions are compiled in, so each count is a set of its own, and each set's id
/// names its count.
pub static STOP_KIND_SETS: [QuestionSet; MAX_PARTS + 1] = [
    QuestionSet {
        id: "stop_kind_0/1",
        model: DEFAULT_MODEL,
        questions: &[STOP_KIND_QUESTION],
    },
    QuestionSet {
        id: "stop_kind_1/1",
        model: DEFAULT_MODEL,
        questions: &[STOP_KIND_QUESTION, PART_1],
    },
    QuestionSet {
        id: "stop_kind_2/1",
        model: DEFAULT_MODEL,
        questions: &[STOP_KIND_QUESTION, PART_1, PART_2],
    },
    QuestionSet {
        id: "stop_kind_3/1",
        model: DEFAULT_MODEL,
        questions: &[STOP_KIND_QUESTION, PART_1, PART_2, PART_3],
    },
    QuestionSet {
        id: "stop_kind_4/1",
        model: DEFAULT_MODEL,
        questions: &[STOP_KIND_QUESTION, PART_1, PART_2, PART_3, PART_4],
    },
    QuestionSet {
        id: "stop_kind_5/1",
        model: DEFAULT_MODEL,
        questions: &[STOP_KIND_QUESTION, PART_1, PART_2, PART_3, PART_4, PART_5],
    },
    QuestionSet {
        id: "stop_kind_6/1",
        model: DEFAULT_MODEL,
        questions: &[
            STOP_KIND_QUESTION,
            PART_1,
            PART_2,
            PART_3,
            PART_4,
            PART_5,
            PART_6,
        ],
    },
];

/// How long the stop kind waits for an answer before the row goes on reading `idle`.
const STOP_KIND_DEADLINE: Duration = Duration::from_secs(2);

/// The stop kind (#566), one use under the name `stop_kind` for each of [`STOP_KIND_SETS`].
pub static STOP_KIND: [UseSpec; MAX_PARTS + 1] = [
    UseSpec {
        name: "stop_kind",
        set: &STOP_KIND_SETS[0],
        deadline: STOP_KIND_DEADLINE,
    },
    UseSpec {
        name: "stop_kind",
        set: &STOP_KIND_SETS[1],
        deadline: STOP_KIND_DEADLINE,
    },
    UseSpec {
        name: "stop_kind",
        set: &STOP_KIND_SETS[2],
        deadline: STOP_KIND_DEADLINE,
    },
    UseSpec {
        name: "stop_kind",
        set: &STOP_KIND_SETS[3],
        deadline: STOP_KIND_DEADLINE,
    },
    UseSpec {
        name: "stop_kind",
        set: &STOP_KIND_SETS[4],
        deadline: STOP_KIND_DEADLINE,
    },
    UseSpec {
        name: "stop_kind",
        set: &STOP_KIND_SETS[5],
        deadline: STOP_KIND_DEADLINE,
    },
    UseSpec {
        name: "stop_kind",
        set: &STOP_KIND_SETS[6],
        deadline: STOP_KIND_DEADLINE,
    },
];

/// The stop kind's use for a prompt of `parts` parts; more than [`MAX_PARTS`] asks about the
/// first ones.
#[must_use]
pub fn stop_kind(parts: usize) -> &'static UseSpec {
    let [.., most] = &STOP_KIND;
    STOP_KIND.get(parts).unwrap_or(most)
}
