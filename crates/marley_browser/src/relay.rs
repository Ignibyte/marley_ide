//! Marley's browser relay (#583): the main process of a project's Chromium unit.
//!
//! Chromium listens on no port. It speaks CDP on its pipe, reading commands from fd 3 and
//! writing answers and events to fd 4, each a JSON message ended by a NUL byte, as Playwright's
//! pipe transport has it. The relay is its parent and its only client there, and it lets several
//! clients share the one pipe. Each client gets a browser session of its own
//! (`Target.attachToBrowserTarget`), which keeps one client's target discovery and auto-attach
//! apart from another's; a client's command ids are rewritten on the way in and restored on the
//! way out; a command names only a session its client owns; each event goes to the client that
//! owns its session. Marley connects through a Unix socket at mode 0600 in the project's folder,
//! other clients (a Playwright script, the e2e stand-in agent) through a WebSocket on 127.0.0.1
//! that takes a token the relay mints at each start and writes, with the address, into a 0600
//! file beside the socket. Both carry the same WebSocket frames as Chromium's own endpoint did.
//!
//! On SIGTERM the relay closes Chromium over the pipe, so the cookies it set last are written; when
//! Chromium exits, the relay removes its socket and file and exits too, and the unit stops.

use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::Context as _;
use async_tungstenite::WebSocketStream;
use async_tungstenite::tungstenite::Message;
use async_tungstenite::tungstenite::handshake::server::{
    Callback, ErrorResponse, Request, Response,
};
use async_tungstenite::tungstenite::http::StatusCode;
use futures::StreamExt as _;
use futures::channel::mpsc;
use futures::future::{self, Either};
use futures::io::{AsyncBufReadExt as _, AsyncRead, AsyncWrite, AsyncWriteExt as _, BufReader};
use serde_json::{Value, json};

/// The first argument that makes Marley's executable run as the relay.
pub const RELAY_FLAG: &str = "--browser-relay";

/// How long a client has to finish its WebSocket handshake, before its token is known to be good
/// (PR-claude-bound-every-read-before-auth-in-size-and-time-001).
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

/// How long Chromium has to answer its first command before the relay gives up on it.
const READY_TIMEOUT: Duration = Duration::from_secs(15);

/// How long Chromium has to exit after `Browser.close` before it is killed.
const CLOSE_TIMEOUT: Duration = Duration::from_secs(5);

/// The CDP error code for a session a client does not own, as Chromium answers for one it does
/// not know.
const NO_SUCH_SESSION: i64 = -32001;

/// Runs the relay when Marley's executable was started as one (`marley --browser-relay …`),
/// before anything of the app starts; the process's exit code. `None` for any other start.
#[must_use]
pub fn run_if_invoked() -> Option<i32> {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some(OsStr::new(RELAY_FLAG)) {
        return None;
    }
    Some(match Args::parse(args) {
        Ok(args) => smol::block_on(async move {
            match relay(args).await {
                Ok(code) => code,
                Err(error) => {
                    report(&format!("{error:#}"));
                    1
                }
            }
        }),
        Err(error) => {
            report(&format!("{error:#}"));
            2
        }
    })
}

/// Writes `message` to the unit's journal: the relay starts before Marley's logger, and a closed
/// stderr has nowhere else to go.
fn report(message: &str) {
    writeln!(std::io::stderr(), "marley relay: {message}").ok();
}

/// What the unit starts the relay with: `--socket <path> --endpoint-file <path> -- <chromium>
/// <arguments>`.
struct Args {
    socket: PathBuf,
    endpoint_file: PathBuf,
    chromium: Vec<OsString>,
}

impl Args {
    fn parse(mut args: impl Iterator<Item = OsString>) -> anyhow::Result<Self> {
        let (mut socket, mut endpoint_file) = (None, None);
        while let Some(arg) = args.next() {
            match arg.to_str() {
                Some("--socket") => socket = args.next().map(PathBuf::from),
                Some("--endpoint-file") => endpoint_file = args.next().map(PathBuf::from),
                Some("--") => break,
                _ => anyhow::bail!("an argument the relay does not take: {}", arg.display()),
            }
        }
        let chromium: Vec<OsString> = args.collect();
        anyhow::ensure!(!chromium.is_empty(), "no Chromium command after `--`");
        Ok(Self {
            socket: socket.context("no --socket")?,
            endpoint_file: endpoint_file.context("no --endpoint-file")?,
            chromium,
        })
    }
}

/// The relay's run: Chromium started and ready, the listeners up, then the switch until Chromium
/// exits; Chromium's exit code.
async fn relay(args: Args) -> anyhow::Result<i32> {
    let (mut child, to_chromium, from_chromium) = spawn_chromium(&args.chromium)?;
    let mut from_chromium = BufReader::new(from_chromium);
    let mut to_chromium = to_chromium;
    let listening = async {
        ready(&mut to_chromium, &mut from_chromium).await?;
        let token = mint_token()?;
        let unix = bind_unix(&args.socket)?;
        let tcp = smol::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .context("listening on 127.0.0.1")?;
        let port = tcp.local_addr().context("reading the relay's port")?.port();
        write_endpoint_file(&args.endpoint_file, port, &token)?;
        anyhow::Ok((token, unix, tcp, port))
    }
    .await;
    // A Chromium no client can reach has no reason to run.
    let (token, unix, tcp, port) = match listening {
        Ok(listening) => listening,
        Err(error) => {
            child.kill().ok();
            remove_if_there(&args.socket);
            return Err(error);
        }
    };

    let (inputs, incoming) = mpsc::unbounded();
    smol::spawn(read_chromium(from_chromium, inputs.clone())).detach();
    smol::spawn(accept_unix(unix, inputs.clone())).detach();
    smol::spawn(accept_tcp(tcp, token, port, inputs.clone())).detach();
    smol::spawn(watch_signals(inputs.clone())).detach();
    Switch::default()
        .run(incoming, to_chromium, &mut child, inputs)
        .await;

    remove_if_there(&args.socket);
    remove_if_there(&args.endpoint_file);
    // The switch also ends when the pipe breaks under a Chromium that still runs.
    if matches!(child.try_status(), Ok(None)) {
        child.kill().ok();
    }
    let status = child.status().await.context("waiting for Chromium")?;
    Ok(status.code().unwrap_or(1))
}

/// Starts Chromium from `command` with its CDP pipe on fds 3 and 4; the child, the pipe the
/// relay writes commands to, and the pipe it reads answers and events from.
fn spawn_chromium(
    command: &[OsString],
) -> anyhow::Result<(
    smol::process::Child,
    smol::Unblock<std::io::PipeWriter>,
    smol::Unblock<std::io::PipeReader>,
)> {
    use command_fds::{CommandFdExt as _, FdMapping};
    use std::process::Stdio;

    let program = command.first().context("no Chromium command")?;
    let (commands_read, commands_write) = std::io::pipe().context("making Chromium's pipe")?;
    let (answers_read, answers_write) = std::io::pipe().context("making Chromium's pipe")?;
    // A fixed program runs Chromium's command, which arrives as positional parameters, each
    // quoted as one word: nothing in the command is read as a program's name or as shell. The
    // exec keeps fds 3 and 4.
    let mut spawn = std::process::Command::new("/bin/sh");
    spawn.arg("-c").arg(r#"exec "$0" "$@""#).args(command);
    spawn
        .fd_mappings(vec![
            FdMapping {
                parent_fd: commands_read.into(),
                child_fd: 3,
            },
            FdMapping {
                parent_fd: answers_write.into(),
                child_fd: 4,
            },
        ])
        .context("mapping Chromium's pipe to fds 3 and 4")?;
    let mut spawn = smol::process::Command::from(spawn);
    let child = spawn
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("starting {}", Path::new(program).display()))?;
    // The command holds the child's ends of the pipe until it drops, and the relay would never
    // read the end of the answers while it held their writer.
    drop(spawn);
    Ok((
        child,
        smol::Unblock::new(commands_write),
        smol::Unblock::new(answers_read),
    ))
}

/// Waits for Chromium to answer `Browser.getVersion`, within [`READY_TIMEOUT`].
async fn ready<W, R>(to_chromium: &mut W, from_chromium: &mut R) -> anyhow::Result<()>
where
    W: AsyncWrite + Unpin,
    R: futures::io::AsyncBufRead + Unpin,
{
    send_to_chromium(
        to_chromium,
        &json!({ "id": 0, "method": "Browser.getVersion" }),
    )
    .await
    .context("writing to Chromium's pipe")?;
    let answered = async {
        loop {
            let Some(message) = read_message(from_chromium).await? else {
                anyhow::bail!("Chromium closed its pipe before it answered");
            };
            if message.get("id") == Some(&json!(0)) {
                return Ok(());
            }
        }
    };
    match future::select(
        pin(answered),
        pin(smol::Timer::at(Instant::now() + READY_TIMEOUT)),
    )
    .await
    {
        Either::Left((answered, _)) => answered,
        Either::Right(_) => anyhow::bail!(
            "Chromium did not answer within {} seconds",
            READY_TIMEOUT.as_secs()
        ),
    }
}

/// `future` pinned on the heap, for `select`.
fn pin<F: Future>(future: F) -> std::pin::Pin<Box<F>> {
    Box::pin(future)
}

/// Writes `message` and its NUL to Chromium's pipe.
async fn send_to_chromium<W: AsyncWrite + Unpin>(
    writer: &mut W,
    message: &Value,
) -> std::io::Result<()> {
    let mut bytes = message.to_string().into_bytes();
    bytes.push(0);
    writer.write_all(&bytes).await?;
    writer.flush().await
}

/// The next message from Chromium's pipe; `None` at its end.
async fn read_message<R>(reader: &mut R) -> anyhow::Result<Option<Value>>
where
    R: futures::io::AsyncBufRead + Unpin,
{
    let mut bytes = Vec::new();
    loop {
        bytes.clear();
        if reader
            .read_until(0, &mut bytes)
            .await
            .context("reading Chromium's pipe")?
            == 0
        {
            return Ok(None);
        }
        if bytes.last() == Some(&0) {
            bytes.pop();
        }
        if bytes.is_empty() {
            continue;
        }
        return serde_json::from_slice(&bytes)
            .map(Some)
            .context("Chromium sent a message that is not JSON");
    }
}

/// A token of 16 bytes from the OS's generator, as hex.
fn mint_token() -> anyhow::Result<String> {
    let mut bytes = [0u8; 16];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut random| random.read_exact(&mut bytes))
        .context("reading /dev/urandom for the relay's token")?;
    Ok(hex::encode(bytes))
}

/// Binds the Unix socket at `path` at mode 0600, a socket the last relay left removed first.
fn bind_unix(path: &Path) -> anyhow::Result<smol::net::unix::UnixListener> {
    remove_if_there(path);
    let listener = smol::net::unix::UnixListener::bind(path)
        .with_context(|| format!("listening at {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .with_context(|| format!("making {} its owner's alone", path.display()))?;
    }
    Ok(listener)
}

/// Writes the loopback endpoint and its token to `path` at mode 0600, through a file renamed
/// into place.
fn write_endpoint_file(path: &Path, port: u16, token: &str) -> anyhow::Result<()> {
    let staged = path.with_extension("json.new");
    let json = json!({
        "url": format!("ws://127.0.0.1:{port}/devtools/browser"),
        "token": token,
    })
    .to_string();
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options
        .open(&staged)
        .with_context(|| format!("writing {}", staged.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))
            .with_context(|| format!("making {} its owner's alone", staged.display()))?;
    }
    file.write_all(json.as_bytes())
        .with_context(|| format!("writing {}", staged.display()))?;
    std::fs::rename(&staged, path).with_context(|| format!("writing {}", path.display()))
}

/// Removes the file at `path` when it is there; a removal that fails is reported.
fn remove_if_there(path: &Path) {
    match std::fs::remove_file(path) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            report(&format!("removing {}: {error}", path.display()));
        }
        _ => {}
    }
}

/// What reaches the switch.
enum Input {
    /// A message from Chromium.
    FromChromium(Value),
    /// Chromium's pipe ended: it exited.
    ChromiumGone,
    /// A client connected; its messages go to `sender`.
    ClientJoined(u64, mpsc::UnboundedSender<String>),
    /// A client's message.
    FromClient(u64, String),
    /// A client left.
    ClientGone(u64),
    /// SIGTERM or SIGINT: close Chromium.
    Stop,
    /// Chromium did not exit within [`CLOSE_TIMEOUT`] of `Browser.close`.
    CloseTimedOut,
}

/// Reads Chromium's messages into the switch until its pipe ends.
async fn read_chromium<R>(mut reader: R, inputs: mpsc::UnboundedSender<Input>)
where
    R: futures::io::AsyncBufRead + Unpin,
{
    loop {
        match read_message(&mut reader).await {
            Ok(Some(message)) => {
                if inputs.unbounded_send(Input::FromChromium(message)).is_err() {
                    return;
                }
            }
            Ok(None) => break,
            Err(error) => {
                report(&format!("{error:#}"));
                break;
            }
        }
    }
    inputs.unbounded_send(Input::ChromiumGone).ok();
}

/// Sends SIGTERM and SIGINT to the switch as [`Input::Stop`].
async fn watch_signals(inputs: mpsc::UnboundedSender<Input>) {
    use async_signal::{Signal, Signals};
    let mut signals = match Signals::new([Signal::Term, Signal::Int]) {
        Ok(signals) => signals,
        Err(error) => {
            report(&format!("watching for SIGTERM: {error}"));
            return;
        }
    };
    while signals.next().await.is_some() {
        if inputs.unbounded_send(Input::Stop).is_err() {
            return;
        }
    }
}

/// Serves Marley's clients on the Unix socket: its 0600 mode admits the user alone.
async fn accept_unix(
    listener: smol::net::unix::UnixListener,
    inputs: mpsc::UnboundedSender<Input>,
) {
    let mut next_client = 0u64;
    loop {
        let stream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(error) => {
                report(&format!("accepting on the socket: {error}"));
                continue;
            }
        };
        // Unix clients count down from the top, so their numbers never meet the loopback's.
        next_client += 1;
        let client = u64::MAX - next_client;
        let inputs = inputs.clone();
        smol::spawn(async move {
            let handshake = async_tungstenite::accept_async(stream);
            match within(HANDSHAKE_TIMEOUT, handshake).await {
                Some(Ok(socket)) => serve_client(client, socket, inputs).await,
                Some(Err(error)) => report(&format!("a client's handshake on the socket: {error}")),
                None => {}
            }
        })
        .detach();
    }
}

/// Serves other clients on 127.0.0.1: each must send the token, and a web page's request is
/// refused.
async fn accept_tcp(
    listener: smol::net::TcpListener,
    token: String,
    port: u16,
    inputs: mpsc::UnboundedSender<Input>,
) {
    let mut next_client = 0u64;
    loop {
        let stream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(error) => {
                report(&format!("accepting on 127.0.0.1: {error}"));
                continue;
            }
        };
        next_client += 1;
        let client = next_client;
        let (inputs, token) = (inputs.clone(), token.clone());
        smol::spawn(async move {
            let handshake = async_tungstenite::accept_hdr_async(stream, Admission { token, port });
            if let Some(Ok(socket)) = within(HANDSHAKE_TIMEOUT, handshake).await {
                serve_client(client, socket, inputs).await;
            }
        })
        .detach();
    }
}

/// The loopback handshake's check ([`refusal`]), as tungstenite takes it.
struct Admission {
    token: String,
    port: u16,
}

impl Callback for Admission {
    fn on_request(self, request: &Request, response: Response) -> Result<Response, ErrorResponse> {
        match refusal(request, &self.token, self.port) {
            None => Ok(response),
            Some((status, reason)) => {
                let mut refused = ErrorResponse::new(Some(reason.to_string()));
                *refused.status_mut() = status;
                Err(refused)
            }
        }
    }
}

/// Why the WebSocket request `request` may not connect, if it may not: a web page's `Origin` or
/// another host's `Host` (403), or no token or a wrong one as `Authorization: Bearer` (401).
fn refusal(request: &Request, token: &str, port: u16) -> Option<(StatusCode, &'static str)> {
    let header = |name: &str| {
        request
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
    };
    let local_host = header("host").is_some_and(|host| {
        host == format!("127.0.0.1:{port}") || host == format!("localhost:{port}")
    });
    let page_origin = header("origin").is_some_and(|origin| {
        !(origin.starts_with("http://127.0.0.1:")
            || origin.starts_with("http://localhost:")
            || origin == "http://127.0.0.1"
            || origin == "http://localhost")
    });
    if !local_host || page_origin {
        return Some((StatusCode::FORBIDDEN, "not from this machine's own clients"));
    }
    let presented = header("authorization")
        .and_then(|value| value.strip_prefix("Bearer "))
        .unwrap_or_default();
    (!same_token(presented, token)).then_some((
        StatusCode::UNAUTHORIZED,
        "the relay's token is missing or wrong",
    ))
}

/// Whether `presented` is `token`, compared in constant time over their bytes.
fn same_token(presented: &str, token: &str) -> bool {
    let (presented, token) = (presented.as_bytes(), token.as_bytes());
    if presented.len() != token.len() {
        return false;
    }
    presented
        .iter()
        .zip(token)
        .fold(0u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}

/// `future`'s output, or `None` when `limit` passes first. The relay runs on smol's own reactor,
/// outside gpui, whose executor's timer the tree otherwise uses.
async fn within<F: Future>(limit: Duration, future: F) -> Option<F::Output> {
    match future::select(pin(future), pin(smol::Timer::at(Instant::now() + limit))).await {
        Either::Left((output, _)) => Some(output),
        Either::Right(_) => None,
    }
}

/// Carries one client's messages both ways until it leaves.
async fn serve_client<S>(
    client: u64,
    socket: WebSocketStream<S>,
    inputs: mpsc::UnboundedSender<Input>,
) where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let (mut sink, mut source) = socket.split();
    let (sender, mut outgoing) = mpsc::unbounded::<String>();
    if inputs
        .unbounded_send(Input::ClientJoined(client, sender))
        .is_err()
    {
        return;
    }
    let writer = smol::spawn(async move {
        while let Some(text) = outgoing.next().await {
            if sink.send(Message::Text(text.into())).await.is_err() {
                break;
            }
        }
        sink.close(None).await.ok();
    });
    while let Some(message) = source.next().await {
        match message {
            Ok(Message::Text(text)) => {
                if inputs
                    .unbounded_send(Input::FromClient(client, text.to_string()))
                    .is_err()
                {
                    break;
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            Ok(_) => {}
        }
    }
    inputs.unbounded_send(Input::ClientGone(client)).ok();
    drop(writer);
}

/// A command sent to Chromium, waiting for its answer.
enum Pending {
    /// A client's: its own id, its method, and whether the relay put it on the client's browser
    /// session.
    Client {
        client: u64,
        id: Value,
        method: String,
        on_browser_session: bool,
    },
    /// The browser session a new client gets.
    BrowserSession(u64),
    /// The relay's own, whose answer goes nowhere.
    Own,
}

/// One client.
#[derive(Default)]
struct Client {
    sender: Option<mpsc::UnboundedSender<String>>,
    browser_session: Option<String>,
    /// The sessions it attached, or that attached for it, besides its browser session.
    sessions: HashSet<String>,
    /// Its commands, while its browser session is being made.
    waiting: Vec<String>,
}

/// The relay's state: the clients, the commands waiting on Chromium, and who owns each session.
#[derive(Default)]
struct Switch {
    clients: HashMap<u64, Client>,
    pending: HashMap<u64, Pending>,
    owners: HashMap<String, u64>,
    next_id: u64,
    stopping: bool,
}

impl Switch {
    async fn run<W: AsyncWrite + Unpin>(
        mut self,
        mut incoming: mpsc::UnboundedReceiver<Input>,
        mut to_chromium: W,
        child: &mut smol::process::Child,
        inputs: mpsc::UnboundedSender<Input>,
    ) {
        while let Some(input) = incoming.next().await {
            let outgoing = match input {
                Input::FromChromium(message) => self.chromium_message(message),
                Input::ChromiumGone => break,
                Input::ClientJoined(client, sender) => self.joined(client, sender),
                Input::FromClient(client, text) => self.client_command(client, &text),
                Input::ClientGone(client) => self.gone(client),
                Input::Stop => {
                    if self.stopping {
                        Vec::new()
                    } else {
                        self.stopping = true;
                        let inputs = inputs.clone();
                        smol::spawn(async move {
                            smol::Timer::at(Instant::now() + CLOSE_TIMEOUT).await;
                            inputs.unbounded_send(Input::CloseTimedOut).ok();
                        })
                        .detach();
                        vec![self.own("Browser.close", &json!({}), None)]
                    }
                }
                Input::CloseTimedOut => {
                    report("Chromium did not close in time; killing it");
                    child.kill().ok();
                    Vec::new()
                }
            };
            for message in outgoing {
                if let Err(error) = send_to_chromium(&mut to_chromium, &message).await {
                    report(&format!("writing to Chromium's pipe: {error}"));
                    return;
                }
            }
        }
    }

    const fn next_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    /// A command of the relay's own, on `session` or the pipe's root.
    fn own(&mut self, method: &str, params: &Value, session: Option<&str>) -> Value {
        let id = self.next_id();
        self.pending.insert(id, Pending::Own);
        let mut message = json!({ "id": id, "method": method, "params": params });
        if let Some(session) = session {
            message["sessionId"] = json!(session);
        }
        message
    }

    /// A client joined: it gets a browser session of its own before its first command goes on.
    fn joined(&mut self, client: u64, sender: mpsc::UnboundedSender<String>) -> Vec<Value> {
        self.clients.insert(
            client,
            Client {
                sender: Some(sender),
                ..Client::default()
            },
        );
        let id = self.next_id();
        self.pending.insert(id, Pending::BrowserSession(client));
        vec![json!({ "id": id, "method": "Target.attachToBrowserTarget", "params": {} })]
    }

    /// A client's command, on its way to Chromium.
    fn client_command(&mut self, client: u64, text: &str) -> Vec<Value> {
        let Some(state) = self.clients.get_mut(&client) else {
            return Vec::new();
        };
        let Some(browser_session) = state.browser_session.clone() else {
            state.waiting.push(text.to_string());
            return Vec::new();
        };
        // A message that is not a command has no id to answer with: it goes nowhere.
        let Ok(Value::Object(mut message)) = serde_json::from_str::<Value>(text) else {
            return Vec::new();
        };
        let Some(id) = message.remove("id") else {
            return Vec::new();
        };
        let method = message
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let on_browser_session = match message.get("sessionId").and_then(Value::as_str) {
            None => {
                message.insert("sessionId".to_string(), json!(browser_session));
                true
            }
            Some(session) if session == browser_session || state.sessions.contains(session) => {
                false
            }
            Some(_) => {
                let refusal = json!({
                    "id": id,
                    "error": { "code": NO_SUCH_SESSION, "message": "Session with given id not found." },
                });
                if let Some(sender) = &state.sender {
                    sender.unbounded_send(refusal.to_string()).ok();
                }
                return Vec::new();
            }
        };
        let relay_id = self.next_id();
        self.pending.insert(
            relay_id,
            Pending::Client {
                client,
                id,
                method,
                on_browser_session,
            },
        );
        message.insert("id".to_string(), json!(relay_id));
        vec![Value::Object(message)]
    }

    /// A client left: the sessions it owned are detached, its browser session last, and what it
    /// waited for is dropped.
    fn gone(&mut self, client: u64) -> Vec<Value> {
        let Some(state) = self.clients.remove(&client) else {
            return Vec::new();
        };
        self.pending.retain(|_, pending| {
            !matches!(pending, Pending::Client { client: owner, .. } | Pending::BrowserSession(owner) if *owner == client)
        });
        state
            .sessions
            .into_iter()
            .chain(state.browser_session)
            .map(|session| {
                self.owners.remove(&session);
                self.own(
                    "Target.detachFromTarget",
                    &json!({ "sessionId": session }),
                    None,
                )
            })
            .collect()
    }

    /// A message from Chromium: an answer goes to the client that asked, under its own id; an
    /// event to the client that owns its session. What it gives back goes to Chromium: a new
    /// client's commands that waited for its browser session.
    fn chromium_message(&mut self, mut message: Value) -> Vec<Value> {
        if let Some(relay_id) = message.get("id").and_then(Value::as_u64) {
            match self.pending.remove(&relay_id) {
                Some(Pending::Client {
                    client,
                    id,
                    method,
                    on_browser_session,
                }) => {
                    if method == "Target.attachToTarget"
                        && let Some(session) =
                            message.pointer("/result/sessionId").and_then(Value::as_str)
                    {
                        self.own_session(client, session);
                    }
                    message["id"] = id;
                    if on_browser_session && let Some(object) = message.as_object_mut() {
                        object.remove("sessionId");
                    }
                    self.send(client, &message);
                }
                Some(Pending::BrowserSession(client)) => {
                    return self.browser_session_made(client, &message);
                }
                Some(Pending::Own) | None => {}
            }
            return Vec::new();
        }
        let Some(session) = message
            .get("sessionId")
            .and_then(Value::as_str)
            .map(str::to_string)
        else {
            return Vec::new();
        };
        let Some(&client) = self.owners.get(&session) else {
            return Vec::new();
        };
        match message.get("method").and_then(Value::as_str) {
            Some("Target.attachedToTarget") => {
                if let Some(child) = message.pointer("/params/sessionId").and_then(Value::as_str) {
                    self.own_session(client, child);
                }
            }
            Some("Target.detachedFromTarget") => {
                if let Some(child) = message.pointer("/params/sessionId").and_then(Value::as_str) {
                    self.owners.remove(child);
                    if let Some(state) = self.clients.get_mut(&client) {
                        state.sessions.remove(child);
                    }
                }
            }
            _ => {}
        }
        let on_browser_session = self
            .clients
            .get(&client)
            .is_some_and(|state| state.browser_session.as_deref() == Some(session.as_str()));
        if on_browser_session && let Some(object) = message.as_object_mut() {
            object.remove("sessionId");
        }
        self.send(client, &message);
        Vec::new()
    }

    /// `session` is `client`'s from now on.
    fn own_session(&mut self, client: u64, session: &str) {
        self.owners.insert(session.to_string(), client);
        if let Some(state) = self.clients.get_mut(&client) {
            state.sessions.insert(session.to_string());
        }
    }

    /// A new client's browser session was made: its waiting commands go on, as the messages it
    /// gives back. A client that got none is sent away.
    fn browser_session_made(&mut self, client: u64, answer: &Value) -> Vec<Value> {
        let Some(session) = answer
            .pointer("/result/sessionId")
            .and_then(Value::as_str)
            .map(str::to_string)
        else {
            report("Chromium gave a client no browser session");
            if let Some(state) = self.clients.get_mut(&client) {
                state.sender = None;
            }
            return Vec::new();
        };
        self.owners.insert(session.clone(), client);
        let waiting = match self.clients.get_mut(&client) {
            Some(state) => {
                state.browser_session = Some(session);
                std::mem::take(&mut state.waiting)
            }
            None => return Vec::new(),
        };
        waiting
            .iter()
            .flat_map(|text| self.client_command(client, text))
            .collect()
    }

    fn send(&self, client: u64, message: &Value) {
        if let Some(sender) = self
            .clients
            .get(&client)
            .and_then(|state| state.sender.as_ref())
        {
            sender.unbounded_send(message.to_string()).ok();
        }
    }
}
