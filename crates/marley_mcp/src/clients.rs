//! PURE — outside clients (#524, #811): programs the user let in, each with a token of its own,
//! minted at each start. One allowed by name in Browser Clients reaches the browser tools, to read
//! pages or also to act in them; one that paired holds a [`Power`] over every tool. A program that
//! holds the shared endpoint's bearer without proving a Marley terminal has not paired, and reads
//! alone. The transport mints the tokens and asks [`ClientTable::resolve`] who holds a bearer;
//! every decision about what a caller may call is [`permits`].

use std::fmt;

use crate::Refusal;
use crate::auth::ct_eq;
use crate::permission::Tier;
use crate::registry::{ToolSpec, lookup};

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

/// The grant classes whose tools run a command or a program, or start or stop an agent: they need
/// [`Power::Run`] (#811).
pub const RUN_CLASSES: [&str; 3] = ["terminal.write", "harness.write", "agents.write"];

/// The write tools of other classes that need [`Power::Run`]: answering an agent's waiting tool
/// call runs that call (#811).
pub const RUN_TOOLS: [&str; 1] = ["thread_answer"];

/// The tools a program pairs with, open to every caller (#811).
pub const PAIRING_TOOLS: [&str; 2] = ["client_pair", "client_pair_status"];

/// The longest name a client may have.
const MAX_NAME: usize = 32;

/// What a paired client may do (#811), each power holding the ones before it.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Power {
    /// The read tools.
    Read,
    /// The read tools and every write tool that runs nothing.
    Write,
    /// Every tool Marley lists, the ones that run a command or an agent too.
    Run,
}

impl Power {
    /// Every power, weakest first.
    pub const ALL: [Self; 3] = [Self::Read, Self::Write, Self::Run];

    /// The power's word, as `clients.json` and the tools write it.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Run => "run",
        }
    }

    /// The power `word` names.
    #[must_use]
    pub fn parse(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|power| power.as_str() == word)
    }

    /// What a client with this power may do, in words.
    #[must_use]
    pub const fn words(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "read and write",
            Self::Run => "read, write and run",
        }
    }
}

/// The power `spec` needs of a paired client: [`Power::Read`] for a read tool, [`Power::Run`] for
/// one of [`RUN_CLASSES`] or [`RUN_TOOLS`], and [`Power::Write`] for any other write tool.
#[must_use]
pub fn power_needed(spec: &ToolSpec) -> Power {
    match spec.tier {
        Tier::Read => Power::Read,
        Tier::Write
            if RUN_CLASSES.contains(&spec.grant_class)
                || RUN_TOOLS.contains(&spec.name().as_str()) =>
        {
            Power::Run
        }
        Tier::Write => Power::Write,
    }
}

/// A client's name and what it may do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientGrant {
    /// The name the user gave it, which its endpoint file and the Browser tab's mark carry.
    pub name: String,
    /// Whether it may act in pages, beside reading them.
    pub write: bool,
    /// The power it paired with (#811); none for a client allowed by name in Browser Clients, which
    /// reaches the browser tools alone.
    pub power: Option<Power>,
}

/// Who holds the bearer of a request.
///
/// Marley's own agents hold the shared bearer and prove a Marley terminal, or hold Marley's own
/// bearer; a program that holds the shared bearer alone has not paired; a client the user let in
/// holds its own token. Beside [`Caller`](crate::Caller), which says where a call comes from and
/// grants nothing, this is what a call may do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Principal {
    /// Marley's own: its terminals' programs, Zed's agents and its helpers, with every tool.
    Marley,
    /// A program with the shared bearer and no proven Marley terminal (#811): the read tools.
    Unpaired,
    /// A client the user let in, by name or by pairing.
    Client(ClientGrant),
}

impl Principal {
    /// The client's name, or `None` for Marley's own and an unpaired program.
    #[must_use]
    pub fn client_name(&self) -> Option<&str> {
        match self {
            Self::Marley | Self::Unpaired => None,
            Self::Client(grant) => Some(&grant.name),
        }
    }

    /// Whether it is Marley's own.
    #[must_use]
    pub const fn is_marley(&self) -> bool {
        matches!(self, Self::Marley)
    }
}

/// Whether `principal` may call `tool` (a wire name).
///
/// Marley may call every tool. Anyone else calls only tools that `tools/list` lists, besides
/// [`PAIRING_TOOLS`]: an unpaired program the read tools, a paired client those its power covers,
/// and a client allowed by name its grant's browser tools.
///
/// # Errors
///
/// The refusal: `not_paired` for an unpaired program, `not_granted` for a client.
pub fn permits(principal: &Principal, tool: &str) -> Result<(), Refusal> {
    if principal.is_marley() || PAIRING_TOOLS.contains(&tool) {
        return Ok(());
    }
    let needed = lookup(tool)
        .filter(|spec| spec.family.is_served())
        .map(|spec| power_needed(&spec));
    match principal {
        Principal::Marley => Ok(()),
        Principal::Unpaired => match needed {
            Some(Power::Read) => Ok(()),
            Some(_) => Err(Refusal::new(
                "not_paired",
                format!(
                    "{tool} acts, and this program has not paired with Marley, so it may only \
                     read"
                ),
            )
            .next(
                "ask the user to pair this program: they run `marley-mcp-bridge pair NAME` and \
                 pick what it may do when Marley shows the code",
            )
            .next("the read tools answer without pairing; tools/list names them")),
            None => Err(Refusal::new(
                "not_paired",
                format!("{tool} is Marley's own, and this program has not paired with Marley"),
            )),
        },
        Principal::Client(grant) => match (grant.power, needed) {
            (Some(power), Some(needed)) if needed <= power => Ok(()),
            (Some(power), Some(needed)) => Err(Refusal::new(
                "not_granted",
                format!(
                    "{tool} needs the power to {}; the client {} may {}",
                    needed.as_str(),
                    grant.name,
                    power.words()
                ),
            )
            .next(format!(
                "ask the user to give {} that power in Outside Clients (marley: outside clients)",
                grant.name
            ))),
            (None, _) if CLIENT_READ_TOOLS.contains(&tool) => Ok(()),
            (None, _) if grant.write && CLIENT_WRITE_TOOLS.contains(&tool) => Ok(()),
            (None, _) if CLIENT_WRITE_TOOLS.contains(&tool) => Err(Refusal::new(
                "not_granted",
                format!(
                    "{tool} needs the grant to act in pages; the client {} may only read them",
                    grant.name
                ),
            )
            .next("tools/list names the tools this client may call")),
            _ => Err(Refusal::new(
                "not_granted",
                format!("{tool} is not open to the client {}", grant.name),
            )
            .next("tools/list names the tools this client may call")),
        },
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

    /// Who holds `presented` (#811): Marley's own `own_bearer`, the `shared_bearer` of the endpoint
    /// file (Marley's own with a `proven_terminal`, else an unpaired program), or a client's
    /// token. Every token is compared with [`ct_eq`] and none stops the loop, so the time taken
    /// does not say which one matched. An empty token matches nothing.
    #[must_use]
    pub fn resolve(
        &self,
        presented: Option<&str>,
        shared_bearer: &str,
        own_bearer: &str,
        proven_terminal: bool,
    ) -> Option<Principal> {
        let presented = presented?;
        let own = !own_bearer.is_empty() && ct_eq(presented, own_bearer);
        let shared = !shared_bearer.is_empty() && ct_eq(presented, shared_bearer);
        let mut found = match (own, shared) {
            (true, _) => Some(Principal::Marley),
            (false, true) if proven_terminal => Some(Principal::Marley),
            (false, true) => Some(Principal::Unpaired),
            (false, false) => None,
        };
        for entry in &self.entries {
            if ct_eq(presented, &entry.token) && !entry.token.is_empty() && found.is_none() {
                found = Some(Principal::Client(entry.grant.clone()));
            }
        }
        found
    }

    /// Gives the client `name` the power `power` (#811); whether it was allowed.
    pub fn set_power(&mut self, name: &str, power: Power) -> bool {
        let mut found = false;
        for entry in &mut self.entries {
            if entry.grant.name == name {
                entry.grant.power = Some(power);
                entry.grant.write = power >= Power::Write;
                found = true;
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
