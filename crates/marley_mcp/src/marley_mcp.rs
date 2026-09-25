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
//! for the answer. The `fleet` and `session` families stay unlisted until prong 2's C1 feeds them.

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

mod auth;
mod config;
pub mod discovery;
mod dispatch;
mod expose;
pub mod jsonrpc;
mod permission;
mod registry;
mod resource;
mod secret;
mod session;
mod tools;
pub mod transport;

pub use auth::{bearer_ok, ct_eq, is_loopback, origin_allowed};
pub use config::{McpConfigError, McpServerConfig, McpTransport};
pub use dispatch::{deferred_response, handle_message, snapshot_changed};
pub use expose::ExposeConfig;
pub use jsonrpc::{RpcRequest, parse_request};
pub use permission::{Decision, GrantTable, Tier, decide};
pub use registry::{Family, ToolSpec, lookup, registry, tool_name, tools_list};
pub use resource::{
    FLEET_RESOURCE_URI, resource_read, resource_updated_notification, resources_list,
};
pub use secret::{EntropyError, hex128, mint_secret};
pub use session::{
    SESSION_CAP, SESSION_TTL_MS, SessionDecision, SessionFull, SessionRegistry, session_decision,
    session_gate,
};
pub use tools::{
    SurfaceAck, fleet_snapshot_result, resolve_surface, surface_receipt, surface_result,
    tool_answer_result, tool_error, tool_result,
};

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

/// A tool call handed to the app. The connection's thread waits for [`AppCall::answer`].
#[derive(Debug)]
pub struct AppCall {
    /// The tool's wire name.
    pub tool: String,
    /// Its arguments, as the client sent them.
    pub arguments: Value,
    answer: SyncSender<Result<ToolAnswer, String>>,
}

impl AppCall {
    /// A call for `tool` with `arguments`, whose answer goes to `answer`.
    #[must_use]
    pub const fn new(
        tool: String,
        arguments: Value,
        answer: SyncSender<Result<ToolAnswer, String>>,
    ) -> Self {
        Self {
            tool,
            arguments,
            answer,
        }
    }

    /// Gives the waiting connection the app's answer: the tool's result, or why it failed. An
    /// answer that comes after the connection stopped waiting goes nowhere.
    pub fn answer(self, result: Result<ToolAnswer, String>) {
        if self.answer.send(result).is_err() {
            log::debug!(
                "marley_mcp: {} was answered after its call stopped waiting",
                self.tool
            );
        }
    }
}

/// What the app answers a tool call with: the structured result, and the text a client reads
/// when it is not the result's JSON (a block's output, say).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolAnswer {
    /// The result, as the tool's output schema describes it.
    pub structured: Value,
    /// The text to show, when not the result's JSON.
    pub text: Option<String>,
}

/// How a call handed to the app ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppOutcome {
    /// The app answered: the tool's result, or why it failed.
    Answered(Result<ToolAnswer, String>),
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
