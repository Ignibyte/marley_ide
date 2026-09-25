//! The transport shim (D-OPEN-SDK / D-OPEN-LISTEN) — the loopback Streamable-HTTP listener, the
//! per-connection threads, the standing SSE stream, and the discovery-file writer.
//!
//! Every decision it makes — parse a request, run the auth guards, route, build a
//! response/notification — is delegated to the PURE core (`handle_message`, `auth`, `jsonrpc`). The
//! plumbing itself is proven by the tests at the foot of this file, which drive a real server over
//! a loopback socket (the fork's mutation gate has no exclusions). Coverage still lists this file
//! as ACCEPTED-UNTESTABLE for the IO-error arms a loopback peer cannot provoke.
//!
//! Security (D1/D9): binds `127.0.0.1:0` ONLY (never 0.0.0.0), validates `Origin`, requires the per-boot
//! bearer on every request, and NEVER logs the bearer.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc::{RecvTimeoutError, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use marley_fleet::FleetSnapshot;

use crate::permission::GrantTable;
use crate::session::{SessionDecision, SessionRegistry, session_gate};
use crate::{
    APP_CALL_TIMEOUT_SECONDS, AppCall, AppCaller, AppOutcome, Effect, Outgoing, PendingCall,
    RequestCtx, Subscriptions, auth, deferred_response, handle_message, mint_secret, parse_request,
    snapshot_changed,
};

/// The shared server state the app updates each pump tick: the current snapshot, the permission
/// grants, and the `(session-id, pane-handle)` surface index.
///
/// Guarded by a `Mutex`; a `Condvar` wakes the SSE threads when the snapshot version bumps.
#[derive(Debug, Default)]
pub struct ServerData {
    /// The current fleet snapshot the read tools + resource serve.
    pub snapshot: FleetSnapshot,
    /// The permission grants (deserialized from #371 settings; fixture in tests).
    pub grants: GrantTable,
    /// The app-built id→pane-handle index for `surface_to_human`.
    pub surface_index: Vec<(String, u64)>,
    /// Monotonic snapshot version — bumped on every change so SSE threads emit exactly once per change.
    pub version: u64,
    /// The live `Mcp-Session-Id`s (#375). Its `Debug` redacts the ids (they are secrets-adjacent).
    pub sessions: SessionRegistry,
}

/// A shared, condvar-signalled `ServerData`.
pub type Shared = Arc<(Mutex<ServerData>, Condvar)>;

/// A handle to a running server (its loopback URL + the per-boot bearer). The bearer's `Debug` is
/// redacted so it can never leak into a log.
pub struct ServerHandle {
    url: String,
    bearer: String,
}

impl std::fmt::Debug for ServerHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerHandle")
            .field("url", &self.url)
            .field("bearer", &"***")
            .finish()
    }
}

impl ServerHandle {
    /// The server's loopback URL.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// The per-boot bearer (for the discovery file only — never log it).
    #[must_use]
    pub fn bearer(&self) -> &str {
        &self.bearer
    }
}

/// Epoch-millis from the system clock — the injected `now` for the #379 session-TTL gate. Ambient
/// authority confined to this masked file; the pure `session` decisions take the resulting `u64`.
fn now_epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

/// Read 16 bytes of OS CSPRNG from `/dev/urandom` (#375, D1) — the bearer + every session id are minted
/// from this. On macOS `/dev/urandom` is the CSPRNG and cannot block. `None` on any read failure → the
/// caller refuses to start (never a weaker fallback — D2). Masked (raw file IO).
fn read_entropy() -> Option<[u8; 16]> {
    let mut buf = [0u8; 16];
    std::fs::File::open("/dev/urandom")
        .ok()?
        .read_exact(&mut buf)
        .ok()?;
    Some(buf)
}

/// Bind a loopback listener on an OS-assigned port and start the accept loop on a background thread.
///
/// Returns the [`ServerHandle`] (url + a CSPRNG bearer) for the discovery file. The `shared` state is read
/// on each request; `effects` carries `surface_to_human` focus requests to the UI thread, and `caller`
/// hands the app the tool calls it answers (#491). Refuses to start (typed `io::Error`, no weaker
/// fallback — D2) when OS entropy is unavailable.
///
/// # Errors
///
/// Any IO error binding the listener or starting its thread, and an `io::Error` wrapping
/// [`EntropyError`](crate::EntropyError) when OS entropy is unavailable.
pub fn spawn(
    shared: Shared,
    effects: Sender<Effect>,
    caller: AppCaller,
) -> std::io::Result<ServerHandle> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let bearer = mint_secret(read_entropy()).map_err(std::io::Error::other)?;
    let url = format!("http://{addr}/mcp");
    let bearer_for_thread = bearer.clone();
    // The accept loop runs for the life of the process; dropping its handle detaches it.
    drop(
        std::thread::Builder::new()
            .name("marley-mcp-server".into())
            .spawn(move || {
                accept_loop(&listener, &shared, &effects, &caller, &bearer_for_thread);
            })?,
    );
    Ok(ServerHandle { url, bearer })
}

/// Accept connections, one thread per connection (blocking IO — the search/syntax-worker idiom).
fn accept_loop(
    listener: &TcpListener,
    shared: &Shared,
    effects: &Sender<Effect>,
    caller: &AppCaller,
    bearer: &str,
) {
    for stream in listener.incoming().flatten() {
        let shared = Arc::clone(shared);
        let effects = effects.clone();
        let caller = Arc::clone(caller);
        let bearer = bearer.to_string();
        let spawned = std::thread::Builder::new()
            .name("marley-mcp-conn".into())
            .spawn(move || {
                if let Err(error) = serve_connection(stream, &shared, &effects, &caller, &bearer) {
                    // Mostly a client that hung up mid-exchange; the connection is over either way.
                    log::debug!("marley_mcp: a connection ended in an IO error: {error}");
                }
            });
        if let Err(error) = spawned {
            // The OS refused a thread: this client gets no reply, and the server keeps accepting.
            log::warn!("marley_mcp: no thread for a connection: {error}");
        }
    }
}

/// Serve one connection: read the HTTP request, run the auth guards, then either answer a POST (via
/// `handle_message`) or hold a GET open as the standing SSE notification stream.
fn serve_connection(
    mut stream: std::net::TcpStream,
    shared: &Shared,
    effects: &Sender<Effect>,
    caller: &AppCaller,
    bearer: &str,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let Some(request) = read_http_request(&mut reader)? else {
        return write_status(&mut stream, 400, "Bad Request");
    };
    // Auth guards (D1/D9) — refuse BEFORE any dispatch. Order: cap (in read_http_request) → Origin →
    // bearer → session → dispatch (#375 REQ-012 — an unauthenticated request never touches the registry).
    if !auth::origin_allowed(request.origin.as_deref())
        || !auth::bearer_ok(request.bearer.as_deref(), bearer)
    {
        return write_status(&mut stream, 403, "Forbidden");
    }
    // Session gate (#375, MCP §Session Management) — POST-auth. Decide + mutate the registry under ONE
    // lock, then release before touching the socket. An `initialize` mints + assigns a fresh id (echoed on
    // the response); a valid session proceeds; a DELETE terminates; a missing/unknown id is refused.
    let is_initialize = parse_request(&request.body).is_ok_and(|req| req.method == "initialize");
    let session_id = {
        let (data, _cv) = &**shared;
        let mut guard = data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // One timeline value per request (#379): the gate's sweep and the Initialize arm's assign
        // stamp share the same `now`.
        let now = now_epoch_ms();
        match session_gate(
            &request.method,
            is_initialize,
            request.session.as_deref(),
            &mut guard.sessions,
            now,
        ) {
            SessionDecision::Reject(code) => {
                drop(guard);
                return write_status(&mut stream, code, status_reason(code));
            }
            SessionDecision::Terminate => {
                if let Some(id) = request.session.as_deref() {
                    let _was_live = guard.sessions.terminate(id);
                }
                drop(guard);
                return write_status(&mut stream, 200, "OK");
            }
            SessionDecision::Initialize => {
                let Some(id) = mint_secret(read_entropy()).ok() else {
                    drop(guard);
                    return write_status(&mut stream, 500, "Internal Server Error");
                };
                if guard.sessions.assign(id.clone(), now).is_err() {
                    drop(guard);
                    return write_status(&mut stream, 503, "Service Unavailable");
                    // at SESSION_CAP
                }
                Some(id)
            }
            SessionDecision::Proceed => None,
        }
    };
    if request.method == "POST" {
        let handled = {
            let (data, _cv) = &**shared;
            let guard = data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let ctx = RequestCtx {
                snapshot: &guard.snapshot,
                grants: &guard.grants,
                surface_index: &guard.surface_index,
            };
            let mut subs = Subscriptions::default();
            let handled = handle_message(&ctx, &mut subs, &request.body);
            drop(guard);
            handled
        };
        if let Some(effect) = handled.effect
            && let Err(error) = effects.send(effect)
        {
            // The app's receiver is gone, so it is shutting down and has no pane to focus.
            log::debug!("marley_mcp: the app dropped a surface effect: {error}");
        }
        let mut wrote_response = false;
        for message in handled.outgoing {
            let body = match message {
                Outgoing::Response(body) => body,
                // The lock is released by now, so the wait holds up no other connection.
                Outgoing::Deferred(pending) => {
                    let outcome = ask_app(caller, &pending);
                    deferred_response(&pending, outcome)
                }
                Outgoing::Notification(_) => continue,
            };
            // The `Mcp-Session-Id` header rides ONLY the initialize response (session_id is Some only
            // for an Initialize decision); subsequent responses carry no new id.
            write_sse_response(&mut stream, &body, session_id.as_deref())?;
            wrote_response = true;
        }
        // Inspect fix: a notification-only POST (no response body) still gets an HTTP reply — 202
        // Accepted, per MCP Streamable-HTTP — so the client's read never hangs waiting for a body.
        if !wrote_response {
            write_status(&mut stream, 202, "Accepted")?;
        }
        Ok(())
    } else {
        // GET → the standing SSE stream (reached only for a valid session — the gate refused otherwise).
        // Its session is reclaimed when the stream drops, so a client that reconnects (re-initializing, per
        // the MCP spec + Marley's own #373 pump) does not leak its old slot toward the cap.
        serve_sse_stream(stream, shared, request.session.as_deref())
    }
}

/// Hands `pending` to the app and waits for its answer, up to [`APP_CALL_TIMEOUT_SECONDS`] (#491).
fn ask_app(caller: &AppCaller, pending: &PendingCall) -> AppOutcome {
    let (answer, answered) = std::sync::mpsc::sync_channel(1);
    caller(AppCall::new(
        pending.tool.clone(),
        pending.arguments.clone(),
        answer,
    ));
    match answered.recv_timeout(Duration::from_secs(APP_CALL_TIMEOUT_SECONDS)) {
        Ok(result) => AppOutcome::Answered(result),
        Err(RecvTimeoutError::Timeout) => AppOutcome::TimedOut,
        Err(RecvTimeoutError::Disconnected) => AppOutcome::Unavailable,
    }
}

/// Terminate `session` (if any) in the shared registry — the standing-stream reap on hang-up (#375).
fn reap_session(shared: &Shared, session: Option<&str>) {
    if let Some(id) = session {
        let (data, _cv) = &**shared;
        let mut guard = data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let _was_live = guard.sessions.terminate(id);
    }
}

/// The reason phrase for a session-gate refusal status.
const fn status_reason(code: u16) -> &'static str {
    match code {
        400 => "Bad Request",
        404 => "Not Found",
        _ => "Error",
    }
}

/// Hold a GET connection open as the SSE stream, emitting one `resources/updated` per snapshot change.
///
/// L1 SIMPLIFICATION (inspect note): there is ONE resource, so opening this stream IS the subscription —
/// it pushes the fleet resource unconditionally (`fleet: true`). The per-connection [`Subscriptions`] gate
/// in the pure `handle_message`/`snapshot_changed` pair is the L2-ready model (a second resource + an
/// `Mcp-Session-Id`-keyed store); it is deliberately NOT wired across the POST/GET split here because an
/// unsubscribed client re-reading a self-healing snapshot is harmless (over-notify, never under-notify).
/// `resources/subscribe` over POST returns success as an advisory ack.
fn serve_sse_stream(
    mut stream: std::net::TcpStream,
    shared: &Shared,
    session: Option<&str>,
) -> std::io::Result<()> {
    let (data, condvar) = &**shared;
    // Read the version before the head goes out: a change that lands while the head is being
    // written still gets its push, where reading it after would drop that push.
    let mut last_seen = {
        data.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .version
    };
    if stream
        .write_all(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n",
        )
        .is_err()
    {
        reap_session(shared, session);
        return Ok(());
    }
    let subs = Subscriptions { fleet: true }; // L1: opening the stream = subscribing to the one resource
    loop {
        let guard = data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let guard = condvar
            .wait_while(guard, |d| d.version == last_seen)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        last_seen = guard.version;
        drop(guard);
        for message in snapshot_changed(subs) {
            if let Outgoing::Notification(body) = message
                && stream
                    .write_all(format!("event: message\r\ndata: {body}\r\n\r\n").as_bytes())
                    .is_err()
            {
                reap_session(shared, session); // client hung up → free its session slot
                return Ok(());
            }
        }
    }
}

/// Signal all SSE threads that the snapshot changed (the app calls this after replacing `snapshot`). Bumps
/// the version under the lock and notifies the condvar.
pub fn signal_change(shared: &Shared) {
    let (data, condvar) = &**shared;
    {
        let mut guard = data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.version = guard.version.wrapping_add(1);
    }
    condvar.notify_all();
}

/// A parsed HTTP request (only the parts L1 needs).
struct HttpRequest {
    method: String,
    origin: Option<String>,
    bearer: Option<String>,
    /// The presented `Mcp-Session-Id` (#375), if any.
    session: Option<String>,
    body: String,
}

/// The largest request body the server reads (1 MiB; an MCP request is small).
///
/// Inspect fix (pre-auth DoS): never allocate an attacker-chosen `Content-Length`. A hostile
/// (unauthenticated, loopback) request with a huge length would otherwise `vec![0u8; N]` → an OOM
/// process abort taking down the whole app, so anything over this cap is a bad request.
const MAX_BODY_BYTES: usize = 1 << 20;

/// Read + parse an HTTP/1.1 request: the request line, the headers we care about (`Origin`,
/// `Authorization`, `Content-Length`), and the body.
fn read_http_request(
    reader: &mut BufReader<std::net::TcpStream>,
) -> std::io::Result<Option<HttpRequest>> {
    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(None);
    }
    let method = request_line
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string();
    let (mut origin, mut bearer, mut session, mut content_length) = (None, None, None, 0usize);
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some(value) = header_value(line, "origin") {
            origin = Some(value.to_string());
        } else if let Some(value) = header_value(line, "authorization") {
            bearer = value.strip_prefix("Bearer ").map(str::to_string);
        } else if let Some(value) = header_value(line, "mcp-session-id") {
            session = Some(value.to_string());
        } else if let Some(value) = header_value(line, "content-length") {
            content_length = value.parse().unwrap_or(0);
        }
    }
    if content_length > MAX_BODY_BYTES {
        return Ok(None); // → 400 Bad Request; no allocation
    }
    // An empty body reads zero bytes, so no length guard is needed around the read.
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body)?;
    Ok(Some(HttpRequest {
        method,
        origin,
        bearer,
        session,
        body: String::from_utf8_lossy(&body).into_owned(),
    }))
}

/// Case-insensitive header match: returns the trimmed value if `line` is `Name: value` for `name`.
fn header_value<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let (key, value) = line.split_once(':')?;
    if key.trim().eq_ignore_ascii_case(name) {
        Some(value.trim())
    } else {
        None
    }
}

/// Write a single JSON-RPC response as a one-shot SSE `application/json`-over-`text/event-stream` reply
/// (the standard Streamable-HTTP shape an MCP client expects: a `data:` line per message).
fn write_sse_response(
    stream: &mut std::net::TcpStream,
    body: &str,
    session_id: Option<&str>,
) -> std::io::Result<()> {
    let payload = format!("event: message\r\ndata: {body}\r\n\r\n");
    // The `Mcp-Session-Id` header is emitted only on the initialize response (#375); other responses omit it.
    let session_header =
        session_id.map_or_else(String::new, |id| format!("Mcp-Session-Id: {id}\r\n"));
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\n{session_header}Content-Length: {}\r\nConnection: close\r\n\r\n{}",
        payload.len(),
        payload
    )
}

/// Write a bare HTTP status line (for a refusal / bad request).
fn write_status(stream: &mut std::net::TcpStream, code: u16, reason: &str) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {code} {reason}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    )
}

/// Build the discovery-file JSON `{url, bearer}` a manager's `.mcp.json` reads to reach this server.
///
/// (Writing it to disk is the app glue's job — kept here as a pure builder so its shape is testable if
/// ever needed; the bearer lands ONLY in this gitignored runtime file, never a log.)
#[must_use]
pub fn discovery_json(url: &str, bearer: &str) -> String {
    serde_json::json!({ "type": "http", "url": url, "headers": { "Authorization": format!("Bearer {bearer}") } })
        .to_string()
}

#[cfg(test)]
mod tests {
    //! The transport on a real loopback socket: every guard, status line, header and stream push
    //! is asserted on the bytes a client receives, so each line of the shim is covered by behavior
    //! rather than by a mask.

    use super::*;
    use crate::permission::GrantTable;
    use serde_json::Value;
    use std::fmt::Write as _;
    use std::net::{Shutdown, TcpStream};
    use std::sync::mpsc::{self, Receiver};
    use std::time::{Duration, Instant};

    const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    const TOOLS_LIST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
    const INITIALIZED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    const REPLY_TIMEOUT: Duration = Duration::from_secs(5);
    /// How long a stream must stay silent to count as "nothing more is coming". Only the
    /// negative checks lean on it; data a test expects is awaited up to `REPLY_TIMEOUT`.
    const QUIET_WINDOW: Duration = Duration::from_millis(300);
    const MAX_READ: usize = 4 << 20;
    /// Ends the response head and every SSE event the server writes.
    const MESSAGE_END: &[u8] = b"\r\n\r\n";

    struct Server {
        handle: ServerHandle,
        shared: Shared,
        effects: Receiver<Effect>,
    }

    fn start() -> Server {
        let shared: Shared = Arc::new((Mutex::new(ServerData::default()), Condvar::new()));
        let (sender, effects) = mpsc::channel();
        // No app answers here: a deferred call's answer channel closes as the call is dropped.
        let caller: AppCaller = Arc::new(drop);
        let handle =
            spawn(Arc::clone(&shared), sender, caller).expect("the loopback server starts");
        Server {
            handle,
            shared,
            effects,
        }
    }

    impl Server {
        fn address(&self) -> String {
            self.handle
                .url()
                .trim_start_matches("http://")
                .trim_end_matches("/mcp")
                .to_string()
        }

        fn connect(&self) -> TcpStream {
            let stream = TcpStream::connect(self.address()).expect("connect to the server");
            stream
                .set_read_timeout(Some(REPLY_TIMEOUT))
                .expect("set the read timeout");
            stream
        }

        /// Send one raw request and read until the server closes the connection.
        fn exchange(&self, request: &[u8]) -> String {
            let mut stream = self.connect();
            stream.write_all(request).expect("write the request");
            read_available(&mut stream)
        }

        fn request(&self, method: &str, headers: &[(&str, &str)], body: &str) -> String {
            let mut raw = format!("{method} /mcp HTTP/1.1\r\nHost: localhost\r\n");
            for (name, value) in headers {
                write!(raw, "{name}: {value}\r\n").expect("formatting into a String");
            }
            write!(raw, "Content-Length: {}\r\n\r\n{body}", body.len())
                .expect("formatting into a String");
            self.exchange(raw.as_bytes())
        }

        fn authorized(&self) -> String {
            format!("Bearer {}", self.handle.bearer())
        }

        fn post(&self, session: Option<&str>, body: &str) -> String {
            let authorization = self.authorized();
            let mut headers = vec![("Authorization", authorization.as_str())];
            if let Some(session) = session {
                headers.push(("Mcp-Session-Id", session));
            }
            self.request("POST", &headers, body)
        }

        fn initialize(&self) -> String {
            let reply = self.post(None, INITIALIZE);
            assert_eq!(status_line(&reply), "HTTP/1.1 200 OK", "{reply}");
            header(&reply, "mcp-session-id").expect("initialize echoes a session id")
        }

        fn session_is_live(&self, session: &str) -> bool {
            let (data, _) = &*self.shared;
            data.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .sessions
                .validate(session)
        }
    }

    /// Read until the peer closes or goes quiet for the stream's read timeout. A peer that never
    /// stops sending (a stream pushing without a change) is cut off by size and time, so the
    /// caller's assertion fails fast instead of the test hanging.
    fn read_available(stream: &mut TcpStream) -> String {
        let started = Instant::now();
        let mut received = Vec::new();
        let mut chunk = [0u8; 4096];
        while received.len() < MAX_READ && started.elapsed() < REPLY_TIMEOUT {
            match stream.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(count) => received.extend_from_slice(&chunk[..count]),
            }
        }
        String::from_utf8_lossy(&received).into_owned()
    }

    /// Read until a whole message has arrived (the text holds `MESSAGE_END`) or `REPLY_TIMEOUT`
    /// passes. A read that times out on the stream's short quiet window is retried, so a slow
    /// server fails the deadline, never the first 300 ms.
    fn read_message(stream: &mut TcpStream) -> String {
        let started = Instant::now();
        let mut received = Vec::new();
        let mut chunk = [0u8; 4096];
        while received.len() < MAX_READ && started.elapsed() < REPLY_TIMEOUT {
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(count) => {
                    let searched_from = received.len().saturating_sub(MESSAGE_END.len());
                    received.extend_from_slice(&chunk[..count]);
                    if received[searched_from..]
                        .windows(MESSAGE_END.len())
                        .any(|window| window == MESSAGE_END)
                    {
                        break;
                    }
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) => {}
                Err(_) => break,
            }
        }
        String::from_utf8_lossy(&received).into_owned()
    }

    fn status_line(reply: &str) -> &str {
        reply.split("\r\n").next().unwrap_or_default()
    }

    fn header(reply: &str, name: &str) -> Option<String> {
        let head = reply.split("\r\n\r\n").next()?;
        head.split("\r\n")
            .skip(1)
            .find_map(|line| header_value(line, name))
            .map(str::to_string)
    }

    /// The JSON payload of the reply's single SSE `data:` line.
    fn sse_json(reply: &str) -> Value {
        let data = reply
            .lines()
            .find_map(|line| line.strip_prefix("data: "))
            .expect("the reply carries an SSE data line");
        serde_json::from_str(data).expect("the data line is JSON")
    }

    #[test]
    fn the_handle_names_a_loopback_url_and_redacts_its_bearer() {
        let server = start();
        let url = server.handle.url();
        assert!(url.starts_with("http://127.0.0.1:"), "{url}");
        assert!(url.ends_with("/mcp"), "{url}");
        let bearer = server.handle.bearer();
        assert_eq!(bearer.len(), 32);
        assert!(bearer.chars().all(|c| c.is_ascii_hexdigit()), "{bearer}");

        let debug = format!("{:?}", server.handle);
        assert!(debug.starts_with("ServerHandle"), "{debug}");
        assert!(debug.contains(url), "{debug}");
        assert!(debug.contains("***"), "{debug}");
        assert!(
            !debug.contains(bearer),
            "the bearer must never reach a log: {debug}"
        );
    }

    #[test]
    fn the_clock_and_the_entropy_are_the_real_ones() {
        // 2023-11-14 in epoch millis; a constant clock would read 0 or 1.
        assert!(now_epoch_ms() > 1_700_000_000_000);
        let first = read_entropy().expect("OS entropy is readable");
        let second = read_entropy().expect("OS entropy is readable");
        assert_ne!(first, second);
        assert_ne!(first, [0; 16]);
        assert_ne!(first, [1; 16]);
        assert_ne!(start().handle.bearer(), start().handle.bearer());
    }

    #[test]
    fn initialize_assigns_and_echoes_a_session() {
        let server = start();
        let reply = server.post(None, INITIALIZE);
        assert_eq!(status_line(&reply), "HTTP/1.1 200 OK", "{reply}");
        assert_eq!(
            header(&reply, "content-type").as_deref(),
            Some("text/event-stream")
        );
        let session = header(&reply, "mcp-session-id").expect("the session header");
        assert_eq!(session.len(), 32);
        assert!(server.session_is_live(&session));
        let body = sse_json(&reply);
        assert_eq!(body["id"], 1);
        assert!(body["result"]["protocolVersion"].is_string(), "{body}");
    }

    #[test]
    fn a_missing_or_wrong_bearer_or_a_foreign_origin_is_forbidden() {
        let server = start();
        let forbidden = "HTTP/1.1 403 Forbidden";
        let no_bearer = server.request("POST", &[], INITIALIZE);
        assert_eq!(status_line(&no_bearer), forbidden);
        let wrong = server.request("POST", &[("Authorization", "Bearer 00")], INITIALIZE);
        assert_eq!(status_line(&wrong), forbidden);
        let authorization = server.authorized();
        let foreign = server.request(
            "POST",
            &[
                ("Authorization", authorization.as_str()),
                ("Origin", "https://attacker.example"),
            ],
            INITIALIZE,
        );
        assert_eq!(status_line(&foreign), forbidden);
        let local = server.request(
            "POST",
            &[
                ("Authorization", authorization.as_str()),
                ("Origin", "http://localhost:1234"),
            ],
            INITIALIZE,
        );
        assert_eq!(status_line(&local), "HTTP/1.1 200 OK", "{local}");
        assert!(header(&foreign, "mcp-session-id").is_none());
    }

    #[test]
    fn a_request_without_a_session_is_bad_and_an_unknown_one_is_not_found() {
        let server = start();
        let missing = server.post(None, TOOLS_LIST);
        assert_eq!(status_line(&missing), "HTTP/1.1 400 Bad Request");
        let unknown = server.post(Some("0123456789abcdef0123456789abcdef"), TOOLS_LIST);
        assert_eq!(status_line(&unknown), "HTTP/1.1 404 Not Found");
    }

    #[test]
    fn a_live_session_gets_exactly_one_answer_and_no_new_session() {
        let server = start();
        let session = server.initialize();
        let reply = server.post(Some(&session), TOOLS_LIST);
        assert_eq!(status_line(&reply), "HTTP/1.1 200 OK", "{reply}");
        assert!(header(&reply, "mcp-session-id").is_none(), "{reply}");
        assert_eq!(reply.matches("HTTP/1.1").count(), 1, "{reply}");
        let body = sse_json(&reply);
        assert_eq!(body["id"], 2);
        assert!(body["result"]["tools"].is_array(), "{body}");
    }

    #[test]
    fn a_notification_only_post_is_accepted() {
        let server = start();
        let session = server.initialize();
        let reply = server.post(Some(&session), INITIALIZED);
        assert_eq!(status_line(&reply), "HTTP/1.1 202 Accepted", "{reply}");
    }

    #[test]
    fn delete_ends_the_session() {
        let server = start();
        let session = server.initialize();
        let authorization = server.authorized();
        let reply = server.request(
            "DELETE",
            &[
                ("Authorization", authorization.as_str()),
                ("Mcp-Session-Id", session.as_str()),
            ],
            "",
        );
        assert_eq!(status_line(&reply), "HTTP/1.1 200 OK", "{reply}");
        assert!(!server.session_is_live(&session));
        let after = server.post(Some(&session), TOOLS_LIST);
        assert_eq!(status_line(&after), "HTTP/1.1 404 Not Found");
    }

    #[test]
    fn a_session_past_the_cap_is_refused() {
        let server = start();
        for _ in 0..crate::SESSION_CAP {
            let _session = server.initialize();
        }
        let reply = server.post(None, INITIALIZE);
        assert_eq!(status_line(&reply), "HTTP/1.1 503 Service Unavailable");
    }

    #[test]
    fn a_granted_surface_call_hands_the_focus_effect_to_the_app() {
        let server = start();
        {
            let (data, _) = &*server.shared;
            let mut data = data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            data.grants = GrantTable::from_classes(["session.write"]);
            data.surface_index = vec![("dev-1/a".to_string(), 7)];
        }
        let session = server.initialize();
        let reply = server.post(
            Some(&session),
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"session_surface_to_human","arguments":{"id":"dev-1/a"}}}"#,
        );
        assert_eq!(status_line(&reply), "HTTP/1.1 200 OK", "{reply}");
        assert_eq!(
            server.effects.recv_timeout(REPLY_TIMEOUT),
            Ok(Effect::SurfacePane(7))
        );
    }

    #[test]
    fn a_body_over_the_cap_is_refused_before_it_is_read() {
        let server = start();
        let authorization = server.authorized();
        let request = format!(
            "POST /mcp HTTP/1.1\r\nAuthorization: {authorization}\r\nContent-Length: {}\r\n\r\n",
            (1usize << 20) + 1
        );
        let reply = server.exchange(request.as_bytes());
        assert_eq!(status_line(&reply), "HTTP/1.1 400 Bad Request", "{reply}");
    }

    #[test]
    fn a_body_of_exactly_the_cap_is_read_whole() {
        let server = start();
        let mut body = INITIALIZE.to_string();
        body.push_str(&" ".repeat((1 << 20) - body.len()));
        let reply = server.post(None, &body);
        assert_eq!(status_line(&reply), "HTTP/1.1 200 OK", "{reply}");
        assert!(header(&reply, "mcp-session-id").is_some());
    }

    #[test]
    fn a_connection_that_sends_nothing_is_a_bad_request() {
        let server = start();
        let mut stream = server.connect();
        stream
            .shutdown(Shutdown::Write)
            .expect("half-close the socket");
        let reply = read_available(&mut stream);
        assert_eq!(status_line(&reply), "HTTP/1.1 400 Bad Request", "{reply}");
    }

    #[test]
    fn the_stream_pushes_one_notification_per_change() {
        let server = start();
        let session = server.initialize();
        let mut stream = server.connect();
        let authorization = server.authorized();
        stream
            .write_all(
                format!(
                    "GET /mcp HTTP/1.1\r\nAuthorization: {authorization}\r\nMcp-Session-Id: {session}\r\n\r\n"
                )
                .as_bytes(),
            )
            .expect("open the stream");
        stream
            .set_read_timeout(Some(QUIET_WINDOW))
            .expect("set the quiet window");
        let head = read_message(&mut stream);
        assert_eq!(status_line(&head), "HTTP/1.1 200 OK", "{head}");
        assert_eq!(
            header(&head, "content-type").as_deref(),
            Some("text/event-stream")
        );
        let before_change = read_available(&mut stream);
        assert!(
            !head.contains("data:") && before_change.is_empty(),
            "no push before a change: {head}{before_change}"
        );

        signal_change(&server.shared);
        let first = read_message(&mut stream);
        assert_eq!(first.matches("data: ").count(), 1, "{first}");
        assert!(first.contains("notifications/resources/updated"), "{first}");

        let quiet = read_available(&mut stream);
        assert!(quiet.is_empty(), "no push without a change: {quiet}");

        signal_change(&server.shared);
        let second = read_message(&mut stream);
        assert_eq!(second.matches("data: ").count(), 1, "{second}");
    }

    #[test]
    fn a_stream_the_client_hangs_up_frees_its_session() {
        let server = start();
        let session = server.initialize();
        {
            let mut stream = server.connect();
            let authorization = server.authorized();
            stream
                .write_all(
                    format!(
                        "GET /mcp HTTP/1.1\r\nAuthorization: {authorization}\r\nMcp-Session-Id: {session}\r\n\r\n"
                    )
                    .as_bytes(),
                )
                .expect("open the stream");
            stream
                .set_read_timeout(Some(QUIET_WINDOW))
                .expect("set the quiet window");
            let head = read_message(&mut stream);
            assert_eq!(status_line(&head), "HTTP/1.1 200 OK", "{head}");
        }
        // The first write after a hang-up can still land in the kernel buffer; keep signalling
        // until the server's write fails and it reaps the session.
        let deadline = Instant::now() + REPLY_TIMEOUT;
        while server.session_is_live(&session) && Instant::now() < deadline {
            signal_change(&server.shared);
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(!server.session_is_live(&session));
    }

    #[test]
    fn signal_change_bumps_the_version() {
        let server = start();
        signal_change(&server.shared);
        signal_change(&server.shared);
        let (data, _) = &*server.shared;
        let version = data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .version;
        assert_eq!(version, 2);
    }

    #[test]
    fn header_value_matches_the_name_case_insensitively_and_trims() {
        assert_eq!(
            header_value("Content-Length: 5", "content-length"),
            Some("5")
        );
        assert_eq!(
            header_value("origin:  http://a  ", "Origin"),
            Some("http://a")
        );
        assert_eq!(header_value("X-Other: y", "origin"), None);
        assert_eq!(header_value("no colon here", "no colon here"), None);
    }

    #[test]
    fn status_reason_names_the_gate_refusals() {
        assert_eq!(status_reason(400), "Bad Request");
        assert_eq!(status_reason(404), "Not Found");
        assert_eq!(status_reason(418), "Error");
    }

    #[test]
    fn discovery_json_carries_the_url_and_the_bearer_header() {
        let json: Value = serde_json::from_str(&discovery_json("http://127.0.0.1:9/mcp", "abc"))
            .expect("discovery JSON parses");
        assert_eq!(json["type"], "http");
        assert_eq!(json["url"], "http://127.0.0.1:9/mcp");
        assert_eq!(json["headers"]["Authorization"], "Bearer abc");
    }
}
