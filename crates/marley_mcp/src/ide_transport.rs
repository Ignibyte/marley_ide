//! Claude Code's IDE server, the sockets (#653).
//!
//! A WebSocket server on a loopback port the system picks, one thread accepting and one per
//! client, as the HTTP transport runs. A client's upgrade request is read through the HTTP
//! transport's bounds before its token is checked; then `tungstenite` carries the frames, each
//! capped at [`MAX_MESSAGE_BYTES`]. Each client's thread reads with a short timeout and, between
//! reads, writes what the app queued for it, answers a tool call the app has answered, and pings
//! every 30 seconds, closing a client that misses two pongs. What a client may be sent is the
//! app's [`Judge`]'s word on its version; what it asks for goes to the app's [`IdeHandler`].

use std::collections::HashMap;
use std::io::{BufReader, Write as _};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, SyncSender, TryRecvError, channel, sync_channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::Value;
use tungstenite::protocol::{Role, WebSocketConfig};
use tungstenite::{Message, WebSocket};

use crate::ide::{self, MAX_CONNECTIONS, MAX_MESSAGE_BYTES, Parts, Step, Tool, Upgrade};
use crate::transport::{
    Deadlined, MAX_HEADER_LINES, REQUEST_READ_TIMEOUT, RequestError, read_bounded_line,
    read_entropy, write_status, write_status_then_drain,
};

/// How often the server pings each client.
const PING_EVERY: Duration = Duration::from_secs(30);

/// How long a read waits before the thread writes what is queued.
const READ_SLICE: Duration = Duration::from_millis(50);

/// How long a write may block before the client is dropped.
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);

/// How long the app has to answer a tool call.
const CALL_TIMEOUT: Duration = Duration::from_secs(crate::APP_CALL_TIMEOUT_SECONDS);

/// One client of one server.
pub type ConnectionId = u64;

/// What a client of the given version may be sent; `None` when it named no version.
pub type Judge = Arc<dyn Fn(Option<&str>) -> Parts + Send + Sync>;

/// Takes each message for the app, on a client's thread.
pub type IdeHandler = Arc<dyn Fn(IdeMessage) + Send + Sync>;

/// What the server tells the app.
#[derive(Debug)]
pub enum IdeMessage {
    /// A client upgraded, or named its version at `initialize`.
    Connected {
        /// The client.
        connection: ConnectionId,
        /// Its version, if it named one.
        version: Option<String>,
    },
    /// A client named its process (`ide_connected`).
    Identified {
        /// The client.
        connection: ConnectionId,
        /// Claude Code's process id.
        pid: u32,
    },
    /// A client left.
    Closed {
        /// The client.
        connection: ConnectionId,
    },
    /// A client called a tool the app answers.
    Call(IdeCall),
}

/// A tool call for the app; [`IdeCall::answer`] sends the reply.
#[derive(Debug)]
pub struct IdeCall {
    /// The client.
    pub connection: ConnectionId,
    /// The tool and its arguments.
    pub tool: Tool,
    answer: SyncSender<Result<String, String>>,
}

impl IdeCall {
    /// Sends the tool's text, or an error the client reads as the tool's failure.
    pub fn answer(self, answer: Result<String, String>) {
        if self.answer.send(answer).is_err() {
            log::debug!("claude ide: an answer came after its client left");
        }
    }
}

/// What a client's thread writes for the app.
enum Out {
    Text(String),
    Close,
}

/// What the server's threads share.
#[derive(Default)]
struct Inner {
    clients: Mutex<HashMap<ConnectionId, Sender<Out>>>,
    closed: AtomicBool,
    next: AtomicU64,
}

/// A running server; dropping it closes the listener and every client.
pub struct IdeServer {
    port: u16,
    token: String,
    inner: Arc<Inner>,
}

impl std::fmt::Debug for IdeServer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("IdeServer")
            .field("port", &self.port)
            .finish_non_exhaustive()
    }
}

impl IdeServer {
    /// The port the server listens on.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// The token a client presents, which the lock file holds.
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    /// The clients connected now.
    #[must_use]
    pub fn connections(&self) -> Vec<ConnectionId> {
        self.inner
            .clients
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .keys()
            .copied()
            .collect()
    }

    /// Queues `message` for `connection`; false when that client has left.
    pub fn notify(&self, connection: ConnectionId, message: String) -> bool {
        self.inner
            .clients
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&connection)
            .is_some_and(|client| client.send(Out::Text(message)).is_ok())
    }
}

impl Drop for IdeServer {
    fn drop(&mut self) {
        self.inner.closed.store(true, Ordering::SeqCst);
        for client in self
            .inner
            .clients
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
        {
            if client.send(Out::Close).is_err() {
                log::debug!("claude ide: a client left before the server closed");
            }
        }
        // The accept loop wakes on a connection and then sees the server closed.
        if let Err(error) = TcpStream::connect(("127.0.0.1", self.port)) {
            log::debug!("claude ide: waking the listener on {}: {error}", self.port);
        }
    }
}

/// Binds a loopback port, mints the token, and serves clients on threads of their own.
///
/// # Errors
///
/// When the port cannot be bound, a thread cannot start, or OS entropy is unavailable.
pub fn spawn_ide(judge: Judge, handler: IdeHandler) -> std::io::Result<IdeServer> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    let token = crate::mint_secret(read_entropy()).map_err(std::io::Error::other)?;
    let inner = Arc::new(Inner::default());
    let server = Served {
        port,
        token: token.clone(),
        inner: Arc::clone(&inner),
        judge,
        handler,
    };
    let _accepting = std::thread::Builder::new()
        .name("marley-ide-server".into())
        .spawn(move || accept_loop(&listener, &server))?;
    Ok(IdeServer { port, token, inner })
}

/// What each client's thread needs.
#[derive(Clone)]
struct Served {
    port: u16,
    token: String,
    inner: Arc<Inner>,
    judge: Judge,
    handler: IdeHandler,
}

fn accept_loop(listener: &TcpListener, server: &Served) {
    for stream in listener.incoming() {
        if server.inner.closed.load(Ordering::SeqCst) {
            break;
        }
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(error) => {
                log::debug!("claude ide: accept: {error}");
                continue;
            }
        };
        let held = server
            .inner
            .clients
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len();
        if held >= MAX_CONNECTIONS {
            if let Err(error) = write_status(&mut stream, 503, "Service Unavailable") {
                log::debug!("claude ide: refusing a ninth client: {error}");
            }
            continue;
        }
        let server = server.clone();
        let spawned = std::thread::Builder::new()
            .name("marley-ide-conn".into())
            .spawn(move || {
                if let Err(error) = serve(stream, &server) {
                    log::debug!("claude ide: a client ended in an IO error: {error}");
                }
            });
        if let Err(error) = spawned {
            log::warn!("claude ide: no thread for a client: {error}");
        }
    }
}

/// The upgrade request, read through the HTTP transport's bounds, with any bytes read past it.
fn read_upgrade(stream: &TcpStream) -> Result<Option<(Upgrade, Vec<u8>)>, RequestError> {
    let mut reader = BufReader::new(Deadlined {
        stream: stream.try_clone()?,
        deadline: Instant::now() + REQUEST_READ_TIMEOUT,
    });
    let mut upgrade = Upgrade::default();
    let mut line = String::new();
    if read_bounded_line(&mut reader, &mut line)? == 0 {
        return Ok(None);
    }
    upgrade.request_line(&line);
    let mut header_lines = 0;
    loop {
        line.clear();
        if read_bounded_line(&mut reader, &mut line)? == 0 {
            return Ok(None);
        }
        let header = line.trim_end();
        if header.is_empty() {
            break;
        }
        header_lines += 1;
        if header_lines > MAX_HEADER_LINES {
            return Err(RequestError::TooLarge);
        }
        if let Some((name, value)) = header.split_once(':') {
            upgrade.header(name, value);
        }
    }
    let rest = reader.buffer().to_vec();
    Ok(Some((upgrade, rest)))
}

/// One client: the upgrade checked and answered, then its messages until it leaves.
fn serve(mut stream: TcpStream, server: &Served) -> std::io::Result<()> {
    let (upgrade, rest) = match read_upgrade(&stream) {
        Ok(Some(read)) => read,
        Ok(None) => return write_status(&mut stream, 400, "Bad Request"),
        Err(RequestError::TooLarge) => {
            return write_status_then_drain(&mut stream, 431, "Request Header Fields Too Large");
        }
        Err(RequestError::Io(error)) => return Err(error),
    };
    let admitted = match ide::admit(&upgrade, &server.token, server.port) {
        Ok(admitted) => admitted,
        Err(refusal) => {
            let (code, reason) = refusal.status();
            return write_status(&mut stream, code, reason);
        }
    };
    let protocol = if admitted.subprotocol {
        format!("Sec-WebSocket-Protocol: {}\r\n", ide::SUBPROTOCOL)
    } else {
        String::new()
    };
    write!(
        stream,
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\
         Sec-WebSocket-Accept: {}\r\n{protocol}\r\n",
        tungstenite::handshake::derive_accept_key(admitted.key.as_bytes())
    )?;
    stream.set_read_timeout(Some(READ_SLICE))?;
    stream.set_write_timeout(Some(WRITE_TIMEOUT))?;
    let config = WebSocketConfig::default()
        .max_message_size(Some(MAX_MESSAGE_BYTES))
        .max_frame_size(Some(MAX_MESSAGE_BYTES));
    let socket = WebSocket::from_partially_read(stream, rest, Role::Server, Some(config));
    let connection = server.inner.next.fetch_add(1, Ordering::SeqCst);
    let (queue, queued) = channel();
    let _previous = server
        .inner
        .clients
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(connection, queue);
    (server.handler)(IdeMessage::Connected {
        connection,
        version: admitted.version.clone(),
    });
    let mut client = Client {
        connection,
        socket,
        queued,
        version: admitted.version,
        pending: Vec::new(),
        last_ping: Instant::now(),
        last_pong: Instant::now(),
    };
    client.run(server);
    let _gone = server
        .inner
        .clients
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .remove(&connection);
    (server.handler)(IdeMessage::Closed { connection });
    Ok(())
}

/// A tool call the app has not answered yet.
struct Pending {
    id: Value,
    answered: Receiver<Result<String, String>>,
    deadline: Instant,
}

/// One client's state on its thread.
struct Client {
    connection: ConnectionId,
    socket: WebSocket<TcpStream>,
    queued: Receiver<Out>,
    version: Option<String>,
    pending: Vec<Pending>,
    last_ping: Instant,
    last_pong: Instant,
}

impl Client {
    /// Reads and writes until the client leaves or the server closes.
    fn run(&mut self, server: &Served) {
        loop {
            if !self.write_queued() || !self.write_answers() || !self.keep_alive() {
                return;
            }
            match self.socket.read() {
                Ok(Message::Text(text)) => {
                    if !self.take(text.as_str(), server) {
                        return;
                    }
                }
                Ok(Message::Pong(_)) => self.last_pong = Instant::now(),
                Ok(_) => {}
                Err(tungstenite::Error::Io(error))
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) => {}
                Err(tungstenite::Error::ConnectionClosed | tungstenite::Error::AlreadyClosed) => {
                    return;
                }
                Err(error) => {
                    log::debug!("claude ide: client {}: {error}", self.connection);
                    return;
                }
            }
        }
    }

    /// Answers one message; false when the client is gone.
    fn take(&mut self, text: &str, server: &Served) -> bool {
        let parts = (server.judge)(self.version.as_deref());
        match ide::step(text, parts) {
            Step::Reply(reply) => self.send(reply),
            Step::Initialized { reply, version } => {
                if version.is_some() {
                    self.version = version;
                    (server.handler)(IdeMessage::Connected {
                        connection: self.connection,
                        version: self.version.clone(),
                    });
                }
                self.send(reply)
            }
            Step::Call { id, tool } => {
                let (answer, answered) = sync_channel(1);
                (server.handler)(IdeMessage::Call(IdeCall {
                    connection: self.connection,
                    tool,
                    answer,
                }));
                self.pending.push(Pending {
                    id,
                    answered,
                    deadline: Instant::now() + CALL_TIMEOUT,
                });
                true
            }
            Step::Identified { pid } => {
                (server.handler)(IdeMessage::Identified {
                    connection: self.connection,
                    pid,
                });
                true
            }
            Step::Nothing => true,
        }
    }

    /// Writes what the app queued; false when the client is gone or the server closed.
    fn write_queued(&mut self) -> bool {
        loop {
            match self.queued.try_recv() {
                Ok(Out::Text(text)) => {
                    if !self.send(text) {
                        return false;
                    }
                }
                Ok(Out::Close) | Err(TryRecvError::Disconnected) => {
                    if let Err(error) = self.socket.close(None) {
                        log::debug!("claude ide: closing client {}: {error}", self.connection);
                    }
                    return false;
                }
                Err(TryRecvError::Empty) => return true,
            }
        }
    }

    /// Writes the replies of the tool calls the app answered or let run out of time.
    fn write_answers(&mut self) -> bool {
        let now = Instant::now();
        let mut replies = Vec::new();
        self.pending.retain(|call| match call.answered.try_recv() {
            Ok(answer) => {
                replies.push(ide::tool_answer(&call.id, answer));
                false
            }
            Err(TryRecvError::Disconnected) => {
                replies.push(ide::tool_answer(
                    &call.id,
                    Err("Marley closed before it answered".to_string()),
                ));
                false
            }
            Err(TryRecvError::Empty) if now >= call.deadline => {
                replies.push(ide::tool_answer(
                    &call.id,
                    Err("Marley did not answer in time".to_string()),
                ));
                false
            }
            Err(TryRecvError::Empty) => true,
        });
        replies.into_iter().all(|reply| self.send(reply))
    }

    /// Pings on schedule, and gives up on a client that missed two pongs.
    fn keep_alive(&mut self) -> bool {
        if self.last_ping.elapsed() < PING_EVERY {
            return true;
        }
        if self.last_pong.elapsed() > PING_EVERY * 2 + READ_SLICE {
            log::debug!("claude ide: client {} missed two pings", self.connection);
            return false;
        }
        self.last_ping = Instant::now();
        self.write(Message::Ping(tungstenite::Bytes::new()))
    }

    fn send(&mut self, text: String) -> bool {
        self.write(Message::text(text))
    }

    fn write(&mut self, message: Message) -> bool {
        match self.socket.send(message) {
            Ok(()) => true,
            Err(error) => {
                log::debug!("claude ide: writing to client {}: {error}", self.connection);
                false
            }
        }
    }
}
