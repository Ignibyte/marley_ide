//! Rusty in Marley (#633, #643).
//!
//! `marley.rusty` is Rusty's one switch, off by default; off, Marley starts no `rusty-mcp`, opens
//! no connection and offers Zed's agents nothing. On, Marley keeps one connection to Rusty's MCP
//! server through Zed's MCP client: `embedded` starts `rusty-mcp` on stdio (`MARLEY_RUSTY_MCP`,
//! else the search path) and ends it with the switch, and `service` connects to Rusty's running
//! service at `service_url`, on this machine only. Zed's client sees no server exit, so the
//! connection is checked with a `ping` every 5 s; a lost one says why and is made again after 1,
//! 2, 4 … 60 s. Rusty's settings are read with `settings_list`, again on the embedded server's
//! `notifications/resources/list_changed` and after each write, and written only with
//! `setting_set`: Rusty stays the only writer of its data. The Settings window's Rusty's Server
//! sub-page draws [`RustyServerView`].
//!
//! While Rusty and `agent_tools` are both on, Marley also offers `rusty-mcp` to Zed's agents as
//! the context server `rusty`, beside its own `marley` server (#501), added to Zed's default
//! settings, so a `context_servers.rusty` of the user's own wins. Zed asks before each call, as
//! for every context server.
//!
//! Once a rail's Brain view has shown ([`brain`], #644), the vault is read with `brain_tree` on
//! each connection, each announcement and each write, into the `Vault` global. A page opens in a
//! center tab ([`page`], #645), which reads it again on each announcement, and the right dock's
//! [`knowledge_panel`] (#646) shows its links and tags and brain search. The vault as a graph is a
//! center tab of its own ([`graph_tab`], #647).

pub mod brain;
mod capture;
pub mod decisions_tab;
mod favourites;
mod follow_up;
mod graph_store;
pub mod graph_tab;
mod import;
mod inline_edit;
pub mod knowledge_panel;
mod memory_tab;
pub mod page;
pub mod page_picker;
pub mod project;
mod properties;
mod slider;
pub mod tasks_tab;

use std::any::TypeId;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::pin::pin;
use std::sync::Arc;
use std::time::Duration;

use command_palette_hooks::CommandPaletteFilter;
use context_server::types::Request;
use context_server::types::requests::CallTool;
use context_server::types::{CallToolParams, CallToolResponse};
use context_server::{ContextServer, ContextServerCommand, ContextServerId};
use futures::StreamExt as _;
use futures::channel::mpsc;
use futures::future::{Either, select};
use gpui::{
    App, AppContext as _, AsyncApp, BorrowAppContext as _, Context, Global, Render, SharedString,
    Task, Window,
};
use http_client::Url;
use marley_rusty::settings::{SETTING_SET, SETTINGS_LIST, setting_set_arguments};
use marley_rusty::vault::{BRAIN_TREE, VAULT_PATH_KEY, VaultNode};
use marley_rusty::{EmbeddingProvider, ServerSettings};
use serde_json::{Value, json};
use settings::settings_content::ContextServerSettingsContent;
use settings::{Settings as _, SettingsStore};
use ui::{
    AiSettingItem, AiSettingItemSource, AiSettingItemStatus, ButtonSize, ContextMenu, DropdownMenu,
    DropdownStyle, IconPosition, prelude::*,
};

use crate::MarleySettings;

/// The context server Zed's agents know Rusty's tools by.
const CONTEXT_SERVER: &str = "rusty";

/// The program Rusty's MCP server is.
const PROGRAM: &str = "rusty-mcp";

/// The variable that names the `rusty-mcp` Marley starts, before the search path.
const PROGRAM_VARIABLE: &str = "MARLEY_RUSTY_MCP";

/// Where Rusty's service listens when the settings name nowhere.
const DEFAULT_SERVICE_URL: &str = "http://127.0.0.1:4174/mcp";

/// The sub-page of the Marley settings page that draws [`RustyServerView`].
const SERVER_PAGE: &str = "rusty";

/// How long a start or a call may take before the connection counts as lost.
const CALL_TIMEOUT: Duration = Duration::from_secs(5);

/// How often the connection is checked with a `ping`.
const PING_EVERY: Duration = Duration::from_secs(5);

/// The longest wait before connecting again, in seconds.
const BACKOFF_MAX_S: u64 = 60;

/// MCP's `ping`, answered with an empty object. Zed's own `Ping` reads its answer as `()`, which
/// takes only `null`, so no server's answer to it parses.
struct Ping;

impl Request for Ping {
    type Params = ();
    type Response = Value;
    const METHOD: &'static str = "ping";
}

/// `marley.rusty` as Marley reads it (#643).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RustySettings {
    /// Where Marley reaches Rusty, or [`Source::Off`].
    pub source: Source,
    /// Whether Zed's agents get Rusty's tools: only while Rusty is on.
    pub agent_tools: bool,
}

impl RustySettings {
    /// The block's value: off unless `enabled` is on, and the agents' tools only while it is.
    pub(crate) fn from_content(marley: Option<&settings::MarleySettingsContent>) -> Self {
        let rusty = marley.and_then(|marley| marley.rusty.as_ref());
        let on = rusty.and_then(|rusty| rusty.enabled) == Some(true);
        let source = match rusty.and_then(|rusty| rusty.connection) {
            _ if !on => Source::Off,
            Some(settings::MarleyRustyConnection::Service) => Source::Service(
                rusty
                    .and_then(|rusty| rusty.service_url.clone())
                    .unwrap_or_else(|| DEFAULT_SERVICE_URL.to_string()),
            ),
            Some(settings::MarleyRustyConnection::Embedded) | None => Source::Embedded,
        };
        Self {
            source,
            agent_tools: on && rusty.and_then(|rusty| rusty.agent_tools) == Some(true),
        }
    }
}

/// Where Marley reaches Rusty.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Source {
    /// Rusty is off.
    #[default]
    Off,
    /// `rusty-mcp` on stdio, a child of Marley's.
    Embedded,
    /// Rusty's running service at this URL.
    Service(String),
}

/// The connection's state, as the Rusty's Server page says it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum State {
    #[default]
    Off,
    Starting,
    /// Connected to the server named, through what the second string says.
    Connected {
        server: SharedString,
        via: SharedString,
    },
    /// Lost, with the reason.
    Down(SharedString),
    /// Nothing to connect to, with the reason: no `rusty-mcp`, or a URL Marley refuses.
    Missing(SharedString),
}

/// Marley's connection to Rusty.
#[derive(Default)]
pub(crate) struct Rusty {
    /// The source the connection follows.
    source: Source,
    state: State,
    /// Rusty's settings as last read, or why they could not be.
    settings: Option<Result<ServerSettings, SharedString>>,
    /// The last write Rusty refused, its first line.
    refused: Option<SharedString>,
    server: Option<Arc<ContextServer>>,
    /// The loop that connects, checks and connects again; dropped with the source.
    keeper: Option<Task<()>>,
}

impl Global for Rusty {}

/// The `rusty-mcp` Marley last offered to Zed's agents, if any.
#[derive(Default)]
struct RustyOffer(Option<PathBuf>);

impl Global for RustyOffer {}

/// Rusty's vault as `brain_tree` last gave it, or why it could not be read, for the rails' Brain
/// views (#644). Written only when a read differs, so its observers redraw for a change alone.
#[derive(Default)]
pub(crate) struct Vault {
    pub(crate) tree: Option<Result<Arc<VaultNode>, SharedString>>,
}

impl Global for Vault {}

/// How many changes Rusty has announced on Marley's connection; the Page tabs read their page
/// again on each (#645).
#[derive(Default)]
pub(crate) struct Announced(u64);

impl Global for Announced {}

/// Whether the vault is wanted, a read is under way, or one more is due when it ends. Kept apart
/// from [`Vault`], so a read's start and end redraw nothing (L-572).
#[derive(Default)]
struct VaultReads {
    wanted: bool,
    reading: bool,
    again: bool,
}

impl Global for VaultReads {}

/// Whether the command palette shows Rusty's commands, as last set; none before the first (#661).
#[derive(Default)]
struct PaletteShown(Option<bool>);

impl Global for PaletteShown {}

/// Follows `marley.rusty` and registers the Rusty's Server page's view; [`crate::init`] calls it
/// once.
pub fn init(cx: &mut App) {
    cx.set_global(Rusty::default());
    cx.set_global(RustyOffer::default());
    cx.set_global(Vault::default());
    cx.set_global(VaultReads::default());
    cx.set_global(Announced::default());
    brain::init(cx);
    page::init(cx);
    page_picker::init(cx);
    project::init(cx);
    knowledge_panel::init(cx);
    graph_tab::init(cx);
    tasks_tab::init(cx);
    decisions_tab::init(cx);
    favourites::init(cx);
    capture::init(cx);
    import::init(cx);
    memory_tab::init(cx);
    let view = cx.new(|cx: &mut Context<RustyServerView>| {
        cx.observe_global::<Rusty>(|_, cx| cx.notify()).detach();
        RustyServerView
    });
    settings_ui::MarleyPageViews::set(SERVER_PAGE, view.into(), cx);
    follow_setting(cx);
    cx.observe_global::<SettingsStore>(follow_setting).detach();
}

/// Starts, ends or moves the connection when the setting's source changes, and offers or
/// withdraws Rusty's tools for Zed's agents.
fn follow_setting(cx: &mut App) {
    let wanted = MarleySettings::get_global(cx).rusty.clone();
    if cx.global::<Rusty>().source != wanted.source {
        let source = wanted.source.clone();
        // Dropping the loop and the server ends the embedded child (its transport kills it).
        update(cx, |rusty| {
            rusty.source = source.clone();
            rusty.keeper = None;
            rusty.server = None;
            rusty.settings = None;
            rusty.refused = None;
            rusty.state = if source == Source::Off {
                State::Off
            } else {
                State::Starting
            };
        });
        if cx.global::<Vault>().tree.is_some() {
            cx.update_global::<Vault, _>(|vault, _| vault.tree = None);
        }
        if source != Source::Off {
            let keeper = cx.spawn(async move |cx| keep(source, cx).await);
            update(cx, |rusty| rusty.keeper = Some(keeper));
        }
    }
    offer(wanted.agent_tools, cx);
    filter_palette(wanted.source != Source::Off, cx);
}

/// Hides Rusty's commands from the command palette while Rusty is off, as Zed hides its AI
/// commands while `disable_ai` is set (#661): the `rusty` namespace, and the Brain view's
/// toggle, whose namespace is Marley's own. A key bound to one still answers with the toast.
fn filter_palette(on: bool, cx: &mut App) {
    if cx.default_global::<PaletteShown>().0 == Some(on) {
        return;
    }
    // Before the palette has its filter there is nothing to set; the next call sets it.
    if CommandPaletteFilter::try_global(cx).is_none() {
        return;
    }
    cx.default_global::<PaletteShown>().0 = Some(on);
    let toggle = [TypeId::of::<crate::ToggleBrainView>()];
    CommandPaletteFilter::update_global(cx, |filter, _| {
        if on {
            filter.show_namespace("rusty");
            filter.show_action_types(&toggle);
        } else {
            filter.hide_namespace("rusty");
            filter.hide_action_types(&toggle);
        }
    });
}

/// Changes the `Rusty` global, so its observers, the Rusty's Server page among them, hear it.
fn update(cx: &mut App, change: impl FnOnce(&mut Rusty)) {
    cx.update_global::<Rusty, _>(|rusty, _| change(rusty));
}

/// Where `rusty-mcp` is: `MARLEY_RUSTY_MCP`, else the first on `search_path`. The error says
/// where Marley looked. Blocking: run it off the main thread.
fn find(search_path: Option<OsString>) -> Result<PathBuf, String> {
    if let Some(named) = std::env::var_os(PROGRAM_VARIABLE).filter(|named| !named.is_empty()) {
        let path = PathBuf::from(named);
        return if path.is_file() {
            Ok(path)
        } else {
            Err(format!(
                "{PROGRAM_VARIABLE} names {}, which is not there",
                path.display()
            ))
        };
    }
    which::which_in(PROGRAM, search_path, "/")
        .map_err(|_| format!("no {PROGRAM} on the PATH, and {PROGRAM_VARIABLE} is unset"))
}

/// `url` as an `http` URL on this machine; Rusty's server has no authentication and its tools
/// read and write the brain, so nothing else is reached (D10).
fn loopback(url: &str) -> Result<Url, String> {
    let parsed = Url::parse(url).map_err(|error| format!("{url} is not a URL: {error}"))?;
    let local = matches!(
        parsed.host_str(),
        Some("127.0.0.1" | "localhost" | "[::1]" | "::1")
    );
    if parsed.scheme() == "http" && local {
        Ok(parsed)
    } else {
        Err(format!(
            "{url} is not an http URL on this machine; Marley reaches Rusty's service only there"
        ))
    }
}

/// Why a connection ended.
enum Lost {
    /// Nothing to connect to.
    Missing(String),
    /// The server did not start, or stopped answering.
    Down(String),
}

/// Connects, then keeps the connection, again and again: after 1 s, doubling to 60 s, from the
/// last connection that held.
async fn keep(source: Source, cx: &AsyncApp) {
    let mut failures = 0_u32;
    loop {
        let state = match connected(&source, &mut failures, cx).await {
            Lost::Missing(reason) => State::Missing(reason.into()),
            Lost::Down(reason) => State::Down(reason.into()),
        };
        cx.update(|cx| {
            update(cx, |rusty| {
                rusty.server = None;
                rusty.state = state;
            });
        });
        let wait = 1_u64
            .checked_shl(failures)
            .unwrap_or(u64::MAX)
            .min(BACKOFF_MAX_S);
        failures = failures.saturating_add(1);
        cx.background_executor()
            .timer(Duration::from_secs(wait))
            .await;
    }
}

/// The server `source` names, and how it is reached, for the page; not yet started.
async fn server_for(source: &Source, cx: &AsyncApp) -> Result<(ContextServer, String), Lost> {
    let id = ContextServerId(Arc::from("marley-rusty"));
    Ok(match source {
        Source::Off => return Err(Lost::Missing("Rusty is off".to_string())),
        Source::Embedded => {
            let search_path = cx.update(|cx| crate::agents::launcher(cx).search_path);
            let found = cx
                .background_spawn(futures::future::lazy(move |_| find(search_path)))
                .await;
            let path = match found {
                Ok(path) => path,
                Err(reason) => return Err(Lost::Missing(reason)),
            };
            let via = format!("embedded: {}", path.display());
            let command = ContextServerCommand {
                path,
                args: Vec::new(),
                env: None,
                timeout: None,
            };
            (ContextServer::stdio(id, command, None), via)
        }
        Source::Service(url) => {
            let url = match loopback(url) {
                Ok(url) => url,
                Err(reason) => return Err(Lost::Missing(reason)),
            };
            let (http, executor) =
                cx.update(|cx| (cx.http_client(), cx.background_executor().clone()));
            match ContextServer::http(
                id,
                &url,
                HashMap::default(),
                http,
                executor,
                Some(CALL_TIMEOUT),
            ) {
                Ok(server) => (server, format!("service: {url}")),
                Err(error) => return Err(Lost::Down(first_line(&error.to_string()))),
            }
        }
    })
}

/// One connection, from its start to its loss.
async fn connected(source: &Source, failures: &mut u32, cx: &AsyncApp) -> Lost {
    let (server, via) = match server_for(source, cx).await {
        Ok(found) => found,
        Err(lost) => return lost,
    };
    let server = Arc::new(server);
    let executor = cx.background_executor().clone();
    match select(pin!(server.start(cx)), pin!(executor.timer(CALL_TIMEOUT))).await {
        Either::Left((Ok(()), _)) => {}
        Either::Left((Err(error), _)) => {
            return Lost::Down(format!(
                "{PROGRAM} did not start: {}",
                first_line(&error.to_string())
            ));
        }
        Either::Right(_) => return Lost::Down(format!("{PROGRAM} did not answer within 5 s")),
    }
    let Some(protocol) = server.client() else {
        return Lost::Down(format!("{PROGRAM} is not running"));
    };
    let info = &protocol.initialize.server_info;
    let name = format!("{} {}", info.name, info.version);
    // An embedded server announces each change it sees; Zed's HTTP transport opens no stream for
    // a service's announcements, so a service is read on connect and after each write.
    let (announce, mut announced) = mpsc::unbounded::<()>();
    let _subscription = matches!(source, Source::Embedded).then(|| {
        protocol.on_notification(
            "notifications/resources/list_changed",
            Box::new(move |_, _| {
                announce.unbounded_send(()).ok();
            }),
        )
    });
    *failures = 0;
    cx.update(|cx| {
        update(cx, |rusty| {
            rusty.server = Some(Arc::clone(&server));
            rusty.state = State::Connected {
                server: name.into(),
                via: via.into(),
            };
        });
    });
    read_settings(&server, cx).await;
    cx.update(reread_vault);
    loop {
        match select(pin!(executor.timer(PING_EVERY)), announced.next()).await {
            Either::Right((Some(()), _)) => {
                read_settings(&server, cx).await;
                cx.update(|cx| {
                    cx.update_global::<Announced, _>(|announced, _| announced.0 += 1);
                    reread_vault(cx);
                });
                continue;
            }
            Either::Left(_) | Either::Right((None, _)) => {}
        }
        let ping = protocol.request::<Ping>(());
        match select(pin!(ping), pin!(executor.timer(CALL_TIMEOUT))).await {
            Either::Left((Ok(_), _)) => {}
            Either::Left((Err(error), _)) => {
                return Lost::Down(format!(
                    "{PROGRAM} stopped answering: {}",
                    first_line(&error.to_string())
                ));
            }
            Either::Right(_) => {
                return Lost::Down(format!("{PROGRAM} stopped answering within 5 s"));
            }
        }
    }
}

/// Reads Rusty's settings with `settings_list` into the `Rusty` global.
async fn read_settings(server: &ContextServer, cx: &AsyncApp) {
    let read = call(server, SETTINGS_LIST, json!({}), cx)
        .await
        .and_then(|text| {
            ServerSettings::from_answer(&text)
                .map_err(|error| format!("{SETTINGS_LIST}'s answer did not parse: {error}"))
        });
    cx.update(|cx| {
        update(cx, |rusty| {
            rusty.settings = Some(read.map_err(SharedString::from));
        });
    });
}

/// Writes Rusty's embedding provider with `setting_set`, then reads the settings again; a
/// refusal stays on the page.
fn set_provider(provider: EmbeddingProvider, cx: &App) {
    let Some(server) = cx.global::<Rusty>().server.clone() else {
        return;
    };
    cx.spawn(async move |cx| {
        let arguments = setting_set_arguments(EmbeddingProvider::KEY, provider.as_setting());
        let written = call(&server, SETTING_SET, arguments, cx).await;
        cx.update(|cx| {
            update(cx, |rusty| {
                rusty.refused = written.err().map(SharedString::from);
            });
        });
        read_settings(&server, cx).await;
    })
    .detach();
}

/// Calls one of Rusty's tools within [`CALL_TIMEOUT`] and gives the text it answered, which
/// `rusty-mcp` writes as JSON; a failure gives its first line.
async fn call(
    server: &ContextServer,
    tool: &str,
    arguments: Value,
    cx: &AsyncApp,
) -> Result<String, String> {
    call_within(server, tool, arguments, CALL_TIMEOUT, cx).await
}

/// [`call`] with its own deadline, for the tools that fetch or import (#663).
async fn call_within(
    server: &ContextServer,
    tool: &str,
    arguments: Value,
    deadline: Duration,
    cx: &AsyncApp,
) -> Result<String, String> {
    let protocol = server
        .client()
        .ok_or_else(|| format!("{PROGRAM} is not running"))?;
    // Zed's client gives up after 60 s of its own unless told otherwise.
    let asking = protocol.request_with::<CallTool>(
        CallToolParams {
            name: tool.to_string(),
            arguments: Some(arguments),
            meta: None,
        },
        None,
        Some(deadline),
    );
    let executor = cx.background_executor().clone();
    let response: CallToolResponse =
        match select(pin!(asking), pin!(executor.timer(deadline))).await {
            Either::Left((Ok(response), _)) => response,
            Either::Left((Err(error), _)) => {
                return Err(format!("{tool} failed: {}", first_line(&error.to_string())));
            }
            Either::Right(_) => {
                return Err(format!("{tool} took more than {} s", deadline.as_secs()));
            }
        };
    let text = response.text_contents();
    if response.is_error == Some(true) {
        return Err(format!("{tool} was refused: {}", first_line(&text)));
    }
    Ok(text)
}

/// Calls one of Rusty's tools on Marley's connection; a refusal's error is Rusty's own message.
pub(crate) fn call_tool(
    tool: &'static str,
    arguments: Value,
    cx: &App,
) -> Task<Result<String, String>> {
    call_tool_within(tool, arguments, CALL_TIMEOUT, cx)
}

/// [`call_tool`] with its own deadline (#663).
pub(crate) fn call_tool_within(
    tool: &'static str,
    arguments: Value,
    deadline: Duration,
    cx: &App,
) -> Task<Result<String, String>> {
    let Some(server) = cx.global::<Rusty>().server.clone() else {
        return Task::ready(Err("Rusty is not connected".to_string()));
    };
    cx.spawn(async move |cx| {
        call_within(&server, tool, arguments, deadline, cx)
            .await
            .map_err(|error| {
                let failed = format!("{tool} failed: ");
                let refused = format!("{tool} was refused: ");
                error
                    .strip_prefix(&failed)
                    .or_else(|| error.strip_prefix(&refused))
                    .map_or_else(|| error.clone(), str::to_string)
            })
    })
}

/// Whether Marley is connected to Rusty.
pub(crate) fn is_connected(cx: &App) -> bool {
    matches!(cx.global::<Rusty>().state, State::Connected { .. })
}

/// Whether `marley.rusty.enabled` is on.
pub(crate) fn is_on(cx: &App) -> bool {
    cx.global::<Rusty>().source != Source::Off
}

/// Why the rail's Brain view cannot show now, if it can't: Rusty off, or not connected.
pub(crate) fn unavailable(cx: &App) -> Option<SharedString> {
    match &cx.global::<Rusty>().state {
        State::Connected { .. } => None,
        State::Off => Some(SharedString::new_static(
            "Rusty is off. Turn it on in the Rusty section of the Marley settings.",
        )),
        State::Starting => Some(SharedString::new_static(
            "Rusty is still connecting; the Brain view shows once it is connected.",
        )),
        State::Down(reason) | State::Missing(reason) => {
            Some(format!("Rusty is not connected: {reason}").into())
        }
    }
}

/// The vault's folder: `brain_vault_path` as Rusty stores it, else Rusty's own default,
/// `.rusty/brain` in the home folder.
pub(crate) fn vault_folder(cx: &App) -> PathBuf {
    let stored = match &cx.global::<Rusty>().settings {
        Some(Ok(settings)) => settings.get(VAULT_PATH_KEY).map(PathBuf::from),
        _ => None,
    };
    stored.unwrap_or_else(|| paths::home_dir().join(".rusty").join("brain"))
}

/// Asks for the vault once a Brain view shows: it is read now, and again on each change.
pub(crate) fn want_vault(cx: &mut App) {
    if cx.global::<VaultReads>().wanted {
        return;
    }
    cx.global_mut::<VaultReads>().wanted = true;
    reread_vault(cx);
}

/// Reads the vault with `brain_tree` while it is wanted and Marley is connected; with a read under
/// way, one more runs when it ends.
pub(crate) fn reread_vault(cx: &mut App) {
    let reads = cx.global_mut::<VaultReads>();
    if !reads.wanted {
        return;
    }
    if reads.reading {
        reads.again = true;
        return;
    }
    let rusty = cx.global::<Rusty>();
    let Some(server) = rusty.server.clone() else {
        return;
    };
    let source = rusty.source.clone();
    cx.global_mut::<VaultReads>().reading = true;
    cx.spawn(async move |cx| {
        let read = match call(&server, BRAIN_TREE, json!({}), cx).await {
            Ok(text) => cx
                .background_spawn(futures::future::lazy(move |_| {
                    VaultNode::from_answer(&text)
                }))
                .await
                .map(Arc::new)
                .map_err(|error| format!("{BRAIN_TREE}'s answer did not parse: {error}")),
            Err(error) => Err(error),
        };
        cx.update(|cx| {
            // A read begun before the switch moved belongs to a connection that is gone.
            let tree = Some(read.map_err(SharedString::from));
            if cx.global::<Rusty>().source == source && cx.global::<Vault>().tree != tree {
                cx.update_global::<Vault, _>(|vault, _| vault.tree = tree);
            }
            let reads = cx.global_mut::<VaultReads>();
            reads.reading = false;
            if std::mem::take(&mut reads.again) {
                reread_vault(cx);
            }
        });
    })
    .detach();
}

/// The first line of `text`, trimmed.
fn first_line(text: &str) -> String {
    text.lines().next().unwrap_or_default().trim().to_string()
}

/// Offers Rusty's server to Zed's agents while `wanted`, found as the connection finds it, off the
/// main thread, then settles Zed's defaults.
fn offer(wanted: bool, cx: &App) {
    let search_path = crate::agents::launcher(cx).search_path;
    let found = cx.background_spawn(futures::future::lazy(move |_| {
        wanted.then(|| find(search_path).ok()).flatten()
    }));
    cx.spawn(async move |cx| {
        let found = found.await;
        cx.update(|cx| settle(found.as_ref(), cx));
    })
    .detach();
}

/// Puts `found` into Zed's defaults as `rusty`, or takes Marley's entry out, when that changes
/// what was offered. The defaults' change notifies the settings again, which finds it unchanged.
fn settle(found: Option<&PathBuf>, cx: &mut App) {
    if cx.global::<RustyOffer>().0.as_ref() == found {
        return;
    }
    cx.global_mut::<RustyOffer>().0 = found.cloned();
    cx.update_global::<SettingsStore, _>(|store, cx| {
        store.update_default_settings(cx, |defaults| match found {
            Some(program) => {
                log::info!(
                    "rusty: Zed's agents get Rusty's tools through the context server \
                     {CONTEXT_SERVER}, which runs {}",
                    program.display()
                );
                defaults.project.context_servers.insert(
                    CONTEXT_SERVER.into(),
                    ContextServerSettingsContent::Stdio {
                        enabled: true,
                        remote: false,
                        command: ContextServerCommand {
                            path: program.clone(),
                            args: Vec::new(),
                            env: None,
                            timeout: None,
                        },
                    },
                );
            }
            None => {
                defaults.project.context_servers.remove(CONTEXT_SERVER);
            }
        });
    });
}

/// The Rusty's Server sub-page of the Marley settings page (#643): the connection's state, then
/// Rusty's own settings, read from and written to it.
#[derive(Debug)]
pub struct RustyServerView;

impl Render for RustyServerView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rusty = cx.global::<Rusty>();
        // The row keeps a short word; a path or a reason, often wider than the window, wraps under
        // it.
        let (status, detail, under, under_color): (_, SharedString, SharedString, _) =
            match &rusty.state {
                State::Off => (
                    AiSettingItemStatus::Stopped,
                    SharedString::new_static("off"),
                    SharedString::new_static(
                        "Rusty is off: turn it on in the Rusty section of the Marley settings.",
                    ),
                    Color::Muted,
                ),
                State::Starting => (
                    AiSettingItemStatus::Starting,
                    SharedString::new_static("connecting"),
                    SharedString::new_static("Connecting…"),
                    Color::Muted,
                ),
                State::Connected { server, via } => (
                    AiSettingItemStatus::Running,
                    server.clone(),
                    via.clone(),
                    Color::Muted,
                ),
                State::Down(reason) | State::Missing(reason) => (
                    AiSettingItemStatus::Error,
                    SharedString::new_static("not connected"),
                    reason.clone(),
                    Color::Error,
                ),
            };
        let connected = matches!(rusty.state, State::Connected { .. });
        let settings = rusty.settings.clone();
        let refused = rusty.refused.clone();
        let item = AiSettingItem::new(
            "marley-rusty-server",
            PROGRAM,
            status,
            AiSettingItemSource::Custom,
        )
        .detail_label(detail)
        .details(
            div()
                .pl_7()
                .child(Label::new(under).size(LabelSize::Small).color(under_color)),
        );
        let provider_row = match (connected, settings) {
            (true, Some(Ok(settings))) => {
                Some(provider_row(settings.embedding_provider(), window, cx))
            }
            (true, Some(Err(reason))) => Some(muted(reason)),
            (true, None) => Some(muted(SharedString::new_static("Reading Rusty's settings…"))),
            (false, _) => None,
        };
        v_flex()
            .gap_4()
            .child(
                v_flex().child(Label::new("Rusty's Server")).child(
                    Label::new("Marley's connection to rusty-mcp, Rusty's MCP server.")
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                ),
            )
            .child(item)
            .child(
                v_flex()
                    .gap_2()
                    .child(Label::new("Rusty's Settings"))
                    .child(provider_row.unwrap_or_else(|| {
                        muted(SharedString::new_static(
                            "Rusty's settings show once Marley is connected.",
                        ))
                    }))
                    .when_some(refused, |this, refused| {
                        this.child(
                            Label::new(refused)
                                .size(LabelSize::Small)
                                .color(Color::Error),
                        )
                    }),
            )
    }
}

/// A muted line on the Rusty's Server page.
fn muted(text: SharedString) -> AnyElement {
    Label::new(text)
        .size(LabelSize::Small)
        .color(Color::Muted)
        .into_any_element()
}

/// The embedding provider's row: its name, Rusty's words for the current one, and a dropdown
/// that writes Rusty's setting.
fn provider_row(
    current: EmbeddingProvider,
    window: &mut Window,
    cx: &mut Context<RustyServerView>,
) -> AnyElement {
    let menu = window.use_keyed_state(("marley-rusty-provider", current as usize), cx, {
        move |window, cx| {
            ContextMenu::new(window, cx, move |mut menu, _, _| {
                for provider in EmbeddingProvider::ALL {
                    menu = menu.toggleable_entry(
                        provider.label(),
                        provider == current,
                        IconPosition::End,
                        None,
                        move |_, cx| set_provider(provider, cx),
                    );
                }
                menu
            })
        }
    });
    h_flex()
        .w_full()
        .justify_between()
        .gap_4()
        .child(
            v_flex()
                .min_w_0()
                .child(Label::new("Embedding Provider"))
                .child(
                    Label::new(format!(
                        "Rusty's embedding_provider: what its brain search embeds pages with. {}",
                        current.description()
                    ))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
                ),
        )
        .child(
            DropdownMenu::new("marley-rusty-provider", current.label(), menu)
                .trigger_size(ButtonSize::Medium)
                .style(DropdownStyle::Outlined),
        )
        .into_any_element()
}
