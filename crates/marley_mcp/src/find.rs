//! The find tools' match by words (#567): `browser_find` and `terminal_find` look for an item
//! whose text holds every word of the query before anything is asked of a model. Pure.

use std::collections::HashSet;
use std::ops::Range;

/// Words a query carries that say nothing of what it looks for, left out of the match.
const STOP_WORDS: [&str; 22] = [
    "a", "an", "and", "are", "at", "did", "do", "does", "for", "is", "it", "of", "on", "or",
    "that", "the", "this", "to", "was", "what", "where", "which",
];

/// What the words of a query find among the items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Local {
    /// One item holds every word: its index.
    Sure(usize),
    /// Several do: their indexes, in order.
    Several(Vec<usize>),
    /// None does, or the query has no word to look for.
    Nothing,
}

/// The words of `text`, lowercased, split at everything but letters and digits.
fn words_of(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
}

/// The words of `query` the match looks for: its words without the stop words.
#[must_use]
pub fn words(query: &str) -> Vec<String> {
    words_of(query)
        .filter(|word| !STOP_WORDS.contains(&word.as_str()))
        .collect()
}

/// The items among `items` that hold every one of `words` as a whole word. No word matches
/// nothing.
#[must_use]
pub fn matches(items: &[String], words: &[String]) -> Vec<usize> {
    if words.is_empty() {
        return Vec::new();
    }
    items
        .iter()
        .enumerate()
        .filter(|(_, item)| {
            let held: HashSet<String> = words_of(item).collect();
            words.iter().all(|word| held.contains(word))
        })
        .map(|(index, _)| index)
        .collect()
}

/// What `query`'s words find among `items`.
#[must_use]
pub fn local(items: &[String], query: &str) -> Local {
    let found = matches(items, &words(query));
    match found.as_slice() {
        [] => Local::Nothing,
        [one] => Local::Sure(*one),
        _ => Local::Several(found),
    }
}

/// `count` items cut into windows of at most `size`, in order. A size of 0 is taken as 1.
#[must_use]
pub fn windows(count: usize, size: usize) -> Vec<Range<usize>> {
    let size = size.max(1);
    (0..count)
        .step_by(size)
        .map(|start| start..count.min(start + size))
        .collect()
}
