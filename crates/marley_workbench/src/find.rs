//! What the find tools share (#567): `browser_find` finds an element of a page and
//! `terminal_find` a line of a block, each from a query in words.
//!
//! The query's words come first ("local first"): an item that alone holds every one of them is
//! the answer, and no call is made. What the words leave open goes to the System One layer, by
//! the use's mode, as a choice over the items and a noul on whether anything matches. A layer
//! holds at most [`FIND_WINDOW`] items in a question, so more go in windows, a request each,
//! asked together. The answer never acts: the agent decides what to do with it.

use std::path::PathBuf;
use std::time::Duration;

use futures::future::join_all;
use gpui::AsyncApp;
use marley_mcp::find::{self, Local};
use marley_system_one::policy::Refusal;
use marley_system_one::reading::{Reading, Signal};
use marley_system_one::request::Answer;
use marley_system_one::state::Detail;
use marley_system_one::{FIND_WINDOW, UseSpec, find_label, find_set};
use settings::SystemOneMode;

use crate::system_one::{self, Asked, Asking};

/// How long a find waits on the model: the agent's own call waits on it (#567 D8).
const DEADLINE: Duration = Duration::from_secs(3);

/// The most candidates a find answers.
const MAX_CANDIDATES: usize = 3;

/// The project a find looks in, which the layer's lists allow or refuse.
pub(crate) struct Place {
    /// The project's name.
    pub(crate) project: String,
    /// Its folders.
    pub(crate) folders: Vec<PathBuf>,
    /// Whether it is on this machine.
    pub(crate) local: bool,
}

/// What a find found, by the items' indexes.
pub(crate) struct Found {
    /// What found it: `rules` (the query's words), `model` or `none`.
    pub(crate) source: &'static str,
    /// Whether the answer can be acted on without a look.
    pub(crate) sure: bool,
    /// The item to act on, when the answer is sure.
    pub(crate) top: Option<usize>,
    /// The candidates, best first, with the model's probability when it gave one.
    pub(crate) candidates: Vec<(usize, Option<f64>)>,
    /// The model's reading of whether anything matches, when it answered.
    pub(crate) present: Option<&'static str>,
    /// Whether the candidates are the model's suggestions, to look at before acting.
    pub(crate) verify: bool,
    /// Why the model was not asked or did not answer.
    pub(crate) note: Option<String>,
}

impl Found {
    /// The answer from the query's words alone: the items that hold them, none of them sure.
    fn by_words(candidates: &[usize], note: Option<String>) -> Self {
        Self {
            source: if candidates.is_empty() {
                "none"
            } else {
                "rules"
            },
            sure: false,
            top: None,
            candidates: candidates
                .iter()
                .take(MAX_CANDIDATES)
                .map(|index| (*index, None))
                .collect(),
            present: None,
            verify: false,
            note,
        }
    }
}

/// Finds `query` among `items` for the use `name` (`browser_find` or `terminal_find`), in
/// `place`, with `subject` naming what is searched for #565's repeat check.
pub(crate) async fn find_items(
    name: &'static str,
    subject: &str,
    query: &str,
    items: &[String],
    place: Place,
    cx: &AsyncApp,
) -> Found {
    let (asked, candidates) = match find::local(items, query) {
        // One item holds every word: code's answer, with no call.
        Local::Sure(index) => {
            return Found {
                source: "rules",
                sure: true,
                top: Some(index),
                candidates: vec![(index, None)],
                present: None,
                verify: false,
                note: None,
            };
        }
        // Several do: the model ranks them alone.
        Local::Several(indexes) => (indexes.clone(), indexes),
        // None does: the model looks among them all.
        Local::Nothing => ((0..items.len()).collect(), Vec::new()),
    };
    let mode = cx.update(|cx| system_one::use_mode(name, cx));
    if mode == SystemOneMode::Off {
        return Found::by_words(&candidates, Some(format!("{name} is off")));
    }
    if asked.is_empty() {
        return Found::by_words(&candidates, None);
    }
    let place_of = |subject: String, texts: Vec<(&'static str, String)>| Asking {
        subject,
        project: place.project.clone(),
        folders: place.folders.clone(),
        local: place.local,
        facts: Vec::new(),
        texts,
        verdict: None,
    };
    let detail = cx.update(|cx| system_one::detail(&place_of(String::new(), Vec::new()), cx));
    match detail {
        Ok(Detail::Full) => {}
        Ok(Detail::Facts) => {
            return Found::by_words(
                &candidates,
                Some(
                    "System One: a metadata-only project sends no text, and a find's items are \
                     text"
                        .to_string(),
                ),
            );
        }
        Err(Refusal::Refused(reason) | Refusal::Unavailable(reason)) => {
            return Found::by_words(&candidates, Some(format!("System One: {reason}")));
        }
    }
    let mut asks = Vec::new();
    for (number, range) in find::windows(asked.len(), FIND_WINDOW)
        .into_iter()
        .enumerate()
    {
        let Some((window, set)) = asked
            .get(range)
            .and_then(|window| Some((window.to_vec(), find_set(window.len())?)))
        else {
            continue;
        };
        let texts = std::iter::once(("query", query.to_string()))
            .chain(window.iter().enumerate().filter_map(|(position, index)| {
                Some((find_label(position + 1)?, items.get(*index)?.clone()))
            }))
            .collect();
        let asking = place_of(format!("{subject}:{number}"), texts);
        let spec = UseSpec {
            name,
            set,
            deadline: DEADLINE,
        };
        let task = cx.update(|cx| system_one::ask(spec, &asking, cx));
        asks.push((window, task));
    }
    // Shadow answers from the words alone; the asks go on, and System One calls shows what they read.
    if mode == SystemOneMode::Shadow {
        for (_, task) in asks {
            task.detach();
        }
        return Found::by_words(&candidates, None);
    }
    let answered = join_all(
        asks.into_iter()
            .map(|(window, task)| async move { (window, task.await) }),
    )
    .await;
    read_windows(&answered, &candidates, mode)
}

/// What the windows' answers say, in `mode`: whether anything matches, the candidates by the
/// model's probability, and in `act` the item to act on when the model found one and chose it
/// with confidence.
fn read_windows(
    answered: &[(Vec<usize>, Asked)],
    candidates: &[usize],
    mode: SystemOneMode,
) -> Found {
    let mut read = 0;
    let mut found = false;
    let mut every_absent = true;
    let mut reason = None;
    let mut chosen: Option<(usize, f64)> = None;
    let mut scored: Vec<(usize, f64)> = Vec::new();
    for (window, asked) in answered {
        let reads = match &asked.reading {
            Reading::Model(reads) => reads,
            Reading::Refused(why) | Reading::Unavailable(why) => {
                reason.get_or_insert_with(|| format!("System One: {why}"));
                every_absent = false;
                continue;
            }
            // A find has no rules of its own for the `rules` provider to answer with.
            Reading::Rules(_) => {
                reason.get_or_insert_with(|| {
                    "System One's provider is each use's own rules, and a find has none".to_string()
                });
                every_absent = false;
                continue;
            }
            Reading::Off => {
                every_absent = false;
                continue;
            }
        };
        read += 1;
        let item = |option: &str| {
            option
                .parse::<usize>()
                .ok()
                .and_then(|label| label.checked_sub(1))
                .and_then(|position| window.get(position))
                .copied()
        };
        for each in reads {
            match (each.key.as_str(), &each.signal) {
                ("present", Signal::Noul { holds, .. }) => {
                    found |= *holds;
                    every_absent &= !*holds;
                }
                ("present", _) => every_absent = false,
                ("which", Signal::Choice { option, confidence }) => {
                    if let Some(index) = item(option.as_str())
                        && chosen.is_none_or(|(_, best)| *confidence > best)
                    {
                        chosen = Some((index, *confidence));
                    }
                }
                _ => {}
            }
        }
        if let Some(Answer::Choice { probabilities, .. }) = asked
            .answers
            .as_ref()
            .and_then(|answers| answers.by_key.get("which"))
        {
            scored.extend(
                probabilities.iter().filter_map(|(option, probability)| {
                    Some((item(option.as_str())?, *probability))
                }),
            );
        }
    }
    if read == 0 {
        return Found::by_words(
            candidates,
            Some(reason.unwrap_or_else(|| "System One gave no answer".to_string())),
        );
    }
    scored.sort_by(|left, right| right.1.total_cmp(&left.1));
    let mut ranked: Vec<(usize, Option<f64>)> = scored
        .into_iter()
        .take(MAX_CANDIDATES)
        .map(|(index, probability)| (index, Some(probability)))
        .collect();
    if ranked.is_empty()
        && let Some((index, confidence)) = chosen
    {
        ranked.push((index, Some(confidence)));
    }
    let present = if found {
        "found"
    } else if every_absent {
        "absent"
    } else {
        "unsure"
    };
    // A choice reads as one only at the layer's confidence floor, so `chosen` is confident.
    let top = chosen
        .filter(|_| found && mode == SystemOneMode::Act)
        .map(|(index, _)| index);
    Found {
        source: "model",
        sure: top.is_some(),
        top,
        candidates: ranked,
        present: Some(present),
        verify: mode == SystemOneMode::Suggest,
        note: reason,
    }
}
