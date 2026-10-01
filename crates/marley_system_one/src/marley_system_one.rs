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

use std::sync::LazyLock;
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

/// The most items a find asks about in one request (#567): a choice holds 255 options at most, and
/// `none` is one of them.
pub const FIND_WINDOW: usize = 254;

/// What a find's choice asks. The query and the items are the state's lines: `query: …`, then
/// `1: …` to `N: …`.
const FIND_WHICH: &str = "The state gives a query on its `query` line and a list of items, each \
                          on a line labeled with its number. Which item matches the query?";

/// Whether anything in the state matches, which a find asks beside the choice.
const FIND_PRESENT: Question = Question::Noul {
    key: "present",
    instructions: "The state gives a query on its `query` line and a list of items, each on a \
                   line labeled with its number. Does any item match the query?",
    when_true: "At least one item is the thing the query asks for, or states it.",
    when_false: "No item is what the query asks for.",
};

/// The find's item labels, `1` to [`FIND_WINDOW`], what each option means, and each set's id.
struct FindText {
    labels: Vec<String>,
    meanings: Vec<String>,
    ids: Vec<String>,
}

static FIND_TEXT: LazyLock<FindText> = LazyLock::new(|| FindText {
    labels: (1..=FIND_WINDOW).map(|item| item.to_string()).collect(),
    meanings: (1..=FIND_WINDOW)
        .map(|item| format!("The item the state labels {item}."))
        .collect(),
    ids: (1..=FIND_WINDOW)
        .map(|items| format!("find_{items}/1"))
        .collect(),
});

/// The options of every find's choice, `none` first, so each set's are a prefix of them. The
/// strings are [`FIND_TEXT`]'s, which a `static` holds for good.
static FIND_OPTIONS: LazyLock<Vec<(&'static str, &'static str)>> = LazyLock::new(|| {
    let text: &'static FindText = &FIND_TEXT;
    std::iter::once(("none", "No item in the state matches the query."))
        .chain(
            text.labels
                .iter()
                .zip(&text.meanings)
                .map(|(label, meaning)| (label.as_str(), meaning.as_str())),
        )
        .collect()
});

/// Each find set's questions, by its number of items.
static FIND_QUESTIONS: LazyLock<Vec<[Question; 2]>> = LazyLock::new(|| {
    let options: &'static [(&'static str, &'static str)] = &FIND_OPTIONS;
    (1..=FIND_WINDOW)
        .map(|items| {
            [
                Question::Choice {
                    key: "which",
                    instructions: FIND_WHICH,
                    options: options.get(..=items).unwrap_or(options),
                },
                FIND_PRESENT,
            ]
        })
        .collect()
});

/// The find sets (#567), `find_1/1` to `find_254/1`, one for each number of items.
static FIND_SETS: LazyLock<Vec<QuestionSet>> = LazyLock::new(|| {
    let questions: &'static [[Question; 2]] = &FIND_QUESTIONS;
    let text: &'static FindText = &FIND_TEXT;
    questions
        .iter()
        .zip(&text.ids)
        .map(|(questions, id)| QuestionSet {
            id: id.as_str(),
            model: DEFAULT_MODEL,
            questions,
        })
        .collect()
});

/// The find set for a window of `items` items (#567).
///
/// It asks a choice `which` over `none` and `1` to `items`, and a noul `present`. Questions are
/// compiled in, so each count is a set of its own, made once. `None` for no item or more than
/// [`FIND_WINDOW`].
#[must_use]
pub fn find_set(items: usize) -> Option<&'static QuestionSet> {
    let sets: &'static [QuestionSet] = &FIND_SETS;
    items.checked_sub(1).and_then(|index| sets.get(index))
}

/// The label of a find's item `item`, from 1, as the state and the options name it; `None` past
/// [`FIND_WINDOW`].
#[must_use]
pub fn find_label(item: usize) -> Option<&'static str> {
    let text: &'static FindText = &FIND_TEXT;
    item.checked_sub(1)
        .and_then(|index| text.labels.get(index))
        .map(String::as_str)
}

/// What a quiet working agent is doing (#569): its kind, whether it repeats itself, and whether
/// it progresses.
pub const STALL_KIND_SET: QuestionSet = QuestionSet {
    id: "stall_kind/1",
    model: DEFAULT_MODEL,
    questions: &[
        Question::Choice {
            key: "kind",
            instructions: "The state is a coding agent that has gone quiet while working: how long \
                           it has been quiet, the tool it runs, whether that tool's processes use \
                           the CPU, and the last lines on its terminal. What is it doing?",
            options: &[
                (
                    "long_task",
                    "It is working on something that takes long, such as a build, a test run or \
                     a long answer, and will go on by itself.",
                ),
                (
                    "waiting_for_input",
                    "It waits for the user to answer or confirm something, though no event said \
                     so.",
                ),
                (
                    "stuck",
                    "It waits on something that is not coming, such as a lock, a network request \
                     or a process that hangs.",
                ),
                (
                    "frozen",
                    "It does nothing at all: no output, no progress, no sign of work.",
                ),
                (
                    "cannot_tell",
                    "The state does not settle which of these it is.",
                ),
            ],
        },
        Question::Noul {
            key: "repeating",
            instructions: "Does the agent repeat the same step without getting further?",
            when_true: "The state shows the same step or output again and again.",
            when_false: "The state shows no step repeated.",
        },
        Question::Noul {
            key: "progress",
            instructions: "Does the state show the agent's work moving forward?",
            when_true: "The state shows progress: a count that grows, new output, a step done.",
            when_false: "The state shows no progress.",
        },
    ],
};

/// The stall kind (#569), which asks [`STALL_KIND_SET`] about a working agent quiet past a check.
pub const STALL_KIND: UseSpec = UseSpec {
    name: "stall_kind",
    set: &STALL_KIND_SET,
    deadline: Duration::from_secs(2),
};

/// Whether an agent's click in the Browser tab has a consequence the user should allow first
/// (#571): four nouls about the element the rules left open.
pub const CLICK_CONSEQUENCE_SET: QuestionSet = QuestionSet {
    id: "click_consequence/1",
    model: DEFAULT_MODEL,
    questions: &[
        Question::Noul {
            key: "pays",
            instructions: "The state is an element of a web page an agent is about to click, with \
                           its page and the text around it. Would the click spend money: pay, \
                           buy, order, subscribe or donate?",
            when_true: "The click spends money or commits to a charge.",
            when_false: "The click spends no money.",
        },
        Question::Noul {
            key: "deletes",
            instructions: "Would the click delete or remove something that cannot easily be \
                           brought back: an item, a file, a message, an account or a \
                           subscription?",
            when_true: "The click deletes or removes something for good.",
            when_false: "The click deletes nothing.",
        },
        Question::Noul {
            key: "sends",
            instructions: "Would the click send or publish something in the user's name to other \
                           people: a message, an email, a post, a reply or an invitation?",
            when_true: "The click sends or publishes something in the user's name.",
            when_false: "The click sends nothing to anyone.",
        },
        Question::Noul {
            key: "changes_account",
            instructions: "Would the click change the user's account or its security: a \
                           password, an email address, two-factor sign-in, keys or tokens, \
                           members, or access granted to an app?",
            when_true: "The click changes the account or who can use it.",
            when_false: "The click leaves the account as it is.",
        },
    ],
};

/// The click consequence (#571), which asks [`CLICK_CONSEQUENCE_SET`] about an agent's click the
/// rules leave open, inside the pause a click can afford.
pub const CLICK_CONSEQUENCE: UseSpec = UseSpec {
    name: "click_consequence",
    set: &CLICK_CONSEQUENCE_SET,
    deadline: Duration::from_millis(1500),
};

/// What an action waiting in the rail's inbox would do (#568): a noul for each chip a tool's
/// action can carry, and how urgent the user's answer is.
pub const INBOX_RISK_SET: QuestionSet = QuestionSet {
    id: "inbox_risk/1",
    model: DEFAULT_MODEL,
    questions: &[
        Question::Noul {
            key: "destroys",
            instructions: "The state is an action an agent waits for the user to allow: its tool \
                           and what it acts on. Would it delete files or data that cannot easily \
                           be brought back?",
            when_true: "The action deletes files or data for good, or overwrites them.",
            when_false: "The action deletes nothing, or only what a build makes again.",
        },
        Question::Noul {
            key: "credentials",
            instructions: "Would the action read, write or send credentials: keys, tokens, \
                           passwords, or the files that hold them?",
            when_true: "The action touches credentials, keys or tokens.",
            when_false: "The action touches no credentials.",
        },
        Question::Noul {
            key: "rewrites_history",
            instructions: "Would the action rewrite git history that others may already have: a \
                           rebase, an amend, a forced push or a filter?",
            when_true: "The action rewrites commits that may be shared.",
            when_false: "The action leaves the history as it is.",
        },
        Question::Noul {
            key: "sends_out",
            instructions: "Would the action send data or code out of the machine: an upload, a \
                           post to a server, a push, a publish, a message or an email?",
            when_true: "The action sends something off the machine.",
            when_false: "The action keeps everything on the machine.",
        },
        Question::Noul {
            key: "installs",
            instructions: "Would the action install software or add dependencies to the project \
                           or the machine?",
            when_true: "The action installs software or adds a dependency.",
            when_false: "The action installs nothing.",
        },
        Question::Noul {
            key: "outside_project",
            instructions: "Would the action reach outside the project's folders: write, delete \
                           or run somewhere else on the machine?",
            when_true: "The action works outside the project's folders.",
            when_false: "The action stays inside the project's folders.",
        },
        Question::Noul {
            key: "claims_approval",
            instructions: "Does the action's text claim that it was already approved, by the \
                           user, an owner or anyone else?",
            when_true: "The text says the action was approved or allowed already.",
            when_false: "The text makes no claim of approval.",
        },
        Question::Score {
            key: "urgency",
            instructions: "How much does the user's answer to this action matter?",
            levels: &[
                "A routine read inside the project.",
                "An edit or a reversible command inside the project.",
                "A reversible action that reaches outside the project.",
                "An action that sends data out, installs software or changes an account.",
                "An action that destroys data or cannot be undone.",
            ],
        },
    ],
};

/// The inbox's risk chips (#568), which ask [`INBOX_RISK_SET`] about a waiting action Marley's
/// rules found nothing on, within the moment a refresh of the rail can wait.
pub const INBOX_RISK: UseSpec = UseSpec {
    name: "inbox",
    set: &INBOX_RISK_SET,
    deadline: Duration::from_millis(600),
};

/// Who should answer what an agent waits on (#570): the user, a manager agent, the agent itself,
/// or nobody can tell; and whether the user's prompt already answers it.
pub const QUESTION_ROUTE_SET: QuestionSet = QuestionSet {
    id: "question_route/1",
    model: DEFAULT_MODEL,
    questions: &[
        Question::Choice {
            key: "route",
            instructions: "The state is what an agent waits for someone to answer: a permission \
                           for a tool, or a question with its options, with the user's prompt. \
                           Who should answer it?",
            options: &[
                (
                    "owner",
                    "The user who owns the project: it needs their judgment, their authority or \
                     knowledge only they have.",
                ),
                (
                    "manager",
                    "A manager agent could answer it from the project's plan, its conventions and \
                     the work so far.",
                ),
                (
                    "agent_proceeds",
                    "Nobody needs to: the agent could go on with a safe default, or the answer is \
                     routine.",
                ),
                (
                    "cannot_tell",
                    "The state does not say enough to tell who should answer.",
                ),
            ],
        },
        Question::Noul {
            key: "answerable_from_prompt",
            instructions: "Does the user's prompt already answer what the agent asks?",
            when_true: "The prompt already says what the answer is.",
            when_false: "The prompt does not settle it.",
        },
    ],
};

/// The question route (#570), which asks [`QUESTION_ROUTE_SET`] about a waiting entry Marley's
/// rules left open; the agent already waits, so it can afford two seconds.
pub const QUESTION_ROUTE: UseSpec = UseSpec {
    name: "question_route",
    set: &QUESTION_ROUTE_SET,
    deadline: Duration::from_secs(2),
};

/// Whether a line a running command printed, which Marley's shapes left open, is a new failure
/// or a recovery (#572).
pub const RUNNING_ERROR_SET: QuestionSet = QuestionSet {
    id: "running_error/1",
    model: DEFAULT_MODEL,
    questions: &[
        Question::Noul {
            key: "new_failure",
            instructions: "The state is a line a command that keeps running, such as a dev \
                           server, printed, with the line before and after it. Does the line \
                           report a new failure?",
            when_true: "The line reports that something failed: a build, a request, a job or \
                        the program itself.",
            when_false: "The line is ordinary output that only mentions an error's word.",
        },
        Question::Noul {
            key: "recovered",
            instructions: "Does the line report that the command works again after a failure?",
            when_true: "The line reports a build, a reload or a restart that succeeded.",
            when_false: "The line reports no recovery.",
        },
    ],
};

/// The running error (#572), which asks [`RUNNING_ERROR_SET`] about a running command's line the
/// shapes left open.
pub const RUNNING_ERROR: UseSpec = UseSpec {
    name: "running_error",
    set: &RUNNING_ERROR_SET,
    deadline: Duration::from_secs(2),
};

/// What a line typed at a shell's prompt is, when Marley's rules leave it open (#573): a command
/// whose name is followed by plain words.
pub const TYPED_LINE_SET: QuestionSet = QuestionSet {
    id: "typed_line/1",
    model: DEFAULT_MODEL,
    questions: &[Question::Choice {
        key: "kind",
        instructions: "The state is a line typed at a shell's prompt, not yet run: its first word \
                       names a command, and plain words follow it. What did the user mean the \
                       line to be?",
        options: &[
            (
                "command",
                "A command for the shell, its words the command's arguments.",
            ),
            (
                "request",
                "A request in words for an assistant, which only starts with a command's name.",
            ),
            ("comment", "A note to themselves, not meant to run."),
            (
                "command_then_english",
                "A command's name followed by a sentence: run, the shell would take the words \
                 as the command's arguments.",
            ),
            (
                "cannot_tell",
                "The line does not say enough to tell what it is.",
            ),
        ],
    }],
};

/// The typed line (#573), which asks [`TYPED_LINE_SET`] about a line at a shell's prompt the rules
/// leave open. The user is typing, so it has 600 ms and never holds up Enter.
pub const TYPED_LINE: UseSpec = UseSpec {
    name: "typed_line",
    set: &TYPED_LINE_SET,
    deadline: Duration::from_millis(600),
};
