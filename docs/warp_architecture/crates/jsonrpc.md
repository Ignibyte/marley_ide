# jsonrpc

> Per-crate reference (Marley round 2). Crate dir: `crates/jsonrpc`. Marley is forked from Warp (warpdotdev/warp).
> Provenance: [Warp-derived/AGPL] impl of [permissive/public: JSON-RPC 2.0 spec] · Marley status: gap — reimplement over the public spec if needed; used by LSP, NOT the agent transport, and NOT the brain.

| | |
|---|---|
| **Subsystem** | [agent-ai-mcp](../subsystems/04-agent-ai-mcp.md) |
| **License** | AGPL v3 (workspace `AGPL-3.0-only`; no per-crate LICENSE marker) |
| **Internal deps** | 1 |
| **Used by** | 1 |

## Purpose

`jsonrpc` is a small, transport-agnostic **JSON-RPC 2.0 client** runtime. It implements the
request/response/notification machinery (correlating responses to requests by id, dispatching
server-initiated notifications and requests) on top of an abstract byte transport, leaving
*framing* and *connection* to the caller. In this repo its consumer is the **LSP** subsystem
(language-server communication) — note it is *not* what `mcp` uses (MCP rides on the `rmcp`
SDK's own JSON-RPC).

## Key types, modules & public API

Two modules, re-exported from `src/lib.rs`:

- **`service::JsonRpcService`** (`src/service.rs`) — the client. Public API:
  - `new(...)` — constructs the service over a `Transport`, spawning a read loop on a `warpui_core::async::executor::Background` executor.
  - `next_id() -> RequestId` (`RequestId = i32`, monotonic via `AtomicI32`).
  - `async send_request(...)` — issues a request and awaits the correlated response (via a `oneshot` channel keyed by id).
  - `send_notification(method: String, params: Value) -> Result<()>` — fire-and-forget.
  - `async subscribe(key: String, on_notification: Subscription)` — register a handler for inbound notifications.
  - `set_server_request_handler(handler: impl Fn(String, Value, RequestId) -> Result<()> + Send + Sync + 'static)` — handle server→client requests.
  - `async shutdown(timeout: Duration) -> Result<()>`.
  - Wire structs: `Request<T>`, `Notification<T>`, `AnyRequest`/`AnyNotification`/`AnyResponse` (borrowed, `RawValue`-based for zero-copy dispatch); constant `JSON_RPC_VERSION = "2.0"`.
- **`transport::Transport`** (`src/transport.rs`) — the `#[async_trait]` abstraction the service runs on:
  - `async read(&self) -> Result<String>` (empty string = EOF),
  - `async write(&self, message: &str) -> Result<()>` (caller owns framing),
  - `async shutdown(&self, timeout: Duration) -> Result<()>`.
- **`ServerNotificationEvent`** — re-exported event type for surfaced notifications.

## Depends on (internal)

- [`warpui_core`](./warpui_core.md) — provides `r#async::executor::Background`, the executor that runs the JSON-RPC read loop. This is the *only* internal dependency.

> Otherwise leaf-level: `anyhow`, `async-channel`, `async-trait`, `futures`, `serde`/`serde_json` (incl. `RawValue`), `log`.

## Used by (internal dependents)

- [`lsp`](./lsp.md) — the language-server-protocol client crate is the sole internal consumer; it implements a concrete `Transport` over the language server's stdio and speaks LSP over this JSON-RPC core.

## Related crates

- [`lsp`](./lsp.md) — the consumer; read alongside to see a real `Transport` impl and method dispatch.
- [`mcp`](./mcp.md) — conceptually similar (also JSON-RPC over a transport) but uses `rmcp` instead of this crate; a candidate for *consolidation* but currently independent.

## Marley relevance

**Classify: KEEP.** This is generic, brand-neutral protocol infrastructure with a single
narrow internal dependency (`warpui_core`) and a single dependent (`lsp`). None of the four
Marley goals touch it:

1. **Custom panel** — n/a.
2. **Session spawn/write/read** — not the agent session transport (that's `warp_multi_agent_client`); leave as-is for LSP.
3. **De-auth + login stub** — no auth involved.
4. **De-Warp rebrand** — package name `jsonrpc` is already generic; the only Warp coupling is the `warpui_core` executor dependency, which is structural, not branding.

Lowest-risk crate in the batch. Keep verbatim. (Long-term, one *could* evaluate unifying it
with `rmcp`'s JSON-RPC to drop a dependency, but there's no Marley driver to do so.)

## Notes / gotchas

- **Framing is the transport's job** — `read()` must return exactly one complete message and `write()` must add appropriate framing (e.g. LSP `Content-Length` headers). The service itself is framing-agnostic; bugs here usually live in the `Transport` impl, not this crate.
- Uses `serde_json::value::RawValue` and borrowed `Any*` structs for low-allocation inbound dispatch — be careful with lifetimes when extending the dispatcher.
- Runs its read loop on `warpui_core`'s `Background` executor, so it's coupled to that async runtime rather than bare Tokio.
- IDs are `i32` only (`RequestId`), with a code comment noting the spec allows string ids but the client always assigns integer ids.
