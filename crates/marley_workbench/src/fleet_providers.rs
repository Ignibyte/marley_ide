//! The fleet's workflow stores over MCP and HTTP (#611, `docs/marley/fleet-contract.md`).
//!
//! Each `mcp` or `http` provider in the settings is a `Remote`: its client, its handshake, what it
//! last answered and how it stands. The fleet's loop polls every remote that is due, together,
//! between reads; each call gives up after a few seconds, so a store that stops answering reads
//! stale, one that fails reads unreachable and is tried again later, and one that speaks another
//! contract reads incompatible. One store's trouble touches only its own agents.
//!
//! A bearer token comes from the environment variable a provider names, and is never logged, shown
//! or put in an error.

use std::collections::HashMap;
use std::fmt;
use std::pin::pin;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context as _;
use context_server::types::requests::CallTool;
use context_server::types::{CallToolParams, CallToolResponse};
use context_server::{ContextServer, ContextServerCommand, ContextServerId};
use futures::AsyncReadExt as _;
use futures::future::{Either, join_all, select};
use gpui::AsyncApp;
use http_client::{AsyncBody, HttpClient, HttpRequestExt as _, Method, Request, Url};
use marley_sdk::stale::DEFAULT_POLL_S;
use marley_sdk::work::WORK_CONTRACT;
use marley_sdk::{
    AgentDetail, AgentList, AgentSummary, Changes, Handshake, HostSnapshot, is_stale,
};
use serde::de::DeserializeOwned;
use settings::FleetProviderContent;
use util::ResultExt as _;

use crate::fleet::{Selected, Source, SourceState};

/// How long one call to a store may take before the poll gives up on it for this round.
const CALL_TIMEOUT: Duration = Duration::from_secs(5);

/// The longest wait before a store that failed is tried again, in seconds.
const BACKOFF_MAX_S: u64 = 30;

/// How long an unreachable store's reason may be on its header.
const REASON_CHARS: usize = 120;

/// One workflow store the settings name, and where Marley stands with it.
#[derive(Clone)]
pub(crate) struct Remote {
    /// The settings entry, which is also how the remote is found again when the settings change.
    pub(crate) entry: FleetProviderContent,
    /// The name its agents are known by, and its header.
    pub(crate) name: String,
    client: Option<Client>,
    handshake: Option<Handshake>,
    status: Status,
    agents: Vec<AgentSummary>,
    /// The detail of its agents a panel or tab wants.
    details: Vec<AgentDetail>,
    cursor: Option<String>,
    /// When it last answered a poll whole.
    answered_ms: Option<u64>,
    failures: u32,
    /// When it may be polled next.
    next_ms: u64,
}

impl fmt::Debug for Remote {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The client holds the bearer, so it is left out.
        formatter
            .debug_struct("Remote")
            .field("name", &self.name)
            .field("status", &self.status)
            .field("agents", &self.agents.len())
            .finish_non_exhaustive()
    }
}

/// How Marley reaches a store.
#[derive(Clone)]
enum Client {
    Mcp(Arc<ContextServer>),
    Http {
        base: String,
        bearer: Option<String>,
        http: Arc<dyn HttpClient>,
    },
}

/// Where Marley stands with a store.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Status {
    Connecting,
    Ready,
    Unreachable(String),
    Incompatible(String),
}

/// Why a poll did not finish.
enum Failure {
    /// A call gave no answer in time: the store may be busy, or silent.
    TimedOut,
    /// The store speaks another contract.
    Incompatible(String),
    /// The store could not be reached, or answered with an error.
    Failed(anyhow::Error),
}

impl From<anyhow::Error> for Failure {
    fn from(error: anyhow::Error) -> Self {
        Self::Failed(error)
    }
}

/// The name a provider entry is shown and known by.
pub(crate) fn name_of(entry: &FleetProviderContent) -> String {
    let host = |url: &str| {
        Url::parse(url)
            .ok()
            .and_then(|url| url.host_str().map(str::to_string))
            .unwrap_or_else(|| url.to_string())
    };
    match entry {
        FleetProviderContent::Pseudo => "Pseudo provider".to_string(),
        FleetProviderContent::Mcp {
            name: Some(name), ..
        }
        | FleetProviderContent::Http {
            name: Some(name), ..
        } => name.clone(),
        FleetProviderContent::Mcp {
            command: Some(command),
            ..
        } => command
            .rsplit('/')
            .next()
            .unwrap_or(command.as_str())
            .to_string(),
        FleetProviderContent::Mcp { url: Some(url), .. }
        | FleetProviderContent::Http { url, .. } => host(url),
        FleetProviderContent::Mcp { .. } => "MCP provider".to_string(),
    }
}

impl Remote {
    fn new(entry: FleetProviderContent) -> Self {
        Self {
            name: name_of(&entry),
            entry,
            client: None,
            handshake: None,
            status: Status::Connecting,
            agents: Vec::new(),
            details: Vec::new(),
            cursor: None,
            answered_ms: None,
            failures: 0,
            next_ms: 0,
        }
    }

    /// Stops its MCP server, if it runs one.
    fn stop(&mut self) {
        if let Some(Client::Mcp(server)) = self.client.take() {
            server.stop().log_err();
        }
        self.handshake = None;
    }

    fn poll_s(&self) -> u64 {
        self.handshake
            .as_ref()
            .and_then(|handshake| handshake.poll_s)
            .unwrap_or(DEFAULT_POLL_S)
            .max(1)
    }
}

/// The remotes for the settings' `entries`, in their order: a remote whose entry is still set is
/// kept, a new entry gets a new one, and one whose entry went is stopped.
pub(crate) fn sync(mut remotes: Vec<Remote>, entries: &[FleetProviderContent]) -> Vec<Remote> {
    let synced = entries
        .iter()
        .filter(|entry| !matches!(entry, FleetProviderContent::Pseudo))
        .map(|entry| {
            remotes
                .iter()
                .position(|remote| &remote.entry == entry)
                .map_or_else(|| Remote::new(entry.clone()), |at| remotes.swap_remove(at))
        })
        .collect();
    for mut gone in remotes {
        gone.stop();
    }
    synced
}

/// Polls every remote that is due, together, and gives them back.
pub(crate) async fn poll_all(
    remotes: Vec<Remote>,
    wanted: &[Selected],
    now: u64,
    cx: &AsyncApp,
) -> Vec<Remote> {
    join_all(
        remotes
            .into_iter()
            .map(|remote| poll(remote, wanted, now, cx)),
    )
    .await
}

async fn poll(mut remote: Remote, wanted: &[Selected], now: u64, cx: &AsyncApp) -> Remote {
    if now < remote.next_ms {
        return remote;
    }
    match poll_once(&mut remote, wanted, cx).await {
        Ok(()) => {
            remote.status = Status::Ready;
            remote.failures = 0;
            remote.answered_ms = Some(now);
            remote.next_ms = now + remote.poll_s() * 1000;
        }
        // No answer in time: what it last said stays, and three such polls read stale.
        Err(Failure::TimedOut) => {
            remote.next_ms = now + remote.poll_s() * 1000;
        }
        Err(Failure::Incompatible(contract)) => {
            remote.status = Status::Incompatible(contract);
            remote.handshake = None;
            remote.agents.clear();
            remote.details.clear();
            back_off(&mut remote, now);
        }
        Err(Failure::Failed(error)) => {
            if !matches!(remote.status, Status::Unreachable(_)) {
                log::info!(
                    "fleet: provider {:?} is unreachable: {error:#}",
                    remote.name
                );
            }
            // The root cause says the most in the few words a header has.
            let reason = error.root_cause().to_string();
            let reason = reason.lines().next().unwrap_or_default();
            remote.status = Status::Unreachable(reason.chars().take(REASON_CHARS).collect());
            remote.stop();
            back_off(&mut remote, now);
        }
    }
    remote
}

/// Waits 1, 2, 4, 8 and 16 seconds after each failure in a row, then 30.
fn back_off(remote: &mut Remote, now: u64) {
    let wait_s = 1_u64
        .checked_shl(remote.failures)
        .unwrap_or(BACKOFF_MAX_S)
        .min(BACKOFF_MAX_S);
    remote.failures = remote.failures.saturating_add(1);
    remote.next_ms = now + wait_s * 1000;
}

/// One poll: the client and the handshake when there are none, then the list, or the changes
/// since the cursor, and the detail of each wanted agent.
async fn poll_once(remote: &mut Remote, wanted: &[Selected], cx: &AsyncApp) -> Result<(), Failure> {
    if remote.client.is_none() {
        remote.client = Some(connect(&remote.entry, &remote.name, cx).await?);
    }
    let client = remote
        .client
        .clone()
        .context("the store's client did not start")?;
    let handshake = if let Some(handshake) = remote.handshake.clone() {
        handshake
    } else {
        let handshake: Handshake = call(
            &client,
            "work_handshake",
            serde_json::json!({}),
            "/marley/v1/handshake",
            cx,
        )
        .await?;
        if handshake.contract != WORK_CONTRACT {
            return Err(Failure::Incompatible(handshake.contract));
        }
        remote.cursor = None;
        remote.handshake = Some(handshake.clone());
        handshake
    };
    let mut read_list = true;
    let mut changed_agents = Vec::new();
    if handshake.offers("changes")
        && let Some(cursor) = remote.cursor.clone()
    {
        let changes: Changes = call(
            &client,
            "work_changes",
            serde_json::json!({ "after": cursor }),
            &format!("/marley/v1/changes?after={}", encode(&cursor)),
            cx,
        )
        .await?;
        read_list = changes.reset
            || changes
                .changed
                .iter()
                .any(|changed| changed.kind == "agent");
        changed_agents = changes
            .changed
            .iter()
            .filter(|changed| changed.kind == "agent")
            .map(|changed| changed.id.clone())
            .collect();
        remote.cursor = changes.cursor.or(Some(cursor));
    }
    if read_list {
        let list: AgentList = call(
            &client,
            "work_agents",
            serde_json::json!({}),
            "/marley/v1/agents",
            cx,
        )
        .await?;
        remote.agents = list.agents;
        if handshake.offers("changes") && (remote.cursor.is_none() || list.cursor.is_some()) {
            remote.cursor = list.cursor;
        }
    }
    let wanted: Vec<&str> = wanted
        .iter()
        .filter(|selected| selected.source == remote.name)
        .map(|selected| selected.agent.as_str())
        .filter(|id| remote.agents.iter().any(|agent| agent.id == *id))
        .collect();
    let mut details = Vec::with_capacity(wanted.len());
    for id in wanted {
        let known = remote.details.iter().find(|detail| detail.agent.id == id);
        let fresh = read_list || changed_agents.iter().any(|changed| changed == id);
        match known {
            Some(detail) if !fresh => details.push(detail.clone()),
            _ => details.push(
                call::<AgentDetail>(
                    &client,
                    "work_agent",
                    serde_json::json!({ "id": id }),
                    &format!("/marley/v1/agents/{}", encode(id)),
                    cx,
                )
                .await?,
            ),
        }
    }
    remote.details = details;
    Ok(())
}

/// Starts the store's client.
async fn connect(
    entry: &FleetProviderContent,
    name: &str,
    cx: &AsyncApp,
) -> anyhow::Result<Client> {
    let id = ContextServerId(Arc::from(format!("marley-fleet-{name}")));
    match entry {
        FleetProviderContent::Mcp {
            command: Some(command),
            args,
            ..
        } => {
            let server = ContextServer::stdio(
                id,
                ContextServerCommand {
                    path: command.into(),
                    args: args.clone().unwrap_or_default(),
                    env: None,
                    timeout: None,
                },
                None,
                None,
            );
            server.start(cx).await?;
            Ok(Client::Mcp(Arc::new(server)))
        }
        FleetProviderContent::Mcp {
            url: Some(url),
            bearer_env,
            ..
        } => {
            let url = web_url(url)?;
            let mut headers = HashMap::default();
            if let Some(token) = bearer(bearer_env.as_deref(), name) {
                headers.insert("Authorization".to_string(), format!("Bearer {token}"));
            }
            let (http, executor) =
                cx.update(|cx| (cx.http_client(), cx.background_executor().clone()));
            let server =
                ContextServer::http(id, &url, headers, http, executor, Some(CALL_TIMEOUT))?;
            server.start(cx).await?;
            Ok(Client::Mcp(Arc::new(server)))
        }
        FleetProviderContent::Mcp { .. } => {
            anyhow::bail!("an mcp provider needs a \"command\" or a \"url\"")
        }
        FleetProviderContent::Http {
            url, bearer_env, ..
        } => {
            let url = web_url(url)?;
            Ok(Client::Http {
                base: url.as_str().trim_end_matches('/').to_string(),
                bearer: bearer(bearer_env.as_deref(), name),
                http: cx.update(|cx| cx.http_client()),
            })
        }
        FleetProviderContent::Pseudo => anyhow::bail!("the pseudo provider has no client"),
    }
}

/// `url` as an HTTP or HTTPS URL.
fn web_url(url: &str) -> anyhow::Result<Url> {
    let parsed = Url::parse(url).with_context(|| format!("{url:?} is not a URL"))?;
    anyhow::ensure!(
        matches!(parsed.scheme(), "http" | "https"),
        "{url:?} is not an http or https URL"
    );
    Ok(parsed)
}

/// The bearer token in the variable `env` names. Only the variable's name is ever logged.
fn bearer(env: Option<&str>, name: &str) -> Option<String> {
    let variable = env?;
    match std::env::var(variable) {
        Ok(token) if !token.is_empty() => {
            log::info!("fleet: provider {name:?} reads its bearer from ${variable}");
            Some(token)
        }
        _ => {
            log::warn!(
                "fleet: provider {name:?} names ${variable} for its bearer, and it is not set"
            );
            None
        }
    }
}

/// A path segment or query value, percent-encoded: everything but the unreserved characters.
fn encode(text: &str) -> String {
    text.bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
                char::from(byte).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}

/// Calls one of the contract's tools, or its HTTP path, and reads the answer; gives up after
/// `CALL_TIMEOUT`.
async fn call<T: DeserializeOwned>(
    client: &Client,
    tool: &str,
    arguments: serde_json::Value,
    path: &str,
    cx: &AsyncApp,
) -> Result<T, Failure> {
    let executor = cx.background_executor().clone();
    let asking = async {
        let value = match client {
            Client::Mcp(server) => {
                let protocol = server.client().context("the MCP server is not running")?;
                let response = protocol
                    .request::<CallTool>(CallToolParams {
                        name: tool.to_string(),
                        arguments: Some(arguments),
                        meta: None,
                    })
                    .await?;
                mcp_value(tool, response)?
            }
            Client::Http { base, bearer, http } => {
                get(http, &format!("{base}{path}"), bearer.as_deref()).await?
            }
        };
        serde_json::from_value::<T>(value).with_context(|| format!("{tool}'s answer did not parse"))
    };
    match select(pin!(asking), pin!(executor.timer(CALL_TIMEOUT))).await {
        Either::Left((answer, _)) => answer.map_err(Failure::Failed),
        Either::Right(_) => Err(Failure::TimedOut),
    }
}

/// A tool's answer: its structured content, else the JSON of its text.
fn mcp_value(tool: &str, response: CallToolResponse) -> anyhow::Result<serde_json::Value> {
    let text = response.text_contents();
    if response.is_error == Some(true) {
        anyhow::bail!("{tool} failed: {}", text.lines().next().unwrap_or_default());
    }
    response.structured_content.map_or_else(
        || serde_json::from_str(&text).with_context(|| format!("{tool} gave no JSON")),
        Ok,
    )
}

/// `GET url` with the bearer, the body as JSON.
async fn get(
    http: &Arc<dyn HttpClient>,
    url: &str,
    bearer: Option<&str>,
) -> anyhow::Result<serde_json::Value> {
    let mut builder = Request::builder()
        .method(Method::GET)
        .uri(url)
        .header("Accept", "application/json")
        .timeout(CALL_TIMEOUT);
    if let Some(token) = bearer {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }
    let request = builder.body(AsyncBody::empty())?;
    let mut response = http.send(request).await?;
    let status = response.status();
    let mut text = String::new();
    response.body_mut().read_to_string(&mut text).await?;
    anyhow::ensure!(status.is_success(), "HTTP {}", status.as_u16());
    serde_json::from_str(&text).context("the answer is not JSON")
}

/// The remote as the panel's source at `now`.
pub(crate) fn source(remote: &Remote, now: u64) -> Source {
    let handshake = remote.handshake.as_ref();
    let poll_s = handshake.and_then(|handshake| handshake.poll_s);
    let stale_after_s = handshake.and_then(|handshake| handshake.stale_after_s);
    let state = match &remote.status {
        Status::Connecting => SourceState::Connecting,
        Status::Unreachable(reason) => SourceState::Unreachable(reason.clone()),
        Status::Incompatible(contract) => SourceState::Incompatible(contract.clone()),
        Status::Ready if is_stale(remote.answered_ms, now, poll_s, stale_after_s) => {
            SourceState::Stale
        }
        Status::Ready => SourceState::Ready,
    };
    let mut hosts: Vec<HostSnapshot> = Vec::new();
    for host in remote
        .details
        .iter()
        .filter_map(|detail| detail.host.as_ref())
    {
        if !hosts.iter().any(|known| known.host.id == host.host.id) {
            hosts.push(host.clone());
        }
    }
    let mut built = Source::for_state(&remote.name, now, state);
    built.capabilities = handshake
        .map(|handshake| handshake.capabilities.clone())
        .unwrap_or_default();
    built.agents.clone_from(&remote.agents);
    built.details.clone_from(&remote.details);
    built.hosts = hosts;
    built.poll_s = poll_s;
    built.stale_after_s = stale_after_s;
    built
}
