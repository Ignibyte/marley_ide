//! Rusty's own server settings, as `rusty-mcp`'s `settings_list` gives them, and the one Marley
//! shows first, the embedding provider (#643).

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
