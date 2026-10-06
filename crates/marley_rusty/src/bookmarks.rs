//! Rusty's bookmarks as its tools answer them, and the writes Marley makes (#662).
//!
//! Rusty keeps one ordered list in the vault (its TICKET-037). A bookmark is a page (`file`), a
//! folder, a search or a heading of a page; page and folder bookmarks are the favourites. Rusty
//! knows a bookmark by its kind and its path, query or heading, fills a blank title, and drops a
//! second of the same; every write answers with the whole list.

use serde::Deserialize;
use serde_json::{Map, Value};

/// Rusty's tool for the list.
pub const BOOKMARK_LIST: &str = "bookmark_list";
/// Adds one bookmark at the end.
pub const BOOKMARK_ADD: &str = "bookmark_add";
/// Removes one bookmark.
pub const BOOKMARK_REMOVE: &str = "bookmark_remove";
/// Replaces the whole list, in order.
pub const BOOKMARK_SET: &str = "bookmark_set";

/// What a bookmark points at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BookmarkKind {
    /// A page.
    File,
    /// A folder.
    Folder,
    /// A brain search's query.
    Search,
    /// A heading of a page.
    Heading,
    /// A kind Rusty may add later, as written.
    Other(String),
}

impl BookmarkKind {
    /// The kind as Rusty writes it.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        match text {
            "file" => Self::File,
            "folder" => Self::Folder,
            "search" => Self::Search,
            "heading" => Self::Heading,
            other => Self::Other(other.to_string()),
        }
    }
}

/// One bookmark, as Rusty serves it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Bookmark {
    /// `file`, `folder`, `search` or `heading`.
    pub kind: String,
    /// What it is called.
    #[serde(default)]
    pub title: String,
    /// The page's or the folder's path in the vault (a page by its slug).
    #[serde(default)]
    pub path: String,
    /// A search's query.
    #[serde(default)]
    pub query: String,
    /// A heading's text.
    #[serde(default)]
    pub heading: String,
}

impl Bookmark {
    /// A bookmark of the page `slug`, titled by Rusty.
    #[must_use]
    pub fn page(slug: &str) -> Self {
        Self {
            kind: "file".to_string(),
            title: String::new(),
            path: slug.to_string(),
            query: String::new(),
            heading: String::new(),
        }
    }

    /// What it points at.
    #[must_use]
    pub fn kind(&self) -> BookmarkKind {
        BookmarkKind::parse(&self.kind)
    }

    /// Whether it is the page `slug`'s bookmark.
    #[must_use]
    pub fn is_page(&self, slug: &str) -> bool {
        self.kind() == BookmarkKind::File && self.path == slug
    }

    /// Rusty's identity for it: the kind, and the path, the query or the page and its heading.
    #[must_use]
    pub fn key(&self) -> String {
        match self.kind() {
            BookmarkKind::Search => format!("search:{}", self.query),
            BookmarkKind::Heading => format!("heading:{}#{}", self.path, self.heading),
            _ => format!("{}:{}", self.kind, self.path),
        }
    }

    /// Its title, or what Rusty would call it when it has none.
    #[must_use]
    pub fn shown_title(&self) -> &str {
        if !self.title.is_empty() {
            return &self.title;
        }
        match self.kind() {
            BookmarkKind::Search => &self.query,
            BookmarkKind::Heading => &self.heading,
            _ => self.path.rsplit('/').next().unwrap_or(&self.path),
        }
    }

    /// The bookmark in Rusty's parameters, the empty fields left out.
    #[must_use]
    pub fn arguments(&self) -> Value {
        let fields = [
            ("kind", &self.kind),
            ("title", &self.title),
            ("path", &self.path),
            ("query", &self.query),
            ("heading", &self.heading),
        ];
        let object: Map<String, Value> = fields
            .into_iter()
            .filter(|(_, value)| !value.is_empty())
            .map(|(key, value)| (key.to_string(), Value::from(value.as_str())))
            .collect();
        Value::Object(object)
    }
}

/// `bookmark_list`'s answer, and every write's.
///
/// # Errors
///
/// When the answer is not a list of bookmarks.
pub fn bookmarks_from_answer(text: &str) -> Result<Vec<Bookmark>, serde_json::Error> {
    serde_json::from_str(text)
}

/// The list with the bookmark whose key is `key` retitled, every other as it was, in order: what
/// `bookmark_set` sends for a rename.
#[must_use]
pub fn retitled(list: &[Bookmark], key: &str, title: &str) -> Vec<Bookmark> {
    list.iter()
        .map(|bookmark| {
            if bookmark.key() == key {
                Bookmark {
                    title: title.trim().to_string(),
                    ..bookmark.clone()
                }
            } else {
                bookmark.clone()
            }
        })
        .collect()
}

/// One change to the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BookmarkWrite {
    /// A bookmark added at the end.
    Add(Bookmark),
    /// A bookmark removed.
    Remove(Bookmark),
    /// The whole list, in order.
    Set(Vec<Bookmark>),
}

impl BookmarkWrite {
    /// Rusty's tool for it.
    #[must_use]
    pub const fn tool(&self) -> &'static str {
        match self {
            Self::Add(_) => BOOKMARK_ADD,
            Self::Remove(_) => BOOKMARK_REMOVE,
            Self::Set(_) => BOOKMARK_SET,
        }
    }

    /// Its arguments, in Rusty's parameter names.
    #[must_use]
    pub fn arguments(&self) -> Value {
        match self {
            Self::Add(bookmark) | Self::Remove(bookmark) => bookmark.arguments(),
            Self::Set(list) => {
                let bookmarks: Vec<Value> = list.iter().map(Bookmark::arguments).collect();
                Value::Object(
                    std::iter::once(("bookmarks".to_string(), Value::Array(bookmarks))).collect(),
                )
            }
        }
    }
}
