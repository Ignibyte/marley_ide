//! The project join (#655): a workspace's folders to a brain project page, the page's task group,
//! its follow-ups due and its summary.
//!
//! A project page lists the folders it belongs to in its `path:` property; the first tier takes
//! the page that lists one of the project's folders, the second the one page named like a folder
//! by Rusty's slug rule. Both read only what Rusty answered and the home folder the caller passes,
//! so nothing here touches the disk.

use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;
use serde_json::{Map, Value};

use crate::decisions::DecisionSummary;
use crate::knowledge::PageLinks;
use crate::tasks::TaskGroup;

/// Rusty's tool for one page with its frontmatter.
pub const BRAIN_READ_PAGE: &str = "brain_read_page";

/// Rusty's tool for one frontmatter property's change.
pub const BRAIN_SET_PROPERTY: &str = "brain_set_property";

/// The page type of a project page.
pub const PROJECT_TYPE: &str = "project";

/// The most project pages listed: Rusty's default of 50 would miss some.
pub const PAGE_LIMIT: u64 = 1000;

/// The property listing a project page's folders.
pub const PATH_KEY: &str = "path";

/// The property naming a project page's task groups.
pub const TASK_GROUP_KEY: &str = "task_group";

/// The property holding a project page's summary.
pub const SUMMARY_KEY: &str = "summary";

/// The properties `brain_list_pages` is asked for.
pub const PROPERTIES: [&str; 3] = [PATH_KEY, TASK_GROUP_KEY, SUMMARY_KEY];

/// The longest summary shown, in characters.
const SUMMARY_CHARS: usize = 300;

/// One project page as `brain_list_pages` lists it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ListedPage {
    /// The page's slug.
    pub slug: String,
    /// Its title.
    #[serde(default)]
    pub title: String,
    /// Its aliases, from a Rusty that lists them.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// When Rusty last saw it change.
    #[serde(default)]
    pub updated_at: Value,
    /// The properties asked for, from a Rusty that answers them; absent from an older one.
    #[serde(default)]
    pub properties: Option<Map<String, Value>>,
}

/// `brain_list_pages`' answer.
///
/// # Errors
///
/// When the answer is not a list of pages.
pub fn listed_from_answer(text: &str) -> Result<Vec<ListedPage>, serde_json::Error> {
    serde_json::from_str(text)
}

/// The parts of `brain_read_page`'s answer the project view reads.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PageRead {
    /// The page's slug.
    pub slug: String,
    /// Its title.
    #[serde(default)]
    pub title: String,
    /// Its body above the timeline.
    #[serde(default)]
    pub compiled_truth: String,
    /// Its frontmatter, the extra keys among the rest.
    #[serde(default)]
    pub frontmatter: Map<String, Value>,
}

/// `brain_read_page`'s answer: `None` for Rusty's `null`, a page that does not exist.
///
/// # Errors
///
/// When the answer is neither a page nor `null`.
pub fn read_from_answer(text: &str) -> Result<Option<PageRead>, serde_json::Error> {
    serde_json::from_str(text)
}

/// A `path:` value as the page holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathValue {
    /// No `path:`, or an empty one.
    Absent,
    /// Text, its parts split on commas.
    Text(String),
    /// A list of text.
    List(Vec<String>),
    /// Anything else, which no link changes.
    Other,
}

impl PathValue {
    /// The value `value` holds.
    #[must_use]
    pub fn from_value(value: Option<&Value>) -> Self {
        match value {
            None | Some(Value::Null) => Self::Absent,
            Some(Value::String(text)) if text.trim().is_empty() => Self::Absent,
            Some(Value::String(text)) => Self::Text(text.clone()),
            Some(Value::Array(items)) => items
                .iter()
                .map(|item| item.as_str().map(str::to_string))
                .collect::<Option<Vec<_>>>()
                .map_or(Self::Other, Self::List),
            Some(_) => Self::Other,
        }
    }

    fn parts(&self) -> Vec<String> {
        match self {
            Self::Text(text) => text.split(',').map(clean_part).collect(),
            Self::List(items) => items.iter().map(|item| clean_part(item)).collect(),
            Self::Absent | Self::Other => Vec::new(),
        }
    }
}

/// A part trimmed and stripped of quotes.
fn clean_part(part: &str) -> String {
    part.trim()
        .trim_matches(|character| character == '"' || character == '\'')
        .trim()
        .to_string()
}

/// One project page, as the join reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectPage {
    /// Its slug.
    pub slug: String,
    /// Its title.
    pub title: String,
    /// Its aliases.
    pub aliases: Vec<String>,
    /// Its `path:`.
    pub path: PathValue,
    /// The groups its `task_group` names.
    pub task_groups: Vec<String>,
    /// Its `summary:`, when it has one.
    pub summary: Option<String>,
}

impl ProjectPage {
    /// The page from its listing, when the listing carries the properties; `None` from a Rusty
    /// that does not answer them.
    #[must_use]
    pub fn from_listed(listed: &ListedPage) -> Option<Self> {
        let properties = listed.properties.as_ref()?;
        Some(Self::from_properties(
            &listed.slug,
            &listed.title,
            listed.aliases.clone(),
            properties,
        ))
    }

    /// The page from its read.
    #[must_use]
    pub fn from_read(read: &PageRead) -> Self {
        let aliases = text_items(read.frontmatter.get("aliases"));
        Self::from_properties(&read.slug, &read.title, aliases, &read.frontmatter)
    }

    fn from_properties(
        slug: &str,
        title: &str,
        aliases: Vec<String>,
        properties: &Map<String, Value>,
    ) -> Self {
        let summary = properties
            .get(SUMMARY_KEY)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|summary| !summary.is_empty())
            .map(str::to_string);
        Self {
            slug: slug.to_string(),
            title: title.to_string(),
            aliases,
            path: PathValue::from_value(properties.get(PATH_KEY)),
            task_groups: text_items(properties.get(TASK_GROUP_KEY)),
            summary,
        }
    }

    /// The page's names for the name tier: the slug's last part, the title and the aliases.
    fn name_keys(&self) -> HashSet<String> {
        let last = self.slug.rsplit('/').next().unwrap_or(&self.slug);
        std::iter::once(last)
            .chain(std::iter::once(self.title.as_str()))
            .chain(self.aliases.iter().map(String::as_str))
            .filter_map(name_key)
            .collect()
    }
}

/// A text value as one item, a list of text as its items, anything else as none.
fn text_items(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::String(text)) if !text.trim().is_empty() => vec![text.trim().to_string()],
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

/// `path` with `.` dropped, `..` taken back and a trailing `/` gone, read without the disk.
#[must_use]
pub fn lexical(path: &Path) -> PathBuf {
    let mut normal = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let _above = normal.pop();
            }
            other => normal.push(other.as_os_str()),
        }
    }
    normal
}

/// The parts of `value` that are paths on this machine: absolute, or `~` and under it, `~` being
/// `home`; anything else, such as another machine's path in words, is none.
#[must_use]
pub fn local_paths(value: &PathValue, home: &Path) -> Vec<PathBuf> {
    value
        .parts()
        .into_iter()
        .filter_map(|part| {
            if part.starts_with('/') {
                Some(lexical(Path::new(&part)))
            } else if part == "~" {
                Some(lexical(home))
            } else {
                part.strip_prefix("~/")
                    .map(|rest| lexical(&home.join(rest)))
            }
        })
        .collect()
}

/// Rusty's slug rule for a title: lowercase, letters and digits kept, space, `_` and `-` as one
/// `-`, anything else dropped, the ends trimmed; `None` when nothing is left.
#[must_use]
pub fn name_key(text: &str) -> Option<String> {
    let mut key = String::new();
    for character in text.to_lowercase().chars() {
        if character.is_alphanumeric() {
            key.push(character);
        } else if matches!(character, ' ' | '_' | '-') && !key.ends_with('-') && !key.is_empty() {
            key.push('-');
        }
    }
    let key = key.trim_end_matches('-').to_string();
    (!key.is_empty()).then_some(key)
}

/// A workspace's project: its folders and whether they are on another machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    /// The project's folders.
    pub folders: Vec<PathBuf>,
    /// Whether the folders are on another machine.
    pub remote: bool,
}

impl Project {
    /// The project's names: each folder's last part, a `.git` extension dropped.
    #[must_use]
    pub fn names(&self) -> Vec<String> {
        self.folders
            .iter()
            .filter_map(|folder| folder.file_name())
            .map(|name| {
                let name = name.to_string_lossy();
                name.strip_suffix(".git").unwrap_or(&name).to_string()
            })
            .collect()
    }
}

/// How a page matched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Matched {
    /// Its `path:` lists a folder.
    Path,
    /// It is named like a folder.
    Name,
}

/// What a project resolves to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// One page.
    Page {
        /// Its slug.
        slug: String,
        /// How it matched.
        by: Matched,
        /// The other pages that list a folder too.
        also: Vec<String>,
    },
    /// Several pages named like a folder, none chosen.
    Candidates {
        /// The name they share.
        name: String,
        /// Their slugs.
        slugs: Vec<String>,
    },
    /// No page.
    Unmatched {
        /// The folders looked for.
        folders: Vec<PathBuf>,
        /// The names looked for.
        names: Vec<String>,
    },
}

/// The page `project` belongs to among `pages`: by `path:`, else by name; a page under `archive/`
/// is never one. A remote project matches by name alone.
#[must_use]
pub fn resolve(project: &Project, pages: &[ProjectPage], home: &Path) -> Resolution {
    let pages: Vec<&ProjectPage> = pages
        .iter()
        .filter(|page| !page.slug.starts_with("archive/"))
        .collect();
    let names = project.names();
    let keys: HashSet<String> = names.iter().filter_map(|name| name_key(name)).collect();
    let is_named = |page: &ProjectPage| !page.name_keys().is_disjoint(&keys);
    if !project.remote {
        let folders: Vec<PathBuf> = project
            .folders
            .iter()
            .map(|folder| lexical(folder))
            .collect();
        let mut listing: Vec<&ProjectPage> = pages
            .iter()
            .copied()
            .filter(|page| {
                local_paths(&page.path, home)
                    .iter()
                    .any(|path| folders.contains(path))
            })
            .collect();
        listing.sort_by(|a, b| a.slug.cmp(&b.slug));
        let chosen = listing
            .iter()
            .position(|page| is_named(page))
            .unwrap_or_default();
        if let Some(page) = listing.get(chosen) {
            let also = listing
                .iter()
                .filter(|other| other.slug != page.slug)
                .map(|other| other.slug.clone())
                .collect();
            return Resolution::Page {
                slug: page.slug.clone(),
                by: Matched::Path,
                also,
            };
        }
    }
    let mut candidates: Vec<String> = pages
        .iter()
        .filter(|page| is_named(page))
        .map(|page| page.slug.clone())
        .collect();
    candidates.sort();
    match candidates.len() {
        0 => Resolution::Unmatched {
            folders: project.folders.clone(),
            names,
        },
        1 => Resolution::Page {
            slug: candidates.remove(0),
            by: Matched::Name,
            also: Vec::new(),
        },
        _ => Resolution::Candidates {
            name: names
                .first()
                .and_then(|name| name_key(name))
                .unwrap_or_default(),
            slugs: candidates,
        },
    }
}

/// The `path:` value that adds `folders` to `value`, keeping what was there; `None` when every
/// folder is listed already.
///
/// # Errors
///
/// When `value` is neither text nor a list, which a link does not change.
pub fn path_value_with(
    slug: &str,
    value: &PathValue,
    folders: &[PathBuf],
) -> Result<Option<Value>, String> {
    let listed: Vec<PathBuf> = value
        .parts()
        .iter()
        .map(|part| lexical(Path::new(part)))
        .collect();
    let missing: Vec<String> = folders
        .iter()
        .filter(|folder| !listed.contains(&lexical(folder)))
        .map(|folder| folder.to_string_lossy().into_owned())
        .collect();
    if missing.is_empty() && *value != PathValue::Absent {
        return Ok(None);
    }
    match value {
        PathValue::Absent => Ok(Some(Value::String(missing.join(", ")))),
        PathValue::Text(text) => Ok(Some(Value::String(format!(
            "{}, {}",
            text.trim_end(),
            missing.join(", ")
        )))),
        PathValue::List(items) => Ok(Some(Value::Array(
            items
                .iter()
                .cloned()
                .chain(missing)
                .map(Value::String)
                .collect(),
        ))),
        PathValue::Other => Err(format!(
            "path on {slug} is not text or a list; edit the page"
        )),
    }
}

/// The project page's task groups among `groups`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupJoin {
    /// The groups its `task_group` names: those found, and the names that match none.
    Named {
        /// The groups found.
        found: Vec<TaskGroup>,
        /// The names that match no group.
        missing: Vec<String>,
    },
    /// No `task_group`: the group named like the page.
    Like(TaskGroup),
    /// Neither.
    Nothing,
}

/// `page`'s groups: those its `task_group` names, by Rusty's slug rule, else the one named like
/// its title or its slug's last part.
#[must_use]
pub fn group_join(page: &ProjectPage, groups: &[TaskGroup]) -> GroupJoin {
    let group_named = |name: &str| {
        let key = name_key(name);
        groups
            .iter()
            .find(|group| key.is_some() && name_key(&group.name) == key)
    };
    if !page.task_groups.is_empty() {
        let mut found = Vec::new();
        let mut missing = Vec::new();
        for name in &page.task_groups {
            match group_named(name) {
                Some(group) => found.push(group.clone()),
                None => missing.push(name.clone()),
            }
        }
        return GroupJoin::Named { found, missing };
    }
    let last = page.slug.rsplit('/').next().unwrap_or(&page.slug);
    group_named(&page.title)
        .or_else(|| group_named(last))
        .map_or(GroupJoin::Nothing, |group| GroupJoin::Like(group.clone()))
}

/// The follow-ups due among the decisions linked to or from the page whose links are `links`,
/// in `due`'s order.
#[must_use]
pub fn due_for(due: &[DecisionSummary], links: &PageLinks) -> Vec<DecisionSummary> {
    let linked: HashSet<&str> = links
        .backlinks
        .iter()
        .map(|link| link.from_slug.as_str())
        .chain(links.outbound.iter().map(|link| link.to_slug.as_str()))
        .collect();
    due.iter()
        .filter(|decision| linked.contains(decision.slug.as_str()))
        .cloned()
        .collect()
}

/// The page's summary: its `summary:`, else the first paragraph of its body that is not a heading,
/// list, table, quote or code, its wikilinks as their words; cut at a word near
/// 300 characters.
#[must_use]
pub fn summary(property: Option<&str>, body: &str) -> String {
    let text = property
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map_or_else(|| first_paragraph(body), str::to_string);
    cut(&text)
}

fn first_paragraph(body: &str) -> String {
    let mut in_code = false;
    let mut paragraph: Vec<&str> = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code = !in_code;
            if !paragraph.is_empty() {
                break;
            }
            continue;
        }
        if in_code {
            continue;
        }
        if trimmed.is_empty() {
            if !paragraph.is_empty() {
                break;
            }
            continue;
        }
        let block = trimmed.starts_with(['#', '-', '*', '|', '>', '+'])
            || trimmed
                .split_once(". ")
                .is_some_and(|(number, _)| number.chars().all(|c| c.is_ascii_digit()));
        if block {
            if !paragraph.is_empty() {
                break;
            }
            continue;
        }
        paragraph.push(trimmed);
    }
    wikilinks_as_words(&paragraph.join(" "))
}

/// `[[target|words]]` as its words, `[[target]]` as its target.
fn wikilinks_as_words(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find("[[") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("]]") else {
            out.push_str(&rest[start..]);
            return out;
        };
        let inner = &after[..end];
        out.push_str(inner.split_once('|').map_or(inner, |(_, words)| words));
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    out
}

fn cut(text: &str) -> String {
    if text.chars().count() <= SUMMARY_CHARS {
        return text.to_string();
    }
    let kept: String = text.chars().take(SUMMARY_CHARS).collect();
    let at_word = kept
        .rfind(' ')
        .map_or(kept.as_str(), |space| &kept[..space]);
    format!("{}…", at_word.trim_end_matches([',', ';', ':', '.']))
}
