# http_client

> Per-crate reference (Marley round 2) — crate dir `crates/http_client`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance `[permissive substance / AGPL file: standard `reqwest` wrapper]`:** the Warp *file* is AGPL, but its substance is a thin shim over permissive `reqwest` (+ IAP/SSE). Marley has no HTTP today; a future brain would **rebuild from `reqwest` directly**, not adopt this file. See [subsystem 05 → Marley status & Provenance](../subsystems/05-cloud-auth-networking.md#provenance--licensing).

| Field | Value |
|---|---|
| Subsystem | [Cloud, Auth & Networking](../subsystems/05-cloud-auth-networking.md) |
| License | AGPL v3 (`license.workspace = true` → `AGPL-3.0-only`) |
| Internal deps | 2 |
| Used by | 6 |

## Purpose

Warp's **shared HTTP client** — a thin, opinionated wrapper around `reqwest` that every networked subsystem (AI, LSP, node runtime, GraphQL, server client) routes through instead of using `reqwest` directly. Centralizing here lets Warp uniformly: attach client/OS identity headers (`X-Warp-Client-Version`, OS category/name/version, client id), inject before-request / after-response hooks, handle **GCP IAP** auth challenges via a pluggable token provider, support Server-Sent-Events (`eventsource`) for streaming AI responses, prevent system sleep during long requests, and compile to both native and wasm32 (it swaps `reqwest`'s async runtime via `async-compat` natively and uses `gloo` on wasm).

## Key types, modules & public API

From `src/lib.rs`:

- `Client` — the wrapper. `Client::new()`, `Client::new_for_test()`, `Client::from_client_builder(reqwest::ClientBuilder)`; verb builders `get/post/put/patch/delete(url) -> RequestBuilder`; `execute(request) -> reqwest::Result<Response>`. Configurable hooks: `set_before_request_fn(RequestHookFn)`, `set_after_response_fn(ResponseHookFn)`, `set_iap_token_provider(Arc<dyn IapTokenProvider>)`.
- `RequestBuilder<'a>` — fluent builder: `json` (Serde), `proto` (`prost::Message`), `form`, `multipart`, `body`, `header`, `basic_auth`, `bearer_auth`, `timeout`, `prevent_sleep(reason)`, `eventsource() -> EventSourceStream`, and `send`/`build`/`build_split`.
- `Response` — wraps `reqwest::Response`: `text`, `json::<T>`, `bytes`, `bytes_stream`, `status`, `headers`, `url`, plus `error_for_status` / `error_for_status_with_body` / `error_for_status_ref` returning `ResponseError`.
- `Request`, `ResponseError`, and re-exports `AUTHORIZATION`, `HeaderMap`, `StatusCode`.
- `pub mod headers` — `CLIENT_RELEASE_VERSION_HEADER_KEY = "X-Warp-Client-Version"` (public) plus the private `X-Warp-*` OS/client-id header constants attached to every request.
- `pub mod iap` — `IapTokenProvider` (trait the host implements to mint Identity-Aware-Proxy tokens), `proxy_auth_header(token)`, `is_iap_challenge(status, headers)`.

`feature = "test-util"` and `Client::new_for_test()` exist for test harnesses. The `WARP_EXTRA_HTTP_HEADERS` env var (read only on `Channel::Integration`) injects extra headers.

## Depends on (internal)

- [`warp_core`](./warp_core.md) — `channel::{Channel, ChannelState}`, `operating_system_info::OperatingSystemInfo`, `execution_mode`, `report_error` — used to populate identity headers and report failures.
- [`prevent_sleep`](./prevent_sleep.md) — backs `RequestBuilder::prevent_sleep` so long downloads/streams keep the machine awake.

## Used by (internal dependents)

- [`ai`](./ai.md) — model API calls + SSE streaming.
- [`lsp`](./lsp.md), [`node_runtime`](./node_runtime.md) — download/runtime fetches.
- [`warp_graphql`](./warp_graphql.md), [`warp_server_client`](./warp_server_client.md) — backend transport.
- [`warp`](./warp.md) — the app.

## Related crates

- [`websocket`](./websocket.md) — sibling realtime transport; shares the IAP-challenge concept and native TLS/proxy stack.
- [`prevent_sleep`](./prevent_sleep.md) — power-management hook.
- [`ai`](./ai.md) — heaviest consumer; the `eventsource()` path was built largely for it.

## Marley relevance

**Classification: KEEP (RENAME deferred; targeted STUB of identity headers).** This is foundational, mostly vendor-neutral plumbing used by **6 crates** — ripping it out is a non-starter, and Marley's own networking (model calls in the agent path, downloads) should keep flowing through one wrapper. Relevant changes per goal: (3) **de-auth/login-stub** — the `iap` module and the `IapTokenProvider` hook can be left unset/short-circuited (`set_iap_token_provider` simply never called), so no Warp/GCP auth is attempted; this is a clean offline degrade, not a removal. (4) **de-Warp rebrand** — the `X-Warp-*` header constants and `CLIENT_RELEASE_VERSION_HEADER_KEY = "X-Warp-Client-Version"` leak the Warp brand on the wire and should be renamed to `X-Marley-*` (and the crate kept as `http_client` since it has no `warp_` prefix). (1) the custom panel and (2) session I/O don't touch this directly. Net: KEEP the crate, STUB the auth header injection + IAP provider, rebrand the header names.

## Notes / gotchas

- Native builds wrap `reqwest` futures in `async-compat`'s `Compat`/`CompatExt` because `reqwest` expects a `tokio` reactor while the app's runtime may differ — a subtle source of "works on wasm, hangs natively" bugs if removed.
- wasm32 swaps to `gloo` + `wasm-bindgen-futures`; not all features (e.g. some streaming paths) behave identically across targets.
- The OS identity headers are populated from `warp_core::operating_system_info` — rebranding the header *names* is easy, but the *values* still describe the host OS.
- `WARP_EXTRA_HTTP_HEADERS` only takes effect on the Integration channel — an easy-to-miss test hook.
