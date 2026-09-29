//! PURE — a block's output lines filtered by a query (#528).
//!
//! The query is text or a regular expression, its case ignored unless asked, inverted, and with
//! context lines around each line it picks, in grep's `-v` and `-C` meanings. Nothing is deleted:
//! the caller shows what comes back beside the block, whose own lines stay as they are.

use std::fmt;
use std::ops::Range;

use regex::RegexBuilder;

/// What a filter keeps.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilterQuery {
    /// The text or the pattern; an empty one keeps every line.
    pub text: String,
    /// Whether `text` is a regular expression rather than plain text.
    pub regex: bool,
    /// Whether the case must match.
    pub case_sensitive: bool,
    /// Whether to keep the lines that do not match instead.
    pub invert: bool,
    /// How many lines to keep before and after each line the query picks.
    pub context: usize,
}

/// One row of a filtered list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilteredRow {
    /// A line of the block.
    Line {
        /// Its place in the block's output, from 0.
        index: usize,
        /// Its text.
        text: String,
        /// Whether the query picked it, rather than keeping it as context.
        picked: bool,
        /// Where the query matched in it, in bytes; none for an inverted query.
        matches: Vec<Range<usize>>,
    },
    /// A break between groups of lines that do not touch, grep's `--`.
    Gap,
}

/// A block's lines after a filter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filtered {
    /// The kept lines, with a gap between groups when there is context.
    pub rows: Vec<FilteredRow>,
    /// How many lines the query picked.
    pub picked: usize,
    /// How many lines the block's output has.
    pub total: usize,
}

/// A query the filter cannot use: its regular expression does not parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterError(pub String);

impl fmt::Display for FilterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for FilterError {}

/// Filters the lines of `output` by `query`.
///
/// # Errors
///
/// [`FilterError`] when `query` is a regular expression that does not parse.
pub fn filter_lines(output: &str, query: &FilterQuery) -> Result<Filtered, FilterError> {
    let lines: Vec<&str> = output.lines().collect();
    let total = lines.len();
    let matcher = if query.text.is_empty() {
        None
    } else {
        let pattern = if query.regex {
            query.text.clone()
        } else {
            regex::escape(&query.text)
        };
        let matcher = RegexBuilder::new(&pattern)
            .case_insensitive(!query.case_sensitive)
            .build()
            .map_err(|error| FilterError(error.to_string()))?;
        Some(matcher)
    };
    let picked: Vec<bool> = lines
        .iter()
        .map(|line| {
            matcher
                .as_ref()
                .is_none_or(|matcher| matcher.is_match(line) != query.invert)
        })
        .collect();
    let mut kept = vec![false; total];
    for (index, _) in picked.iter().enumerate().filter(|(_, picked)| **picked) {
        let start = index.saturating_sub(query.context);
        let end = index
            .saturating_add(query.context)
            .saturating_add(1)
            .min(total);
        for keep in kept.iter_mut().skip(start).take(end - start) {
            *keep = true;
        }
    }
    let mut rows = Vec::new();
    let mut previous: Option<usize> = None;
    for (index, (line, (keep, picked))) in lines.iter().zip(kept.iter().zip(&picked)).enumerate() {
        if !keep {
            continue;
        }
        if query.context > 0 && previous.is_some_and(|previous| index > previous + 1) {
            rows.push(FilteredRow::Gap);
        }
        let found = match &matcher {
            Some(matcher) if *picked && !query.invert => {
                matcher.find_iter(line).map(|hit| hit.range()).collect()
            }
            _ => Vec::new(),
        };
        rows.push(FilteredRow::Line {
            index,
            text: (*line).to_string(),
            picked: *picked,
            matches: found,
        });
        previous = Some(index);
    }
    Ok(Filtered {
        rows,
        picked: picked.iter().filter(|picked| **picked).count(),
        total,
    })
}
