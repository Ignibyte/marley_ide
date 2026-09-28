//! What a stopped Claude Code turn needs (#566).
//!
//! A `Stop` makes a seat idle with the agent's last message. The stop kind says what that stop
//! needs from the user: the work done and checked, done and only claimed, a question for them, a
//! block, work still going, or an interrupt. Code decides the clear cases from the turn's own
//! events ([`rules`]); the System One layer is asked the rest, and a noul per part of the prompt
//! says which parts the message covers. The answer rides on the seat as labels, so the rail's row
//! and `fleet_snapshot` read one thing. The kind only changes what a row says: nothing acts on it.

use std::collections::BTreeMap;

use crate::claude_events::TurnFacts;

/// The label for the stop's kind, such as `done_checked`.
pub const STOP_KIND_LABEL: &str = "stop_kind";
/// The label for who decided the kind: `rules` or `model`.
pub const STOP_KIND_SOURCE_LABEL: &str = "stop_kind_source";
/// The label for how sure the decision is, such as `0.91`.
pub const STOP_KIND_CONFIDENCE_LABEL: &str = "stop_kind_confidence";
/// The label for the prompt's parts the message does not cover, as a JSON array of the parts.
pub const STOP_PARTS_MISSING_LABEL: &str = "stop_parts_missing";
/// The stop's labels, which every turn's start and end clears.
pub(crate) const STOP_LABELS: [&str; 4] = [
    STOP_KIND_LABEL,
    STOP_KIND_SOURCE_LABEL,
    STOP_KIND_CONFIDENCE_LABEL,
    STOP_PARTS_MISSING_LABEL,
];

/// The most parts of a prompt asked about.
const MAX_PARTS: usize = 6;

/// The openings of a last sentence that asks the user something without a question mark.
const ASKING: [&str; 7] = [
    "should i",
    "do you want",
    "would you like",
    "which",
    "shall i",
    "let me know",
    "can you confirm",
];

/// Words a period follows without ending a sentence.
const ABBREVIATIONS: [&str; 5] = ["e.g.", "i.e.", "etc.", "vs.", "cf."];

/// What code decides about a stop before anything is asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The user interrupted the turn.
    Interrupted,
    /// A permission asked for in the turn never finished: refused, or never answered.
    Blocked,
    /// The last message asks the user something.
    AsksYou,
    /// Code cannot tell; the layer may be asked.
    Open,
}

impl Verdict {
    /// The kind a settled verdict gives; `Open` gives none.
    #[must_use]
    pub const fn kind(self) -> Option<Kind> {
        match self {
            Self::Interrupted => Some(Kind::Interrupted),
            Self::Blocked => Some(Kind::Blocked),
            Self::AsksYou => Some(Kind::AsksYou),
            Self::Open => None,
        }
    }
}

/// Decides what code can about a stop from the turn's `facts` and its last `message`: an
/// interrupt, a permission never finished, or a message whose last sentence asks.
#[must_use]
pub fn rules(message: Option<&str>, facts: &TurnFacts) -> Verdict {
    if facts.interrupted {
        Verdict::Interrupted
    } else if !facts.pending.is_empty() {
        Verdict::Blocked
    } else if message.is_some_and(asks) {
        Verdict::AsksYou
    } else {
        Verdict::Open
    }
}

/// Whether `message`'s last sentence asks the user something: it ends in a question mark, or it
/// opens with an asking phrase.
fn asks(message: &str) -> bool {
    let sentence = last_sentence(message);
    let ending = sentence.trim_end_matches(|character: char| {
        matches!(character, '*' | '_' | '`' | '"' | '\'' | ')' | '”' | '’')
    });
    let opening = sentence
        .trim_start_matches(|character: char| !character.is_alphanumeric())
        .to_lowercase();
    ending.ends_with('?')
        || ASKING.iter().any(|phrase| {
            opening.strip_prefix(phrase).is_some_and(|rest| {
                rest.chars()
                    .next()
                    .is_none_or(|character| !character.is_alphanumeric())
            })
        })
}

/// The last sentence of `text`: what follows its last `.`, `?` or `!` that a space follows.
fn last_sentence(text: &str) -> &str {
    let text = text.trim();
    let start = sentence_breaks(text).last().unwrap_or(0);
    text.get(start..).unwrap_or(text)
}

/// Where each sentence after the first starts in `text`: after a `.`, `?`, `!` or `;` and the
/// space after it, except after an abbreviation or a list's number.
fn sentence_breaks(text: &str) -> impl Iterator<Item = usize> + '_ {
    text.char_indices()
        .zip(text.chars().skip(1))
        .filter(move |&((index, end), next)| {
            let word = text
                .get(..index + end.len_utf8())
                .and_then(|before| before.rsplit(char::is_whitespace).next())
                .unwrap_or_default();
            matches!(end, '.' | '?' | '!' | ';')
                && next.is_whitespace()
                && !is_numbered(word)
                && !ABBREVIATIONS
                    .iter()
                    .any(|abbreviation| word.eq_ignore_ascii_case(abbreviation))
        })
        .map(|((index, end), next)| index + end.len_utf8() + next.len_utf8())
}

/// The parts of `prompt` asked about: its sentences and list items, at most six.
///
/// Each part has two words or more. A part that ends in `:` introduces the others and is left
/// out, and so is a last part the plugin cut short (it ends in `…`), as its words are not all
/// there.
#[must_use]
pub fn parts(prompt: &str) -> Vec<String> {
    let prompt = prompt.trim();
    let mut starts: Vec<usize> = std::iter::once(0)
        .chain(sentence_breaks(prompt))
        .chain(list_items(prompt))
        .collect();
    starts.sort_unstable();
    starts.dedup();
    let ends = starts.iter().skip(1).copied().chain(Some(prompt.len()));
    let mut parts: Vec<String> = starts
        .iter()
        .zip(ends)
        .filter_map(|(start, end)| prompt.get(*start..end))
        .map(|part| strip_marker(part.trim()).to_string())
        .filter(|part| part.split_whitespace().count() >= 2 && !part.ends_with(':'))
        .collect();
    if parts.last().is_some_and(|part| part.ends_with('…')) {
        let _cut_short = parts.pop();
    }
    parts.truncate(MAX_PARTS);
    parts
}

/// Where each list item after the first word starts in `prompt`: at a number such as `1.` or
/// `2)` that stands alone as a word, or at the bullet (`-`, `*` or `•`) the prompt opens with.
/// The plugin sends a prompt on one line, so a bullet that opens no list is a dash in a sentence.
fn list_items(prompt: &str) -> impl Iterator<Item = usize> + '_ {
    let bullet = prompt
        .split_whitespace()
        .next()
        .filter(|word| is_bullet(word));
    prompt
        .char_indices()
        .filter(|&(index, character)| index > 0 && character.is_whitespace())
        .map(|(index, character)| index + character.len_utf8())
        .filter(move |start| {
            prompt
                .get(*start..)
                .and_then(|rest| rest.split_whitespace().next())
                .is_some_and(|word| is_numbered(word) || bullet == Some(word))
        })
}

/// Whether `word` is a list's bullet.
fn is_bullet(word: &str) -> bool {
    matches!(word, "-" | "*" | "•")
}

/// Whether `word` numbers a list item: one or two digits and `.` or `)`.
fn is_numbered(word: &str) -> bool {
    word.strip_suffix(['.', ')']).is_some_and(|number| {
        (1..=2).contains(&number.len())
            && number.chars().all(|character| character.is_ascii_digit())
    })
}

/// `part` without the list marker it opens with.
fn strip_marker(part: &str) -> &str {
    match part.split_once(char::is_whitespace) {
        Some((word, rest)) if is_bullet(word) || is_numbered(word) => rest.trim_start(),
        _ => part,
    }
}

/// What a stop needs, as its label names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The work is done, and a check ran after the last edit.
    DoneChecked,
    /// The work is reported done with no check behind it.
    DoneClaimed,
    /// The agent asks the user something.
    AsksYou,
    /// The agent cannot go on.
    Blocked,
    /// The agent says it goes on by itself.
    StillGoing,
    /// The user interrupted the turn.
    Interrupted,
}

impl Kind {
    /// The kind's label value, which is also its option in the question.
    #[must_use]
    pub const fn value(self) -> &'static str {
        match self {
            Self::DoneChecked => "done_checked",
            Self::DoneClaimed => "done_claimed",
            Self::AsksYou => "asks_you",
            Self::Blocked => "blocked",
            Self::StillGoing => "still_going",
            Self::Interrupted => "interrupted",
        }
    }

    /// The kind a label value or an answer's option names; `cannot_tell` and anything else name
    /// none.
    #[must_use]
    pub fn from_value(value: &str) -> Option<Self> {
        [
            Self::DoneChecked,
            Self::DoneClaimed,
            Self::AsksYou,
            Self::Blocked,
            Self::StillGoing,
            Self::Interrupted,
        ]
        .into_iter()
        .find(|kind| kind.value() == value)
    }

    /// The kind as a row says it, such as `done · checked`.
    #[must_use]
    pub const fn words(self) -> &'static str {
        match self {
            Self::DoneChecked => "done · checked",
            Self::DoneClaimed => "done · claimed",
            Self::AsksYou => "asks you",
            Self::Blocked => "blocked",
            Self::StillGoing => "still going",
            Self::Interrupted => "interrupted",
        }
    }
}

/// `kind` held to what code saw: a turn that ran no command after its last edit is not
/// `done_checked`, whatever the model read. The model can take a check away, never add one.
#[must_use]
pub const fn apply_evidence(kind: Kind, facts: &TurnFacts) -> Kind {
    match kind {
        Kind::DoneChecked if !facts.checked_after_edit => Kind::DoneClaimed,
        kind => kind,
    }
}

/// Who decided a stop's kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// Code, from the turn's events.
    Rules,
    /// The System One layer.
    Model,
}

impl Source {
    const fn value(self) -> &'static str {
        match self {
            Self::Rules => "rules",
            Self::Model => "model",
        }
    }
}

/// The labels a stop lands on its seat: its kind with who decided it and how sure, when there is
/// one, and the parts of the prompt the message does not cover, when there are any.
#[must_use]
pub fn labels(
    kind: Option<(Kind, Source, f64)>,
    missing_parts: &[String],
) -> Vec<(&'static str, String)> {
    let mut labels = Vec::new();
    if let Some((kind, source, confidence)) = kind {
        labels.push((STOP_KIND_LABEL, kind.value().to_string()));
        labels.push((STOP_KIND_SOURCE_LABEL, source.value().to_string()));
        labels.push((STOP_KIND_CONFIDENCE_LABEL, format!("{confidence:.2}")));
    }
    if !missing_parts.is_empty() {
        let parts = serde_json::Value::from(missing_parts.to_vec());
        labels.push((STOP_PARTS_MISSING_LABEL, parts.to_string()));
    }
    labels
}

/// How a stop's kind shows on its row, from the use's mode: not at all (`off`, and `shadow`,
/// which only logs), after `idle` with a question mark (`suggest`), or in place of `idle` (`act`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StopKindShown {
    /// The row says `idle`.
    #[default]
    Hidden,
    /// The row says `idle · <kind>?`.
    Suggest,
    /// The row says the kind.
    Act,
}

/// The state word of an idle seat's row whose `labels` hold a stop kind, as `shown`; `None`
/// leaves the row at `idle`.
#[must_use]
pub fn row_word(labels: &BTreeMap<String, String>, shown: StopKindShown) -> Option<String> {
    let kind = labels
        .get(STOP_KIND_LABEL)
        .and_then(|value| Kind::from_value(value))?;
    match shown {
        StopKindShown::Hidden => None,
        StopKindShown::Suggest => Some(format!("idle · {}?", kind.words())),
        StopKindShown::Act => Some(kind.words().to_string()),
    }
}

/// The parts of the prompt an idle seat's `labels` say the message does not cover, as its row
/// says them: `not covered: "Add a license."`.
#[must_use]
pub fn not_covered(labels: &BTreeMap<String, String>, shown: StopKindShown) -> Option<String> {
    if shown == StopKindShown::Hidden {
        return None;
    }
    let parts: Vec<String> = labels
        .get(STOP_PARTS_MISSING_LABEL)
        .and_then(|parts| serde_json::from_str(parts).ok())?;
    if parts.is_empty() {
        return None;
    }
    let quoted: Vec<String> = parts.iter().map(|part| format!("\"{part}\"")).collect();
    Some(format!("not covered: {}", quoted.join(", ")))
}

/// The facts of `facts` a stop's state carries, at `now_ms`.
///
/// They are the tools by name, the failures, the permissions still pending, whether a command
/// ran after the last edit, and how long the turn took, in words.
#[must_use]
pub fn state_facts(facts: &TurnFacts, now_ms: u64) -> Vec<(&'static str, String)> {
    let mut tools: Vec<String> = facts
        .tools
        .iter()
        .map(|(tool, count)| format!("{tool} {count}"))
        .collect();
    if facts.subagent_tools > 0 {
        tools.push(format!("subagents' tools {}", facts.subagent_tools));
    }
    let tools = if tools.is_empty() {
        "none".to_string()
    } else {
        tools.join(", ")
    };
    let checked = if facts.checked_after_edit {
        "yes"
    } else {
        "no"
    };
    vec![
        ("agent", "Claude Code".to_string()),
        ("tools", tools),
        ("failed tools", facts.failures.to_string()),
        ("permissions pending", facts.pending.len().to_string()),
        ("checked after edit", checked.to_string()),
        (
            "turn",
            // A request with no prompt the user typed, as after a resume, has no start.
            if facts.started_ms == 0 {
                "unknown".to_string()
            } else {
                duration_words(now_ms.saturating_sub(facts.started_ms))
            },
        ),
    ]
}

/// A turn's length in words, since a phrase reads better to a model than a count of
/// milliseconds: `under a minute`, `4 minutes`, `2 hours`.
fn duration_words(milliseconds: u64) -> String {
    let minutes = milliseconds / 60_000;
    match minutes {
        0 => "under a minute".to_string(),
        1 => "1 minute".to_string(),
        2..=59 => format!("{minutes} minutes"),
        60..=119 => "1 hour".to_string(),
        _ => format!("{} hours", minutes / 60),
    }
}
