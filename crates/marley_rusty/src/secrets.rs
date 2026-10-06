//! Rusty's secrets vault as its tools serve it, and the writes Marley makes (#667).
//!
//! `secret_list` gives the names only. A value shows, changes or goes only with a live token from
//! `secret_unlock`, which takes the PIN; once a PIN is set, setting and deleting need the token too
//! (Rusty's TICKET-049). None of these answers or arguments may reach a log: a parse failure here
//! says what failed and never quotes the answer, since serde's message can carry a value.

use serde::Deserialize;
use serde_json::{Map, Value};

/// Rusty's tool for the secrets' names.
pub const SECRET_LIST: &str = "secret_list";
/// Whether a PIN is set, whether the vault is unlocked, and any lockout.
pub const SECRET_PIN_STATUS: &str = "secret_pin_status";
/// Sets the PIN, or changes it with the token.
pub const SECRET_PIN_SET: &str = "secret_pin_set";
/// Unlocks the vault with the PIN and answers the token.
pub const SECRET_UNLOCK: &str = "secret_unlock";
/// Locks the vault; the token stops working.
pub const SECRET_LOCK: &str = "secret_lock";
/// One secret's value, with the token.
pub const SECRET_REVEAL: &str = "secret_reveal";
/// Replaces one secret's value, with the token.
pub const SECRET_UPDATE: &str = "secret_update";
/// Sets a secret; the token once a PIN is set.
pub const SECRET_SET: &str = "secret_set";
/// Deletes a secret; the token once a PIN is set.
pub const SECRET_DELETE: &str = "secret_delete";

/// The lock's state, as `secret_pin_status` answers it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct PinStatus {
    /// A PIN exists.
    pub set: bool,
    /// A token is live right now, held by some client.
    pub unlocked: bool,
    /// Seconds left on a lockout after wrong PINs, or zero.
    pub locked_out_seconds: u64,
}

/// `secret_unlock`'s answer.
#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct Unlock {
    /// The token every guarded call takes.
    pub token: String,
    /// How long the token lives.
    #[serde(default)]
    pub expires_in_seconds: u64,
}

impl std::fmt::Debug for Unlock {
    // The token never reaches a log through a `{:?}`.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Unlock")
            .field("expires_in_seconds", &self.expires_in_seconds)
            .finish_non_exhaustive()
    }
}

/// `secret_reveal`'s answer.
#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct Revealed {
    /// The secret's name.
    pub key: String,
    /// Its value.
    pub value: String,
}

impl std::fmt::Debug for Revealed {
    // The value never reaches a log through a `{:?}`.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Revealed")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

/// `secret_list`'s answer.
///
/// # Errors
///
/// When the answer is not a list of names; the error does not quote it.
pub fn names_from_answer(text: &str) -> Result<Vec<String>, String> {
    parsed(SECRET_LIST, text)
}

/// `secret_pin_status`'s answer.
///
/// # Errors
///
/// When the answer is not the lock's state; the error does not quote it.
pub fn status_from_answer(text: &str) -> Result<PinStatus, String> {
    parsed(SECRET_PIN_STATUS, text)
}

/// `secret_unlock`'s answer.
///
/// # Errors
///
/// When the answer is not an unlock; the error does not quote it.
pub fn unlock_from_answer(text: &str) -> Result<Unlock, String> {
    parsed(SECRET_UNLOCK, text)
}

/// `secret_reveal`'s answer.
///
/// # Errors
///
/// When the answer is not a secret and its value; the error does not quote it.
pub fn revealed_from_answer(text: &str) -> Result<Revealed, String> {
    parsed(SECRET_REVEAL, text)
}

fn parsed<T: for<'de> Deserialize<'de>>(tool: &str, text: &str) -> Result<T, String> {
    serde_json::from_str(text).map_err(|_| format!("{tool}'s answer did not parse"))
}

/// One write to the vault or its PIN. Its `Debug` names the tool and the key only.
#[derive(Clone, PartialEq, Eq)]
pub enum SecretWrite {
    /// A secret set; the token once a PIN is set.
    Set {
        /// The secret's name.
        key: String,
        /// Its value.
        value: String,
        /// The live token, if the tab holds one.
        token: Option<String>,
    },
    /// One secret's value replaced.
    Update {
        /// The secret's name.
        key: String,
        /// Its new value.
        value: String,
        /// The live token.
        token: String,
    },
    /// A secret deleted; the token once a PIN is set.
    Delete {
        /// The secret's name.
        key: String,
        /// The live token, if the tab holds one.
        token: Option<String>,
    },
    /// The PIN set, or changed with the token.
    PinSet {
        /// The new PIN.
        pin: String,
        /// The live token, needed to change a PIN.
        token: Option<String>,
    },
}

impl std::fmt::Debug for SecretWrite {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key = match self {
            Self::Set { key, .. } | Self::Update { key, .. } | Self::Delete { key, .. } => {
                key.as_str()
            }
            Self::PinSet { .. } => "",
        };
        formatter
            .debug_struct("SecretWrite")
            .field("tool", &self.tool())
            .field("key", &key)
            .finish_non_exhaustive()
    }
}

impl SecretWrite {
    /// Rusty's tool for it.
    #[must_use]
    pub const fn tool(&self) -> &'static str {
        match self {
            Self::Set { .. } => SECRET_SET,
            Self::Update { .. } => SECRET_UPDATE,
            Self::Delete { .. } => SECRET_DELETE,
            Self::PinSet { .. } => SECRET_PIN_SET,
        }
    }

    /// Its arguments, in Rusty's parameter names; a token the tab does not hold is left out.
    #[must_use]
    pub fn arguments(&self) -> Value {
        let fields: Vec<(&str, Option<&str>)> = match self {
            Self::Set { key, value, token } => vec![
                ("key", Some(key)),
                ("value", Some(value)),
                ("token", token.as_deref()),
            ],
            Self::Update { key, value, token } => vec![
                ("key", Some(key)),
                ("value", Some(value)),
                ("token", Some(token)),
            ],
            Self::Delete { key, token } => vec![("key", Some(key)), ("token", token.as_deref())],
            Self::PinSet { pin, token } => vec![("pin", Some(pin)), ("token", token.as_deref())],
        };
        let object: Map<String, Value> = fields
            .into_iter()
            .filter_map(|(name, value)| value.map(|value| (name.to_string(), Value::from(value))))
            .collect();
        Value::Object(object)
    }
}
