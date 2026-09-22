# warp_multi_agent_client

> Per-crate reference (Marley round 2). Crate dir: `crates/warp_multi_agent_client`. Marley is forked from Warp (warpdotdev/warp).
> Provenance: [Warp-derived/AGPL] · Marley status: SKIP → clean-room-target — the app.warp.dev multi-agent transport; INVENT a Marley direct-provider streaming client (BYO-key), keep only the public *shape* (a typed event stream), none of the body.

| | |
|---|---|
| **Subsystem** | [agent-ai-mcp](../subsystems/04-agent-ai-mcp.md) |
| **License** | AGPL v3 (workspace `AGPL-3.0-only`; no per-crate LICENSE marker) |
| **Internal deps** | 2 |
| **Used by** | 1 |

## Purpose

`warp_multi_agent_client` is the **network transport for Agent Mode** — the thin layer
that actually sends an agent request to Warp's cloud "multi-agent" backend and streams the
response back as a decoded event stream. Where [`ai`](./ai.md) owns the local *model* of an
agent conversation and `warp_multi_agent_api` owns the *protobuf wire types*, this crate is
the *plumbing* in between: authenticate, build the HTTP request, open a Server-Sent-Events
stream, and decode each event (base64 → protobuf → `ResponseEvent`).

It is small and single-purpose: one public async function plus an error enum and a stream
type alias.

## Key types, modules & public API

Everything lives in `src/lib.rs`:

- **`generate_multi_agent_output(client: &BaseClient, request: &warp_multi_agent_api::Request) -> Result<OutputStream, Error>`** — the sole entry point. It:
  1. gets/refreshes the access token via `BaseClient::get_or_refresh_access_token`,
  2. picks the endpoint based on `is_passive_suggestion_request` (active vs passive/suggestion traffic),
  3. POSTs the protobuf body with bearer auth + ambient headers (`AmbientHeaderPolicy`),
  4. wraps the eventsource with IAP-detection (`wrap_eventsource_with_iap_detection`),
  5. filters/decodes each SSE message into `ResponseEvent`s and instruments the trace span on the init event.
- **`OutputStream`** — cfg-split type alias: `BoxStream` on native, `LocalBoxStream` on wasm, both yielding `Result<warp_multi_agent_api::ResponseEvent, Error>`.
- **`Error`** — `thiserror` enum: `Authentication`, `AmbientHeaders`, `Base64Decode`, `ProtobufDecode`, `EventSource(Box<reqwest_eventsource::Error>)`.
- Feature **`agent_mode_evals`** — forwards to `warp_server_client`'s eval mode and adds an `EVAL_USER_ID_HEADER` to outgoing requests.

## Depends on (internal)

- [`warp_core`](./warp_core.md) — `channel::ChannelState` (request context/cancellation plumbing).
- [`warp_server_client`](./warp_server_client.md) — the `BaseClient` that owns auth tokens, the HTTP client, ambient-header policy, and the IAP-detection eventsource wrapper. This crate is effectively a specialization of `warp_server_client` for the multi-agent endpoint.

> Also depends on `warp_multi_agent_api` (the proto request/response types) via Cargo; that edge is tracked outside the internal depgraph.

## Used by (internal dependents)

- [`warp`](./warp.md) — the only internal dependent; the app calls `generate_multi_agent_output` to run an agent turn.

## Related crates

- [`ai`](./ai.md) — converts these `ResponseEvent`s into its local action/action-result model.
- [`warp_server_client`](./warp_server_client.md) — parent HTTP/auth client.
- `warp_multi_agent_api` *(out-of-repo: warpdotdev/warp-proto-apis)* — the protobuf request/response definitions exchanged here.

## Marley relevance

**Classify: STUB (rename deferred).** This crate is the **most Warp-cloud-coupled** in the
batch — it points straight at Warp's hosted agent backend and requires a Warp access token.
It is the primary blocker for goals (2) and (3):

1. **Custom panel** — n/a directly (no UI).
2. **Session spawn/write/read** — `generate_multi_agent_output` returning an `OutputStream` is the *shape* we want, but the destination is wrong. For Marley we **replace the body** to point at our own/local model provider (e.g. an Anthropic-direct or local endpoint) while keeping the `OutputStream`/`ResponseEvent` contract so [`ai`](./ai.md) and `warp` stay unchanged. This is the cleanest seam to swap the brain without touching the whole agent model.
3. **De-auth + login stub** — the `get_or_refresh_access_token` / ambient-header / IAP path must be short-circuited for offline boot; STUB it to skip Warp auth and inject a local key.
4. **De-Warp rebrand** — package name `warp_multi_agent_client` would become e.g. `marley_agent_client`, but it has a `warp_*` dependency surface and a `warp_multi_agent_api` proto contract; **defer the rename** until the endpoint swap lands so we don't churn names twice.

Net: keep the *interface*, stub/replace the *implementation*. This is where "de-Warp the
network" concretely happens.

## Notes / gotchas

- **Edition 2024** crate (newer than most of its siblings on 2021).
- Endpoint selection hinges on `is_passive_suggestion_request` — active agent turns and passive autosuggestions hit *different* URLs; a stub must honor both or disable passive.
- Decoding is **base64-of-protobuf over SSE** (`BASE64_URL_SAFE` → `prost::Message`); any replacement transport must reproduce that framing or change `decode_response_event` too.
- `wrap_eventsource_with_iap_detection` exists to detect Google IAP login-wall HTML masquerading as the stream — a cloud-auth artifact that becomes dead weight once de-authed.
