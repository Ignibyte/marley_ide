//! A brain page as `brain_render` gives it, and the pass that lets Zed's Markdown renderer draw
//! it (#645).
//!
//! Zed's parser leaves wikilinks off, so [`page_markdown`] finds them with Rusty's own parse
//! options and rewrites each into an ordinary link whose address is Rusty's: `rusty:page/SLUG#H`
//! for a page `brain_render` resolved, `rusty:new/TARGET` for one it did not. [`PageLink`] sorts a
//! clicked address, and [`PageHistory`] is a tab's Back and Forward. The frontmatter split, the
//! fragment split and the target's normal form follow Rusty's own (`frontmatter.rs` `split_raw`,
//! `render.rs` `split_fragment`, `links.rs` `normalise_target`).

use std::ops::Range;
use std::path::{Component, Path, PathBuf};

use pulldown_cmark::{Event, LinkType, Options, Parser, Tag, TagEnd};
use serde::Deserialize;
use serde_json::Value;

/// The tool that renders a page.
pub const BRAIN_RENDER: &str = "brain_render";

/// How many pages Back keeps.
const HISTORY_LIMIT: usize = 100;

/// A page as `brain_render` answers it, with the fields the Page tab reads.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RenderedPage {
    /// The page's slug.
    pub slug: String,
    /// Its title: the frontmatter's, else its name.
    #[serde(default)]
    pub title: String,
    /// Its frontmatter, key by key in file order.
    #[serde(default)]
    pub properties: Vec<Property>,
    /// The whole file.
    #[serde(default)]
    pub raw: String,
    /// The file's absolute path, which Rusty gives since its TICKET-042; empty before it.
    #[serde(default)]
    pub file: String,
    /// Every wikilink's target and the slug it resolved to.
    #[serde(default)]
    pub links: Vec<LinkOut>,
}

impl RenderedPage {
    /// Reads `brain_render`'s answer: the page, or `None` for `null`, Rusty's answer for a slug
    /// with no page.
    ///
    /// # Errors
    ///
    /// When the answer is neither.
    pub fn from_answer(text: &str) -> Result<Option<Self>, serde_json::Error> {
        serde_json::from_str(text)
    }
}

/// One frontmatter key and its value.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Property {
    /// The key.
    pub key: String,
    /// The value as YAML gave it: a string, a number, a boolean, a list, an object or null.
    #[serde(default)]
    pub value: Value,
}

/// A wikilink's target, normalised, and the slug Rusty resolved it to.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LinkOut {
    /// The target as Rusty normalises it.
    pub target: String,
    /// The page it names, or `None` when there is none.
    #[serde(default)]
    pub slug: Option<String>,
}

/// The page's body: `raw` after its frontmatter, by Rusty's rule; without frontmatter, all of it.
#[must_use]
pub fn body_of(raw: &str) -> &str {
    let trimmed = raw.trim_start();
    let Some(after_open) = trimmed.strip_prefix("---") else {
        return raw;
    };
    let after_open = after_open.trim_start_matches(['\r', '\n']);
    let Some(close) = after_open.find("\n---") else {
        return raw;
    };
    let after_close = after_open.get(close + 4..).unwrap_or_default();
    let newline = after_close
        .find('\n')
        .map_or(after_close.len(), |at| at + 1);
    after_close.get(newline..).unwrap_or_default()
}

/// A link's destination without its `#heading` or `^block` part, and that part, trimmed.
#[must_use]
pub fn split_fragment(destination: &str) -> (&str, Option<&str>) {
    destination
        .find(['#', '^'])
        .map_or((destination, None), |cut| {
            (
                destination.get(..cut).unwrap_or_default(),
                destination
                    .get(cut + 1..)
                    .map(str::trim)
                    .filter(|fragment| !fragment.is_empty()),
            )
        })
}

/// A link target in Rusty's normal form: trimmed, no leading `/` or `./`, no `.md`.
#[must_use]
pub fn normalise_target(target: &str) -> String {
    let target = target.trim().trim_start_matches('/');
    let target = target.strip_prefix("./").unwrap_or(target);
    target.strip_suffix(".md").unwrap_or(target).to_string()
}

/// A page to make at a path the user named, from a link's target or the page picker's query
/// (#654).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewPage {
    /// The page's path in the vault, without `.md`: `projects/marley/review`.
    pub path: String,
}

impl NewPage {
    /// The page `target` names: its heading cut off, normalised as Rusty normalises a link, and
    /// `None` for a path with an empty part, a `..`, or nothing at all.
    #[must_use]
    pub fn from_target(target: &str) -> Option<Self> {
        let path = normalise_target(split_fragment(target).0);
        let parts_ok = path
            .split('/')
            .all(|part| !part.trim().is_empty() && part != "..");
        (!path.is_empty() && parts_ok).then_some(Self { path })
    }

    /// `brain_new_page`'s arguments: the path, which a Rusty with its TICKET-041 takes as given
    /// (folders made, an existing page returned), and the folder and the name, which an older
    /// Rusty takes instead and the newer one ignores.
    #[must_use]
    pub fn arguments(&self) -> Value {
        serde_json::json!({
            "path": self.path,
            "folder": crate::vault::folder_of(&self.path),
            "name": crate::vault::name_of(&self.path),
        })
    }
}

/// Rusty's parse options, so the pass finds the wikilinks Rusty found.
fn rusty_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
        | Options::ENABLE_WIKILINKS
        | Options::ENABLE_MATH
}

/// `body` with each wikilink, and each `![[embed]]`, turned into a Markdown link Zed's renderer
/// draws.
///
/// A wikilink becomes `[TEXT](<rusty:page/SLUG#HEADING>)` when `links` resolves its target (an
/// empty target is the page itself, `slug`), else `[TEXT](<rusty:new/TARGET>)`. The text is the
/// alias after `|`, or the target as written. Everything else is left byte for byte.
#[must_use]
pub fn page_markdown(body: &str, slug: &str, links: &[LinkOut]) -> String {
    let mut replacements: Vec<(Range<usize>, String)> = Vec::new();
    let mut open: Option<(Range<usize>, String, String)> = None;
    for (event, range) in Parser::new_ext(body, rusty_options()).into_offset_iter() {
        match event {
            Event::Start(
                Tag::Link {
                    link_type: LinkType::WikiLink { .. },
                    dest_url,
                    ..
                }
                | Tag::Image {
                    link_type: LinkType::WikiLink { .. },
                    dest_url,
                    ..
                },
            ) => open = Some((range, dest_url.to_string(), String::new())),
            Event::Text(text) | Event::Code(text) => {
                if let Some((_, _, written)) = &mut open {
                    written.push_str(&text);
                }
            }
            Event::End(TagEnd::Link | TagEnd::Image) => {
                if let Some((range, destination, text)) = open.take() {
                    let address = address_for(&destination, slug, links);
                    let text = if text.is_empty() { destination } else { text };
                    replacements.push((range, format!("[{}](<{address}>)", escaped(&text))));
                }
            }
            _ => {}
        }
    }
    let mut markdown = body.to_string();
    for (range, replacement) in replacements.into_iter().rev() {
        if markdown.get(range.clone()).is_some() {
            markdown.replace_range(range, &replacement);
        }
    }
    markdown
}

/// The address a wikilink to `destination` gets.
fn address_for(destination: &str, slug: &str, links: &[LinkOut]) -> String {
    let (target, fragment) = split_fragment(destination);
    let target = normalise_target(target);
    let resolved = if target.is_empty() {
        Some(slug.to_string())
    } else {
        links
            .iter()
            .find(|link| link.target == target)
            .and_then(|link| link.slug.clone())
    };
    resolved.map_or_else(
        || format!("rusty:new/{}", encoded(&target)),
        |page| {
            let mut address = format!("rusty:page/{}", encoded(&page));
            if let Some(fragment) = fragment {
                address.push('#');
                address.push_str(&encoded(fragment));
            }
            address
        },
    )
}

/// Link text with each ASCII punctuation mark backslash-escaped, so it stays text.
fn escaped(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        if character.is_ascii_punctuation() {
            out.push('\\');
        }
        out.push(character);
    }
    out
}

/// What may not stand in an angle-bracket destination, percent-encoded.
fn encoded(text: &str) -> String {
    text.replace('%', "%25")
        .replace('<', "%3C")
        .replace('>', "%3E")
        .replace('\\', "%5C")
}

/// [`encoded`] undone.
fn decoded(text: &str) -> String {
    text.replace("%3C", "<")
        .replace("%3E", ">")
        .replace("%5C", "\\")
        .replace("%25", "%")
}

/// What a clicked address in a page asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageLink {
    /// A page, at a heading when one is named.
    Page {
        /// The page's slug.
        slug: String,
        /// The heading's text.
        heading: Option<String>,
    },
    /// A wikilink to no page yet.
    Missing {
        /// The target as written, normalised.
        target: String,
    },
    /// A heading of this page.
    Heading(String),
    /// A plain link to a page of the vault by its path, such as `other.md`.
    Local {
        /// The target, normalised.
        target: String,
        /// The heading's text.
        heading: Option<String>,
    },
    /// Anything else: a web address or another scheme, for the system to open.
    External(String),
}

impl PageLink {
    /// Sorts `address`, decoding what [`page_markdown`] encoded.
    #[must_use]
    pub fn parse(address: &str) -> Self {
        if let Some(rest) = address.strip_prefix("rusty:page/") {
            let (slug, heading) = rest
                .split_once('#')
                .map_or((rest, None), |(slug, heading)| {
                    (slug, Some(decoded(heading)))
                });
            return Self::Page {
                slug: decoded(slug),
                heading,
            };
        }
        if let Some(target) = address.strip_prefix("rusty:new/") {
            return Self::Missing {
                target: decoded(target),
            };
        }
        if let Some(heading) = address.strip_prefix('#') {
            return Self::Heading(heading.to_string());
        }
        if has_scheme(address) {
            return Self::External(address.to_string());
        }
        let (target, heading) = split_fragment(address);
        Self::Local {
            target: normalise_target(target),
            heading: heading.map(str::to_string),
        }
    }
}

/// Whether `address` starts with a URL scheme (`https:`, `mailto:`, `rusty:` …).
fn has_scheme(address: &str) -> bool {
    let Some((scheme, _)) = address.split_once(':') else {
        return false;
    };
    let mut characters = scheme.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
        })
}

/// One place in a tab's history: a page, and the heading it was opened at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Visit {
    /// The page's slug.
    pub slug: String,
    /// The heading's text.
    pub heading: Option<String>,
}

impl Visit {
    /// A visit to `slug`'s top.
    #[must_use]
    pub fn page(slug: impl Into<String>) -> Self {
        Self {
            slug: slug.into(),
            heading: None,
        }
    }
}

/// A Page tab's Back and Forward.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageHistory {
    behind: Vec<Visit>,
    current: Visit,
    ahead: Vec<Visit>,
}

impl PageHistory {
    /// A history that starts at `first`.
    #[must_use]
    pub const fn new(first: Visit) -> Self {
        Self {
            behind: Vec::new(),
            current: first,
            ahead: Vec::new(),
        }
    }

    /// Where the tab is.
    #[must_use]
    pub const fn current(&self) -> &Visit {
        &self.current
    }

    /// Goes to `visit`, which drops what was ahead.
    pub fn visit(&mut self, visit: Visit) {
        let left = std::mem::replace(&mut self.current, visit);
        self.behind.push(left);
        if self.behind.len() > HISTORY_LIMIT {
            self.behind = self.behind.split_off(1);
        }
        self.ahead.clear();
    }

    /// Goes back one place, if there is one.
    pub fn back(&mut self) -> bool {
        let Some(previous) = self.behind.pop() else {
            return false;
        };
        let left = std::mem::replace(&mut self.current, previous);
        self.ahead.push(left);
        true
    }

    /// Goes forward one place, if there is one.
    pub fn forward(&mut self) -> bool {
        let Some(next) = self.ahead.pop() else {
            return false;
        };
        let left = std::mem::replace(&mut self.current, next);
        self.behind.push(left);
        true
    }

    /// Whether Back has somewhere to go.
    #[must_use]
    pub const fn can_back(&self) -> bool {
        !self.behind.is_empty()
    }

    /// Whether Forward has somewhere to go.
    #[must_use]
    pub const fn can_forward(&self) -> bool {
        !self.ahead.is_empty()
    }
}

/// The file of page `slug` in the vault at `root`, or `None` for a slug that would leave it.
#[must_use]
pub fn page_file_in(root: &Path, slug: &str) -> Option<PathBuf> {
    let relative = Path::new(slug);
    let inside = !slug.is_empty()
        && relative
            .components()
            .all(|component| matches!(component, Component::Normal(_)));
    inside.then(|| root.join(format!("{slug}.md")))
}
