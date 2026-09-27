//! What the page logged and fetched, kept for agents (#492): its console messages and uncaught
//! errors, and its requests, each in a ring of the newest [`RING`].
//!
//! Nothing secret enters: requests keep no headers and no bodies, and [`redact_url`] hides the
//! values of query and fragment parameters whose names look secret, and the user and password a
//! URL may carry.

use std::collections::VecDeque;

use serde::Serialize;
use serde_json::Value;
use url::Url;

/// How many entries each ring keeps.
pub const RING: usize = 200;

/// The most characters of a message kept.
const MAX_TEXT: usize = 2_000;

/// Parts of a parameter's name that mark its value secret; a picked element's attributes are read
/// by them too (#518).
pub(crate) const SECRET_NAMES: &[&str] = &[
    "token", "key", "secret", "password", "auth", "code", "sig", "session",
];

/// What stands for a hidden value.
const HIDDEN: &str = "\u{2026}";

/// A console message or an uncaught error.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConsoleEntry {
    /// `log`, `info`, `warning`, `error`, `debug`, the browser log's levels, or `exception`.
    pub level: String,
    /// The message.
    pub text: String,
    /// The script or page it came from.
    pub source: Option<String>,
    /// Its line there, from 1.
    pub line: Option<u64>,
    /// When it came, in Unix milliseconds.
    pub time_ms: Option<f64>,
}

/// The page's console messages and errors, newest last.
#[derive(Debug, Clone, Default)]
pub struct ConsoleLog {
    entries: VecDeque<ConsoleEntry>,
}

impl ConsoleLog {
    /// Keeps what the event `method` with `params` says, when it is a console message, an
    /// uncaught error or a browser log entry; the entry kept.
    pub fn apply(&mut self, method: &str, params: &Value) -> Option<&ConsoleEntry> {
        let entry = match method {
            "Runtime.consoleAPICalled" => console_call(params),
            "Runtime.exceptionThrown" => exception(params),
            "Log.entryAdded" => log_entry(params),
            _ => None,
        }?;
        if self.entries.len() == RING {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
        self.entries.back()
    }

    /// The entries, oldest first.
    pub fn entries(&self) -> impl Iterator<Item = &ConsoleEntry> {
        self.entries.iter()
    }
}

fn console_call(params: &Value) -> Option<ConsoleEntry> {
    let level = params.get("type").and_then(Value::as_str)?.to_string();
    let text = params
        .get("args")
        .and_then(Value::as_array)
        .map(|args| args.iter().map(argument).collect::<Vec<_>>().join(" "))
        .unwrap_or_default();
    let frame = params.pointer("/stackTrace/callFrames/0");
    Some(ConsoleEntry {
        level,
        text: shorten(&text),
        source: frame
            .and_then(|frame| frame.get("url"))
            .and_then(Value::as_str)
            .filter(|url| !url.is_empty())
            .map(redact_url),
        line: frame
            .and_then(|frame| frame.get("lineNumber"))
            .and_then(Value::as_u64)
            .map(|line| line + 1),
        time_ms: params.get("timestamp").and_then(Value::as_f64),
    })
}

/// A console argument as text: a string as it is, another primitive as JSON, an object by its
/// description.
fn argument(argument: &Value) -> String {
    match argument.get("value") {
        Some(Value::String(text)) => text.clone(),
        Some(value) if !value.is_null() => value.to_string(),
        _ => argument
            .get("description")
            .or_else(|| argument.get("unserializableValue"))
            .or_else(|| argument.get("type"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
    }
}

fn exception(params: &Value) -> Option<ConsoleEntry> {
    let details = params.get("exceptionDetails")?;
    let text = details
        .pointer("/exception/description")
        .or_else(|| details.get("text"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    Some(ConsoleEntry {
        level: "exception".to_string(),
        text: shorten(text),
        source: details
            .get("url")
            .and_then(Value::as_str)
            .filter(|url| !url.is_empty())
            .map(redact_url),
        line: details
            .get("lineNumber")
            .and_then(Value::as_u64)
            .map(|line| line + 1),
        time_ms: params.get("timestamp").and_then(Value::as_f64),
    })
}

fn log_entry(params: &Value) -> Option<ConsoleEntry> {
    let entry = params.get("entry")?;
    Some(ConsoleEntry {
        level: entry
            .get("level")
            .and_then(Value::as_str)
            .unwrap_or("info")
            .to_string(),
        text: shorten(
            entry
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default(),
        ),
        source: entry
            .get("url")
            .and_then(Value::as_str)
            .filter(|url| !url.is_empty())
            .map(redact_url),
        line: entry
            .get("lineNumber")
            .and_then(Value::as_u64)
            .map(|line| line + 1),
        time_ms: entry.get("timestamp").and_then(Value::as_f64),
    })
}

/// One request the page made.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NetworkEntry {
    /// The browser's id for it.
    #[serde(skip)]
    pub id: String,
    /// Its method.
    pub method: String,
    /// Its URL, with secret-looking values hidden.
    pub url: String,
    /// What it fetched: `Document`, `Script`, `Fetch` and so on.
    pub kind: Option<String>,
    /// The response's status, once one came.
    pub status: Option<u64>,
    /// How long it took, once it ended.
    pub duration_ms: Option<u64>,
    /// Why it failed, when it did.
    pub failure: Option<String>,
    /// When it started, on the browser's clock, in seconds.
    #[serde(skip)]
    started: Option<f64>,
}

/// The page's requests, newest last.
#[derive(Debug, Clone, Default)]
pub struct NetworkLog {
    entries: VecDeque<NetworkEntry>,
}

impl NetworkLog {
    /// Keeps what the `Network` event `method` with `params` says of a request.
    pub fn apply(&mut self, method: &str, params: &Value) {
        let Some(id) = params.get("requestId").and_then(Value::as_str) else {
            return;
        };
        if method == "Network.requestWillBeSent" {
            let url = params
                .pointer("/request/url")
                .and_then(Value::as_str)
                .map(redact_url)
                .unwrap_or_default();
            let method = params
                .pointer("/request/method")
                .and_then(Value::as_str)
                .unwrap_or("GET")
                .to_string();
            let kind = params
                .get("type")
                .and_then(Value::as_str)
                .map(str::to_string);
            let started = params.get("timestamp").and_then(Value::as_f64);
            // A redirect sends the same id again, for the next URL.
            if let Some(entry) = self.entry_mut(id) {
                entry.url = url;
                entry.status = params
                    .pointer("/redirectResponse/status")
                    .and_then(Value::as_u64);
                return;
            }
            if self.entries.len() == RING {
                self.entries.pop_front();
            }
            self.entries.push_back(NetworkEntry {
                id: id.to_string(),
                method,
                url,
                kind,
                status: None,
                duration_ms: None,
                failure: None,
                started,
            });
            return;
        }
        let ended = params.get("timestamp").and_then(Value::as_f64);
        let Some(entry) = self.entry_mut(id) else {
            return;
        };
        match method {
            "Network.responseReceived" => {
                entry.status = params.pointer("/response/status").and_then(Value::as_u64);
                if entry.kind.is_none() {
                    entry.kind = params
                        .get("type")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
            }
            "Network.loadingFinished" | "Network.loadingFailed" => {
                entry.duration_ms = entry
                    .started
                    .zip(ended)
                    .map(|(started, ended)| milliseconds(ended - started));
                if method == "Network.loadingFailed" {
                    entry.failure = params
                        .get("errorText")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
            }
            _ => {}
        }
    }

    /// The requests, oldest first.
    pub fn entries(&self) -> impl Iterator<Item = &NetworkEntry> {
        self.entries.iter()
    }

    fn entry_mut(&mut self, id: &str) -> Option<&mut NetworkEntry> {
        self.entries.iter_mut().rev().find(|entry| entry.id == id)
    }
}

/// Seconds as whole milliseconds, none below zero.
fn milliseconds(seconds: f64) -> u64 {
    let milliseconds = (seconds * 1000.0).round().max(0.0);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // rounded and at least 0
    let whole = milliseconds as u64;
    whole
}

/// `url` with its user and password dropped, the values of query and fragment parameters whose
/// names look secret hidden, and a `data:` URL cut to its first 100 characters.
#[must_use]
pub fn redact_url(url: &str) -> String {
    if url.starts_with("data:") {
        return shorten_to(url, 100);
    }
    let Ok(mut parsed) = Url::parse(url) else {
        return url.to_string();
    };
    if (!parsed.username().is_empty() || parsed.password().is_some())
        && (parsed.set_username("").is_err() || parsed.set_password(None).is_err())
    {
        // A URL that carries a user has a host to drop it from; if not, none of it is shown.
        return HIDDEN.to_string();
    }
    let mut redacted = parsed[..url::Position::AfterPath].to_string();
    if let Some(query) = parsed.query() {
        redacted.push('?');
        redacted.push_str(&redact_pairs(query));
    }
    if let Some(fragment) = parsed.fragment() {
        redacted.push('#');
        if fragment.contains('=') {
            redacted.push_str(&redact_pairs(fragment));
        } else {
            redacted.push_str(fragment);
        }
    }
    redacted
}

fn redact_pairs(pairs: &str) -> String {
    pairs
        .split('&')
        .map(|pair| match pair.split_once('=') {
            Some((name, _)) if looks_secret(name) => format!("{name}={HIDDEN}"),
            _ => pair.to_string(),
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn looks_secret(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    SECRET_NAMES.iter().any(|secret| name.contains(secret))
}

fn shorten(text: &str) -> String {
    shorten_to(text, MAX_TEXT)
}

fn shorten_to(text: &str, most: usize) -> String {
    if text.chars().count() <= most {
        return text.to_string();
    }
    let kept: String = text.chars().take(most).collect();
    format!("{kept}{HIDDEN}")
}
