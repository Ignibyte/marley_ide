# websocket

> Per-crate reference (Marley round 2) — crate dir `crates/websocket`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance `[permissive substance / AGPL file: standard `tungstenite`/`ws_stream_wasm`]`:** the AGPL Warp *file* unifies permissive websocket libs behind `graphql_ws_client`. Marley streams nothing today; a future brain would **rebuild from `tungstenite` directly**, not adopt this file. See [subsystem 05 → Marley status & Provenance](../subsystems/05-cloud-auth-networking.md#provenance--licensing).

| Field | Value |
|---|---|
| Subsystem | [Cloud, Auth & Networking](../subsystems/05-cloud-auth-networking.md) |
| License | AGPL v3 (`license.workspace = true` → `AGPL-3.0-only`) |
| Internal deps | 0 |
| Used by | 4 |

## Purpose

A **single, target-agnostic WebSocket abstraction** that compiles to both native (desktop) and `wasm` (web). It papers over two completely different backing stacks — `async-tungstenite` + `tokio-rustls` + `rustls-platform-verifier` natively, and `ws_stream_wasm` on the browser — behind one `WebSocket` type. The returned socket implements `graphql_ws_client`'s `WebsocketMessage`, so it can be the transport for Warp's GraphQL-over-WebSocket subscriptions. It also handles native HTTP/proxy concerns: corporate proxy resolution + tunneling, custom handshake headers, and surfacing non-101 handshake responses (e.g. GCP IAP auth challenges).

## Key types, modules & public API

From `src/lib.rs`:

- `WebSocket` — the public socket. `WebSocket::connect(request, protocols)` (native: any `IntoClientRequest`, enriched with headers; wasm: just a URL + protocols), `WebSocket::connect_with_headers(url, protocols, headers)` (extra handshake headers; ignored on wasm), `WebSocket::split() -> (impl Sink, impl Stream)`, and `into_graphql_client_builder() -> graphql_ws_client::ClientBuilder`.
- `Message` — opaque wrapper over the per-target message; implements the `WebsocketMessage` trait (`new`/`new_text`/`new_binary`/`text`/`binary`).
- `Sink` and `Stream` — marker traits (blanket-impl'd) for the split halves; `Error` — a `thiserror` newtype over `anyhow::Error` (needed because `anyhow::Error` doesn't impl `std::error::Error`).
- `connect_error_http_response(&anyhow::Error)` (native only) — downcasts a failed handshake to its `tungstenite::http::Response`, letting callers inspect status/headers (used to detect IAP challenges).
- Re-exports (native): `tungstenite`, `tungstenite::client::IntoClientRequest`.

Modules: `imp` (cfg-routed to `native.rs` or `wasm.rs`), `sink_map_err` (a `Sink` adapter mapping the error type), and `proxy.rs` — `ProxyInfo`, `resolve_proxy(uri) -> Option<ProxyInfo>`, `connect_via_proxy(...)` for native HTTP `CONNECT` tunneling.

## Depends on (internal)

None — this is a leaf utility crate with only third-party dependencies.

## Used by (internal dependents)

- [`warp`](./warp.md) — the app, for live connections.
- [`warp_core`](./warp_core.md) — core runtime networking.
- [`warp_graphql`](./warp_graphql.md) — GraphQL subscription transport (`into_graphql_client_builder`).
- [`warp_server_client`](./warp_server_client.md) — the backend client layer.

## Related crates

- [`http_client`](./http_client.md) — sibling HTTP transport; shares the IAP-challenge concept (`http_client::iap`) and the proxy/rustls native stack.
- [`warp_graphql`](./warp_graphql.md) — the primary consumer; read together to see how subscriptions flow.

## Marley relevance

**Classification: KEEP (likely RENAME later).** This is generic, vendor-neutral infrastructure with **no internal dependencies** and a clean cross-target API — exactly the kind of plumbing Marley keeps. It is not tied to Warp's backend; whether Marley uses it depends on whether the rebranded app still talks to any realtime service. For the **de-auth/login-stub** goal (goal 3) the crate itself needs no change — you simply stop opening cloud subscriptions; the proxy/IAP code becomes dormant rather than wrong. For **session spawn/read/write** (goal 2), Marley's local sessions don't need a websocket, so this can sit idle. Already crate-named plainly `websocket` (no `warp_` prefix), so the **de-Warp rebrand** is a no-op here aside from the `graphql_ws_client`/Warp-subscription coupling noted below. Recommended: KEEP as-is for now; if Marley drops GraphQL subscriptions entirely, this becomes removable, but the cost of keeping it is near zero.

## Notes / gotchas

- The crate **doesn't actually assume the GraphQL protocol** yet still depends on `graphql_ws_client` solely to implement `WebsocketMessage` and reuse `wasm_websocket_combined_split` — the module docs call this out as an undesirable coupling that would require upstreaming the trait to remove.
- Heavy cfg-gating: native vs wasm pull entirely different dependency sets (`async-tungstenite`/`tokio-rustls`/`rustls-platform-verifier`/`hyper` vs `ws_stream_wasm`). Custom handshake headers and proxy support are **native-only**; wasm silently ignores them.
- TLS root verification uses `rustls-platform-verifier` (OS trust store) — relevant for any rebrand that ships a custom binary on locked-down corporate machines.
