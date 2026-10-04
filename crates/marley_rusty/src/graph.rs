//! Rusty's vault as a graph, and the part of it the Graph tab shows (#647).
//!
//! `brain_graph`'s answer read into nodes and edges; Rusty's graph filter (`tag:`, `path:`,
//! `type:` and words, every term matching); and [`shown`], which applies the filter, the legend's
//! hidden types, tags as nodes, the decision edges and orphans switches, and the cap.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::Deserialize;

/// The tool that names the vault's page types in Rusty's order.
pub const BRAIN_PAGE_TYPES: &str = "brain_page_types";

/// The most nodes the Graph tab lays out; past it, the most linked are kept.
pub const CAP: usize = 2_000;

/// What a node stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(from = "String")]
pub enum NodeKind {
    /// A page of the vault.
    Page,
    /// A tag, as a node joined to its pages.
    Tag,
    /// A link's target that names no page yet.
    Unresolved,
    /// A kind this version does not know.
    Other,
}

impl From<String> for NodeKind {
    fn from(kind: String) -> Self {
        match kind.as_str() {
            "page" => Self::Page,
            "tag" => Self::Tag,
            "unresolved" => Self::Unresolved,
            _ => Self::Other,
        }
    }
}

/// What an edge stands for: a link, or one of a decision's typed edges.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
#[serde(from = "String")]
pub enum EdgeKind {
    /// A wikilink between two pages.
    Link,
    /// A decision to a page it consulted.
    Consulted,
    /// A decision to the decision it replaces.
    Supersedes,
    /// A superseded decision to its successor.
    FollowsUp,
    /// A kind this version does not know, drawn as a link.
    Other(String),
}

impl From<String> for EdgeKind {
    fn from(kind: String) -> Self {
        match kind.as_str() {
            "link" => Self::Link,
            "consulted" => Self::Consulted,
            "supersedes" => Self::Supersedes,
            "follows_up" => Self::FollowsUp,
            _ => Self::Other(kind),
        }
    }
}

impl EdgeKind {
    /// Whether it is one of a decision's typed edges.
    #[must_use]
    pub const fn is_decision(&self) -> bool {
        matches!(self, Self::Consulted | Self::Supersedes | Self::FollowsUp)
    }
}

const fn link_kind() -> EdgeKind {
    EdgeKind::Link
}

/// One node of `brain_graph`'s answer.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Node {
    /// A page's slug, `tag:<name>` or `new:<target>`.
    pub id: String,
    /// What the node stands for.
    pub kind: NodeKind,
    /// The page's title, `#tag`, or the target as written.
    #[serde(default)]
    pub title: String,
    /// The page's type; empty for the other kinds.
    #[serde(default)]
    pub page_type: String,
    /// The page's tags, frontmatter and inline.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// One edge of `brain_graph`'s answer, from a page.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Edge {
    /// The linking page's slug.
    pub from: String,
    /// The target node's id.
    pub to: String,
    /// What the edge stands for; `link` when Rusty names none.
    #[serde(default = "link_kind")]
    pub kind: EdgeKind,
}

/// `brain_graph`'s answer.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Graph {
    /// Every node.
    #[serde(default)]
    pub nodes: Vec<Node>,
    /// Every edge.
    #[serde(default)]
    pub edges: Vec<Edge>,
}

impl Graph {
    /// Reads `brain_graph`'s answer.
    ///
    /// # Errors
    /// When the text is not `brain_graph`'s JSON.
    pub fn from_answer(text: &str) -> serde_json::Result<Self> {
        serde_json::from_str(text)
    }

    /// Each page type in the graph with its page count, by type.
    #[must_use]
    pub fn type_counts(&self) -> BTreeMap<String, usize> {
        let mut counts = BTreeMap::new();
        for node in self.nodes.iter().filter(|node| node.kind == NodeKind::Page) {
            *counts.entry(node.page_type.clone()).or_insert(0) += 1;
        }
        counts
    }
}

/// One entry of `brain_page_types`' answer.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct PageType {
    page_type: String,
}

/// The page types `brain_page_types` names, in Rusty's order.
///
/// # Errors
/// When the text is not `brain_page_types`' JSON.
pub fn page_types_from_answer(text: &str) -> serde_json::Result<Vec<String>> {
    let types: Vec<PageType> = serde_json::from_str(text)?;
    Ok(types.into_iter().map(|each| each.page_type).collect())
}

/// The order page types take their colours in: Rusty's own types first, as `brain_page_types`
/// names them, then the graph's other types by name, so a type keeps its colour as the vault grows.
#[must_use]
pub fn type_order(known: &[String], graph: &Graph) -> Vec<String> {
    let mut order: Vec<String> = known.to_vec();
    let others: BTreeSet<&str> = graph
        .nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Page && !known.contains(&node.page_type))
        .map(|node| node.page_type.as_str())
        .collect();
    order.extend(others.into_iter().map(str::to_string));
    order
}

/// One term of a graph filter.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Term {
    /// The tag, or one nested under it.
    Tag(String),
    /// A prefix of the slug.
    Path(String),
    /// The page type.
    Type(String),
    /// Text in the title or the slug.
    Text(String),
}

impl Term {
    /// One term, lowercased.
    fn read(term: &str) -> Self {
        term.strip_prefix("tag:")
            .map(|tag| Self::Tag(tag.trim_start_matches('#').to_string()))
            .or_else(|| {
                term.strip_prefix("path:")
                    .map(|path| Self::Path(path.to_string()))
            })
            .or_else(|| {
                term.strip_prefix("type:")
                    .map(|page_type| Self::Type(page_type.to_string()))
            })
            .unwrap_or_else(|| Self::Text(term.to_string()))
    }
}

/// Rusty's graph filter: terms separated by spaces, every one of which must match, case aside.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Query {
    terms: Vec<Term>,
}

impl Query {
    /// Reads a filter as typed.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let terms = text
            .split_whitespace()
            .map(|term| Term::read(&term.to_lowercase()))
            .collect();
        Self { terms }
    }

    /// Whether the filter is empty, and so lets every node through.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    /// Whether `node` passes every term.
    #[must_use]
    pub fn matches(&self, node: &Node) -> bool {
        self.terms.iter().all(|term| match term {
            Term::Tag(tag) => node.tags.iter().any(|carried| {
                let carried = carried.to_lowercase();
                carried == *tag || carried.starts_with(&format!("{tag}/"))
            }),
            Term::Path(path) => node.id.to_lowercase().starts_with(path.as_str()),
            Term::Type(page_type) => node.page_type.to_lowercase() == *page_type,
            Term::Text(text) => {
                node.title.to_lowercase().contains(text.as_str())
                    || node.id.to_lowercase().contains(text.as_str())
            }
        })
    }
}

/// What the Graph tab's panel asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filters {
    /// The filter field's query.
    pub query: Query,
    /// The page types turned off in the legend.
    pub hidden_types: BTreeSet<String>,
    /// Tags as nodes, joined to the pages shown.
    pub tags: bool,
    /// A decision's typed edges.
    pub decision_edges: bool,
    /// Pages with no shown edge.
    pub orphans: bool,
}

impl Default for Filters {
    fn default() -> Self {
        Self {
            query: Query::default(),
            hidden_types: BTreeSet::new(),
            tags: false,
            decision_edges: true,
            orphans: true,
        }
    }
}

/// A node the tab shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShownNode {
    /// Its id in the graph.
    pub id: String,
    /// What it stands for.
    pub kind: NodeKind,
    /// What its label says.
    pub title: String,
    /// A page's type; empty for the other kinds.
    pub page_type: String,
}

/// The part of the graph the tab lays out and draws.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Shown {
    /// The nodes, the centre's place in it given by `centre`.
    pub nodes: Vec<ShownNode>,
    /// The edges between them, by their places in `nodes`.
    pub edges: Vec<(usize, usize, EdgeKind)>,
    /// Each node's shown edges.
    pub degree: Vec<usize>,
    /// How many nodes passed the filters before the cap.
    pub total: usize,
    /// The local graph's centre, when shown.
    pub centre: Option<usize>,
}

impl Shown {
    /// The nodes joined to `index` by a shown edge, and `index` itself.
    #[must_use]
    pub fn neighbourhood(&self, index: usize) -> BTreeSet<usize> {
        let mut near: BTreeSet<usize> = self
            .edges
            .iter()
            .filter_map(|(from, to, _)| {
                if *from == index {
                    Some(*to)
                } else if *to == index {
                    Some(*from)
                } else {
                    None
                }
            })
            .collect();
        let _inserted = near.insert(index);
        near
    }
}

/// What `graph` shows under `filters`, around `centre` when there is one.
///
/// The pages and unresolved targets the query and the legend let through, the centre always; tags
/// as nodes for the pages kept (built here, so a local graph stays local); the edges between kept
/// nodes; orphans dropped when asked; and past `cap`, the most linked, ties by id.
#[must_use]
pub fn shown(graph: &Graph, filters: &Filters, centre: Option<&str>, cap: usize) -> Shown {
    let is_centre = |node: &Node| centre.is_some_and(|centre| node.id == centre);
    let mut kept: Vec<ShownNode> = Vec::new();
    let mut tags_of: Vec<Option<&[String]>> = Vec::new();
    for node in &graph.nodes {
        let passes = match node.kind {
            NodeKind::Page => {
                !filters.hidden_types.contains(&node.page_type) && filters.query.matches(node)
            }
            NodeKind::Unresolved => filters.query.matches(node),
            NodeKind::Tag | NodeKind::Other => false,
        };
        if passes || is_centre(node) {
            tags_of.push((node.kind == NodeKind::Page).then_some(node.tags.as_slice()));
            kept.push(ShownNode {
                id: node.id.clone(),
                kind: node.kind,
                title: node.title.clone(),
                page_type: node.page_type.clone(),
            });
        }
    }
    let mut index: HashMap<String, usize> = kept
        .iter()
        .enumerate()
        .map(|(at, node)| (node.id.clone(), at))
        .collect();
    let mut edges: Vec<(usize, usize, EdgeKind)> = graph
        .edges
        .iter()
        .filter(|edge| filters.decision_edges || !edge.kind.is_decision())
        .filter_map(|edge| {
            let from = *index.get(&edge.from)?;
            let to = *index.get(&edge.to)?;
            (from != to).then(|| (from, to, edge.kind.clone()))
        })
        .collect();
    if filters.tags {
        let pages = kept.len();
        for page in 0..pages {
            for tag in tags_of.get(page).copied().flatten().unwrap_or_default() {
                let id = format!("tag:{}", tag.to_lowercase());
                let at = *index.entry(id.clone()).or_insert_with(|| {
                    kept.push(ShownNode {
                        id,
                        kind: NodeKind::Tag,
                        title: format!("#{tag}"),
                        page_type: String::new(),
                    });
                    kept.len() - 1
                });
                edges.push((page, at, EdgeKind::Link));
            }
        }
    }
    let mut degree = degrees(kept.len(), &edges);
    let centre_at = centre.and_then(|centre| index.get(centre).copied());
    if !filters.orphans {
        let keep: Vec<bool> = kept
            .iter()
            .enumerate()
            .map(|(at, node)| Some(at) == centre_at || node.kind == NodeKind::Tag || degree[at] > 0)
            .collect();
        (kept, edges) = retain(kept, edges, &keep);
        degree = degrees(kept.len(), &edges);
    }
    let total = kept.len();
    if total > cap {
        let centre_at = centre.and_then(|centre| kept.iter().position(|node| node.id == centre));
        let mut order: Vec<usize> = (0..total).collect();
        order.sort_by_key(|at| {
            (
                Some(*at) != centre_at,
                Reverse(degree[*at]),
                kept[*at].id.clone(),
            )
        });
        let mut keep = vec![false; total];
        for at in order.into_iter().take(cap) {
            keep[at] = true;
        }
        (kept, edges) = retain(kept, edges, &keep);
        degree = degrees(kept.len(), &edges);
    }
    let centre = centre.and_then(|centre| kept.iter().position(|node| node.id == centre));
    Shown {
        nodes: kept,
        edges,
        degree,
        total,
        centre,
    }
}

fn degrees(count: usize, edges: &[(usize, usize, EdgeKind)]) -> Vec<usize> {
    let mut degree = vec![0; count];
    for (from, to, _) in edges {
        degree[*from] += 1;
        degree[*to] += 1;
    }
    degree
}

/// The nodes `keep` marks, and the edges between them renumbered.
fn retain(
    nodes: Vec<ShownNode>,
    edges: Vec<(usize, usize, EdgeKind)>,
    keep: &[bool],
) -> (Vec<ShownNode>, Vec<(usize, usize, EdgeKind)>) {
    let mut moved = vec![None; nodes.len()];
    let mut kept = Vec::new();
    for (at, node) in nodes.into_iter().enumerate() {
        if keep.get(at).copied().unwrap_or(false) {
            moved[at] = Some(kept.len());
            kept.push(node);
        }
    }
    let edges = edges
        .into_iter()
        .filter_map(|(from, to, kind)| Some((moved[from]?, moved[to]?, kind)))
        .collect();
    (kept, edges)
}
