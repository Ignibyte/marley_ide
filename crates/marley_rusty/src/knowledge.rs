//! What the Knowledge panel shows of a brain page (#646).
//!
//! Its tags with their counts, the pages that link to it with the line each link sits on, and its
//! own links in order; and brain search's snippets with their marks as ranges.
//!
//! [`Backlink`], [`mention`] and [`trimmed`] are ported from Ely GPUI Components
//! (`src/documents/knowledge.rs` and `src/editor/search.rs` at `2f8b2f6`), on Rusty's answers in
//! place of Ely's; Ely's notice is below.

// The portions named above are ported from Ely GPUI Components, under this notice:
//
// MIT License
//
// Copyright (c) 2026 Ely GPUI Component contributors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::ops::Range;

use serde::Deserialize;

use crate::page::normalise_target;
use crate::vault::name_of;

/// The tool that gives a page's links both ways.
pub const BRAIN_GET_LINKS: &str = "brain_get_links";

/// The tool that gives the link graph, or one page's neighbourhood of it.
pub const BRAIN_GRAPH: &str = "brain_graph";

/// The tool that gives every tag with its page count.
pub const BRAIN_TAGS: &str = "brain_tags";

/// How many hits the Knowledge panel asks brain search for, as Rusty's own search pane does.
pub const SEARCH_LIMIT: usize = 60;

/// Characters kept before a backlink's mention when its line is long (Ely's `LEAD`).
const LEAD: usize = 24;

/// One link as `brain_get_links` gives it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LinkEntry {
    /// The page the link is in.
    pub from_slug: String,
    /// The page it names, or its target normalised when no page has that name.
    pub to_slug: String,
    /// The line the link sits on, trimmed.
    #[serde(default)]
    pub context: String,
    /// Whether `to_slug` names a page that exists.
    #[serde(default)]
    pub resolved: bool,
}

/// `brain_get_links`'s answer: the page's links in document order, and the links to it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct PageLinks {
    /// The page's own links, one per target.
    #[serde(default)]
    pub outbound: Vec<LinkEntry>,
    /// The links to the page, by the linking page's slug.
    #[serde(default)]
    pub backlinks: Vec<LinkEntry>,
}

/// One tag and how many pages carry it, as `brain_tags` gives it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TagCount {
    /// The tag, without `#`.
    pub tag: String,
    /// Its pages, its nested tags' included.
    pub count: usize,
}

/// One node of `brain_graph`'s answer.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GraphNode {
    /// A page's slug, or `tag:…` and `new:…` for the other kinds.
    pub id: String,
    /// `page`, `tag` or `unresolved`.
    #[serde(default)]
    pub kind: String,
    /// A page's title.
    #[serde(default)]
    pub title: String,
    /// A page's tags as Rusty's index holds them, frontmatter and inline.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// `brain_graph`'s nodes; the edges are not read.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Graph {
    /// The pages, and tags and unresolved targets when asked for.
    #[serde(default)]
    pub nodes: Vec<GraphNode>,
}

/// A page that links to the shown one: its title, the line its link sits on, and the link's
/// byte range in that line when it can be found (Ely's `Backlink`, the icon dropped).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backlink {
    /// The linking page's slug.
    pub slug: String,
    /// Its title.
    pub title: String,
    /// The line, cut to start near the link when it is long.
    pub context: String,
    /// The link's bytes in `context`.
    pub mention: Option<Range<usize>>,
}

/// One of the page's own links.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outgoing {
    /// A link to a page that exists.
    Page {
        /// Its slug.
        slug: String,
        /// Its title.
        title: String,
    },
    /// A link to no page yet, as written.
    Missing {
        /// The target, normalised.
        target: String,
    },
}

/// What the Knowledge panel shows of a page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageKnowledge {
    /// The page's slug.
    pub slug: String,
    /// Its title, else its name.
    pub title: String,
    /// Its tags, each with its page count.
    pub tags: Vec<(String, usize)>,
    /// The pages that link to it.
    pub backlinks: Vec<Backlink>,
    /// Its own links, in order.
    pub outgoing: Vec<Outgoing>,
}

/// The page `slug`'s knowledge from `brain_get_links`, `brain_graph { around: slug, depth: 1 }`
/// and `brain_tags`: titles from the graph, the slug's last part where it names none.
#[must_use]
pub fn page_knowledge(
    slug: &str,
    links: PageLinks,
    graph: &Graph,
    tag_counts: &[TagCount],
) -> PageKnowledge {
    let node = |id: &str| {
        graph
            .nodes
            .iter()
            .find(|node| node.id == id && node.kind == "page")
    };
    let title_of = |id: &str| {
        node(id)
            .map(|node| node.title.clone())
            .filter(|title| !title.is_empty())
            .unwrap_or_else(|| name_of(id).to_string())
    };
    let title = title_of(slug);
    let tags = node(slug)
        .map(|node| {
            node.tags
                .iter()
                .map(|tag| {
                    let count = tag_counts
                        .iter()
                        .find(|counted| counted.tag.eq_ignore_ascii_case(tag))
                        .map_or(0, |counted| counted.count);
                    (tag.clone(), count)
                })
                .collect()
        })
        .unwrap_or_default();
    let backlinks = links
        .backlinks
        .into_iter()
        .map(|link| {
            let found = mention(&link.context, slug, &title);
            let (context, mention) = match found {
                Some(range) => {
                    let (context, mut ranges) = trimmed(&link.context, &[range], LEAD);
                    (context, ranges.pop())
                }
                None => (link.context.clone(), None),
            };
            Backlink {
                title: title_of(&link.from_slug),
                slug: link.from_slug,
                context,
                mention,
            }
        })
        .collect();
    let outgoing = links
        .outbound
        .into_iter()
        .map(|link| {
            if link.resolved {
                Outgoing::Page {
                    title: title_of(&link.to_slug),
                    slug: link.to_slug,
                }
            } else {
                Outgoing::Missing {
                    target: link.to_slug,
                }
            }
        })
        .collect();
    PageKnowledge {
        slug: slug.to_string(),
        title,
        tags,
        backlinks,
        outgoing,
    }
}

/// The bytes of the first `[[…]]` in `line` that names the page `slug`: by its slug, its last
/// part or its title, case aside, its target read up to `|` or `#`.
#[must_use]
pub fn mention(line: &str, slug: &str, title: &str) -> Option<Range<usize>> {
    let names = [
        slug.to_lowercase(),
        name_of(slug).to_lowercase(),
        title.to_lowercase(),
    ];
    let mut at = 0;
    while let Some(open) = line.get(at..).and_then(|rest| rest.find("[[")) {
        let start = at + open;
        let inner_start = start + 2;
        let close = line.get(inner_start..)?.find("]]")?;
        let end = inner_start + close + 2;
        let inner = line
            .get(inner_start..inner_start + close)
            .unwrap_or_default();
        let target = inner.split(['|', '#']).next().unwrap_or_default();
        if names.contains(&normalise_target(target).to_lowercase()) {
            return Some(start..end);
        }
        at = end;
    }
    None
}

/// `text` cut to start near its first range when more than `lead` characters come before it,
/// with `…` in front and the ranges moved along; otherwise only its leading space dropped
/// (Ely's `trimmed`).
#[must_use]
pub fn trimmed(text: &str, ranges: &[Range<usize>], lead: usize) -> (String, Vec<Range<usize>>) {
    let first = ranges
        .first()
        .map_or(0, |range| range.start)
        .min(text.len());
    let before = text.get(..first).map_or(0, |head| head.chars().count());
    let last = ranges.iter().map(|range| range.end).max().unwrap_or(0);
    let kept = text
        .get(..text.trim_end().len().max(last).min(text.len()))
        .unwrap_or(text);
    if before <= lead {
        let start = (kept.len() - kept.trim_start().len()).min(first);
        let moved = ranges
            .iter()
            .map(|range| range.start.saturating_sub(start)..range.end.saturating_sub(start))
            .collect();
        return (kept.get(start..).unwrap_or_default().to_string(), moved);
    }
    let cut = text
        .get(..first)
        .and_then(|head| head.char_indices().nth(before - lead))
        .map_or(first, |(at, _)| at);
    let shift = '…'.len_utf8();
    let moved = ranges
        .iter()
        .filter(|range| range.start >= cut)
        .map(|range| range.start - cut + shift..range.end - cut + shift)
        .collect();
    (format!("…{}", kept.get(cut..).unwrap_or_default()), moved)
}

/// A snippet with Rusty's `<b>` and `</b>` taken out, and the marked spans as byte ranges.
#[must_use]
pub fn snippet(marked: &str) -> (String, Vec<Range<usize>>) {
    let mut text = String::with_capacity(marked.len());
    let mut ranges = Vec::new();
    let mut open = None;
    let mut rest = marked;
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix("<b>") {
            open = Some(text.len());
            rest = after;
        } else if let Some(after) = rest.strip_prefix("</b>") {
            if let Some(start) = open.take() {
                ranges.push(start..text.len());
            }
            rest = after;
        } else {
            let mut characters = rest.chars();
            if let Some(character) = characters.next() {
                text.push(character);
            }
            rest = characters.as_str();
        }
    }
    (text, ranges)
}
