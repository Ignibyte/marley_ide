# mcp

> Per-crate reference (Marley round 2). Crate dir: `crates/mcp`. Marley is forked from Warp (warpdotdev/warp).
> Provenance: [Warp-derived/AGPL] wrapper over [permissive/public: rmcp MIT/Apache; MCP is an open protocol] · Marley status: clean-room-target — reimplement the thin wrapper directly over upstream rmcp; reuse the open protocol + SDK, not Warp's façade.

| | |
|---|---|
| **Subsystem** | [agent-ai-mcp](../subsystems/04-agent-ai-mcp.md) |
| **License** | AGPL v3 (workspace `AGPL-3.0-only`; no per-crate LICENSE marker) |
| **Internal deps** | 5 |
| **Used by** | 1 |

## Purpose

`mcp` is Warp's **Model Context Protocol client runtime**. It connects to external MCP
servers (stdio child processes, streamable-HTTP, or SSE), negotiates capabilities,
enumerates the server's **tools** and **resources**, handles OAuth (including GitHub and
Dynamic Client Registration), and hands back a live handle the agent uses to invoke tools.
It is the concrete fulfilment of the `CallMCPTool` / `ReadMCPResource` actions modeled in
the [`ai`](./ai.md) crate.

It is essentially a thin, Warp-flavoured façade over the upstream **`rmcp`** SDK: it adds
capability gating, fail-soft listing, credential persistence to secure storage, a custom
SSE transport, and user-friendly error messages.

## Key types, modules & public API

- **`TemplatableMCPServerInfo`** (`src/lib.rs`) — the central handle for one connected server. Wraps an `rmcp::service::RunningService<RoleClient, Box<dyn DynService>>` plus its `resources: Vec<rmcp::model::Resource>`, `tools: Vec<rmcp::model::Tool>`, `installation_id: Uuid`, `description`, and `is_authenticated_transport`. Methods: `name()`, `tools()`, `resources()`, `peer()` / `peer_if_connected()` (the `rmcp::Peer` you call tools through), `has_tool()`, `has_resource()`, `has_resource_name_or_uri()`, `tool_input_schema()`, `shutdown()`.
- **`runtime`** (`src/runtime.rs`, native only) — startup logic:
  - `spawn_server(server_name, description, uuid, transport_type: TransportType, logger: SimpleLogger, auth_context: Option<oauth::AuthContext>) -> Result<TemplatableMCPServerInfo, rmcp::RmcpError>` — the main entry point; branches on `TransportType::{CLIServer, …}` to build stdio/HTTP/SSE transports (wraps commands in `cmd.exe /c` on Windows).
  - `build_client_with_headers(&HashMap<String,String>)` — reqwest client with custom default headers.
  - `error_to_user_message(&rmcp::RmcpError) -> String` — maps SDK errors to human text.
  - Internal `query_*_for` helpers gate tool/resource listing on advertised capability and fail soft.
- **`oauth`** (`src/oauth.rs`, native only) — `PersistedCredentials`, `AuthContext`, GitHub issuer/scopes, secure-storage keys `TEMPLATABLE_MCP_CREDENTIALS_KEY` / `FILE_BASED_MCP_CREDENTIALS_KEY`; built on `rmcp::transport::auth` (`AuthorizationManager`, `OAuthState`, DCR support).
- **`sse_transport`** (`src/sse_transport/`) — a bespoke `SseClientTransport<reqwest::Client>` (client-side SSE, reqwest impl, auth impl) for servers that speak SSE rather than streamable-HTTP.

## Depends on (internal)

- [`cloud_object_models`](./cloud_object_models.md) — `TransportType`, `StaticEnvVar`, and the MCP server config object models.
- [`simple_logger`](./simple_logger.md) — the `SimpleLogger` passed into `spawn_server` for redactable per-server logs.
- [`warp_core`](./warp_core.md) — `channel::ChannelState` used by the OAuth flow.
- [`warpui`](./warpui.md) — UI runtime integration (app context).
- [`warpui_extras`](./warpui_extras.md) — `secure_storage` (feature `user_preferences-toml`) for persisting OAuth credentials.

## Used by (internal dependents)

- [`warp`](./warp.md) — the only internal dependent; the app drives `spawn_server` and routes agent tool calls through the returned handles.

## Related crates

- [`ai`](./ai.md) — defines the `CallMCPTool`/`ReadMCPResource` actions this runtime executes.
- [`jsonrpc`](./jsonrpc.md) — a separate, lower-level JSON-RPC layer (used by `lsp`, not MCP); MCP uses `rmcp`'s own JSON-RPC instead.

## Marley relevance

**Classify: KEEP.** MCP is a major selling point and is provider-neutral — it is how
Marley would let an agent reach external tools (files, GitHub, browsers, our own forge).
None of the four goals require changing it structurally:

1. **Custom panel** — our panel can list `tools()`/`resources()` from `TemplatableMCPServerInfo` to show connected servers; additive UI only.
2. **Session spawn/write/read** — `spawn_server` + `peer()` is exactly the spawn/invoke surface; reuse as-is.
3. **De-auth + login stub** — the `oauth` module is *per-server* MCP auth (e.g. GitHub), **not** Warp account auth, so it is largely orthogonal to our login stub. We keep it; only the secure-storage backing needs to work offline.
4. **De-Warp rebrand** — only cosmetic: secure-storage key strings (`"TemplatableMcpCredentials"`) and the `Templatable*` naming are Warp-isms we may rename for cleanliness (low priority; renaming storage keys would orphan existing creds).

The package name `mcp` is already generic — no rename. Treat this as load-bearing
infrastructure to preserve.

## Notes / gotchas

- **Entirely native** — every public module is `#[cfg(not(target_family = "wasm"))]`; the wasm build of this crate is essentially empty.
- Tightly coupled to a specific **`rmcp`** version and its feature flags (`client`, `client-side-sse`, `auth`, `transport-streamable-http-client-reqwest`, `transport-child-process`). Upgrading `rmcp` is the main maintenance risk.
- The custom `sse_transport` exists because `rmcp`'s built-in SSE didn't meet Warp's needs — don't assume it's redundant.
- `spawn_server` logs may contain secrets (it prints a redaction warning up front); be careful surfacing those logs in a UI panel.
- `is_authenticated_transport` is captured but currently `#[allow(dead_code)]` (a TODO to drive a "connected/log-out" toast) — a natural hook for our panel.
