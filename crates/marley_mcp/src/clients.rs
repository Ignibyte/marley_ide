//! PURE — outside clients (#524): programs the user allowed by name to reach Marley's browser tools,
//! each with a token of its own, minted at each start, and a grant to read pages or also to act in
//! them. The transport mints the tokens and asks [`ClientTable::resolve`] who holds a bearer; every
//! decision about what a client may call is [`permits`].

use std::fmt;

use crate::auth::ct_eq;

/// The tools a client may call with the grant to read pages. The list is explicit, so a tool added
/// to the registry later reaches no client until it is named here.
pub const CLIENT_READ_TOOLS: [&str; 10] = [
    "browser_tabs",
    "browser_look",
    "browser_snapshot",
    "browser_console",
    "browser_network",
    "browser_picks",
    "browser_pick",
    "browser_annotations",
    "browser_recordings",
    "browser_recording",
];

/// The tools a client may call besides [`CLIENT_READ_TOOLS`] with the grant to act in pages.
pub const CLIENT_WRITE_TOOLS: [&str; 8] = [
    "browser_navigate",
    "browser_back",
    "browser_click",
    "browser_type",
    "browser_press",
    "browser_scroll",
    "browser_annotate",
    "browser_check_pick",
];

/// The longest name a client may have.
const MAX_NAME: usize = 32;

/// A client's name and what it may do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientGrant {
    /// The name the user gave it, which its endpoint file and the Browser tab's mark carry.
    pub name: String,
    /// Whether it may act in pages, beside reading them.
    pub write: bool,
}

/// Who holds the bearer of a request.
///
/// Marley's own agents hold the per-boot bearer; a client the user allowed holds its own token.
/// Beside [`Caller`](crate::Caller), which says where a call comes from and grants nothing, this is
/// what a call may do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Principal {
    /// The per-boot bearer's holder: Marley's own bridge and helpers, with every tool.
    Marley,
    /// A client the user allowed by name.
    Client(ClientGrant),
}

impl Principal {
    /// The client's name, or `None` for Marley's own.
    #[must_use]
    pub fn client_name(&self) -> Option<&str> {
        match self {
            Self::Marley => None,
            Self::Client(grant) => Some(&grant.name),
        }
    }
}

/// Whether `principal` may call `tool` (a wire name): Marley may call every tool; a client only a
/// tool its grant's list names.
///
/// # Errors
///
/// Why the client may not, as its tool error reads.
pub fn permits(principal: &Principal, tool: &str) -> Result<(), String> {
    let Principal::Client(grant) = principal else {
        return Ok(());
    };
    if CLIENT_READ_TOOLS.contains(&tool) || (grant.write && CLIENT_WRITE_TOOLS.contains(&tool)) {
        Ok(())
    } else if CLIENT_WRITE_TOOLS.contains(&tool) {
        Err(format!(
            "{tool} needs the grant to act in pages; the client {} may only read them",
            grant.name
        ))
    } else {
        Err(format!("{tool} is not open to outside clients"))
    }
}

/// Why a client could not be allowed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientError {
    /// The name is empty, too long, or holds more than letters, digits, `-` and `_`.
    BadName(String),
    /// The name is one the Browser tab's chip would show as Marley's own agents.
    ReservedName(String),
    /// A client of that name is allowed already.
    Taken(String),
    /// The OS gave no entropy for its token.
    NoEntropy,
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadName(name) => write!(
                f,
                "\"{name}\" is not a client name: use 1 to {MAX_NAME} letters, digits, - and _"
            ),
            Self::ReservedName(name) => write!(f, "\"{name}\" is Marley's own; pick another name"),
            Self::Taken(name) => write!(f, "a client named \"{name}\" is allowed already"),
            Self::NoEntropy => write!(f, "the system gave no randomness for the client's token"),
        }
    }
}

impl std::error::Error for ClientError {}

/// Checks `name` as a client's name.
///
/// A name is 1 to 32 ASCII letters, digits, `-` and `_`, and neither `agent` nor `marley` in any
/// case, which the Browser tab's chip would show as Marley's own. The shape keeps the name safe as
/// a file name.
///
/// # Errors
///
/// [`ClientError::BadName`] or [`ClientError::ReservedName`].
pub fn check_client_name(name: &str) -> Result<(), ClientError> {
    let shaped = !name.is_empty()
        && name.len() <= MAX_NAME
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
    if !shaped {
        Err(ClientError::BadName(name.to_string()))
    } else if name.eq_ignore_ascii_case("agent") || name.eq_ignore_ascii_case("marley") {
        Err(ClientError::ReservedName(name.to_string()))
    } else {
        Ok(())
    }
}

/// An allowed client, for the list the user reviews.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientInfo {
    /// Its name and grant.
    pub grant: ClientGrant,
    /// When it last made a request, in epoch milliseconds; `None` before its first.
    pub last_call_ms: Option<u64>,
}

/// One allowed client with its token.
#[derive(Clone, PartialEq, Eq)]
struct ClientEntry {
    grant: ClientGrant,
    token: String,
    last_call_ms: Option<u64>,
}

/// The clients allowed in this run, with their tokens. The tokens are secrets, so the manual
/// [`Debug`] names the clients and never a token.
#[derive(Default, Clone, PartialEq, Eq)]
pub struct ClientTable {
    entries: Vec<ClientEntry>,
}

impl fmt::Debug for ClientTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<&str> = self
            .entries
            .iter()
            .map(|entry| entry.grant.name.as_str())
            .collect();
        f.debug_struct("ClientTable")
            .field("clients", &names)
            .field("tokens", &"***")
            .finish()
    }
}

impl ClientTable {
    /// Lets `grant`'s client in with `token`.
    ///
    /// # Errors
    ///
    /// A name [`check_client_name`] refuses, or one allowed already.
    pub fn allow(&mut self, grant: ClientGrant, token: String) -> Result<(), ClientError> {
        check_client_name(&grant.name)?;
        if self.is_allowed(&grant.name) {
            return Err(ClientError::Taken(grant.name));
        }
        self.entries.push(ClientEntry {
            grant,
            token,
            last_call_ms: None,
        });
        Ok(())
    }

    /// Takes `name` out, so its token opens nothing; whether it was in.
    pub fn cut_off(&mut self, name: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|entry| entry.grant.name != name);
        self.entries.len() != before
    }

    /// Who holds `presented`: Marley's own `marley_bearer`, or an allowed client's token. Every
    /// token is compared with [`ct_eq`] and none stops the loop, so the time taken does not say
    /// which one matched. An empty token matches nothing.
    #[must_use]
    pub fn resolve(&self, presented: Option<&str>, marley_bearer: &str) -> Option<Principal> {
        let presented = presented?;
        let mut found = (!marley_bearer.is_empty() && ct_eq(presented, marley_bearer))
            .then_some(Principal::Marley);
        for entry in &self.entries {
            if ct_eq(presented, &entry.token) && !entry.token.is_empty() && found.is_none() {
                found = Some(Principal::Client(entry.grant.clone()));
            }
        }
        found
    }

    /// Notes that `name` made a request at `now`.
    pub fn touch(&mut self, name: &str, now: u64) {
        for entry in &mut self.entries {
            if entry.grant.name == name {
                entry.last_call_ms = Some(now);
            }
        }
    }

    /// Whether a client named `name` is allowed.
    #[must_use]
    pub fn is_allowed(&self, name: &str) -> bool {
        self.entries.iter().any(|entry| entry.grant.name == name)
    }

    /// The allowed clients, in the order they were allowed.
    #[must_use]
    pub fn clients(&self) -> Vec<ClientInfo> {
        self.entries
            .iter()
            .map(|entry| ClientInfo {
                grant: entry.grant.clone(),
                last_call_ms: entry.last_call_ms,
            })
            .collect()
    }
}
