//! Codex's own App Server for each Codex terminal Marley launches (#650).
//!
//! While `marley.codex_app_server` is on and #648's check finds a Codex the App Server was tested
//! on, a Codex Marley launches in a local project gets a server of its own: Marley starts
//! `codex app-server --listen unix://<socket>` in the terminal's folder, with the environment the
//! terminal's programs get, and types `codex --remote unix://<socket> --cd <folder>` once the
//! socket is there. The server lives exactly as long as its terminal, so every thread on it is
//! that terminal's.
//!
//! Marley joins second: once the TUI's connection shows on the socket, and a second after it, so
//! the TUI's `initialize` names the server's originator. It then follows the lead thread into the
//! terminal's seat (`marley_agent::codex_events`): the status that every connection hears, and,
//! once subscribed, the turns, the token use and the policies. A lead the TUI made before Marley
//! joined is subscribed with `thread/resume` only while it is idle, since a resume renames the
//! thread's client until the TUI's next turn. Marley sends no request that acts and answers none
//! of the server's: any subscriber's answer resolves a request, and an error reads as a denial.

use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::OsStr;
use std::os::unix::fs::DirBuilderExt as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use anyhow::Context as _;
use async_tungstenite::tungstenite::Message;
use futures::channel::{mpsc, oneshot};
use futures::future::Either;
use futures::{AsyncBufReadExt as _, StreamExt as _};
use gpui::{App, AppContext as _, AsyncApp, Entity, EntityId, Global, Task, WeakEntity};
use marley_agent::codex_events::{
    self, Decision, Input, Notification, Request, RequestKind, Thread, ThreadStatus,
};
use marley_agent::versions::{self, CODEX_APP_SERVER, Found};
use marley_agent::{AgentKind, Remote};
use project::Project;
use serde_json::{Value, json};
use settings::Settings as _;
use terminal::Terminal;

use crate::MarleySettings;

/// How long the server has to make its socket before Codex starts without it.
const SOCKET_WAIT: Duration = Duration::from_secs(5);

/// How long the folder's environment may take, when Zed has not read it yet.
const ENVIRONMENT_WAIT: Duration = Duration::from_secs(3);

/// How long a call to the server may take.
const CALL_TIMEOUT: Duration = Duration::from_secs(5);

/// How often the follow looks for a lead, a lead to subscribe, and Codex in the foreground.
const TICK: Duration = Duration::from_secs(1);

/// How often the socket is checked for the TUI's connection.
const CONNECTION_CHECK: Duration = Duration::from_millis(500);

/// How long after the TUI's connection Marley joins, for the TUI's `initialize` to come first.
const AFTER_TUI: Duration = Duration::from_secs(1);

/// How long a stopped server has to end its turns before it is killed.
const STOP_GRACE: Duration = Duration::from_secs(2);

/// How often a lead-thread message the fold does not read marks the seat as heard from.
const HEARD_EVERY: Duration = Duration::from_secs(5);

/// The most characters of the server's last error line a seat keeps.
const LAST_ERROR_MAX: usize = 200;

/// The streaming notifications Marley does not read, which the server drops a slow connection
/// over. `item/started` stays, for #651.
const OPTED_OUT: [&str; 11] = [
    "item/agentMessage/delta",
    "item/plan/delta",
    "command/exec/outputDelta",
    "process/outputDelta",
    "item/commandExecution/outputDelta",
    "item/fileChange/outputDelta",
    "item/reasoning/summaryTextDelta",
    "item/reasoning/textDelta",
    "thread/realtime/item/transcript/delta",
    "thread/realtime/transcript/delta",
    "thread/realtime/outputAudio/delta",
];

/// `marley.codex_app_server`: whether a Codex Marley launches runs against a server of its own.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CodexAppServer {
    /// Codex starts as the user's shell would start it, the default.
    #[default]
    Off,
    /// Codex starts against an App Server Marley owns and joins.
    On,
}

impl CodexAppServer {
    /// The switch as the settings hold it: off unless set.
    pub(crate) fn from_content(marley: Option<&settings::MarleySettingsContent>) -> Self {
        match marley.and_then(|marley| marley.codex_app_server) {
            Some(true) => Self::On,
            Some(false) | None => Self::Off,
        }
    }
}

/// The servers by the terminal they belong to.
#[derive(Default)]
struct CodexServers {
    servers: HashMap<EntityId, Server>,
}

impl Global for CodexServers {}

/// The requests each terminal's Codex waits on that the inbox answers (#651), by the terminal,
/// as the follow last published them.
#[derive(Default)]
pub(crate) struct CodexRequests {
    terminals: HashMap<EntityId, Published>,
}

impl Global for CodexRequests {}

/// One terminal's requests, its connection's generation, and the way back to its follow.
struct Published {
    generation: u64,
    requests: Vec<Shown>,
    answers: mpsc::UnboundedSender<Event>,
}

/// A request as the inbox shows it: what it asks, a file change's paths when its item named them,
/// and the decision Marley sent, until the server reports it resolved.
#[derive(Debug, Clone)]
pub(crate) struct Shown {
    pub(crate) request: Request,
    pub(crate) files: Option<Vec<String>>,
    pub(crate) sent: Option<Decision>,
}

/// The requests `terminal`'s Codex waits on that the inbox answers, with the connection's
/// generation; none while it waits on nothing the inbox lists.
pub(crate) fn requests_of(terminal: EntityId, cx: &App) -> Option<(u64, &[Shown])> {
    let published = cx.try_global::<CodexRequests>()?.terminals.get(&terminal)?;
    (!published.requests.is_empty())
        .then_some((published.generation, published.requests.as_slice()))
}

/// Sends `decision` on `request`, as the inbox showed it on `terminal`'s connection `generation`,
/// to that terminal's follow, which answers only while the same request is unresolved there.
pub(crate) fn answer(
    terminal: EntityId,
    generation: u64,
    request: &Request,
    decision: Decision,
    cx: &App,
) {
    let Some(published) = cx
        .try_global::<CodexRequests>()
        .and_then(|requests| requests.terminals.get(&terminal))
    else {
        return;
    };
    let answer = Answer {
        generation,
        id: request.id.clone(),
        params: request.params.clone(),
        decision,
    };
    if published
        .answers
        .unbounded_send(Event::Answer(answer))
        .is_err()
    {
        log::info!("codex app server: the terminal's follow has ended; nothing was sent");
    }
}

/// A decision the user picked in the inbox, with the request as the entry showed it.
struct Answer {
    generation: u64,
    id: Value,
    params: Value,
    decision: Decision,
}

/// The generation of the next connection Marley makes to a server.
static NEXT_GENERATION: AtomicU64 = AtomicU64::new(1);

/// One terminal's server: its process, which a drop stops, and Marley's follow of it.
struct Server {
    pid: u32,
    socket: PathBuf,
    /// Dropping it asks the process task to stop the server.
    _stop: oneshot::Sender<()>,
    process: Option<Task<()>>,
    _follow: Task<()>,
}

impl Drop for Server {
    fn drop(&mut self) {
        // The process task outlives the server: it stops the program and removes its socket.
        if let Some(process) = self.process.take() {
            process.detach();
        }
    }
}

/// What a Codex launch needs to join a server of its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Prepared {
    program: String,
    socket: String,
    folder: String,
}

/// A Codex launch that joins a server of its own: the server, and the line without it, which goes
/// in when the server does not come up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Joining {
    /// The server.
    pub(crate) prepared: Prepared,
    /// The launch line without it.
    pub(crate) plain: Vec<u8>,
}

impl Prepared {
    /// The launch line's App Server.
    pub(crate) fn remote(&self) -> Remote<'_> {
        Remote {
            program: &self.program,
            socket: &self.socket,
            folder: &self.folder,
        }
    }
}

/// Stops every server when Marley quits.
pub(crate) fn init(cx: &mut App) {
    cx.set_global(CodexServers::default());
    cx.set_global(CodexRequests::default());
    cx.on_app_quit(|cx| {
        let servers = std::mem::take(&mut cx.default_global::<CodexServers>().servers);
        let mut sockets = Vec::new();
        for (_, mut server) in servers {
            // Quitting leaves no time for a drain: the server goes now.
            send_signal(server.pid, rustix::process::Signal::KILL);
            drop(server.process.take());
            sockets.push(server.socket.clone());
        }
        // `marley-<pid>`, which holds nothing but the sockets' folder.
        let folder = socket_folder()
            .parent()
            .map_or_else(socket_folder, Path::to_path_buf);
        cx.background_spawn(futures::future::lazy(move |_| {
            for socket in &sockets {
                remove_leftovers(socket);
            }
            if folder.exists()
                && let Err(error) = std::fs::remove_dir_all(&folder)
            {
                log::debug!("codex app server: {} stays: {error}", folder.display());
            }
        }))
    })
    .detach();
}

/// What a Codex launch in `folder` of `project` needs to join a server of its own, when the switch
/// is on, the project local, the Codex found tested (#648), and every path usable; `None`, the
/// reason logged when it is not the switch or the project, otherwise.
pub(crate) fn prepare(project: &Project, folder: Option<&Path>, cx: &App) -> Option<Prepared> {
    if MarleySettings::get_global(cx).codex_app_server != CodexAppServer::On || !project.is_local()
    {
        return None;
    }
    let refuse = |why: String| {
        log::info!("codex app server: {why}, so Codex starts without one");
        None
    };
    if !cfg!(target_os = "linux") {
        return refuse(
            "Marley sees the TUI join through /proc/net/unix, which only Linux has".into(),
        );
    }
    if !crate::agent_versions::is_on(&CODEX_APP_SERVER, cx) {
        // The agent bar's chip says why.
        return refuse("the Codex found is outside the versions Marley tested".into());
    }
    let Some(program) = crate::agent_versions::program(AgentKind::Codex, cx) else {
        return refuse("no codex was found".into());
    };
    let Some(folder) = folder else {
        return refuse("the terminal has no folder".into());
    };
    let socket = socket_folder().join(format!(
        "{}.sock",
        NEXT_SOCKET.fetch_add(1, Ordering::Relaxed)
    ));
    let words = [program.as_path(), socket.as_path(), folder];
    let unusable = words.iter().find(|path| {
        path.to_str()
            .is_none_or(|text| text.chars().any(char::is_whitespace))
    });
    if let Some(path) = unusable {
        return refuse(format!(
            "{} has a space in it or is not UTF-8",
            path.display()
        ));
    }
    if !marley_browser::service::socket_fits(&socket) {
        return refuse(format!("{} is too long for a socket", socket.display()));
    }
    Some(Prepared {
        program: program.to_string_lossy().into_owned(),
        socket: socket.to_string_lossy().into_owned(),
        folder: folder.to_string_lossy().into_owned(),
    })
}

/// The number of the next socket's name.
static NEXT_SOCKET: AtomicU64 = AtomicU64::new(1);

/// Marley's folder for the servers' sockets: `marley-<pid>/codex` in the runtime directory, else
/// in Zed's temporary folder, made only the user's (Codex refuses a socket's folder others can
/// write).
fn socket_folder() -> PathBuf {
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|folder| folder.is_absolute())
        .unwrap_or_else(|| paths::temp_dir().clone());
    base.join(format!("marley-{}", std::process::id()))
        .join("codex")
}

/// Starts `prepared`'s server for `terminal` of `project` and waits for its socket. True when the
/// socket came up; false leaves no server, the reason logged, and the plain line goes in.
pub(crate) async fn start(
    prepared: &Prepared,
    terminal: &WeakEntity<Terminal>,
    project: &Entity<Project>,
    agent_env: Vec<(String, String)>,
    cx: &mut AsyncApp,
) -> bool {
    match serve(prepared, terminal, project, agent_env, cx).await {
        Ok(()) => true,
        Err(error) => {
            log::warn!("codex app server: {error:#}; Codex starts without one");
            false
        }
    }
}

async fn serve(
    prepared: &Prepared,
    terminal: &WeakEntity<Terminal>,
    project: &Entity<Project>,
    agent_env: Vec<(String, String)>,
    cx: &mut AsyncApp,
) -> anyhow::Result<()> {
    let terminal = terminal.upgrade().context("the terminal closed")?;
    let socket = PathBuf::from(&prepared.socket);
    let folder = PathBuf::from(&prepared.folder);
    let parent = socket
        .parent()
        .context("the socket has no folder")?
        .to_path_buf();
    cx.background_spawn(futures::future::lazy(move |_| {
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&parent)
            .with_context(|| format!("making {}", parent.display()))
    }))
    .await?;
    let env = environment(&folder, &terminal, project, agent_env, cx).await;
    let listen = format!("unix://{}", prepared.socket);
    let mut child = crate::process::serve(
        Path::new(&prepared.program),
        &[
            OsStr::new("app-server"),
            OsStr::new("--listen"),
            OsStr::new(&listen),
        ],
        &folder,
        &env,
    )
    .with_context(|| format!("starting {} app-server", prepared.program))?;
    let pid = child.id();
    let stderr = child.stderr.take();
    let (events, incoming) = mpsc::unbounded();
    let (stop, stopped) = oneshot::channel();
    let process = cx.background_spawn({
        let events = events.clone();
        let socket = socket.clone();
        let executor = cx.background_executor().clone();
        async move {
            let reading = last_line(stderr);
            let watching = watch(&mut child, stopped, &executor);
            let (last, exited) = futures::join!(reading, watching);
            if exited {
                events.unbounded_send(Event::ServerExited(last)).ok();
            }
            remove_leftovers(&socket);
        }
    });
    // On an error here the process task is dropped, and the server killed with its child.
    wait_for_socket(&socket, cx).await?;
    let id = terminal.entity_id();
    let follow = cx.spawn({
        let terminal = terminal.downgrade();
        let socket = socket.clone();
        async move |cx| {
            if let Err(error) = follow(&socket, &terminal, events, incoming, cx).await {
                log::warn!(
                    "codex app server: following {}: {error:#}",
                    socket.display()
                );
            }
            // Whatever ended the follow, its requests can no longer be answered (#651).
            cx.update(|cx| {
                let _gone = cx.default_global::<CodexRequests>().terminals.remove(&id);
            });
        }
    });
    let server = Server {
        pid,
        socket,
        _stop: stop,
        process: Some(process),
        _follow: follow,
    };
    cx.update(|cx| {
        let _previous = cx
            .default_global::<CodexServers>()
            .servers
            .insert(id, server);
    });
    // The server and its socket last exactly as long as the terminal.
    terminal.update(cx, |_, cx| {
        cx.on_release(move |_, cx| {
            let _stopped = cx.default_global::<CodexServers>().servers.remove(&id);
            let _gone = cx.default_global::<CodexRequests>().terminals.remove(&id);
        })
        .detach();
    });
    Ok(())
}

/// Waits up to [`SOCKET_WAIT`] for the server's socket.
async fn wait_for_socket(socket: &Path, cx: &AsyncApp) -> anyhow::Result<()> {
    let waited = Instant::now();
    loop {
        let there = cx
            .background_spawn(futures::future::lazy({
                let socket = socket.to_path_buf();
                move |_| socket.exists()
            }))
            .await;
        if there {
            return Ok(());
        }
        anyhow::ensure!(
            waited.elapsed() < SOCKET_WAIT,
            "{} did not appear within {} s",
            socket.display(),
            SOCKET_WAIT.as_secs()
        );
        cx.background_executor()
            .timer(Duration::from_millis(50))
            .await;
    }
}

/// The environment the terminal's programs get, as Zed's terminal builder makes it: the folder's
/// environment (the login shell's and direnv's), the port a worktree gets (#590), the terminal
/// settings' `env`, the agent's own variables, and the terminal's identity (#520). Added to
/// Marley's own, which the server inherits.
async fn environment(
    folder: &Path,
    terminal: &Entity<Terminal>,
    project: &Entity<Project>,
    agent_env: Vec<(String, String)>,
    cx: &mut AsyncApp,
) -> Vec<(String, String)> {
    let directory = project.update(cx, |project, cx| {
        project.environment().update(cx, |environment, cx| {
            environment.directory_environment(folder.into(), cx)
        })
    });
    let timer = cx.background_executor().timer(ENVIRONMENT_WAIT);
    let mut env: Vec<(String, String)> = match futures::future::select(directory, timer).await {
        Either::Left((Some(directory), _)) => directory.into_iter().collect(),
        Either::Left((None, _)) | Either::Right(_) => Vec::new(),
    };
    let (project_folder, settings_env) = project.read_with(cx, |project, cx| {
        let project_folder = project.first_project_directory(cx);
        let settings_env = project
            .terminal_settings(&Some(folder.to_path_buf()), cx)
            .env
            .clone();
        (project_folder, settings_env)
    });
    env.extend(marley_terminal::ports::variables(project_folder.clone()).await);
    env.extend(settings_env);
    env.extend(agent_env);
    let id = terminal.read_with(cx, |terminal, _| {
        terminal.marley_terminal_id().map(str::to_string)
    });
    env.push((
        marley_terminal::identity::TERMINAL_ID_VARIABLE.to_string(),
        id.unwrap_or_default(),
    ));
    env.push((
        marley_terminal::identity::PROJECT_VARIABLE.to_string(),
        project_folder
            .map(|folder| folder.to_string_lossy().into_owned())
            .unwrap_or_default(),
    ));
    env
}

/// The server's last line on stderr, read until it ends, cut to [`LAST_ERROR_MAX`].
async fn last_line(stderr: Option<smol::process::ChildStderr>) -> Option<String> {
    let mut lines = futures::io::BufReader::new(stderr?).lines();
    let mut last = None;
    while let Some(line) = lines.next().await {
        match line {
            Ok(line) if !line.trim().is_empty() => {
                last = Some(line.trim().chars().take(LAST_ERROR_MAX).collect());
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
    last
}

/// Waits for the server to end, or for its stop: then SIGTERM, which lets it end its turns, and
/// the kill after [`STOP_GRACE`]. True when it ended on its own.
async fn watch(
    child: &mut smol::process::Child,
    stopped: oneshot::Receiver<()>,
    executor: &gpui::BackgroundExecutor,
) -> bool {
    let pid = child.id();
    match futures::future::select(Box::pin(child.status()), stopped).await {
        Either::Left((status, _)) => {
            log::info!("codex app server: process {pid} ended: {status:?}");
            true
        }
        Either::Right((_, status)) => {
            drop(status);
            send_signal(pid, rustix::process::Signal::TERM);
            let grace = executor.timer(STOP_GRACE);
            let ended = matches!(
                futures::future::select(Box::pin(child.status()), grace).await,
                Either::Left(_)
            );
            if !ended && let Err(error) = child.kill() {
                log::debug!("codex app server: killing process {pid}: {error}");
            }
            false
        }
    }
}

/// Sends `signal` to process `pid`, if it is still there.
fn send_signal(pid: u32, signal: rustix::process::Signal) {
    let Some(process) = i32::try_from(pid)
        .ok()
        .and_then(rustix::process::Pid::from_raw)
    else {
        return;
    };
    match rustix::process::kill_process(process, signal) {
        Ok(()) => {}
        Err(error) if error == rustix::io::Errno::SRCH => {}
        Err(error) => log::debug!("codex app server: signalling process {pid}: {error}"),
    }
}

/// Removes what a server left of `socket`: the path, and, when Codex made it a link into its own
/// folder (0.158), the socket it names and that socket's lock.
fn remove_leftovers(socket: &Path) {
    if let Ok(target) = std::fs::read_link(socket) {
        let codex_made = target
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|name| name.to_string_lossy().starts_with("codex-daemon-"));
        if codex_made {
            let mut lock = target.clone().into_os_string();
            lock.push(".lock");
            remove_if_there(&target);
            remove_if_there(Path::new(&lock));
        }
    }
    remove_if_there(socket);
}

fn remove_if_there(path: &Path) {
    match std::fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => log::debug!("codex app server: {} stays: {error}", path.display()),
    }
}

/// Whether a connection to `socket` has been accepted: a connected line under the socket's path
/// in `/proc/net/unix`, whose accepted sockets carry the listening one's path.
fn tui_connected(socket: &Path) -> bool {
    let Ok(real) = std::fs::canonicalize(socket) else {
        return false;
    };
    let Ok(table) = std::fs::read_to_string("/proc/net/unix") else {
        return false;
    };
    let real = real.to_string_lossy();
    table.lines().skip(1).any(|line| {
        let fields: Vec<&str> = line.split_whitespace().collect();
        fields.get(5) == Some(&"03") && fields.get(7) == Some(&real.as_ref())
    })
}

/// What the follow waits on: the connection's messages and the server's end.
enum Event {
    Response {
        id: u64,
        result: Result<Value, String>,
    },
    Notification {
        method: String,
        params: Value,
    },
    Closed(String),
    ServerExited(Option<String>),
    /// A request of the server's (#651).
    Request {
        id: Value,
        method: String,
        params: Value,
    },
    /// A decision the user picked in the inbox (#651).
    Answer(Answer),
}

/// Sorts one message from the server: a response, a notification, or a request of the server's.
fn sort(text: &str, events: &mpsc::UnboundedSender<Event>) {
    let message: Value = match serde_json::from_str(text) {
        Ok(message) => message,
        Err(error) => {
            log::debug!("codex app server: a message that is not JSON: {error}");
            return;
        }
    };
    let method = message.get("method").and_then(Value::as_str);
    let event = match (message.get("id"), method) {
        (Some(id), Some(method)) => Event::Request {
            id: id.clone(),
            method: method.to_string(),
            params: message.get("params").cloned().unwrap_or(Value::Null),
        },
        (Some(id), None) => {
            let Some(id) = id.as_u64() else {
                return;
            };
            let result = message.get("error").map_or_else(
                || Ok(message.get("result").cloned().unwrap_or(Value::Null)),
                |error| {
                    Err(error
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("an error with no message")
                        .to_string())
                },
            );
            Event::Response { id, result }
        }
        (None, Some(method)) => Event::Notification {
            method: method.to_string(),
            params: message.get("params").cloned().unwrap_or(Value::Null),
        },
        (None, None) => return,
    };
    events.unbounded_send(event).ok();
}

/// Marley's connection to a server.
struct Client {
    outgoing: mpsc::UnboundedSender<String>,
    incoming: mpsc::UnboundedReceiver<Event>,
    /// What came while a call waited, for the follow.
    pending: VecDeque<Event>,
    next_id: u64,
    executor: gpui::BackgroundExecutor,
    _reader: Task<()>,
    _writer: Task<()>,
}

impl Client {
    async fn connect(
        socket: &Path,
        events: mpsc::UnboundedSender<Event>,
        incoming: mpsc::UnboundedReceiver<Event>,
        cx: &AsyncApp,
    ) -> anyhow::Result<Self> {
        let stream = smol::net::unix::UnixStream::connect(socket)
            .await
            .with_context(|| format!("connecting to {}", socket.display()))?;
        let (websocket, _response) = async_tungstenite::client_async("ws://localhost/rpc", stream)
            .await
            .context("the server's WebSocket handshake")?;
        let (mut sink, mut source) = websocket.split();
        let (outgoing, mut outgoing_rx) = mpsc::unbounded::<String>();
        let writer = cx.background_spawn(async move {
            while let Some(text) = outgoing_rx.next().await {
                if let Err(error) = sink.send(Message::Text(text.into())).await {
                    log::debug!("codex app server: writing failed: {error}");
                    break;
                }
            }
        });
        let reader = cx.background_spawn(async move {
            let reason = loop {
                match source.next().await {
                    Some(Ok(Message::Text(text))) => sort(text.as_str(), &events),
                    Some(Ok(Message::Close(_))) => break "the server closed it".to_string(),
                    Some(Ok(_)) => {}
                    Some(Err(error)) => break error.to_string(),
                    None => break "the socket ended".to_string(),
                }
            };
            events.unbounded_send(Event::Closed(reason)).ok();
        });
        Ok(Self {
            outgoing,
            incoming,
            pending: VecDeque::new(),
            next_id: 1,
            executor: cx.background_executor().clone(),
            _reader: reader,
            _writer: writer,
        })
    }

    /// The next thing the follow hears.
    async fn next(&mut self) -> Event {
        match self.pending.pop_front() {
            Some(event) => event,
            None => self
                .incoming
                .next()
                .await
                .unwrap_or_else(|| Event::Closed("the connection's tasks ended".to_string())),
        }
    }

    /// Sends `method` with `params` and waits for the answer, keeping what comes meanwhile.
    async fn call(&mut self, method: &str, params: Value) -> anyhow::Result<Value> {
        let id = self.next_id;
        self.next_id += 1;
        let request = json!({ "id": id, "method": method, "params": params });
        self.outgoing
            .unbounded_send(request.to_string())
            .map_err(|_| anyhow::anyhow!("the connection is closed"))?;
        let mut timer = self.executor.timer(CALL_TIMEOUT);
        loop {
            match futures::future::select(self.incoming.next(), &mut timer).await {
                Either::Left((
                    Some(Event::Response {
                        id: answered,
                        result,
                    }),
                    _,
                )) if answered == id => {
                    return result.map_err(|message| anyhow::anyhow!("{method}: {message}"));
                }
                Either::Left((Some(Event::Response { .. }), _)) => {}
                Either::Left((Some(event @ (Event::Closed(_) | Event::ServerExited(_))), _)) => {
                    self.pending.push_back(event);
                    anyhow::bail!("{method}: the server went before it answered");
                }
                Either::Left((Some(event), _)) => self.pending.push_back(event),
                Either::Left((None, _)) => anyhow::bail!("{method}: the connection is closed"),
                Either::Right(_) => {
                    anyhow::bail!("{method}: no answer within {} s", CALL_TIMEOUT.as_secs())
                }
            }
        }
    }

    /// Answers the server's request `id` with `result`.
    fn respond(&self, id: &Value, result: &Value) -> anyhow::Result<()> {
        self.outgoing
            .unbounded_send(json!({ "id": id, "result": result }).to_string())
            .map_err(|_| anyhow::anyhow!("the connection is closed"))
    }

    fn notify(&self, method: &str) {
        self.outgoing
            .unbounded_send(json!({ "method": method }).to_string())
            .ok();
    }
}

/// The thread Marley follows.
struct Lead {
    id: String,
    idle: bool,
    /// Whether Marley has resumed it, which subscribes it and reads its policies.
    resumed: bool,
}

/// One terminal's follow, between messages.
struct Following {
    terminal: WeakEntity<Terminal>,
    view: Option<EntityId>,
    lead: Option<Lead>,
    read: HashSet<String>,
    heard_at: Option<Instant>,
    /// This connection's generation, which an inbox answer must name (#651).
    generation: u64,
    /// The lead's requests the inbox answers, in the order they came (#651).
    requests: Vec<Shown>,
    /// The lead's file changes' paths, by item, from `item/started` (#651).
    files: HashMap<String, Vec<String>>,
    /// The way an inbox answer comes back to this follow.
    answers: mpsc::UnboundedSender<Event>,
}

impl Following {
    /// Publishes the requests for the inbox.
    fn publish(&self, cx: &AsyncApp) {
        let terminal = self.terminal.entity_id();
        let published = Published {
            generation: self.generation,
            requests: self.requests.clone(),
            answers: self.answers.clone(),
        };
        cx.update(|cx| {
            let _previous = cx
                .default_global::<CodexRequests>()
                .terminals
                .insert(terminal, published);
        });
    }

    /// Forgets the requests and the items, which belong to a lead that went.
    fn forget_requests(&mut self, cx: &AsyncApp) {
        self.files.clear();
        if !self.requests.is_empty() {
            self.requests.clear();
            self.publish(cx);
        }
    }

    /// Folds `input` into the seat of the terminal's view.
    fn fold(&mut self, input: &Input<'_>, cx: &AsyncApp) {
        let terminal = self.terminal.entity_id();
        let known = self.view;
        let view = cx.update(|cx| {
            let view = known.or_else(|| view_of(terminal, cx))?;
            crate::agent_events::fold_codex(view, input, cx);
            Some(view)
        });
        self.view = view;
    }

    /// Whether Codex is the terminal's foreground program.
    fn codex_runs(&self, cx: &AsyncApp) -> bool {
        let terminal = self.terminal.clone();
        cx.update(|cx| {
            terminal.upgrade().is_some_and(|terminal| {
                crate::agent_bar::agent_in(terminal.read(cx)) == Some(AgentKind::Codex)
            })
        })
    }
}

/// The terminal view showing `terminal`.
fn view_of(terminal: EntityId, cx: &App) -> Option<EntityId> {
    crate::mcp::terminals(cx)
        .into_iter()
        .find(|(_, view)| view.read(cx).terminal().entity_id() == terminal)
        .map(|(_, view)| view.entity_id())
}

/// Joins the server on `socket` once the TUI has, and follows its lead thread into the terminal's
/// seat until the server or the terminal goes.
async fn follow(
    socket: &Path,
    terminal: &WeakEntity<Terminal>,
    events: mpsc::UnboundedSender<Event>,
    mut incoming: mpsc::UnboundedReceiver<Event>,
    cx: &AsyncApp,
) -> anyhow::Result<()> {
    loop {
        let connected = cx
            .background_spawn(futures::future::lazy({
                let socket = socket.to_path_buf();
                move |_| tui_connected(&socket)
            }))
            .await;
        if connected {
            break;
        }
        if terminal.upgrade().is_none() {
            return Ok(());
        }
        let check = cx.background_executor().timer(CONNECTION_CHECK);
        if let Either::Left((Some(Event::ServerExited(_)) | None, _)) =
            futures::future::select(incoming.next(), check).await
        {
            return Ok(());
        }
    }
    cx.background_executor().timer(AFTER_TUI).await;
    let answers = events.clone();
    let mut client = Client::connect(socket, events, incoming, cx).await?;
    let version = cx.update(|cx| release_channel::AppVersion::global(cx).to_string());
    let answer = client
        .call(
            "initialize",
            json!({
                "clientInfo": { "name": "marley", "title": "Marley", "version": version },
                "capabilities": {
                    "experimentalApi": true,
                    "optOutNotificationMethods": OPTED_OUT,
                },
            }),
        )
        .await?;
    if !server_tested(&answer, cx) {
        return Ok(());
    }
    client.notify("initialized");
    let mut following = Following {
        terminal: terminal.clone(),
        view: None,
        lead: None,
        read: HashSet::new(),
        heard_at: None,
        generation: NEXT_GENERATION.fetch_add(1, Ordering::Relaxed),
        requests: Vec::new(),
        files: HashMap::new(),
        answers,
    };
    loop {
        let tick = cx.background_executor().timer(TICK);
        let event = match futures::future::select(Box::pin(client.next()), tick).await {
            Either::Left((event, _)) => Some(event),
            Either::Right(_) => None,
        };
        match event {
            None => {
                if terminal.upgrade().is_none() {
                    return Ok(());
                }
                tick_over(&mut client, &mut following, cx).await?;
            }
            Some(Event::Notification { method, params }) => {
                heard(&method, &params, &mut client, &mut following, cx).await?;
            }
            Some(Event::Response { .. }) => {}
            Some(Event::Request { id, method, params }) => {
                requested(&id, &method, &params, &mut following, cx);
            }
            Some(Event::Answer(answer)) => answered(&answer, &client, &mut following, cx),
            Some(Event::ServerExited(last)) => {
                server_stopped(last.as_deref(), &mut following, cx);
                return Ok(());
            }
            Some(Event::Closed(reason)) => {
                log::info!("codex app server: the connection closed: {reason}");
                // The server's own end usually follows, with its last line.
                let wait = cx.background_executor().timer(Duration::from_secs(1));
                let last = match futures::future::select(Box::pin(client.next()), wait).await {
                    Either::Left((Event::ServerExited(last), _)) => last,
                    _ => None,
                };
                server_stopped(last.as_deref(), &mut following, cx);
                return Ok(());
            }
        }
    }
}

/// Whether the server's version, after the first `/` of its `userAgent`, is one Marley tested
/// the App Server on (#648), or allowed; logged when it is not.
fn server_tested(answer: &Value, cx: &AsyncApp) -> bool {
    let user_agent = answer
        .get("userAgent")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let found = user_agent
        .split_once('/')
        .map(|(_, rest)| rest)
        .ok_or_else(|| format!("named no version in \"{user_agent}\""))
        .and_then(versions::parse_version)
        .map_or_else(Found::Unreadable, Found::Version);
    let allowed = cx.update(|cx| {
        MarleySettings::get_global(cx)
            .allow_untested_versions
            .contains(CODEX_APP_SERVER.id)
    });
    let verdict = versions::verdict(&CODEX_APP_SERVER, Some(&found), allowed);
    if !verdict.is_on() {
        log::info!(
            "codex app server: the server reports {found:?}, outside the versions Marley tested; \
             Marley leaves it"
        );
    }
    verdict.is_on()
}

/// A tick: Codex gone from the foreground lets its lead go; with no lead, the loaded threads are
/// read for one; a lead not yet resumed is resumed once idle.
async fn tick_over(
    client: &mut Client,
    following: &mut Following,
    cx: &AsyncApp,
) -> anyhow::Result<()> {
    if !following.codex_runs(cx) {
        if let Some(lead) = following.lead.take() {
            unsubscribe(client, &lead.id).await;
            following.forget_requests(cx);
        }
        return Ok(());
    }
    if following.lead.is_none() {
        let listed = client.call("thread/loaded/list", json!({})).await?;
        let ids: Vec<String> = listed
            .get("data")
            .and_then(Value::as_array)
            .map(|ids| {
                ids.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        let mut newest: Option<Thread> = None;
        for id in ids {
            if !following.read.insert(id.clone()) {
                continue;
            }
            let answer = client
                .call("thread/read", json!({ "threadId": id }))
                .await?;
            match codex_events::read_answer(&answer) {
                Ok(thread) if thread.is_lead() => {
                    if newest
                        .as_ref()
                        .is_none_or(|newest| thread.created_at >= newest.created_at)
                    {
                        newest = Some(thread);
                    }
                }
                Ok(_) => {}
                Err(error) => log::debug!("codex app server: {error}"),
            }
        }
        if let Some(thread) = newest {
            follow_lead(&thread, following, cx);
        }
    }
    let due = following
        .lead
        .as_ref()
        .filter(|lead| lead.idle && !lead.resumed)
        .map(|lead| lead.id.clone());
    if let Some(id) = due {
        let answer = client
            .call(
                "thread/resume",
                json!({ "threadId": id, "excludeTurns": true }),
            )
            .await?;
        match codex_events::resume_answer(&answer) {
            Ok((_, policies)) => following.fold(&Input::Policies(&policies), cx),
            Err(error) => log::debug!("codex app server: {error}"),
        }
        if let Some(lead) = following.lead.as_mut().filter(|lead| lead.id == id) {
            lead.resumed = true;
        }
    }
    Ok(())
}

/// Follows `thread` as the lead from here.
fn follow_lead(thread: &Thread, following: &mut Following, cx: &AsyncApp) {
    following.forget_requests(cx);
    following.lead = Some(Lead {
        id: thread.id.clone(),
        idle: thread.status == ThreadStatus::Idle,
        resumed: false,
    });
    following.fold(&Input::Lead(thread), cx);
}

/// A notification: a new lead, a side thread to let go, or the lead's own messages for its seat.
async fn heard(
    method: &str,
    params: &Value,
    client: &mut Client,
    following: &mut Following,
    cx: &AsyncApp,
) -> anyhow::Result<()> {
    // The inbox's requests: one resolved, by whichever client answered first, and the paths of a
    // file change a request will name by its item (#651).
    if method == "serverRequest/resolved" {
        if let Some((thread, request)) = codex_events::resolved(params) {
            let before = following.requests.len();
            following
                .requests
                .retain(|shown| !(shown.request.thread == thread && shown.request.id == request));
            if following.requests.len() != before {
                following.publish(cx);
            }
        }
        return Ok(());
    }
    if method == "item/started"
        && let Some((thread, item, paths)) = codex_events::file_change_started(params)
        && following
            .lead
            .as_ref()
            .is_some_and(|lead| lead.id == thread)
    {
        let _previous = following.files.insert(item, paths);
    }
    let notification = match Notification::decode(method, params) {
        Ok(Some(notification)) => notification,
        Ok(None) => {
            let about_lead = params
                .get("threadId")
                .and_then(Value::as_str)
                .is_some_and(|thread| {
                    following
                        .lead
                        .as_ref()
                        .is_some_and(|lead| lead.id == thread)
                });
            if about_lead
                && following
                    .heard_at
                    .is_none_or(|at| at.elapsed() >= HEARD_EVERY)
            {
                following.heard_at = Some(Instant::now());
                if let Some(view) = following.view {
                    cx.update(|cx| crate::agent_events::heard_from(view, cx));
                }
            }
            return Ok(());
        }
        Err(error) => {
            log::debug!("codex app server: {error}");
            return Ok(());
        }
    };
    if let Notification::ThreadStarted(thread) = &notification {
        let _known = following.read.insert(thread.id.clone());
        // The server subscribes every joined connection to each thread it makes; Marley keeps only
        // the lead, since a subscriber keeps a thread loaded.
        if thread.is_lead() && following.codex_runs(cx) {
            if let Some(old) = following.lead.take() {
                unsubscribe(client, &old.id).await;
            }
            follow_lead(thread, following, cx);
        } else {
            unsubscribe(client, &thread.id).await;
        }
        return Ok(());
    }
    let about_lead = following
        .lead
        .as_ref()
        .is_some_and(|lead| lead.id == notification.thread());
    if !about_lead {
        return Ok(());
    }
    match &notification {
        Notification::StatusChanged { status, .. } => {
            if let Some(lead) = following.lead.as_mut() {
                lead.idle = *status == ThreadStatus::Idle;
            }
        }
        Notification::ThreadClosed { .. } => {
            following.lead = None;
            following.forget_requests(cx);
        }
        _ => {}
    }
    following.fold(&Input::Notification(&notification), cx);
    Ok(())
}

/// A request of the server's: one the inbox answers, on the lead, joins its list; any other is
/// left to the TUI, unanswered (#651).
fn requested(id: &Value, method: &str, params: &Value, following: &mut Following, cx: &AsyncApp) {
    let request = match Request::decode(id, method, params) {
        Ok(Some(request)) => request,
        Ok(None) => {
            log::debug!("codex app server: left the server's {method} request to the TUI");
            return;
        }
        Err(error) => {
            log::debug!("codex app server: {error}");
            return;
        }
    };
    let on_lead = following
        .lead
        .as_ref()
        .is_some_and(|lead| lead.id == request.thread);
    if !on_lead
        || following
            .requests
            .iter()
            .any(|shown| shown.request.id == request.id)
    {
        return;
    }
    let files = match &request.kind {
        RequestKind::FileChange { item, .. } => following.files.get(item).cloned(),
        _ => None,
    };
    following.requests.push(Shown {
        request,
        files,
        sent: None,
    });
    following.publish(cx);
}

/// A decision the user picked: sent only while the same request, with the params its entry
/// showed, is unresolved on this connection and not answered yet; it then shows as sent until the
/// server reports it resolved (#651).
fn answered(answer: &Answer, client: &Client, following: &mut Following, cx: &AsyncApp) {
    let generation = following.generation;
    let Some(shown) = following.requests.iter_mut().find(|shown| {
        answer.generation == generation
            && shown.request.id == answer.id
            && shown.request.params == answer.params
            && shown.sent.is_none()
    }) else {
        log::info!("codex app server: the request changed or went before the answer; nothing sent");
        return;
    };
    let Some(result) = shown.request.response(answer.decision) else {
        log::warn!(
            "codex app server: {} does not take {}",
            shown.request.ask(None),
            answer.decision.word()
        );
        return;
    };
    match client.respond(&shown.request.id, &result) {
        Ok(()) => {
            shown.sent = Some(answer.decision);
            following.publish(cx);
        }
        Err(error) => log::warn!("codex app server: answering: {error:#}"),
    }
}

/// Lets `thread` go; a failure only reaches the log, since the server unloads it with the rest.
async fn unsubscribe(client: &mut Client, thread: &str) {
    if let Err(error) = client
        .call("thread/unsubscribe", json!({ "threadId": thread }))
        .await
    {
        log::debug!("codex app server: {error:#}");
    }
}

/// The server stopped: a seat it fed reads failed, with the server's last error line.
fn server_stopped(last: Option<&str>, following: &mut Following, cx: &AsyncApp) {
    if following.lead.is_some() {
        following.fold(&Input::ServerStopped(last), cx);
    }
}
