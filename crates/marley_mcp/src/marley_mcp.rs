//! `marley_mcp` — Marley's EXPOSE-side MCP server (M23 #370, orchestration-shell.md §4), so a manager
//! seat (an MCP client) can READ the fleet and SURFACE sessions to the human. Layer-1 item ④: insight
//! before control.
//!
//! House pattern (the retired forge sidecar client's precedent, #411): a PURE protocol core — parse a
//! request, route it, check permission, serialize the fleet, build the response/notification — all at
//! cov/MSI 100; plus a MASKED `std::net` transport shim (the listener / SSE / discovery-file loops). No
//! async runtime, no third-party MCP SDK (D-OPEN-SDK: hand-rolled — the L1 surface is `initialize` + 5
//! methods + 1 notification, and adopting `rmcp` would inject tokio+axum into a gpui app).
//!
//! The `marley_fleet` (#367) types ARE the schema (D2 — one seam, three consumers). `marley_mcp` OWNS the
//! permission [`GrantTable`] (S1): #371's settings round-trip deserializes INTO it, so this crate ships
//! and tests on fixture grants with no dependency on #371.
//!
//! Since #491 Marley starts the server at startup (`marley_workbench::mcp`), and the family it serves
//! is `terminal`, whose answers are the app's: the core returns each such call as
//! [`Outgoing::Deferred`], and the transport hands it to the app through an [`AppCaller`] and waits
//! for the answer. The `browser` family (#492) is the app's too. The `fleet` and `session` families
//! stay unlisted until prong 2's C1 feeds them. Since #524 programs outside Marley that the user
//! allowed by name reach a list of browser tools with tokens of their own ([`Principal`]). Since
//! #567 `browser_find` and `terminal_find` are listed and called only while the user turns them
//! on ([`CONDITIONAL_TOOLS`]), with [`find`]'s match by words as their first step.

// gate:21 runs Zed's dylint lints (`tooling/lints`) with these as errors in the Marley crates;
// Zed's crates keep them at warn (CONSTITUTION §0).
#![cfg_attr(
    dylint_lib = "lints",
    deny(
        async_block_without_await,
        blocking_io_on_foreground,
        entity_update_in_render,
        map_lookup_then_insert,
        notify_in_render,
        owned_string_into_shared,
        shared_string_from_str_literal
    )
)]

#[cfg(unix)]
pub mod agent_socket;
mod auth;
mod clients;
mod config;
pub mod discovery;
mod dispatch;
mod expose;
pub mod find;
pub mod ide;
pub mod ide_transport;
pub mod jsonrpc;
mod permission;
pub mod redact;
mod registry;
mod resource;
mod secret;
mod session;
mod tools;
pub mod transport;

pub use auth::{bearer_ok, ct_eq, is_loopback, origin_allowed};
pub use clients::{
    CLIENT_READ_TOOLS, CLIENT_WRITE_TOOLS, ClientError, ClientGrant, ClientInfo, ClientTable,
    Principal, check_client_name, permits,
};
pub use config::{McpConfigError, McpServerConfig, McpTransport};
pub use dispatch::{deferred_response, handle_message, snapshot_changed};
pub use expose::ExposeConfig;
pub use jsonrpc::{RpcRequest, parse_request};
pub use permission::{Decision, GrantTable, Tier, decide};
pub use registry::{
    CONDITIONAL_TOOLS, Family, ToolSpec, is_off, lookup, registry, tool_name, tools_list,
    tools_list_for,
};
pub use resource::{
    FLEET_RESOURCE_URI, resource_read, resource_updated_notification, resources_list,
};
pub use secret::{EntropyError, hex128, mint_secret};
pub use session::{
    CLIENT_SESSION_CAP, MAX_CLIENT_NAME, SESSION_CAP, SESSION_TTL_MS, SessionDecision, SessionFull,
    SessionRegistry, client_name_of, session_decision, session_gate,
};
pub use tools::{
    fleet_snapshot_result, resolve_surface, surface_receipt, surface_result, tool_answer_result,
    tool_refusal, tool_result,
};

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::mpsc::SyncSender;

use marley_fleet::FleetSnapshot;
use serde_json::Value;

/// The MCP protocol revision this server speaks (the 2025-06-18 MCP spec revision).
pub const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

/// How long a connection waits for the app to answer a tool call, in seconds (#491 D6).
pub const APP_CALL_TIMEOUT_SECONDS: u64 = 30;

/// Read-only context for handling ONE request.
///
/// The current fleet snapshot, the permission grants, and the app's `(session-id, pane-handle)`
/// surface index (the handle is an opaque `u64` = the app's `PaneId.0`, so `marley_mcp` never
/// depends on `marley_app`).
#[derive(Debug)]
pub struct RequestCtx<'a> {
    /// The current fleet snapshot (what `fleet_snapshot` / the resource serve).
    pub snapshot: &'a FleetSnapshot,
    /// The permission grants (what gates the write verbs).
    pub grants: &'a GrantTable,
    /// The app-provided id→pane-handle index for `surface_to_human` resolution.
    pub surface_index: &'a [(String, u64)],
    /// Who holds the request's bearer (#524): what it may list and call.
    pub principal: &'a Principal,
    /// The conditional tools the user turned on (#567): each of [`CONDITIONAL_TOOLS`] is listed
    /// and called only while it is here.
    pub enabled: &'a BTreeSet<String>,
}

/// Per-connection subscription state. L1 has ONE resource, so this is a single flag.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Subscriptions {
    /// Whether this connection subscribed to the fleet resource.
    pub fleet: bool,
}

/// An outbound message to the client: a response (to a request), a server-initiated notification,
/// or a response the app gives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outgoing {
    /// A JSON-RPC response body.
    Response(String),
    /// A JSON-RPC notification body (pushed on the standing stream).
    Notification(String),
    /// A `tools/call` the app answers (#491): the transport hands it to the app, holding no lock,
    /// and sends [`deferred_response`] of the outcome.
    Deferred(PendingCall),
}

/// A tool call the pure core cannot answer, because the answer is the app's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingCall {
    /// The request's id, for its response.
    pub id: Value,
    /// The tool's wire name.
    pub tool: String,
    /// Its arguments, as the client sent them.
    pub arguments: Value,
}

/// Who made a tool call, as the Claude Code plugin's bridge reports it (#520).
///
/// It holds the Marley terminal the client runs in, that terminal's project, the folder the
/// client runs in, and the name the client gave at `initialize` (#571). It is a convenience for
/// what a call that names no terminal acts on and for sorting callers, never an authority: the
/// bearer gates every call, and a shell can set its own variables.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Caller {
    /// The caller's `MARLEY_TERMINAL_ID`.
    pub terminal: Option<String>,
    /// The caller's `MARLEY_PROJECT`.
    pub project: Option<String>,
    /// The folder the caller's client runs in.
    pub cwd: Option<String>,
    /// The name the client gave at `initialize` (#571), such as `Zed`, which its session keeps.
    pub client: Option<String>,
}

/// A tool call handed to the app. The connection's thread waits for [`AppCall::answer`].
#[derive(Debug)]
pub struct AppCall {
    /// The tool's wire name.
    pub tool: String,
    /// Its arguments, as the client sent them.
    pub arguments: Value,
    caller: Caller,
    principal: Principal,
    answer: SyncSender<Result<ToolAnswer, Refusal>>,
}

impl AppCall {
    /// A call for `tool` with `arguments` from `caller`, made with `principal`'s bearer, whose
    /// answer goes to `answer`.
    #[must_use]
    pub const fn new(
        tool: String,
        arguments: Value,
        caller: Caller,
        principal: Principal,
        answer: SyncSender<Result<ToolAnswer, Refusal>>,
    ) -> Self {
        Self {
            tool,
            arguments,
            caller,
            principal,
            answer,
        }
    }

    /// Who made the call, as far as the client said (#520).
    #[must_use]
    pub const fn caller(&self) -> &Caller {
        &self.caller
    }

    /// Whose bearer the call came with (#524): Marley's own, or an outside client's.
    #[must_use]
    pub const fn principal(&self) -> &Principal {
        &self.principal
    }

    /// Gives the waiting connection the app's answer: the tool's result, or why it failed. A
    /// refusal given as words alone answers [`Refusal::REFUSED`]. An answer that comes after the
    /// connection stopped waiting goes nowhere.
    pub fn answer<E: Into<Refusal>>(self, result: Result<ToolAnswer, E>) {
        if self.answer.send(result.map_err(Into::into)).is_err() {
            log::debug!(
                "marley_mcp: {} was answered after its call stopped waiting",
                self.tool
            );
        }
    }
}

/// What the app answers a tool call with: the structured result, the text a client reads when it
/// is not the result's JSON (a block's output, say), and an image (the browser's frame, #492).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolAnswer {
    /// The result, as the tool's output schema describes it.
    pub structured: Value,
    /// The text to show, when not the result's JSON.
    pub text: Option<String>,
    /// An image the answer carries.
    pub image: Option<ToolImage>,
}

/// An image in an answer, base64, with its media type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolImage {
    /// Its media type: `image/jpeg`, `image/png`.
    pub mime_type: String,
    /// Its bytes, base64.
    pub data: String,
}

/// Why a tool call was refused, as the client gets it (#680): a code the tools' descriptions
/// name, so an agent can act on it without parsing words, the reason in words, and what the agent
/// can do instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// The refusal's code, [`Refusal::REFUSED`] when it names none of its own. Owned when it comes
    /// from another program at run time, as the harness's do (#693).
    pub code: Cow<'static, str>,
    /// Why, in words.
    pub reason: String,
    /// What the agent can do instead, each a sentence that names a tool or an argument.
    pub next_steps: Vec<String>,
}

impl Refusal {
    /// The code of a refusal that names no code of its own.
    pub const REFUSED: &'static str = "refused";

    /// A refusal with `code` and `reason`, and no next steps yet.
    #[must_use]
    pub fn new(code: impl Into<Cow<'static, str>>, reason: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            reason: reason.into(),
            next_steps: Vec::new(),
        }
    }

    /// The refusal with `step` added to its next steps.
    #[must_use]
    pub fn next(mut self, step: impl Into<String>) -> Self {
        self.next_steps.push(step.into());
        self
    }
}

impl From<String> for Refusal {
    fn from(reason: String) -> Self {
        Self::new(Self::REFUSED, reason)
    }
}

impl From<&str> for Refusal {
    fn from(reason: &str) -> Self {
        Self::new(Self::REFUSED, reason)
    }
}

/// How a call handed to the app ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppOutcome {
    /// The app answered: the tool's result, or why it refused.
    Answered(Result<ToolAnswer, Refusal>),
    /// No answer came within [`APP_CALL_TIMEOUT_SECONDS`].
    TimedOut,
    /// The app takes no calls: it is shutting down.
    Unavailable,
}

/// How the transport hands a call to the app. It must return at once: the app answers from its own
/// thread, through [`AppCall::answer`].
pub type AppCaller = Arc<dyn Fn(AppCall) + Send + Sync>;

/// An app-side effect the transport hands to the UI thread (it cannot touch the shell itself). L1 has
/// one: focus a pane (the `surface_to_human` effect).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// Focus the pane with this handle (`PaneId.0`).
    SurfacePane(u64),
}

/// The result of handling one message: the outbound messages + an optional app-side effect.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Handled {
    /// Messages to send back to the client.
    pub outgoing: Vec<Outgoing>,
    /// An effect for the app's UI thread, if any.
    pub effect: Option<Effect>,
}
