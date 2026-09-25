//! A client of CDP, Chromium's debugging protocol, over a WebSocket on loopback.
//!
//! One connection to the browser's endpoint carries every session: a request goes out with an
//! id and, for a page, the session it is for (CDP's flat sessions); its response comes back to
//! the call waiting on that id, and events go to one channel, tagged with their session. When
//! the socket ends, every waiting call fails with the reason and the event channel closes, so
//! a browser that went away reads as such rather than as silence.

use std::collections::HashMap;
use std::fmt;
use std::pin::pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use anyhow::Context as _;
use async_tungstenite::tungstenite::Message;
use futures::StreamExt as _;
use futures::channel::{mpsc, oneshot};
use futures::future::{self, Either};
use gpui::{BackgroundExecutor, Task};
use parking_lot::Mutex;
use serde::Deserialize;
use serde_json::{Map, Value};

/// How long a call waits for its response.
const CALL_TIMEOUT: Duration = Duration::from_secs(15);

/// Why a call failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CdpError {
    /// The connection is gone, for the reason given.
    #[error("the connection to the browser closed: {0}")]
    Closed(String),
    /// The browser answered the call with an error.
    #[error("{method} failed: {message}")]
    Protocol {
        /// The call's method.
        method: String,
        /// The browser's message.
        message: String,
    },
    /// No answer came in time.
    #[error("{0} got no answer within 15 seconds")]
    Timeout(String),
    /// The answer lacked what the call needed.
    #[error("{0}")]
    Unexpected(String),
}

/// An event the browser sent: its method, its parameters, and the session it belongs to (none
/// for the browser's own events).
#[derive(Debug)]
pub struct Event {
    /// The event's name, `Page.screencastFrame` say.
    pub method: String,
    /// Its parameters.
    pub params: Value,
    /// The flat session it came from.
    pub session_id: Option<String>,
}

/// The calls waiting on their answers, until the connection ends.
enum Waiting {
    Open(HashMap<u64, (String, oneshot::Sender<Result<Value, CdpError>>)>),
    Closed(String),
}

/// A connection to the browser. Clones share it, and the socket closes when the last drops.
#[derive(Clone)]
pub struct Connection {
    inner: Arc<Inner>,
}

struct Inner {
    next_id: AtomicU64,
    waiting: Arc<Mutex<Waiting>>,
    outgoing: mpsc::UnboundedSender<String>,
    executor: BackgroundExecutor,
    _reader: Task<()>,
    _writer: Task<()>,
}

impl fmt::Debug for Connection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Connection").finish_non_exhaustive()
    }
}

/// Connects to the browser's endpoint `path` on 127.0.0.1 at `port`, and returns the
/// connection and the channel its events arrive on, which ends when the connection does.
///
/// # Errors
///
/// When the TCP connection or the WebSocket handshake fails.
pub async fn connect(
    port: u16,
    path: &str,
    executor: BackgroundExecutor,
) -> anyhow::Result<(Connection, mpsc::UnboundedReceiver<Event>)> {
    let stream = smol::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .with_context(|| format!("connecting to the browser on port {port}"))?;
    let (socket, _response) =
        async_tungstenite::client_async(format!("ws://127.0.0.1:{port}{path}"), stream)
            .await
            .context("opening the browser's DevTools socket")?;
    let (mut sink, mut source) = socket.split();
    let (outgoing, mut outgoing_rx) = mpsc::unbounded::<String>();
    let (events, events_rx) = mpsc::unbounded();
    let waiting = Arc::new(Mutex::new(Waiting::Open(HashMap::new())));

    let writer = executor.spawn(async move {
        while let Some(text) = outgoing_rx.next().await {
            if let Err(error) = sink.send(Message::Text(text.into())).await {
                log::warn!("browser: writing to the DevTools socket failed: {error}");
                break;
            }
        }
    });
    let reader = executor.spawn({
        let waiting = Arc::clone(&waiting);
        async move {
            let reason = loop {
                match source.next().await {
                    Some(Ok(Message::Text(text))) => dispatch(text.as_str(), &waiting, &events),
                    Some(Ok(Message::Close(_))) => break "the browser closed it".to_string(),
                    Some(Ok(_)) => {}
                    Some(Err(error)) => break error.to_string(),
                    None => break "the socket ended".to_string(),
                }
            };
            let previous = std::mem::replace(&mut *waiting.lock(), Waiting::Closed(reason.clone()));
            let waiters = match previous {
                Waiting::Open(waiters) => waiters,
                Waiting::Closed(_) => HashMap::new(),
            };
            for (_, (_, waiter)) in waiters {
                waiter.send(Err(CdpError::Closed(reason.clone()))).ok();
            }
        }
    });

    let connection = Connection {
        inner: Arc::new(Inner {
            next_id: AtomicU64::new(1),
            waiting,
            outgoing,
            executor,
            _reader: reader,
            _writer: writer,
        }),
    };
    Ok((connection, events_rx))
}

/// One message from the browser, a response or an event.
#[derive(Deserialize)]
struct Incoming {
    id: Option<u64>,
    method: Option<String>,
    #[serde(default)]
    params: Value,
    result: Option<Value>,
    error: Option<ErrorBody>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
}

#[derive(Deserialize)]
struct ErrorBody {
    message: String,
}

/// Hands a response to the call waiting on its id, and an event to the event channel.
fn dispatch(text: &str, waiting: &Mutex<Waiting>, events: &mpsc::UnboundedSender<Event>) {
    let incoming = match serde_json::from_str::<Incoming>(text) {
        Ok(incoming) => incoming,
        Err(error) => {
            log::warn!("browser: a DevTools message that is not JSON-RPC: {error}");
            return;
        }
    };
    if let Some(id) = incoming.id {
        let waiter = match &mut *waiting.lock() {
            Waiting::Open(waiters) => waiters.remove(&id),
            Waiting::Closed(_) => None,
        };
        if let Some((method, waiter)) = waiter {
            let result = match incoming.error {
                Some(error) => Err(CdpError::Protocol {
                    method,
                    message: error.message,
                }),
                None => Ok(incoming.result.unwrap_or(Value::Null)),
            };
            waiter.send(result).ok();
        }
    } else if let Some(method) = incoming.method {
        events
            .unbounded_send(Event {
                method,
                params: incoming.params,
                session_id: incoming.session_id,
            })
            .ok();
    }
}

impl Connection {
    /// Sends `method` with `params`, for the page session `session` or for the browser itself,
    /// and waits for the answer.
    ///
    /// # Errors
    ///
    /// When the connection is closed, the browser answers with an error, or no answer comes
    /// within 15 seconds.
    pub async fn call(
        &self,
        method: &str,
        params: Value,
        session: Option<&str>,
    ) -> Result<Value, CdpError> {
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        let (answer, answered) = oneshot::channel();
        match &mut *self.inner.waiting.lock() {
            Waiting::Open(waiters) => {
                waiters.insert(id, (method.to_string(), answer));
            }
            Waiting::Closed(reason) => return Err(CdpError::Closed(reason.clone())),
        }
        let mut message = Map::new();
        message.insert("id".into(), Value::from(id));
        message.insert("method".into(), Value::from(method));
        message.insert("params".into(), params);
        if let Some(session) = session {
            message.insert("sessionId".into(), Value::from(session));
        }
        if self
            .inner
            .outgoing
            .unbounded_send(Value::Object(message).to_string())
            .is_err()
        {
            self.forget(id);
            return Err(CdpError::Closed("the writer stopped".to_string()));
        }
        let timeout = pin!(self.inner.executor.timer(CALL_TIMEOUT));
        match future::select(answered, timeout).await {
            Either::Left((Ok(result), _)) => result,
            Either::Left((Err(_), _)) => Err(CdpError::Closed("the reader stopped".to_string())),
            Either::Right(_) => {
                self.forget(id);
                Err(CdpError::Timeout(method.to_string()))
            }
        }
    }

    /// Stops waiting for the answer to call `id`.
    fn forget(&self, id: u64) {
        if let Waiting::Open(waiters) = &mut *self.inner.waiting.lock() {
            waiters.remove(&id);
        }
    }
}
