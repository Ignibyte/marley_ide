//! Rusty's long-term memories as its tools serve them, and the writes Marley makes (#664).
//!
//! Rusty's agents read the memories at the start of a conversation. `list_memories` answers them
//! high importance first, then normal, then low, the newest first within each; the Memory tab
//! draws that order as it comes. Importance is one of three words since Rusty's TICKET-052.

use serde::Deserialize;
use serde_json::{Value, json};

/// Rusty's tool for the memories.
pub const LIST_MEMORIES: &str = "list_memories";
/// Stores a memory and answers its id.
pub const STORE_MEMORY: &str = "store_memory";
/// Changes a memory and answers it.
pub const UPDATE_MEMORY: &str = "update_memory";
/// Deletes a memory.
pub const DELETE_MEMORY: &str = "delete_memory";

/// The category a memory added with none is filed under, as Rusty's app files it.
pub const DEFAULT_CATEGORY: &str = "context";

/// One memory, as Rusty serves it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Memory {
    /// Rusty's id for it.
    pub id: String,
    /// What it is about: `preference`, `fact`, `context` or any word.
    #[serde(default)]
    pub category: String,
    /// `low`, `normal` or `high`; an older memory may hold another word.
    #[serde(default)]
    pub importance: String,
    /// What is remembered.
    #[serde(default)]
    pub content: String,
    /// Who stored it: `mcp`, `app`, an agent.
    #[serde(default)]
    pub source: String,
    /// When it last changed, in seconds since the epoch.
    #[serde(default)]
    pub updated_at: i64,
}

impl Memory {
    /// Its importance, when it is one of Rusty's three.
    #[must_use]
    pub fn importance(&self) -> Option<Importance> {
        Importance::parse(&self.importance)
    }
}

/// `list_memories`' answer.
///
/// # Errors
///
/// When the answer is not a list of memories.
pub fn memories_from_answer(text: &str) -> Result<Vec<Memory>, serde_json::Error> {
    serde_json::from_str(text)
}

/// How much a memory matters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Importance {
    /// Kept, read last.
    Low,
    /// Rusty's default.
    Normal,
    /// Read first.
    High,
}

impl Importance {
    /// Lowest first, as the toggle shows them.
    pub const ALL: [Self; 3] = [Self::Low, Self::Normal, Self::High];

    /// The word Rusty stores.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::High => "high",
        }
    }

    /// The toggle's label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Normal => "Normal",
            Self::High => "High",
        }
    }

    /// One of the three words, as Rusty reads them: `medium` is `normal`.
    #[must_use]
    pub fn parse(word: &str) -> Option<Self> {
        match word.trim().to_ascii_lowercase().as_str() {
            "low" => Some(Self::Low),
            "normal" | "medium" => Some(Self::Normal),
            "high" => Some(Self::High),
            _ => None,
        }
    }
}

/// The categories the memories hold, sorted, each once.
#[must_use]
pub fn categories(memories: &[Memory]) -> Vec<String> {
    let mut found: Vec<String> = memories
        .iter()
        .map(|memory| memory.category.clone())
        .filter(|category| !category.is_empty())
        .collect();
    found.sort();
    found.dedup();
    found
}

/// "1 memory", "5 memories".
#[must_use]
pub fn count_line(count: usize) -> String {
    if count == 1 {
        "1 memory".to_string()
    } else {
        format!("{count} memories")
    }
}

/// The category as typed, trimmed, or [`DEFAULT_CATEGORY`] when empty.
#[must_use]
pub fn category_or_default(typed: &str) -> String {
    let typed = typed.trim();
    if typed.is_empty() {
        DEFAULT_CATEGORY.to_string()
    } else {
        typed.to_string()
    }
}

/// One write to the memories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryWrite {
    /// A new memory.
    Store {
        /// What to remember.
        content: String,
        /// Its category as typed; empty files it as [`DEFAULT_CATEGORY`].
        category: String,
        /// How much it matters.
        importance: Importance,
    },
    /// A memory's content, category and importance, sent together as Rusty's app sends them.
    Update {
        /// Rusty's id for it.
        id: String,
        /// Its content.
        content: String,
        /// Its category as typed; empty files it as [`DEFAULT_CATEGORY`].
        category: String,
        /// How much it matters; none keeps what Rusty holds.
        importance: Option<Importance>,
    },
    /// A memory removed.
    Delete {
        /// Rusty's id for it.
        id: String,
    },
}

impl MemoryWrite {
    /// Rusty's tool for it.
    #[must_use]
    pub const fn tool(&self) -> &'static str {
        match self {
            Self::Store { .. } => STORE_MEMORY,
            Self::Update { .. } => UPDATE_MEMORY,
            Self::Delete { .. } => DELETE_MEMORY,
        }
    }

    /// Its arguments, in Rusty's parameter names: the content trimmed, an empty category filed as
    /// [`DEFAULT_CATEGORY`].
    #[must_use]
    pub fn arguments(&self) -> Value {
        match self {
            Self::Store {
                content,
                category,
                importance,
            } => json!({
                "content": content.trim(),
                "category": category_or_default(category),
                "importance": importance.word(),
            }),
            Self::Update {
                id,
                content,
                category,
                importance,
            } => {
                let mut fields = vec![
                    ("id", json!(id)),
                    ("content", json!(content.trim())),
                    ("category", json!(category_or_default(category))),
                ];
                if let Some(importance) = importance {
                    fields.push(("importance", json!(importance.word())));
                }
                Value::Object(
                    fields
                        .into_iter()
                        .map(|(key, value)| (key.to_string(), value))
                        .collect(),
                )
            }
            Self::Delete { id } => json!({ "id": id }),
        }
    }
}
