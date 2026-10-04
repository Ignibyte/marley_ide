//! Marley's agent socket (#652): the way an agent's report reaches Marley from inside one of its
//! terminals.
//!
//! A Unix stream socket, mode 0600, in a folder only the user can open: one JSON line in, one line
//! out, a connection per request. The program Marley writes for its terminals (`marley-agent`)
//! sends the line; the peer's credentials say which process sent it, and only a process of the user
//! Marley runs as is heard. What the line means is the app's: the socket hands each request, with
//! the peer's process, to a handler on the connection's own thread, and writes back whatever answer
//! the app gives before the deadline.

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::os::unix::fs::PermissionsExt as _;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::time::Duration;

/// The longest line a request may be.
const MAX_LINE: u64 = 64 * 1024;

/// How long a peer has to send its line.
const READ_TIMEOUT: Duration = Duration::from_secs(2);

/// How long the app has to answer, under the program's 4 s for a report.
const ANSWER_TIMEOUT: Duration = Duration::from_millis(3_500);

/// One request that came in, with the peer's process; [`AgentRequest::answer`] sends the reply.
#[derive(Debug)]
pub struct AgentRequest {
    /// The line the peer sent, without its newline.
    pub line: String,
    /// The peer's process, from its credentials.
    pub pid: u32,
    answer: SyncSender<String>,
}

impl AgentRequest {
    /// Writes `reply` back to the peer, if it still waits.
    pub fn answer(self, reply: String) {
        if self.answer.send(reply).is_err() {
            log::debug!("agent socket: an answer came after its request stopped waiting");
        }
    }
}

/// Takes each request, on its connection's thread.
pub type AgentHandler = Arc<dyn Fn(AgentRequest) + Send + Sync>;

/// The running socket; the file is removed when it is dropped.
#[derive(Debug)]
pub struct AgentSocket {
    path: PathBuf,
}

impl AgentSocket {
    /// The socket's path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for AgentSocket {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_file(&self.path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            log::debug!("agent socket: {} stays: {error}", self.path.display());
        }
    }
}

/// Binds `path`, replacing a socket a Marley left there, makes it the user's alone, and serves it
/// on a thread of its own, each request handed to `handler`.
///
/// # Errors
///
/// When the path cannot be bound or its mode set.
pub fn spawn(path: &Path, handler: AgentHandler) -> std::io::Result<AgentSocket> {
    if let Err(error) = std::fs::remove_file(path)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        return Err(error);
    }
    let listener = UnixListener::bind(path)?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    let uid = rustix::process::getuid();
    let _accepting = std::thread::Builder::new()
        .name("marley-agent-socket".to_string())
        .spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let handler = Arc::clone(&handler);
                        let spawned = std::thread::Builder::new()
                            .name("marley-agent-conn".to_string())
                            .spawn(move || serve(stream, uid, &handler));
                        if let Err(error) = spawned {
                            log::warn!("agent socket: a connection's thread: {error}");
                        }
                    }
                    Err(error) => log::debug!("agent socket: accept: {error}"),
                }
            }
        })?;
    Ok(AgentSocket {
        path: path.to_path_buf(),
    })
}

/// One connection: the peer checked, one line read, the app's answer written.
fn serve(stream: UnixStream, uid: rustix::process::Uid, handler: &AgentHandler) {
    let peer = match rustix::net::sockopt::socket_peercred(&stream) {
        Ok(peer) => peer,
        Err(error) => {
            log::debug!("agent socket: the peer's credentials: {error}");
            return;
        }
    };
    if peer.uid != uid {
        log::warn!("agent socket: refused a peer of another user");
        return;
    }
    let pid = peer.pid.as_raw_nonzero().get().unsigned_abs();
    if let Err(error) = stream.set_read_timeout(Some(READ_TIMEOUT)) {
        log::debug!("agent socket: {error}");
        return;
    }
    let mut line = String::new();
    let read = match stream.try_clone() {
        Ok(reader) => BufReader::new(reader.take(MAX_LINE)).read_line(&mut line),
        Err(error) => Err(error),
    };
    if let Err(error) = read {
        log::debug!("agent socket: reading a request: {error}");
        return;
    }
    let (answer, answered): (SyncSender<String>, Receiver<String>) = sync_channel(1);
    handler(AgentRequest {
        line: line.trim_end().to_string(),
        pid,
        answer,
    });
    let reply = answered.recv_timeout(ANSWER_TIMEOUT).unwrap_or_else(|_| {
        r#"{"refused":"marley_not_running","reason":"Marley did not answer in time"}"#.to_string()
    });
    let mut stream = stream;
    if let Err(error) = stream.write_all(format!("{reply}\n").as_bytes()) {
        log::debug!("agent socket: writing an answer: {error}");
    }
}
