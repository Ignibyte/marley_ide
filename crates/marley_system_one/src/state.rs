//! The state a question is asked about.
//!
//! A state is labeled lines, `label: value`, each value passed through the host's mask before it
//! is added, and the SHA-256 that tells one state from the next.

use std::fmt;

use sha2::{Digest as _, Sha256};

/// The most characters of one text value a state carries: the masked text is cut after them.
pub const TEXT_LIMIT: usize = 300;

/// How much of a project's state may leave the machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Detail {
    /// Facts and text.
    Full,
    /// Facts alone, for a metadata-only project.
    Facts,
}

/// A state as it is sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    /// The labeled lines.
    pub text: String,
    /// The text's SHA-256, in hex.
    pub hash: String,
}

/// Builds a [`State`] from facts computed in code and from text, each value through the mask.
pub struct StateBuilder<'mask> {
    detail: Detail,
    mask: &'mask dyn Fn(&str) -> String,
    lines: Vec<String>,
}

impl fmt::Debug for StateBuilder<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StateBuilder")
            .field("detail", &self.detail)
            .field("lines", &self.lines.len())
            .finish_non_exhaustive()
    }
}

impl<'mask> StateBuilder<'mask> {
    /// A state of `detail`, each value passed through `mask`.
    #[must_use]
    pub fn new(detail: Detail, mask: &'mask dyn Fn(&str) -> String) -> Self {
        Self {
            detail,
            mask,
            lines: Vec::new(),
        }
    }

    /// A fact computed in code, such as a name, a count or an exit code, kept at every detail.
    #[must_use]
    pub fn fact(mut self, label: &str, value: &str) -> Self {
        let value = (self.mask)(value);
        self.lines.push(format!("{label}: {}", one_line(&value)));
        self
    }

    /// Text the user or an agent wrote, such as a command, masked whole and then cut to
    /// [`TEXT_LIMIT`] characters. A [`Detail::Facts`] state leaves it out.
    #[must_use]
    pub fn text(mut self, label: &str, value: &str) -> Self {
        if self.detail == Detail::Facts {
            return self;
        }
        // Masked before it is cut, so a secret the cut would split is still found whole.
        let value = (self.mask)(value);
        self.lines
            .push(format!("{label}: {}", cut(&one_line(&value), TEXT_LIMIT)));
        self
    }

    /// The state, or `None` when nothing was added: an empty state is refused, never asked.
    #[must_use]
    pub fn build(self) -> Option<State> {
        if self.lines.is_empty() {
            return None;
        }
        let text = self.lines.join("\n");
        let hash = hex(&Sha256::digest(text.as_bytes()));
        Some(State { text, hash })
    }
}

/// `text` cut to `limit` characters, ending in `…` when it was longer.
#[must_use]
pub fn cut(text: &str, limit: usize) -> String {
    match text.char_indices().nth(limit) {
        Some((end, _)) => format!("{}…", text.get(..end).unwrap_or(text)),
        None => text.to_string(),
    }
}

/// A value on one line, since each line of a state is one label's.
fn one_line(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|character| {
            if character == '\n' || character == '\r' {
                ' '
            } else {
                character
            }
        })
        .collect()
}

/// `bytes` as lowercase hex.
fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push(char::from(DIGITS[usize::from(byte >> 4)]));
        text.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    text
}
