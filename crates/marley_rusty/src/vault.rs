//! Rusty's vault and the rail's Brain view's rows over it (#644).
//!
//! The vault as `brain_tree` gives it and brain search's hits; the rows the open folders show,
//! where Zed's list keys move, and the paths a new folder, a rename and a drop ask Rusty to write.
//!
//! [`rows`], [`step`], [`VaultNode::holds`] and the drop guard in [`move_target`] are ported from
//! Ely GPUI Components (`src/lists/tree/model.rs` at `2f8b2f6`), on vault paths and Rusty's node in
//! place of Ely's keys and nodes; Ely's notice is below.

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

use std::collections::HashSet;
use std::hash::BuildHasher;

use serde::Deserialize;

/// The tool that gives the whole vault as one nested node.
pub const BRAIN_TREE: &str = "brain_tree";

/// The tool that searches the brain.
pub const BRAIN_SEARCH: &str = "brain_search";

/// The tool that gives a day's note, made when missing.
pub const BRAIN_DAILY_NOTE: &str = "brain_daily_note";

/// The tool that makes a page in a folder.
pub const BRAIN_NEW_PAGE: &str = "brain_new_page";

/// The tool that makes a folder.
pub const BRAIN_NEW_FOLDER: &str = "brain_new_folder";

/// The tool that renames or moves a page or a folder, rewriting the links to it.
pub const BRAIN_RENAME: &str = "brain_rename";

/// The tool that moves a page into `archive/`.
pub const BRAIN_DELETE_PAGE: &str = "brain_delete_page";

/// The tool that moves a folder, with everything in it, into `archive/`.
pub const BRAIN_DELETE_FOLDER: &str = "brain_delete_folder";

/// Rusty's setting for the vault's folder; unset, Rusty uses `.rusty/brain` in the home folder.
pub const VAULT_PATH_KEY: &str = "brain_vault_path";

/// How many hits the Brain view asks brain search for.
pub const SEARCH_LIMIT: usize = 50;

/// What a node of the vault is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
    /// A folder, the vault's root among them.
    Folder,
    /// A Markdown page; its path is its slug.
    Page,
    /// Any other file, and any kind Rusty may add.
    #[serde(other)]
    File,
}

/// One node of `brain_tree`'s answer: the vault's root, a folder, a page or another file.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct VaultNode {
    /// The name shown: a page's without `.md`.
    pub name: String,
    /// The path in the vault, `""` for the root: a page's is its slug.
    pub path: String,
    /// What the node is.
    pub kind: NodeKind,
    /// The pages at and under it, at any depth.
    #[serde(default)]
    pub pages: usize,
    /// Folders first, then pages and files, each by name, as Rusty orders them.
    #[serde(default)]
    pub children: Vec<Self>,
}

impl VaultNode {
    /// Reads `brain_tree`'s answer.
    ///
    /// # Errors
    ///
    /// When the answer is not one node.
    pub fn from_answer(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Whether `path` is this node or under it (Ely's `holds`).
    #[must_use]
    pub fn holds(&self, path: &str) -> bool {
        self.path == path || self.children.iter().any(|child| child.holds(path))
    }

    /// The node at `path`, at or under this one.
    #[must_use]
    pub fn find(&self, path: &str) -> Option<&Self> {
        if self.path == path {
            return Some(self);
        }
        self.children.iter().find_map(|child| child.find(path))
    }
}

/// One of brain search's hits.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SearchHit {
    /// The page's slug.
    pub slug: String,
    /// The page's title.
    #[serde(default)]
    pub title: String,
    /// The words around the match, each match between `<b>` and `</b>`; empty for a query of
    /// operators alone.
    #[serde(default)]
    pub snippet: String,
}

/// Reads `brain_search`'s answer, its hits in Rusty's order.
///
/// # Errors
///
/// When the answer is not a list of hits.
pub fn hits_from_answer(text: &str) -> Result<Vec<SearchHit>, serde_json::Error> {
    serde_json::from_str(text)
}

/// Reads the slug `brain_new_page` answers, a bare JSON string.
///
/// # Errors
///
/// When the answer is not a string.
pub fn slug_from_answer(text: &str) -> Result<String, serde_json::Error> {
    serde_json::from_str(text)
}

/// The slug of the page `brain_daily_note` answers.
///
/// # Errors
///
/// When the answer is not a page.
pub fn page_slug_from_answer(text: &str) -> Result<String, serde_json::Error> {
    #[derive(Deserialize)]
    struct Page {
        slug: String,
    }
    serde_json::from_str::<Page>(text).map(|page| page.slug)
}

/// What `brain_rename` reports: the path renamed and the path it has now.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RenameReport {
    /// The path before.
    pub from: String,
    /// The path now.
    pub to: String,
    /// How many other pages Rusty rewrote links in (#656).
    #[serde(default)]
    pub pages_rewritten: usize,
}

impl RenameReport {
    /// Reads `brain_rename`'s answer.
    ///
    /// # Errors
    ///
    /// When the answer is not a report.
    pub fn from_answer(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}

/// One visible row of the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultRow {
    /// The node's path.
    pub path: String,
    /// The node's name.
    pub name: String,
    /// A folder or a page.
    pub kind: NodeKind,
    /// The pages at and under it.
    pub pages: usize,
    /// How deep it sits: the root's children are at 0.
    pub depth: usize,
    /// The row of the folder it is in, if it is not at the top.
    pub parent: Option<usize>,
    /// Whether it is a folder that is open.
    pub open: bool,
}

/// The rows in view, depth first, through the open folders (Ely's `rows`). The root itself is not
/// a row, and files that are not pages are left out: no tab shows them.
#[must_use]
pub fn rows<S: BuildHasher>(root: &VaultNode, open: &HashSet<String, S>) -> Vec<VaultRow> {
    fn walk<S: BuildHasher>(
        nodes: &[VaultNode],
        open: &HashSet<String, S>,
        depth: usize,
        parent: Option<usize>,
        out: &mut Vec<VaultRow>,
    ) {
        for node in nodes.iter().filter(|node| node.kind != NodeKind::File) {
            let here = out.len();
            let is_open = node.kind == NodeKind::Folder && open.contains(&node.path);
            out.push(VaultRow {
                path: node.path.clone(),
                name: node.name.clone(),
                kind: node.kind,
                pages: node.pages,
                depth,
                parent,
                open: is_open,
            });
            if is_open {
                walk(&node.children, open, depth + 1, Some(here), out);
            }
        }
    }
    let mut out = Vec::new();
    walk(&root.children, open, 0, None, &mut out);
    out
}

/// One of Zed's list keys, as the Brain view takes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// `menu::SelectNext`.
    Next,
    /// `menu::SelectPrevious`.
    Previous,
    /// `menu::SelectFirst`.
    First,
    /// `menu::SelectLast`.
    Last,
    /// `menu::SelectChild`: opens a folder, or steps into an open one.
    Child,
    /// `menu::SelectParent`: closes a folder, or goes to the folder above.
    Parent,
}

/// What a key does on the selected row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Move {
    /// Selects the row at this index.
    To(usize),
    /// Opens the folder at this path.
    Open(String),
    /// Closes the folder at this path.
    Close(String),
}

/// What `key` does with the row `at` selected, or with none (Ely's `step`).
#[must_use]
pub fn step(rows: &[VaultRow], at: Option<usize>, key: Key) -> Option<Move> {
    let last = rows.len().checked_sub(1)?;
    let Some(at) = at.filter(|at| *at <= last) else {
        return match key {
            Key::Next | Key::First => Some(Move::To(0)),
            Key::Previous | Key::Last => Some(Move::To(last)),
            Key::Child | Key::Parent => None,
        };
    };
    let row = &rows[at];
    let folder = row.kind == NodeKind::Folder;
    match key {
        Key::Next => (at < last).then_some(Move::To(at + 1)),
        Key::Previous => at.checked_sub(1).map(Move::To),
        Key::First => Some(Move::To(0)),
        Key::Last => Some(Move::To(last)),
        Key::Child if folder && !row.open => Some(Move::Open(row.path.clone())),
        Key::Child if row.open => rows
            .get(at + 1)
            .filter(|child| child.depth > row.depth)
            .map(|_| Move::To(at + 1)),
        Key::Child => None,
        Key::Parent if row.open => Some(Move::Close(row.path.clone())),
        Key::Parent => row.parent.map(Move::To),
    }
}

/// The folder `path` is in, `""` at the top.
#[must_use]
pub fn folder_of(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(folder, _)| folder)
}

/// The last part of `path`.
#[must_use]
pub fn name_of(path: &str) -> &str {
    path.rsplit_once('/').map_or(path, |(_, name)| name)
}

/// The folders above `path`, the top one first.
#[must_use]
pub fn folders_above(path: &str) -> Vec<String> {
    let mut folders = Vec::new();
    let mut folder = folder_of(path);
    while !folder.is_empty() {
        folders.push(folder.to_string());
        folder = folder_of(folder);
    }
    folders.reverse();
    folders
}

/// A typed name as one part of a path: trimmed, a `/` made `-`, as Rusty's own app and
/// `brain_new_page` take names; `None` when nothing is left.
fn one_part(typed: &str) -> Option<String> {
    let name = typed.trim().trim_end_matches(".md").replace('/', "-");
    (!name.is_empty()).then_some(name)
}

/// The path a folder named `typed` gets in `parent`, or `None` for an empty name.
#[must_use]
pub fn child_path(parent: &str, typed: &str) -> Option<String> {
    let name = one_part(typed)?;
    Some(if parent.is_empty() {
        name
    } else {
        format!("{parent}/{name}")
    })
}

/// The `to` that renames `path` to `typed` in its own folder, or `None` when the name is empty or
/// unchanged, which calls nothing.
#[must_use]
pub fn rename_target(path: &str, typed: &str) -> Option<String> {
    let to = child_path(folder_of(path), typed)?;
    (to != path).then_some(to)
}

/// The `to` that moves `dragged` onto `onto`, as `"<folder>/"` or `"/"`.
///
/// A drop onto a folder goes into it, onto a page into its folder, and `None`, the tree's empty
/// space, to the top. A drop onto itself or under itself (Ely's `can_drop`), or into the folder it
/// is already in, gives `None` and calls nothing.
#[must_use]
pub fn move_target(root: &VaultNode, dragged: &str, onto: Option<&str>) -> Option<String> {
    let node = root.find(dragged)?;
    let folder = match onto {
        None => "",
        Some(onto) => match root.find(onto)?.kind {
            NodeKind::Folder => onto,
            NodeKind::Page | NodeKind::File => folder_of(onto),
        },
    };
    if node.holds(folder) || folder_of(dragged) == folder {
        return None;
    }
    Some(format!("{folder}/"))
}

/// The open folders after the folder `from` became `to`: those at and under it follow it.
#[must_use]
pub fn reopen<S: BuildHasher + Default>(
    open: &HashSet<String, S>,
    from: &str,
    to: &str,
) -> HashSet<String, S> {
    open.iter()
        .map(|path| match path.strip_prefix(from) {
            Some("") => to.to_string(),
            Some(rest) if rest.starts_with('/') => format!("{to}{rest}"),
            _ => path.clone(),
        })
        .collect()
}
