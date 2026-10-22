//! Outside clients of Marley's tools (#524, #811).
//!
//! A program pairs itself (#811): `marley-mcp-bridge pair NAME` calls `client_pair`, Marley shows
//! the name and a six-digit code in every window, and the user picks what it may do (read; read
//! and write; read, write and run) or refuses; `client_pair_status` gives the command the answer.
//! The user may also allow a program by name in Outside Clients (`marley: outside clients`), to
//! read pages or also to act in them, the browser tools alone (#524). Marley keeps the list in
//! `<data>/mcp/clients.json` (0600, no token), never in Zed's settings, which a project's own
//! settings could add to. At each start it mints each client a token and writes it with the
//! server's URL into `<data>/mcp/clients/<name>.json` (0600, in a 0700 folder), the file a client
//! points Marley's bridge at; the files go at quit. Cut Off takes a client out at once: the main
//! thread's copy of the list changes first, which the Browser tab's mark and a call still waiting
//! read, then the server's table and the files, off the main thread, as the server's locks are
//! never taken here.

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context as _, anyhow};
use editor::Editor;
use futures::FutureExt as _;
use futures::channel::oneshot;
use gpui::{
    App, AppContext as _, ClipboardItem, Context, DismissEvent, Entity, EventEmitter, FocusHandle,
    Focusable, Global, Render, Task, Window,
};
use marley_mcp::{
    AppCall, ClientError, ClientInfo, Power, Refusal, ToolAnswer, check_client_name, discovery,
    transport,
};
use rand::Rng as _;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use ui::{Checkbox, TintColor, ToggleState, prelude::*};
use util::ResultExt as _;
use util::shell::ShellKind;
use workspace::notifications::simple_message_notification::MessageNotification;
use workspace::notifications::{NotificationId, dismiss_app_notification, show_app_notification};
use workspace::{ModalView, Workspace};

use crate::asks::Ask;
use crate::browser::BrowserHub;
use crate::prompts::Role;
use crate::{AllowBrowserClient, BrowserClients, OutsideClients};

/// The registry's file in `<data>/mcp`.
const REGISTRY_FILE: &str = "clients.json";

/// The folder of the clients' endpoint files in `<data>/mcp`.
const CLIENTS_DIR: &str = "clients";

/// How often an open Outside Clients reads when each client last called.
const LAST_CALLS_EVERY: Duration = Duration::from_secs(5);

/// How long a pairing waits for the user (#811).
const PAIR_WAIT: Duration = Duration::from_secs(120);

/// How many pairings may wait at once, so a program cannot fill the screen with them.
const MAX_PAIRINGS: usize = 3;

/// The clients the user allowed, as `clients.json` keeps them: no token.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Registry {
    /// The clients, in the order they were allowed.
    #[serde(default)]
    pub clients: Vec<RegisteredClient>,
}

/// One allowed client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisteredClient {
    /// Its name.
    pub name: String,
    /// Whether it may act in pages, beside reading them.
    pub write: bool,
    /// When the user allowed it, in epoch seconds.
    pub allowed_at: u64,
    /// The power it paired with (#811); none for one allowed by name, which reaches the browser
    /// tools alone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub power: Option<Power>,
}

/// `<data>/mcp`.
fn mcp_dir_in(data_dir: &Path) -> PathBuf {
    data_dir.join("mcp")
}

/// The endpoint file of the client `name`.
fn client_file_in(data_dir: &Path, name: &str) -> PathBuf {
    mcp_dir_in(data_dir)
        .join(CLIENTS_DIR)
        .join(format!("{name}.json"))
}

/// The registry in `data_dir`, an empty one when there is none.
///
/// # Errors
///
/// When the file cannot be read or is not a registry.
pub fn read_registry_in(data_dir: &Path) -> anyhow::Result<Registry> {
    let path = mcp_dir_in(data_dir).join(REGISTRY_FILE);
    match std::fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text)
            .with_context(|| format!("{} is not a client list Marley reads", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Registry::default()),
        Err(error) => Err(error).with_context(|| format!("reading {}", path.display())),
    }
}

/// Writes `registry` into `data_dir` owner-only, through a file renamed into place, so a crash
/// leaves the old list or the new one.
///
/// # Errors
///
/// When a folder or the file cannot be written.
pub fn write_registry_in(data_dir: &Path, registry: &Registry) -> anyhow::Result<()> {
    let dir = mcp_dir_in(data_dir);
    std::fs::create_dir_all(&dir).with_context(|| format!("making {}", dir.display()))?;
    let json = serde_json::to_string_pretty(registry)?;
    let staged = format!(".{REGISTRY_FILE}.new");
    discovery::write_endpoint_file_in(&dir, &staged, &json)
        .with_context(|| format!("writing {}", dir.join(&staged).display()))?;
    std::fs::rename(dir.join(&staged), dir.join(REGISTRY_FILE))
        .with_context(|| format!("replacing {}", dir.join(REGISTRY_FILE).display()))
}

/// The folder of the clients' endpoint files, made owner-only.
fn clients_dir_in(data_dir: &Path) -> std::io::Result<PathBuf> {
    let dir = mcp_dir_in(data_dir).join(CLIENTS_DIR);
    std::fs::create_dir_all(&dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(dir)
}

/// Writes the endpoint file of the client `name`, the server's `url` with its `token`; its path.
fn write_client_file_in(
    data_dir: &Path,
    name: &str,
    url: &str,
    token: &str,
) -> std::io::Result<PathBuf> {
    let dir = clients_dir_in(data_dir)?;
    let file = format!("{name}.json");
    discovery::write_endpoint_file_in(&dir, &file, &transport::discovery_json(url, token))?;
    Ok(dir.join(file))
}

/// Removes every endpoint file of the clients' folder: at start, since a crash leaves them, and at
/// quit, since their tokens die with the run.
///
/// # Errors
///
/// When the folder cannot be read or a file removed.
pub fn clear_client_files_in(data_dir: &Path) -> std::io::Result<()> {
    let dir = mcp_dir_in(data_dir).join(CLIENTS_DIR);
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    for entry in entries {
        let path = entry?.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            std::fs::remove_file(&path)?;
        }
    }
    Ok(())
}

/// Epoch seconds now.
fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

/// Marley's outside clients while its server runs.
struct ClientsState {
    data_dir: PathBuf,
    url: String,
    shared: transport::Shared,
    table: transport::Clients,
    /// The registry as the main thread keeps it, or why it could not be read; none until read.
    registry: Option<Result<Registry, SharedString>>,
    /// The pairings programs asked for, until their command reads the answer (#811).
    pairings: Vec<Pairing>,
}

impl Global for ClientsState {}

/// Lets in the clients the user allowed, once the server runs (#524): off the main thread the
/// last run's endpoint files go and the registry is read; then each client gets a new token and
/// its file. The files go at quit.
pub(crate) fn start(
    data_dir: PathBuf,
    url: String,
    shared: transport::Shared,
    table: transport::Clients,
    cx: &mut App,
) {
    cx.set_global(ClientsState {
        data_dir: data_dir.clone(),
        url: url.clone(),
        shared,
        table: Arc::clone(&table),
        registry: None,
        pairings: Vec::new(),
    });
    let read = cx.background_spawn({
        let data_dir = data_dir.clone();
        futures::future::lazy(move |_| {
            clear_client_files_in(&data_dir)
                .with_context(|| "removing the last run's client files".to_string())
                .log_err();
            read_registry_in(&data_dir)
        })
    });
    let quit_dir = data_dir.clone();
    cx.spawn(async move |cx| {
        let registry = read.await;
        if let Err(error) = &registry {
            log::error!("mcp: no outside client is let in: {error:#}");
        }
        let clients = registry
            .as_ref()
            .map(|registry| registry.clients.clone())
            .unwrap_or_default();
        cx.update(|cx| {
            cx.global_mut::<ClientsState>().registry =
                Some(registry.map_err(|error| format!("{error:#}").into()));
        });
        cx.background_spawn(futures::future::lazy(move |_| {
            for client in &clients {
                let let_in =
                    transport::allow_client(&table, &client.name, client.write, client.power)
                        .map_err(anyhow::Error::from)
                        .and_then(|token| {
                            write_client_file_in(&data_dir, &client.name, &url, &token)
                                .map_err(anyhow::Error::from)
                        });
                if let Err(error) = let_in {
                    log::error!("mcp: the client {} is not let in: {error:#}", client.name);
                }
            }
        }))
        .await;
    })
    .detach();
    cx.on_app_quit(move |cx| {
        let data_dir = quit_dir.clone();
        cx.background_spawn(futures::future::lazy(move |_| {
            clear_client_files_in(&data_dir).log_err();
        }))
    })
    .detach();
}

/// Whether the client `name` is allowed, as the main thread knows.
pub(crate) fn is_allowed(name: &str, cx: &App) -> bool {
    cx.try_global::<ClientsState>()
        .and_then(|state| state.registry.as_ref()?.as_ref().ok())
        .is_some_and(|registry| registry.clients.iter().any(|client| client.name == name))
}

/// The address Marley's MCP server listens on, while it runs (#885).
pub(crate) fn server_url(cx: &App) -> Option<String> {
    cx.try_global::<ClientsState>()
        .map(|state| state.url.clone())
}

/// The names of the programs paired with Marley's MCP server, as the main thread knows them;
/// none before the list is read (#885).
pub(crate) fn paired_clients(cx: &App) -> Vec<String> {
    cx.try_global::<ClientsState>()
        .and_then(|state| state.registry.as_ref()?.as_ref().ok())
        .map(|registry| {
            registry
                .clients
                .iter()
                .map(|client| client.name.clone())
                .collect()
        })
        .unwrap_or_default()
}

/// The registry as the main thread keeps it, for Allow, Cut Off and the list.
fn current_registry(cx: &App) -> anyhow::Result<Registry> {
    cx.try_global::<ClientsState>()
        .ok_or_else(|| anyhow!("Marley's MCP server is not running"))
        .and_then(registry_of)
}

/// `state`'s registry, or why there is none to change.
fn registry_of(state: &ClientsState) -> anyhow::Result<Registry> {
    match &state.registry {
        None => Err(anyhow!("Marley is still reading its client list")),
        Some(Err(reason)) => Err(anyhow!(
            "Marley's client list could not be read, so it is left as it is: {reason}"
        )),
        Some(Ok(registry)) => Ok(registry.clone()),
    }
}

/// Lets `name` in (#524), with the grant to act in pages when `write`, or with the `power` it
/// paired with (#811): its token, its endpoint file and its entry in the registry, off the main
/// thread; the file's path.
pub(crate) fn allow(
    name: String,
    write: bool,
    power: Option<Power>,
    cx: &App,
) -> Task<anyhow::Result<PathBuf>> {
    let Some(state) = cx.try_global::<ClientsState>() else {
        return Task::ready(Err(anyhow!("Marley's MCP server is not running")));
    };
    let mut registry = match registry_of(state) {
        Ok(registry) => registry,
        Err(error) => return Task::ready(Err(error)),
    };
    if let Err(error) = check_client_name(&name) {
        return Task::ready(Err(error.into()));
    }
    if registry.clients.iter().any(|client| client.name == name) {
        return Task::ready(Err(ClientError::Taken(name).into()));
    }
    let write = write || power.is_some_and(|power| power >= Power::Write);
    registry.clients.push(RegisteredClient {
        name: name.clone(),
        write,
        allowed_at: now_seconds(),
        power,
    });
    let (data_dir, url) = (state.data_dir.clone(), state.url.clone());
    let (shared, table) = (Arc::clone(&state.shared), Arc::clone(&state.table));
    let written = cx.background_spawn(futures::future::lazy(move |_| {
        let token = transport::allow_client(&table, &name, write, power)?;
        let kept = write_client_file_in(&data_dir, &name, &url, &token)
            .map_err(anyhow::Error::from)
            .and_then(|file| write_registry_in(&data_dir, &registry).map(|()| file));
        if kept.is_err() {
            // A client the list does not keep holds no token either.
            let _was_allowed = transport::cut_off_client(&shared, &table, &name);
            discovery::remove_endpoint_file_in(
                &mcp_dir_in(&data_dir).join(CLIENTS_DIR),
                &format!("{name}.json"),
            )
            .log_err();
        }
        anyhow::Ok((kept?, registry))
    }));
    cx.spawn(async move |cx| {
        let (file, registry) = written.await?;
        cx.update(|cx| {
            cx.global_mut::<ClientsState>().registry = Some(Ok(registry));
        });
        Ok(file)
    })
}

/// Cuts the client `name` off (#524): at once for the main thread (the Browser tab's mark, a call
/// still waiting), then off it for the server (its token opens nothing, its sessions end), its
/// entry in the registry and then its endpoint file.
pub(crate) fn cut_off(name: String, cx: &mut App) -> Task<anyhow::Result<()>> {
    if !cx.has_global::<ClientsState>() {
        return Task::ready(Err(anyhow!("Marley's MCP server is not running")));
    }
    let state = cx.global_mut::<ClientsState>();
    let mut registry = match registry_of(state) {
        Ok(registry) => registry,
        Err(error) => return Task::ready(Err(error)),
    };
    registry.clients.retain(|client| client.name != name);
    state.registry = Some(Ok(registry.clone()));
    let (data_dir, shared, table) = (
        state.data_dir.clone(),
        Arc::clone(&state.shared),
        Arc::clone(&state.table),
    );
    if let Some(hub) = BrowserHub::try_global(cx) {
        hub.update(cx, |hub, cx| hub.forget_client(&name, cx));
    }
    cx.background_spawn(futures::future::lazy(move |_| {
        let _was_allowed = transport::cut_off_client(&shared, &table, &name);
        // The list changes before the file goes, so a bridge that finds the file gone finds the
        // client unlisted too, and says it is cut off rather than that Marley is not running
        // (#584).
        write_registry_in(&data_dir, &registry)?;
        discovery::remove_endpoint_file_in(
            &mcp_dir_in(&data_dir).join(CLIENTS_DIR),
            &format!("{name}.json"),
        )
        .with_context(|| format!("removing {}", client_file_in(&data_dir, &name).display()))
    }))
}

/// Gives the paired client `name` the power `power` (#811): at once for the server, then in the
/// registry, off the main thread.
pub(crate) fn set_power(name: String, power: Power, cx: &mut App) -> Task<anyhow::Result<()>> {
    if !cx.has_global::<ClientsState>() {
        return Task::ready(Err(anyhow!("Marley's MCP server is not running")));
    }
    let state = cx.global_mut::<ClientsState>();
    let mut registry = match registry_of(state) {
        Ok(registry) => registry,
        Err(error) => return Task::ready(Err(error)),
    };
    let Some(client) = registry
        .clients
        .iter_mut()
        .find(|client| client.name == name && client.power.is_some())
    else {
        return Task::ready(Err(anyhow!("{name} is not a paired client")));
    };
    client.power = Some(power);
    client.write = power >= Power::Write;
    state.registry = Some(Ok(registry.clone()));
    let (data_dir, table) = (state.data_dir.clone(), Arc::clone(&state.table));
    cx.background_spawn(futures::future::lazy(move |_| {
        transport::set_client_power(&table, &name, power);
        write_registry_in(&data_dir, &registry)
    }))
}

/// A program's request to pair (#811), until its command reads the answer.
struct Pairing {
    id: String,
    name: String,
    state: PairState,
    asked_at: std::time::Instant,
    /// The wait for the user's answer.
    _waiting: Task<()>,
}

/// Where a pairing stands.
#[derive(Clone)]
enum PairState {
    Waiting,
    Trusted { power: Power, file: PathBuf },
    Refused(String),
    Expired,
}

/// The user's answer to a pairing.
#[derive(Clone, Copy)]
enum PairAnswer {
    Trust(Power),
    Refuse,
}

/// What a pairing's notification shows.
#[derive(Clone)]
struct PairQuestion {
    name: String,
    code: String,
    folder: Option<String>,
}

/// The pairing notifications' kind; each has its own id within it.
struct PairingQuestion;

static NEXT_PAIRING: AtomicUsize = AtomicUsize::new(0);

/// The answer, given once from whichever button is clicked in whichever window.
type PairSlot = Arc<Mutex<Option<oneshot::Sender<PairAnswer>>>>;

/// Answers the pairing tools (#811): `client_pair` and `client_pair_status`.
pub(crate) fn answer(call: AppCall, cx: &mut App) {
    match call.tool.as_str() {
        "client_pair" => {
            let name = call
                .arguments
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let begun = begin_pairing(&name, &call, cx);
            call.answer(begun.map(|(id, code)| ToolAnswer {
                structured: json!({
                    "id": id,
                    "code": code,
                    "expires_in_seconds": PAIR_WAIT.as_secs(),
                }),
                text: None,
                image: None,
            }));
        }
        "client_pair_status" => {
            let id = call
                .arguments
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let status = pairing_status(&id, cx);
            call.answer(status.map(|structured| ToolAnswer {
                structured,
                text: None,
                image: None,
            }));
        }
        tool => {
            let refusal = Refusal::new("bad_argument", format!("{tool} is no pairing tool"));
            call.answer(Err::<ToolAnswer, _>(refusal));
        }
    }
}

/// Shows the pairing `name` asks for in every window and waits for the user: its id and code.
///
/// # Errors
///
/// A name that is no client's name, one let in already or already waiting, too many pairings
/// waiting, or no list of clients to add to.
fn begin_pairing(name: &str, call: &AppCall, cx: &mut App) -> Result<(String, String), Refusal> {
    check_client_name(name).map_err(|error| Refusal::new("bad_argument", error.to_string()))?;
    let Some(state) = cx.try_global::<ClientsState>() else {
        return Err(Refusal::new(
            "not_ready",
            "Marley's MCP server is not running",
        ));
    };
    let registry = registry_of(state).map_err(|error| {
        Refusal::new("not_ready", format!("{error:#}")).next("call client_pair again in a moment")
    })?;
    if registry.clients.iter().any(|client| client.name == name) {
        return Err(Refusal::new(
            "already_paired",
            format!("a client named {name} is let in already"),
        )
        .next(format!(
            "ask the user to cut {name} off in Outside Clients first, or pair under another name"
        )));
    }
    let waiting = |pairing: &&Pairing| matches!(pairing.state, PairState::Waiting);
    if state
        .pairings
        .iter()
        .filter(waiting)
        .any(|pairing| pairing.name == name)
    {
        return Err(Refusal::new(
            "pairing_pending",
            format!("a pairing as {name} already waits on the user"),
        )
        .next("wait for the user's answer, or for the pairing to expire"));
    }
    if state.pairings.iter().filter(waiting).count() >= MAX_PAIRINGS {
        return Err(Refusal::new(
            "too_many_pairings",
            format!("{MAX_PAIRINGS} pairings already wait on the user"),
        )
        .next("wait for the user to answer them, or for them to expire"));
    }
    let id = format!("{:032x}", rand::random::<u128>());
    let number = rand::rng().random_range(0..1_000_000_u32);
    let code = format!("{:03} {:03}", number / 1000, number % 1000);
    let question = PairQuestion {
        name: name.to_string(),
        code: code.clone(),
        folder: call.caller().cwd.clone(),
    };
    let waiting_for_user = cx.spawn({
        let (id, name) = (id.clone(), name.to_string());
        async move |cx| {
            let state = match ask_pairing(question, cx).await {
                None => PairState::Expired,
                Some(PairAnswer::Refuse) => PairState::Refused("the user refused".to_string()),
                Some(PairAnswer::Trust(power)) => {
                    let letting_in = cx.update(|cx| allow(name, false, Some(power), cx));
                    match letting_in.await {
                        Ok(file) => PairState::Trusted { power, file },
                        Err(error) => {
                            PairState::Refused(format!("Marley could not let it in: {error:#}"))
                        }
                    }
                }
            };
            cx.update(|cx| {
                if let Some(pairing) = cx
                    .global_mut::<ClientsState>()
                    .pairings
                    .iter_mut()
                    .find(|pairing| pairing.id == id)
                {
                    pairing.state = state;
                }
            });
        }
    });
    let pairings = &mut cx.global_mut::<ClientsState>().pairings;
    // An answer no command read is dropped once it is well past its wait.
    pairings.retain(|pairing| {
        matches!(pairing.state, PairState::Waiting) || pairing.asked_at.elapsed() < PAIR_WAIT * 2
    });
    pairings.push(Pairing {
        id: id.clone(),
        name: name.to_string(),
        state: PairState::Waiting,
        asked_at: std::time::Instant::now(),
        _waiting: waiting_for_user,
    });
    Ok((id, code))
}

/// The answer to the pairing `id`; a final answer is given once, and the pairing goes with it.
///
/// # Errors
///
/// When no pairing has that id.
fn pairing_status(id: &str, cx: &mut App) -> Result<Value, Refusal> {
    let no_pairing = || {
        Refusal::new(
            "no_pairing",
            "no pairing has that id: its answer was read already, or it was never asked",
        )
        .next("call client_pair again")
    };
    if !cx.has_global::<ClientsState>() {
        return Err(no_pairing());
    }
    let pairings = &mut cx.global_mut::<ClientsState>().pairings;
    let index = pairings
        .iter()
        .position(|pairing| pairing.id == id)
        .ok_or_else(no_pairing)?;
    let pairing = pairings.get(index).ok_or_else(no_pairing)?;
    let name = pairing.name.clone();
    let status = match &pairing.state {
        PairState::Waiting => return Ok(json!({ "state": "waiting", "name": name })),
        PairState::Trusted { power, file } => {
            let (command, over_ssh) = client_lines(file);
            json!({
                "state": "trusted",
                "name": name,
                "power": power.as_str(),
                "endpoint": file.to_string_lossy(),
                "command": command,
                "over_ssh": over_ssh,
            })
        }
        PairState::Refused(reason) => json!({ "state": "refused", "name": name, "reason": reason }),
        PairState::Expired => json!({ "state": "expired", "name": name }),
    };
    pairings.remove(index);
    Ok(status)
}

/// Asks `question` in every window and waits for an answer, at most [`PAIR_WAIT`].
async fn ask_pairing(question: PairQuestion, cx: &gpui::AsyncApp) -> Option<PairAnswer> {
    let id =
        NotificationId::composite::<PairingQuestion>(NEXT_PAIRING.fetch_add(1, Ordering::Relaxed));
    let (sender, receiver) = oneshot::channel();
    let slot: PairSlot = Arc::new(Mutex::new(Some(sender)));
    cx.update(|cx| show_pairing(id.clone(), question, slot, cx));
    let timer = cx.background_executor().timer(PAIR_WAIT);
    let answered = futures::select_biased! {
        answered = receiver.fuse() => answered.ok(),
        () = timer.fuse() => None,
    };
    cx.update(|cx| dismiss_app_notification(&id, cx));
    answered
}

fn show_pairing(id: NotificationId, question: PairQuestion, slot: PairSlot, cx: &mut App) {
    show_app_notification(id, cx, move |cx| {
        let answer = |answer: PairAnswer| {
            let slot = Arc::clone(&slot);
            move |_: &mut Window, _: &mut App| give_pairing(&slot, answer)
        };
        let mut ask = Ask::new(format!(
            "A program asks to pair with Marley as \"{}\"",
            question.name
        ))
        .line(format!(
            "Check that it shows the code {}, then choose what it may do.",
            question.code
        ));
        if let Some(folder) = question.folder.clone() {
            ask = ask.footnote(format!("Asked from {folder}"));
        }
        let ask = ask
            .button("Refuse", Role::Quiet, answer(PairAnswer::Refuse))
            .button(
                "Read, Write and Run",
                Role::Plain,
                answer(PairAnswer::Trust(Power::Run)),
            )
            .button(
                "Read and Write",
                Role::Plain,
                answer(PairAnswer::Trust(Power::Write)),
            )
            .button(
                "Read",
                Role::Primary,
                answer(PairAnswer::Trust(Power::Read)),
            );
        cx.new(|cx| MessageNotification::new_from_builder(cx, move |_, cx| ask.render(cx)))
    });
}

/// Gives the user's answer to a pairing, the first one only.
fn give_pairing(slot: &PairSlot, answer: PairAnswer) {
    let sender = slot.lock().ok().and_then(|mut sender| sender.take());
    if let Some(sender) = sender
        && sender.send(answer).is_err()
    {
        log::debug!("pairing: the answer came after the pairing expired");
    }
}

/// When each client last called, as the server's table says, read off the main thread.
fn last_calls(cx: &App) -> Task<Vec<ClientInfo>> {
    let Some(table) = cx
        .try_global::<ClientsState>()
        .map(|state| Arc::clone(&state.table))
    else {
        return Task::ready(Vec::new());
    };
    cx.background_spawn(futures::future::lazy(move |_| {
        table
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clients()
    }))
}

/// A last call's age as the list says it.
fn called(last_call_ms: Option<u64>) -> String {
    let Some(last_call_ms) = last_call_ms else {
        return "no call yet".to_string();
    };
    let now_ms = u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
    )
    .unwrap_or(u64::MAX);
    match now_ms.saturating_sub(last_call_ms) / 1000 {
        0..=59 => "last call just now".to_string(),
        seconds @ 60..=3599 => format!("last call {} min ago", seconds / 60),
        seconds => format!("last call {} h ago", seconds / 3600),
    }
}

/// Registers Outside Clients on each workspace, under both its names.
pub(crate) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action(|workspace, _: &BrowserClients, window, cx| {
            workspace.toggle_modal(window, cx, BrowserClientsModal::new);
        });
        workspace.register_action(|workspace, _: &OutsideClients, window, cx| {
            workspace.toggle_modal(window, cx, BrowserClientsModal::new);
        });
    })
    .detach();
}

/// The command that runs Marley's bridge for the client whose endpoint file is `file`, and the same
/// command run here over SSH from another machine (#584).
fn client_lines(file: &Path) -> (String, String) {
    let file_text = file.to_string_lossy().into_owned();
    let bridge = file
        .parent()
        .and_then(Path::parent)
        .map(|mcp| mcp.join("marley-mcp-bridge"))
        .unwrap_or_default();
    let quote = |text: &str| {
        ShellKind::Posix
            .try_quote(text)
            .map_or_else(|| text.to_string(), Cow::into_owned)
    };
    let command = format!(
        "MARLEY_MCP_ENDPOINT={} {}",
        quote(&file_text),
        quote(&bridge.to_string_lossy())
    );
    // `env` sets the variable in any login shell, csh's included, and `BatchMode` makes an SSH
    // that would ask for a password fail at once rather than prompt inside the client's own
    // terminal.
    let over_ssh = format!(
        "ssh -T -o BatchMode=yes {} {}",
        this_machine(),
        quote(&format!("env {command}"))
    );
    (command, over_ssh)
}

/// Outside Clients: the clients the user let in, paired (#811) or allowed by name (#524), with
/// what each may do, its last call and Cut Off, and a form to allow a browser client by name.
pub(crate) struct BrowserClientsModal {
    name: Entity<Editor>,
    write: bool,
    error: Option<SharedString>,
    /// The client allowed last here, and its endpoint file.
    allowed: Option<(SharedString, PathBuf)>,
    last_calls: Vec<ClientInfo>,
    busy: bool,
    _refresh: Task<()>,
}

impl BrowserClientsModal {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let name = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("A client's name, such as playwright-mcp", window, cx);
            editor
        });
        let refresh = cx.spawn(async move |this, cx| {
            loop {
                let read = cx.update(|cx| last_calls(cx)).await;
                if this
                    .update(cx, |this, cx| {
                        this.last_calls = read;
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
                cx.background_executor().timer(LAST_CALLS_EVERY).await;
            }
        });
        Self {
            name,
            write: false,
            error: None,
            allowed: None,
            last_calls: Vec::new(),
            busy: false,
            _refresh: refresh,
        }
    }

    fn allow_from_field(
        &mut self,
        _: &AllowBrowserClient,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.allow(window, cx);
    }

    fn allow(&mut self, window: &Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let name = self.name.read(cx).text(cx).trim().to_string();
        let task = allow(name.clone(), self.write, None, cx);
        self.busy = true;
        self.error = None;
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let allowed = task.await;
            this.update_in(cx, |this, window, cx| {
                this.busy = false;
                match allowed {
                    Ok(file) => {
                        this.allowed = Some((SharedString::from(name), file));
                        this.write = false;
                        this.name.update(cx, |editor, cx| editor.clear(window, cx));
                    }
                    Err(error) => this.error = Some(format!("{error:#}").into()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn cut_off(&mut self, name: &str, window: &Window, cx: &mut Context<Self>) {
        let task = cut_off(name.to_string(), cx);
        if self
            .allowed
            .as_ref()
            .is_some_and(|(allowed, _)| allowed.as_ref() == name)
        {
            self.allowed = None;
        }
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            if let Err(error) = task.await {
                this.update(cx, |this, cx| {
                    this.error = Some(format!("{error:#}").into());
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    fn change_power(name: &str, power: Power, window: &Window, cx: &mut Context<Self>) {
        let task = set_power(name.to_string(), power, cx);
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            if let Err(error) = task.await {
                this.update(cx, |this, cx| {
                    this.error = Some(format!("{error:#}").into());
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    fn render_client(
        &self,
        index: usize,
        client: &RegisteredClient,
        cx: &Context<Self>,
    ) -> impl IntoElement + use<> {
        let last_call = self
            .last_calls
            .iter()
            .find(|info| info.grant.name == client.name)
            .and_then(|info| info.last_call_ms);
        let grant = match (client.power, client.write) {
            (Some(power), _) => format!("may {}", power.words()),
            (None, true) => "browser: reads and acts".to_string(),
            (None, false) => "browser: reads".to_string(),
        };
        let name = client.name.clone();
        let powers = client.power.map(|current| {
            h_flex()
                .gap_0p5()
                .children(Power::ALL.into_iter().map(|power| {
                    let name = client.name.clone();
                    let label = match power {
                        Power::Read => "Read",
                        Power::Write => "Write",
                        Power::Run => "Run",
                    };
                    Button::new(
                        SharedString::from(format!(
                            "outside-client-power-{index}-{}",
                            power.as_str()
                        )),
                        label,
                    )
                    .style(ButtonStyle::Subtle)
                    .toggle_state(power == current)
                    .on_click(cx.listener(move |_, _, window, cx| {
                        Self::change_power(&name, power, window, cx);
                    }))
                }))
        });
        h_flex()
            .w_full()
            .justify_between()
            .gap_2()
            .py_1()
            .child(
                v_flex()
                    .min_w_0()
                    .child(Label::new(client.name.clone()))
                    .child(
                        Label::new(format!("{grant} · {}", called(last_call)))
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    ),
            )
            .child(
                h_flex().gap_2().children(powers).child(
                    Button::new(("browser-client-cut-off", index), "Cut Off")
                        .style(ButtonStyle::Tinted(TintColor::Error))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.cut_off(&name, window, cx);
                        })),
                ),
            )
    }

    fn render_allowed(file: &Path, name: &str, cx: &App) -> impl IntoElement + use<> {
        let file_text = file.to_string_lossy().into_owned();
        let (command, over_ssh) = client_lines(file);
        let colors = cx.theme().colors();
        let copyable = |id: &'static str, text: String| {
            h_flex()
                .w_full()
                .gap_1()
                .px_2()
                .py_1()
                .rounded_md()
                .bg(colors.editor_background)
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(Label::new(text.clone()).size(LabelSize::Small).truncate()),
                )
                .child(
                    IconButton::new(id, IconName::Copy)
                        .icon_size(IconSize::Small)
                        .tooltip(ui::Tooltip::text("Copy"))
                        .on_click(move |_, _, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
                        }),
                )
        };
        v_flex()
            .gap_1()
            .child(Label::new(format!("{name}'s endpoint file")).size(LabelSize::Small))
            .child(copyable("browser-client-copy-file", file_text))
            .child(Label::new("Point Marley's bridge at it:").size(LabelSize::Small))
            .child(copyable("browser-client-copy-command", command))
            .child(Label::new("From another machine, run it over SSH:").size(LabelSize::Small))
            .child(copyable("browser-client-copy-ssh", over_ssh))
    }
}

/// This machine as another one names it over SSH, the user and the host; a word for the user to
/// replace stands in for either one that cannot be read.
fn this_machine() -> String {
    let user = whoami::fallible::username().unwrap_or_else(|_| "<user>".to_string());
    let host = whoami::fallible::hostname().unwrap_or_else(|_| "<this machine>".to_string());
    format!("{user}@{host}")
}

impl ModalView for BrowserClientsModal {}

impl EventEmitter<DismissEvent> for BrowserClientsModal {}

impl Focusable for BrowserClientsModal {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.name.focus_handle(cx)
    }
}

impl Render for BrowserClientsModal {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let registry = current_registry(cx);
        let colors = cx.theme().colors();
        let clients: Vec<AnyElement> = registry
            .as_ref()
            .map(|registry| {
                registry
                    .clients
                    .iter()
                    .enumerate()
                    .map(|(index, client)| self.render_client(index, client, cx).into_any_element())
                    .collect()
            })
            .unwrap_or_default();
        let listed = registry.as_ref().map_or_else(
            |error| {
                Label::new(format!("{error:#}"))
                    .size(LabelSize::Small)
                    .color(Color::Error)
                    .into_any_element()
            },
            |registry| {
                if registry.clients.is_empty() {
                    Label::new("No client is allowed.")
                        .size(LabelSize::Small)
                        .color(Color::Muted)
                        .into_any_element()
                } else {
                    v_flex().children(clients).into_any_element()
                }
            },
        );
        v_flex()
            .key_context("BrowserClients")
            .on_action(cx.listener(|_, _: &menu::Cancel, _, cx| cx.emit(DismissEvent)))
            .w(rems(36.))
            .elevation_3(cx)
            .p_3()
            .gap_3()
            .child(
                v_flex()
                    .gap_1()
                    .child(Headline::new("Outside Clients").size(HeadlineSize::Small))
                    .child(
                        Label::new(
                            "A program outside Marley's terminals reads Marley's tools and does \
                             nothing more until it pairs: it runs marley-mcp-bridge pair NAME, \
                             and you check its code and choose what it may do. Each client has a \
                             token of its own, new at each start.",
                        )
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                    )
                    .child(
                        Label::new(
                            "A program allowed by name below reaches the browser alone: to read \
                             pages, or also to click, type and navigate, in front of you.",
                        )
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                    ),
            )
            .child(listed)
            .child(
                h_flex()
                    .w_full()
                    .gap_2()
                    .child(
                        div()
                            .key_context("MarleyClientName")
                            .on_action(cx.listener(Self::allow_from_field))
                            .flex_1()
                            .min_w_0()
                            .px_2()
                            .py_0p5()
                            .rounded_md()
                            .border_1()
                            .border_color(colors.border)
                            .bg(colors.editor_background)
                            .child(self.name.clone()),
                    )
                    .child(
                        Checkbox::new("browser-client-write", ToggleState::from(self.write))
                            .label("May act in pages")
                            .on_click(cx.listener(|this, state: &ToggleState, _, cx| {
                                this.write = state.selected();
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("browser-client-allow", "Allow")
                            .disabled(self.busy)
                            .on_click(cx.listener(|this, _, window, cx| this.allow(window, cx))),
                    ),
            )
            .when_some(self.error.clone(), |this, error| {
                this.child(Label::new(error).size(LabelSize::Small).color(Color::Error))
            })
            .when_some(self.allowed.clone(), |this, (name, file)| {
                this.child(Self::render_allowed(&file, &name, cx))
            })
    }
}
