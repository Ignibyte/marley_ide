//! Rusty's own server settings, as `rusty-mcp`'s `settings_list` gives them.
//!
//! The embedding provider came first (#643); since #666 the keys Rusty's app lists, the mask, and
//! the embedding status.

use serde::Deserialize;
use serde_json::{Value, json};

/// The tool that lists Rusty's stored settings.
pub const SETTINGS_LIST: &str = "settings_list";

/// The tool that writes one of Rusty's settings.
pub const SETTING_SET: &str = "setting_set";

/// One row of `settings_list`'s answer.
#[derive(Debug, Deserialize)]
struct Entry {
    key: String,
    value: String,
}

/// Rusty's stored settings: only the keys Rusty stores, a credential-like key's value masked by
/// Rusty itself.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerSettings {
    entries: Vec<(String, String)>,
}

impl ServerSettings {
    /// Reads `settings_list`'s answer, a JSON array of `{"key", "value"}`.
    ///
    /// # Errors
    ///
    /// When the answer is not that array.
    pub fn from_answer(text: &str) -> Result<Self, serde_json::Error> {
        let entries: Vec<Entry> = serde_json::from_str(text)?;
        Ok(Self {
            entries: entries
                .into_iter()
                .map(|entry| (entry.key, entry.value))
                .collect(),
        })
    }

    /// The value Rusty stores for `key`, if it stores one.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(stored, _)| stored == key)
            .map(|(_, value)| value.as_str())
    }

    /// The embedding provider Rusty uses, `Auto` where it stores none.
    #[must_use]
    pub fn embedding_provider(&self) -> EmbeddingProvider {
        EmbeddingProvider::from_stored(self.get(EmbeddingProvider::KEY))
    }

    /// The stored keys outside [`KNOWN`], with their values, in Rusty's order (#666).
    #[must_use]
    pub fn others(&self) -> Vec<(&str, &str)> {
        self.entries
            .iter()
            .filter(|(key, _)| !KNOWN.iter().any(|known| known.key == key))
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect()
    }
}

/// What Rusty hands back in place of a credential's value (its `settings_manager::MASK`).
pub const MASK: &str = "•••";

/// Whether Rusty masks `key`'s value: a key naming a key, token, secret or password, as Rusty's
/// `looks_secret` reads it.
#[must_use]
pub fn looks_secret(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    ["key", "token", "secret", "password", "passwd"]
        .iter()
        .any(|needle| key.contains(needle))
}

/// One of the settings Rusty's own app lists, with its words and what Rusty does when it is
/// unset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownSetting {
    /// Rusty's key.
    pub key: &'static str,
    /// What it does, in Rusty's app's words.
    pub about: &'static str,
    /// What Rusty uses while it is unset.
    pub fallback: &'static str,
}

/// The settings Rusty's app lists, in its order (#666); the Rusty's Server page lists the same.
pub const KNOWN: [KnownSetting; 10] = [
    KnownSetting {
        key: "brain_vault_path",
        about: "The brain vault folder (an Obsidian vault). Restart the service after changing it.",
        fallback: "~/.rusty/brain",
    },
    KnownSetting {
        key: "notes_path",
        about: "The notes folder the notes tools use; inside the vault unless you point it \
                elsewhere (rusty-cli notes adopt moves an older folder in).",
        fallback: "<vault>/notes",
    },
    KnownSetting {
        key: EmbeddingProvider::KEY,
        about: "auto (Ollama when it answers), ollama, openai (needs openai_api_key in Secrets), \
                or off.",
        fallback: "auto",
    },
    KnownSetting {
        key: "embedding_model",
        about: "Overrides the provider's default model (nomic-embed-text, text-embedding-3-small).",
        fallback: "provider default",
    },
    KnownSetting {
        key: "ollama_url",
        about: "Where Ollama listens.",
        fallback: "http://127.0.0.1:11434",
    },
    KnownSetting {
        key: "pin_timeout_minutes",
        about: "How long an unlock of the Secrets tab lasts.",
        fallback: "5",
    },
    KnownSetting {
        key: "skills_enabled",
        about: "Whether the skills store is served to agents.",
        fallback: "true",
    },
    KnownSetting {
        key: "skills_path",
        about: "Where skills live (active/ and staging/).",
        fallback: "~/.rusty/skills",
    },
    KnownSetting {
        key: "brain_auto_enrich",
        about: "Enrich captured pages automatically.",
        fallback: "false",
    },
    KnownSetting {
        key: "default_workflow",
        about: "The default agent workflow name.",
        fallback: "deep",
    },
];

/// What Enter in a setting's field writes, if anything.
///
/// Nothing when the text is what Rusty stores, when the field over a masked value is still empty,
/// or when the text is the mask itself, which Rusty would refuse.
#[must_use]
pub fn value_to_write(stored: Option<&str>, typed: &str) -> Option<String> {
    let masked = stored == Some(MASK);
    if typed == MASK || (masked && typed.is_empty()) || stored.unwrap_or_default() == typed {
        None
    } else {
        Some(typed.to_string())
    }
}

/// The tool that tells how far Rusty's brain search has its pages embedded.
pub const BRAIN_SEMANTIC_STATUS: &str = "brain_semantic_status";

/// `brain_semantic_status`'s answer.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct SemanticStatus {
    /// `provider:model` in use; none while brain search is full-text only.
    pub provider: Option<String>,
    /// What has vectors.
    pub stats: SemanticStats,
    /// Pages whose vectors are missing or older than the page.
    pub stale: usize,
}

/// What Rusty's semantic index holds.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct SemanticStats {
    /// The model the stored vectors came from.
    pub model: Option<String>,
    /// Pages with vectors.
    pub pages: usize,
    /// Chunks with vectors.
    pub chunks: usize,
}

impl SemanticStatus {
    /// `brain_semantic_status`'s answer.
    ///
    /// # Errors
    ///
    /// When the answer is not a status.
    pub fn from_answer(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// The status in one line.
    #[must_use]
    pub fn line(&self) -> String {
        let Some(provider) = &self.provider else {
            return "No embedding provider: brain search is full-text only.".to_string();
        };
        let waiting = if self.stale == 0 {
            String::new()
        } else {
            format!("; {} waiting", count(self.stale, "page"))
        };
        format!(
            "Embedding with {provider}: {} in {} have vectors{waiting}.",
            count(self.stats.pages, "page"),
            count(self.stats.chunks, "chunk"),
        )
    }
}

/// "1 page", "2 pages".
fn count(number: usize, noun: &str) -> String {
    if number == 1 {
        format!("1 {noun}")
    } else {
        format!("{number} {noun}s")
    }
}

/// Which embedder Rusty's brain search uses, from its `embedding_provider` setting. Marley shows
/// it and writes it, and adds no fallback of its own: `openai` runs only when named.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingProvider {
    /// Ollama when it runs on this machine, else none.
    Auto,
    /// Rusty's local Ollama.
    Ollama,
    /// The `openai` embeddings, which send page text off the machine.
    OpenAi,
    /// No vectors: brain search stays full-text.
    Off,
}

impl EmbeddingProvider {
    /// Rusty's key for the provider.
    pub const KEY: &'static str = "embedding_provider";

    /// Every provider, in the order the settings page lists them.
    pub const ALL: [Self; 4] = [Self::Auto, Self::Ollama, Self::OpenAi, Self::Off];

    /// The provider a stored value names, read as Rusty reads it (its `semantic.rs`): trimmed and
    /// lower-cased; `off`, `none` and `false` are off; any other word, and no value, is auto.
    #[must_use]
    pub fn from_stored(value: Option<&str>) -> Self {
        match value
            .map(|value| value.trim().to_ascii_lowercase())
            .as_deref()
        {
            Some("off" | "none" | "false") => Self::Off,
            Some("ollama") => Self::Ollama,
            Some("openai") => Self::OpenAi,
            _ => Self::Auto,
        }
    }

    /// The value written back with `setting_set`.
    #[must_use]
    pub const fn as_setting(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Ollama => "ollama",
            Self::OpenAi => "openai",
            Self::Off => "off",
        }
    }

    /// The provider's name on the settings page.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Ollama => "Ollama",
            Self::OpenAi => "OpenAI",
            Self::Off => "Off",
        }
    }

    /// What the provider does, in Rusty's terms.
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::Auto => "Ollama when it runs on this machine, else full-text search only.",
            Self::Ollama => "Rusty's local Ollama.",
            Self::OpenAi => {
                "OpenAI's embeddings: page text leaves this machine. Needs openai_api_key in \
                 Rusty's secrets."
            }
            Self::Off => "No vectors: brain search stays full-text.",
        }
    }
}

/// `setting_set`'s arguments for `key` and `value`.
#[must_use]
pub fn setting_set_arguments(key: &str, value: &str) -> Value {
    json!({ "key": key, "value": value })
}
