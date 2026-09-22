# warp_graphql

> Per-crate reference (Marley round 2). Crate dir: `crates/graphql` (package name `warp_graphql`). Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance `[Warp-derived/AGPL: Warp's cloud/backend glue — not adopted]`:** typed (cynic) GraphQL operations against Warp's `app.warp.dev` schema. Marley is local-first → **N/A**; the schema is Warp's proprietary backend contract. See [subsystem 05 → Marley status & Provenance](../subsystems/05-cloud-auth-networking.md#provenance--licensing).

| | |
|---|---|
| Subsystem | [cloud-auth-networking](../subsystems/05-cloud-auth-networking.md) |
| License | AGPL v3 (no own `LICENSE` marker; `license.workspace = true`) |
| Internal deps | 5 |
| Used by | 9 |

## Purpose

`warp_graphql` is the **typed GraphQL client and operation library** for talking to the Warp server. It wraps the registered `"warp-server"` schema (from [`warp_graphql_schema`](./warp_graphql_schema.md)) with [`cynic`](https://cynic-rs.dev) query/mutation/subscription fragments, custom scalars, request building, transport (`reqwest` over `http_client`, WebSocket subscriptions via `graphql-ws-client`), and error mapping. Every higher-level cloud crate (auth, cloud objects, AI, billing) issues server calls through the operations defined here.

Note the package name (`warp_graphql`) differs from the directory (`crates/graphql`).

## Key types, modules & public API

- `src/lib.rs` re-exports `api::*` plus `pub mod client`, `pub mod managed_secrets`, `pub mod scalars`.
- **`client`** (`src/client.rs`) — the transport layer:
  - `pub struct RequestOptions { auth_token, timeout, headers, path_prefix }` — per-request config (bearer token, header overrides, URL path prefix).
  - `pub enum GraphQLError` — variants incl. `RequestError`, `ResponseError`, `HttpError { status, body }`, `IapChallengeBlocked`, `StagingAccessBlocked`.
  - `pub trait Operation<QF>` — abstraction over a runnable GraphQL operation.
  - `pub fn get_request_context() -> RequestContext` and `pub fn get_user_facing_error_message(e) -> String`.
  - Internal `build_graphql_request` / `send_graphql_request` POST to `"{server_root_url}{path_prefix}/graphql/v2?op={name}"`, do IAP-challenge and staging-allowlist detection.
- **`api`** (`src/api/`) — the operation surface, one module per domain: `ai`, `billing`, `experiment`, `folder`, `notebook`, `object`, `object_actions`, `object_permissions`, `user`, `workflow`, `workspace`, `request_context` (`RequestContext`/`OsContext`/`ClientContext`), `response_context`, plus `queries/` (37 query modules), `mutations/` (71 mutation modules), `subscriptions/` (e.g. `get_warp_drive_updates`). `api/mod.rs` re-exports `warp_graphql_schema::schema` and wires custom scalars via `impl_scalar!`.
- **`scalars`** (`src/scalars/`) — `pub type Time = ServerTimestamp` (chrono `DateTime<Utc>` wrapper) and `pub struct Uint32(pub u32)`, mapped onto `schema::Time` / `schema::Uint`.
- **`managed_secrets`** (`src/managed_secrets.rs`) — `pub enum ManagedSecretType`, `pub struct ManagedSecretConfig`, `pub struct ManagedSecret`.

## Depends on (internal)

- [`warp_graphql_schema`](./warp_graphql_schema.md) — the registered `"warp-server"` schema this crate builds typed fragments against.
- [`http_client`](./http_client.md) — HTTP transport (`reqwest`-backed `Client`) used to POST operations.
- [`websocket`](./websocket.md) — WebSocket transport for GraphQL subscriptions (non-wasm).
- [`persistence`](./persistence.md) — local persistence helpers used by some operation/response handling.
- [`warp_core`](./warp_core.md) — core primitives incl. `ChannelState::server_root_url()` / staging detection used to build endpoints.

## Used by (internal dependents)

- [`warp_server_client`](./warp_server_client.md) and [`warp_server_auth`](./warp_server_auth.md) — auth/session and main client transport.
- [`cloud_objects`](./cloud_objects.md), [`cloud_object_client`](./cloud_object_client.md), [`cloud_object_models`](./cloud_object_models.md), [`cloud_object_persistence`](./cloud_object_persistence.md) — Warp Drive object CRUD.
- [`ai`](./ai.md) — AI conversation/agent operations.
- [`warp_managed_secrets`](./warp_managed_secrets.md) and the top-level [`warp`](./warp.md) app crate.

(Total internal dependents: 9 — the highest fan-out in this subsystem.)

## Related crates

- [`warp_graphql_schema`](./warp_graphql_schema.md) — the schema half; read together.
- [`http_client`](./http_client.md) / [`websocket`](./websocket.md) — the transports.
- [`warp_server_client`](./warp_server_client.md) — the main consumer that adds auth and retry on top.

## Marley relevance

**Classification: KEEP, partially STUB.**

This is the seam where Marley's offline/de-auth work bites. The crate is the *only* path to Warp's backend, so:

- **(3) de-auth + login stub:** Rather than deleting operations (which would cascade-break 9 dependents), the cleanest approach is to keep the typed API and **stub `client::send_graphql_request`** (and/or have callers in `warp_server_client` short-circuit) to return canned/empty results or a single "offline" error, so the app boots with no live server. `RequestOptions`/`GraphQLError` stay as-is.
- **(2) session spawn/write/read:** Local terminal sessions do not need the server; ensure session paths never hard-depend on a successful GraphQL round-trip. Audit `warp_server_client` for calls that block session start.
- **(4) de-Warp rebrand:** Endpoint strings derive from `warp_core::ChannelState::server_root_url()` (not hard-coded here), so a rebrand of the backend host is a `warp_core` change. The `"warp-server"` schema name must stay in sync with `warp_graphql_schema`. Package rename `warp_graphql → marley_graphql` is feasible but has 9 dependents → **defer to a coordinated sweep.**

Do not REMOVE: too many crates compile against its types.

## Notes / gotchas

- **Codegen-heavy.** Most "types" are `cynic` derive output keyed off the schema; a mismatch between the SDL and a hand-written fragment is a compile error.
- **wasm-aware.** `Cargo.toml` switches `graphql-ws-client` transport features between `tungstenite` (native) and `ws_stream_wasm` (wasm); subscriptions code is `cfg`-gated.
- **IAP / staging detection is baked into the transport.** `send_graphql_request` special-cases `x-goog-iap-*` challenges and Cloud-Armor HTML 403s — relevant if Marley ever points at a non-GCP backend (these branches become dead but harmless).
- Endpoint is hard-pathed to `/graphql/v2`; the host comes from `ChannelState`, not config here.
