//! The page picker's pure part (#654): the list `brain_list_pages` answers, the pages Marley
//! opened most recently, the order on an empty query, and the merge of the title and slug
//! matches.

use std::collections::HashMap;

use serde::Deserialize;

use crate::page::NewPage;

/// Rusty's tool for every page's summary.
pub const BRAIN_LIST_PAGES: &str = "brain_list_pages";

/// The `limit` the picker asks for: every page, as Rusty's own app asks (Rusty's default is 50).
pub const LIST_LIMIT: u64 = 100_000;

/// The most recently opened pages kept.
pub const RECENT_CAP: usize = 20;

/// The most rows a query shows.
pub const MATCH_CAP: usize = 100;

/// One page as `brain_list_pages` lists it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PageSummary {
    /// The page's slug, its path in the vault without `.md`.
    pub slug: String,
    /// Its title, empty when Rusty has none.
    #[serde(default)]
    pub title: String,
}

impl PageSummary {
    /// The title to show: the page's own, else its file's name.
    #[must_use]
    pub fn shown_title(&self) -> &str {
        if self.title.trim().is_empty() {
            crate::vault::name_of(&self.slug)
        } else {
            &self.title
        }
    }
}

/// Reads `brain_list_pages`' answer: the pages, the most recently updated first.
///
/// # Errors
///
/// When the answer is not a list of pages.
pub fn parse_page_list(text: &str) -> Result<Vec<PageSummary>, serde_json::Error> {
    serde_json::from_str(text)
}

/// The pages Marley opened most recently, newest first.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RecentPages(Vec<String>);

impl RecentPages {
    /// Puts `slug` first, once, keeping at most [`RECENT_CAP`].
    pub fn visit(&mut self, slug: &str) {
        self.0.retain(|seen| seen != slug);
        self.0.insert(0, slug.to_string());
        self.0.truncate(RECENT_CAP);
    }

    /// The list as Marley stored it; anything else reads as no list.
    #[must_use]
    pub fn from_json(text: &str) -> Self {
        let mut slugs: Vec<String> = serde_json::from_str(text).unwrap_or_default();
        slugs.truncate(RECENT_CAP);
        Self(slugs)
    }

    /// The list as Marley stores it.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::Value::from(self.0.clone()).to_string()
    }

    /// Where `slug` stands in the list, 0 the newest.
    #[must_use]
    pub fn rank(&self, slug: &str) -> Option<usize> {
        self.0.iter().position(|seen| seen == slug)
    }

    /// The slugs, newest first.
    #[must_use]
    pub fn slugs(&self) -> &[String] {
        &self.0
    }
}

/// The rows on an empty query, as indexes into the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    /// The rows in order.
    pub rows: Vec<usize>,
    /// The row a separator follows: the last of the recent group.
    pub separator_after: Option<usize>,
    /// The row selected first.
    pub selected: usize,
}

/// The empty query's rows: the active tab's page, the recently opened pages, then the rest.
///
/// The recent group holds the pages the list holds, newest first, and a separator follows it; the
/// rest keep the list's order. With the active page first and another row after it, the selection
/// starts on that row, so Enter goes back.
#[must_use]
pub fn empty_order(pages: &[PageSummary], recent: &RecentPages, active: Option<&str>) -> Order {
    let index: HashMap<&str, usize> = pages
        .iter()
        .enumerate()
        .map(|(row, page)| (page.slug.as_str(), row))
        .collect();
    let mut group: Vec<usize> = Vec::new();
    for slug in active
        .into_iter()
        .chain(recent.slugs().iter().map(String::as_str))
    {
        if let Some(&row) = index.get(slug)
            && !group.contains(&row)
        {
            group.push(row);
        }
    }
    let rest: Vec<usize> = (0..pages.len())
        .filter(|row| !group.contains(row))
        .collect();
    let separator_after = (!group.is_empty() && group.len() < pages.len()).then(|| group.len() - 1);
    let active_first = active.is_some_and(|slug| {
        group
            .first()
            .and_then(|&row| pages.get(row))
            .is_some_and(|page| page.slug == slug)
    });
    let mut rows = group;
    rows.extend(rest);
    let selected = usize::from(active_first && rows.len() > 1);
    Order {
        rows,
        separator_after,
        selected,
    }
}

/// One field's match of a page: the page's index, the score and the matched bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct Matched {
    /// The page's index in the list.
    pub page: usize,
    /// The matcher's score, higher better.
    pub score: f64,
    /// The matched bytes of the field.
    pub positions: Vec<usize>,
}

/// One row of a query's matches.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    /// The page's index in the list.
    pub page: usize,
    /// The better of its two fields' scores.
    pub score: f64,
    /// The matched bytes of its title, when the title gave the score.
    pub title_positions: Vec<usize>,
    /// The matched bytes of its slug, when the slug gave the score.
    pub slug_positions: Vec<usize>,
}

/// One row per page from the title and the slug matches, at most `cap`.
///
/// Each page keeps its better score and is lit in the field that gave it (both on a tie); the rows
/// go best first, then the more recently opened, then in the list's order.
#[must_use]
pub fn merge(
    titles: Vec<Matched>,
    slugs: Vec<Matched>,
    pages: &[PageSummary],
    recent: &RecentPages,
    cap: usize,
) -> Vec<Hit> {
    let mut hits: HashMap<usize, Hit> = HashMap::new();
    for (matched, is_title) in titles
        .into_iter()
        .map(|matched| (matched, true))
        .chain(slugs.into_iter().map(|matched| (matched, false)))
    {
        let hit = hits.entry(matched.page).or_insert_with(|| Hit {
            page: matched.page,
            score: f64::NEG_INFINITY,
            title_positions: Vec::new(),
            slug_positions: Vec::new(),
        });
        if matched.score > hit.score {
            hit.score = matched.score;
            hit.title_positions.clear();
            hit.slug_positions.clear();
        }
        if matched.score >= hit.score {
            if is_title {
                hit.title_positions = matched.positions;
            } else {
                hit.slug_positions = matched.positions;
            }
        }
    }
    let recency = |hit: &Hit| {
        pages
            .get(hit.page)
            .and_then(|page| recent.rank(&page.slug))
            .unwrap_or(usize::MAX)
    };
    let mut hits: Vec<Hit> = hits.into_values().collect();
    hits.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| recency(a).cmp(&recency(b)))
            .then_with(|| a.page.cmp(&b.page))
    });
    hits.truncate(cap);
    hits
}

/// The page the query would make: its path, when no listed page has that slug.
#[must_use]
pub fn create_target(query: &str, pages: &[PageSummary]) -> Option<NewPage> {
    NewPage::from_target(query)
        .filter(|new_page| pages.iter().all(|page| page.slug != new_page.path))
}
