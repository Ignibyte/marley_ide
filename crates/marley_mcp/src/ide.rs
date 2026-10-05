//! Claude Code's IDE integration, the protocol (#653).
//!
//! Claude Code links to an IDE through a WebSocket server on a loopback port and a lock file
//! named for that port in its configuration folder's `ide` directory, which holds the token the
//! client presents. Over the link the IDE pushes the editor's selection and a mention of a file's
//! lines, and answers the CLI's tools: diagnostics, the workspace's folders, the selection and the
//! open editors. Claude Code's docs name the server, the lock file's place, the token header and
//! `getDiagnostics`; the rest of what this module speaks is the VS Code extension's wire as public
//! implementations describe it, which Marley turns on per Claude Code version (`marley_agent`'s
//! version table). This module is pure: the sockets are [`crate::ide_transport`]'s.

use serde_json::{Value, json};

use crate::jsonrpc::{METHOD_NOT_FOUND, error_response, parse_request, result_response};

/// The name Marley gives itself in the lock file, which `/ide` shows.
pub const IDE_NAME: &str = "Marley";

/// The WebSocket subprotocol Claude Code asks for and expects back.
pub const SUBPROTOCOL: &str = "mcp";

/// The header carrying the lock file's token.
pub const AUTH_HEADER: &str = "x-claude-code-ide-authorization";

/// The largest frame and the largest message a client may send.
pub const MAX_MESSAGE_BYTES: usize = 1 << 20;

/// The most clients one server holds at once.
pub const MAX_CONNECTIONS: usize = 8;

/// The MCP version answered to a client that names none.
const FALLBACK_PROTOCOL_VERSION: &str = "2024-11-05";

/// The lock file's name for a server on `port`.
#[must_use]
pub fn lock_name(port: u16) -> String {
    format!("{port}.lock")
}

/// The lock file Claude Code reads: Marley's process, the folders it serves, its name, the
/// transport and the token.
#[must_use]
pub fn lock_json(pid: u32, folders: &[String], token: &str) -> String {
    json!({
        "pid": pid,
        "workspaceFolders": folders,
        "ideName": IDE_NAME,
        "transport": "ws",
        "authToken": token,
        "runningInWindows": false,
    })
    .to_string()
}

/// Whether the lock file `json` is one a Marley left behind: it names Marley and a process for
/// which `alive` is false. A file of another IDE, or one Marley cannot read, is never stale here.
#[must_use]
pub fn is_stale_marley_lock(json: &str, alive: impl Fn(u32) -> bool) -> bool {
    let Ok(lock) = serde_json::from_str::<Value>(json) else {
        return false;
    };
    let ours = lock.get("ideName").and_then(Value::as_str) == Some(IDE_NAME);
    let pid = lock
        .get("pid")
        .and_then(Value::as_u64)
        .and_then(|pid| u32::try_from(pid).ok());
    ours && pid.is_some_and(|pid| !alive(pid))
}

/// The parts of a WebSocket upgrade request the admission reads.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Upgrade {
    /// The request's method.
    pub method: String,
    /// `Host`.
    pub host: Option<String>,
    /// `Origin`, which Claude Code does not send.
    pub origin: Option<String>,
    /// The lock file's token as presented.
    pub token: Option<String>,
    /// `Upgrade`: the protocol the request switches to.
    pub requested: Option<String>,
    /// `Sec-WebSocket-Version`.
    pub version: Option<String>,
    /// `Sec-WebSocket-Key`.
    pub key: Option<String>,
    /// The subprotocols asked for.
    pub protocols: Vec<String>,
    /// `User-Agent`, which names Claude Code's version.
    pub user_agent: Option<String>,
}

impl Upgrade {
    /// Takes the request line `line`'s method.
    pub fn request_line(&mut self, line: &str) {
        self.method = line
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_string();
    }

    /// Takes the header `name: value` when it is one the admission reads.
    pub fn header(&mut self, name: &str, value: &str) {
        let value = value.trim().to_string();
        match name.trim().to_ascii_lowercase().as_str() {
            "host" => self.host = Some(value),
            "origin" => self.origin = Some(value),
            AUTH_HEADER => self.token = Some(value),
            "upgrade" => self.requested = Some(value),
            "sec-websocket-version" => self.version = Some(value),
            "sec-websocket-key" => self.key = Some(value),
            "sec-websocket-protocol" => self.protocols.extend(
                value
                    .split(',')
                    .map(|protocol| protocol.trim().to_string())
                    .filter(|protocol| !protocol.is_empty()),
            ),
            "user-agent" => self.user_agent = Some(value),
            _ => {}
        }
    }
}

/// Why an upgrade is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// Not a WebSocket upgrade this server speaks.
    BadRequest,
    /// Another host's name, or a web page's origin.
    Forbidden,
    /// No token, or the wrong one.
    Unauthorized,
}

impl Refusal {
    /// The status line's code and reason.
    #[must_use]
    pub const fn status(self) -> (u16, &'static str) {
        match self {
            Self::BadRequest => (400, "Bad Request"),
            Self::Forbidden => (403, "Forbidden"),
            Self::Unauthorized => (401, "Unauthorized"),
        }
    }
}

/// An upgrade the server takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Admitted {
    /// The client's key, from which the answer's accept key comes.
    pub key: String,
    /// Whether the client asked for [`SUBPROTOCOL`], which the answer then names.
    pub subprotocol: bool,
    /// The client's version, from its `User-Agent`.
    pub version: Option<String>,
}

/// Admits `upgrade` to the server on `port` whose token is `token`, or says why not.
///
/// The host must be the server's own loopback address, an `Origin` must be a loopback one, and
/// the token must match, compared in constant time.
///
/// # Errors
///
/// The [`Refusal`] the answer's status comes from.
pub fn admit(upgrade: &Upgrade, token: &str, port: u16) -> Result<Admitted, Refusal> {
    let websocket = upgrade.method == "GET"
        && upgrade
            .requested
            .as_deref()
            .is_some_and(|value| value.eq_ignore_ascii_case("websocket"))
        && upgrade.version.as_deref() == Some("13");
    let Some(key) = upgrade.key.clone().filter(|_| websocket) else {
        return Err(Refusal::BadRequest);
    };
    let own_host = upgrade.host.as_deref().is_some_and(|host| {
        host == format!("127.0.0.1:{port}") || host == format!("localhost:{port}")
    });
    if !own_host || !crate::auth::origin_allowed(upgrade.origin.as_deref()) {
        return Err(Refusal::Forbidden);
    }
    let presented = upgrade.token.as_deref().unwrap_or_default();
    if !crate::auth::ct_eq(presented, token) {
        return Err(Refusal::Unauthorized);
    }
    Ok(Admitted {
        key,
        subprotocol: upgrade
            .protocols
            .iter()
            .any(|protocol| protocol == SUBPROTOCOL),
        version: upgrade
            .user_agent
            .as_deref()
            .and_then(version_of_user_agent),
    })
}

/// The version a `User-Agent` such as `claude-code/2.1.288` or `claude-cli/2.1.288 (external,
/// cli)` names.
#[must_use]
pub fn version_of_user_agent(user_agent: &str) -> Option<String> {
    user_agent.split_whitespace().find_map(|word| {
        let (product, version) = word.split_once('/')?;
        (product.starts_with("claude") && !version.is_empty()).then(|| version.to_string())
    })
}

/// What a client may be sent, by its version's verdicts.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Parts {
    /// The selection and the open file: `selection_changed` and the selection tools.
    pub selection: bool,
    /// Send selection's mention: `at_mentioned`.
    pub mention: bool,
}

/// A tool a client calls, which the app answers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tool {
    /// `getDiagnostics`: one file's, by its `uri`, or every file's that has any.
    Diagnostics {
        /// The file, as a `file://` URI or a path.
        uri: Option<String>,
    },
    /// `getWorkspaceFolders`.
    WorkspaceFolders,
    /// `getCurrentSelection`.
    CurrentSelection,
    /// `getLatestSelection`.
    LatestSelection,
    /// `getOpenEditors`.
    OpenEditors,
}

impl Tool {
    /// The tool called `name` with `arguments`, if the server serves it to a client with `parts`.
    fn called(name: &str, arguments: &Value, parts: Parts) -> Option<Self> {
        let tool = match name {
            "getDiagnostics" => Self::Diagnostics {
                uri: arguments
                    .get("uri")
                    .and_then(Value::as_str)
                    .filter(|uri| !uri.is_empty())
                    .map(str::to_string),
            },
            "getWorkspaceFolders" => Self::WorkspaceFolders,
            "getCurrentSelection" => Self::CurrentSelection,
            "getLatestSelection" => Self::LatestSelection,
            "getOpenEditors" => Self::OpenEditors,
            _ => return None,
        };
        (parts.selection || !tool.needs_selection()).then_some(tool)
    }

    /// Whether the tool reads the editor's selection or editors, which the selection part gates.
    #[must_use]
    pub const fn needs_selection(&self) -> bool {
        matches!(
            self,
            Self::CurrentSelection | Self::LatestSelection | Self::OpenEditors
        )
    }
}

/// The tools listed to a client with `parts`.
fn tool_list(parts: Parts) -> Value {
    let empty = json!({ "type": "object", "properties": {} });
    let mut tools = vec![
        json!({
            "name": "getDiagnostics",
            "description": "Get language diagnostics from Marley's language servers: one file's, \
                            by its URI, or every file's in the project that has any.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "uri": {
                        "type": "string",
                        "description": "The file's URI; every file's diagnostics when left out."
                    }
                }
            }
        }),
        json!({
            "name": "getWorkspaceFolders",
            "description": "Get the folders of the project Marley serves.",
            "inputSchema": empty,
        }),
    ];
    if parts.selection {
        tools.extend([
            json!({
                "name": "getCurrentSelection",
                "description": "Get the selection in Marley's last focused file editor.",
                "inputSchema": empty,
            }),
            json!({
                "name": "getLatestSelection",
                "description": "Get the most recent selection Marley sent.",
                "inputSchema": empty,
            }),
            json!({
                "name": "getOpenEditors",
                "description": "Get the files open in Marley's editors.",
                "inputSchema": empty,
            }),
        ]);
    }
    json!({ "tools": tools })
}

/// What one message from a client asks of the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// Write this message back.
    Reply(String),
    /// Write this message back; the client named its version, if it did.
    Initialized {
        /// The answer to `initialize`.
        reply: String,
        /// `clientInfo.version`.
        version: Option<String>,
    },
    /// The app answers this tool call; [`tool_answer`] builds the reply.
    Call {
        /// The request's id.
        id: Value,
        /// The tool.
        tool: Tool,
    },
    /// The client named its process (`ide_connected`).
    Identified {
        /// Claude Code's process id.
        pid: u32,
    },
    /// Nothing to answer.
    Nothing,
}

/// What `message`, from a client with `parts`, asks of the server.
#[must_use]
pub fn step(message: &str, parts: Parts) -> Step {
    let request = match parse_request(message) {
        Ok(request) => request,
        Err((code, reason)) => return Step::Reply(error_response(&Value::Null, code, &reason)),
    };
    let id = request.response_id();
    match request.method.as_str() {
        "initialize" => {
            let protocol = request
                .params
                .get("protocolVersion")
                .and_then(Value::as_str)
                .unwrap_or(FALLBACK_PROTOCOL_VERSION)
                .to_string();
            let reply = result_response(
                &id,
                json!({
                    "protocolVersion": protocol,
                    "capabilities": {
                        "tools": { "listChanged": false },
                        "prompts": { "listChanged": false },
                    },
                    "serverInfo": { "name": IDE_NAME, "version": env!("CARGO_PKG_VERSION") },
                }),
            );
            let version = request
                .params
                .pointer("/clientInfo/version")
                .and_then(Value::as_str)
                .map(str::to_string);
            Step::Initialized { reply, version }
        }
        "ide_connected" => request
            .params
            .get("pid")
            .and_then(Value::as_u64)
            .and_then(|pid| u32::try_from(pid).ok())
            .map_or(Step::Nothing, |pid| Step::Identified { pid }),
        "tools/list" => Step::Reply(result_response(&id, tool_list(parts))),
        "tools/call" => {
            let name = request
                .params
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let arguments = request
                .params
                .get("arguments")
                .cloned()
                .unwrap_or(Value::Null);
            Tool::called(name, &arguments, parts).map_or_else(
                || {
                    Step::Reply(tool_answer(
                        &id,
                        Err(format!("Marley does not serve the tool {name}")),
                    ))
                },
                |tool| Step::Call {
                    id: id.clone(),
                    tool,
                },
            )
        }
        "prompts/list" => Step::Reply(result_response(&id, json!({ "prompts": [] }))),
        "ping" => Step::Reply(result_response(&id, json!({}))),
        _ if request.is_request() => Step::Reply(error_response(
            &id,
            METHOD_NOT_FOUND,
            &format!("Marley does not serve {}", request.method),
        )),
        _ => Step::Nothing,
    }
}

/// The reply to a tool call: the app's text, or its error as a tool error.
#[must_use]
pub fn tool_answer(id: &Value, answer: Result<String, String>) -> String {
    let (text, is_error) = match answer {
        Ok(text) => (text, false),
        Err(text) => (text, true),
    };
    let mut result = json!({ "content": [{ "type": "text", "text": text }] });
    if is_error {
        result["isError"] = Value::Bool(true);
    }
    result_response(id, result)
}

/// A place in a file as the LSP counts it: a line from 0 and a character from 0 in UTF-16 units.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Place {
    /// The line, from 0.
    pub line: u32,
    /// The character, from 0, in UTF-16 units.
    pub character: u32,
}

impl Place {
    fn json(self) -> Value {
        json!({ "line": self.line, "character": self.character })
    }
}

/// The selection in a file editor, or its caret alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorSelection {
    /// The file's absolute path.
    pub path: String,
    /// The selected text, empty for a caret.
    pub text: String,
    /// Where the selection starts.
    pub start: Place,
    /// Where it ends.
    pub end: Place,
}

impl EditorSelection {
    fn json(&self) -> Value {
        json!({
            "text": self.text,
            "filePath": self.path,
            "fileUrl": file_url(&self.path),
            "selection": {
                "start": self.start.json(),
                "end": self.end.json(),
                "isEmpty": self.start == self.end,
            },
        })
    }
}

/// The `selection_changed` notification for `selection`.
#[must_use]
pub fn selection_changed(selection: &EditorSelection) -> String {
    notification("selection_changed", &selection.json())
}

/// The `at_mentioned` notification for `path`, its lines from 0 when it has a range.
#[must_use]
pub fn at_mentioned(path: &str, lines: Option<(u32, u32)>) -> String {
    let mut params = json!({ "filePath": path });
    if let Some((start, end)) = lines {
        params["lineStart"] = json!(start);
        params["lineEnd"] = json!(end);
    }
    notification("at_mentioned", &params)
}

fn notification(method: &str, params: &Value) -> String {
    json!({ "jsonrpc": "2.0", "method": method, "params": params }).to_string()
}

/// The answer to `getCurrentSelection` and `getLatestSelection`.
#[must_use]
pub fn selection_answer(selection: Option<&EditorSelection>) -> String {
    selection.map_or_else(
        || json!({ "success": false, "message": "No file editor in Marley" }).to_string(),
        |selection| {
            let mut answer = selection.json();
            answer["success"] = Value::Bool(true);
            answer.to_string()
        },
    )
}

/// A file open in an editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenEditor {
    /// The file's absolute path.
    pub path: String,
    /// Whether it is the editor the selection comes from.
    pub active: bool,
    /// Whether it has unsaved changes.
    pub dirty: bool,
}

/// The answer to `getOpenEditors`.
#[must_use]
pub fn open_editors_answer(editors: &[OpenEditor]) -> String {
    let tabs: Vec<Value> = editors
        .iter()
        .map(|editor| {
            let label = editor
                .path
                .rsplit('/')
                .next()
                .unwrap_or(editor.path.as_str());
            json!({
                "uri": file_url(&editor.path),
                "isActive": editor.active,
                "label": label,
                "fileName": editor.path,
                "isDirty": editor.dirty,
            })
        })
        .collect();
    json!({ "tabs": tabs }).to_string()
}

/// The answer to `getWorkspaceFolders`.
#[must_use]
pub fn folders_answer(folders: &[String]) -> String {
    let entries: Vec<Value> = folders
        .iter()
        .map(|folder| {
            let name = folder.rsplit('/').next().unwrap_or(folder.as_str());
            json!({ "name": name, "uri": file_url(folder), "path": folder })
        })
        .collect();
    json!({
        "success": true,
        "folders": entries,
        "rootPath": folders.first(),
    })
    .to_string()
}

/// How severe a diagnostic is, in the LSP's words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// An error.
    Error,
    /// A warning.
    Warning,
    /// Information.
    Information,
    /// A hint.
    Hint,
}

impl Severity {
    const fn word(self) -> &'static str {
        match self {
            Self::Error => "Error",
            Self::Warning => "Warning",
            Self::Information => "Information",
            Self::Hint => "Hint",
        }
    }
}

/// One diagnostic of a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// What it says.
    pub message: String,
    /// How severe it is.
    pub severity: Severity,
    /// Where it starts.
    pub start: Place,
    /// Where it ends.
    pub end: Place,
    /// The language server or tool that gave it.
    pub source: Option<String>,
    /// Its code.
    pub code: Option<String>,
}

/// A file's diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDiagnostics {
    /// The file's absolute path.
    pub path: String,
    /// Its diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

/// The answer to `getDiagnostics`.
#[must_use]
pub fn diagnostics_answer(files: &[FileDiagnostics]) -> String {
    let files: Vec<Value> = files
        .iter()
        .map(|file| {
            let diagnostics: Vec<Value> = file
                .diagnostics
                .iter()
                .map(|diagnostic| {
                    json!({
                        "message": diagnostic.message,
                        "severity": diagnostic.severity.word(),
                        "range": { "start": diagnostic.start.json(), "end": diagnostic.end.json() },
                        "source": diagnostic.source,
                        "code": diagnostic.code,
                    })
                })
                .collect();
            json!({ "uri": file_url(&file.path), "diagnostics": diagnostics })
        })
        .collect();
    Value::Array(files).to_string()
}

/// `path` as a `file://` URL, each segment percent-encoded.
#[must_use]
pub fn file_url(path: &str) -> String {
    let encoded: Vec<String> = path
        .split('/')
        .map(|segment| urlencoding::encode(segment).into_owned())
        .collect();
    format!("file://{}", encoded.join("/"))
}

/// The path a tool's `uri` names: a `file://` URL decoded, or an absolute path as given.
#[must_use]
pub fn path_of_uri(uri: &str) -> Option<String> {
    let path = match uri.strip_prefix("file://") {
        Some(rest) => urlencoding::decode(rest).ok()?.into_owned(),
        None => uri.to_string(),
    };
    (path.starts_with('/') && !path.contains('\0')).then_some(path)
}
