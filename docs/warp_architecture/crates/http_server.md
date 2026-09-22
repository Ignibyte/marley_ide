# http_server

> Per-crate reference (Marley round 2) — crate dir `crates/http_server`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance `[permissive substance / AGPL file: standard `axum` local server]`:** the AGPL Warp *file* wraps permissive `axum` for local OAuth/web-handoff callbacks. Marley has no login callback; a future brain would **rebuild from `axum`**, not adopt this file. See [subsystem 05 → Marley status & Provenance](../subsystems/05-cloud-auth-networking.md#provenance--licensing).

| Field | Value |
|---|---|
| Subsystem | [Cloud, Auth & Networking](../subsystems/05-cloud-auth-networking.md) |
| License | AGPL v3 (`license.workspace = true` → `AGPL-3.0-only`) |
| Internal deps | 2 |
| Used by | 1 |

## Purpose

A tiny **local HTTP server embedded in the Warp client**. It binds to `127.0.0.1` on a fixed port (per release channel) and serves a set of `axum::Router`s contributed by other parts of the app. The canonical use is to receive **OAuth/login redirect callbacks** (and similar local-loopback handoffs) — an external browser completes a flow and redirects to `http://127.0.0.1:<port>/...`, which this server catches. It is a GPUI singleton, so there is exactly one per app process.

## Key types, modules & public API

Single file `src/lib.rs`:

- `HttpServer` — the singleton model. Implements `warpui_core::{Entity (Event = ()), SingletonEntity}`.
  - `HttpServer::new(routers: impl IntoIterator<Item = axum::Router>, ctx: &mut ModelContext<Self>) -> Self` — merges all supplied routers into one root router and spawns the server; failures are logged (`Failed to start local HTTP server`) and degrade gracefully (server simply absent).
  - `HttpServer::port() -> u16` — returns the channel-specific port: base `PORT_BASE = 9277` ("WARP" on a phone keypad), plus an offset per `warp_core::channel::Channel` (`Stable`=9277, `Preview`=9278, `Dev`=9279, `Local`=9280, `Integration`=9281, `Oss`=9282).
  - Private `spawn_server` builds a **dedicated single-worker `tokio` multi-thread runtime** (held in `_runtime`), binds a `TcpListener` on `127.0.0.1:port`, layers `tower_http::trace::TraceLayer`, and `axum::serve`s the merged router.

The server owns its own tokio runtime only because there is no shared runtime yet (`TODO(vorporeal)` in source).

## Depends on (internal)

- [`warp_core`](./warp_core.md) — `channel::{Channel, ChannelState}` to select the per-channel port.
- [`warpui_core`](./warpui_core.md) — `Entity`/`SingletonEntity`/`ModelContext` GPUI model framework (MIT).

Third-party: `axum`, `tower`, `tower-http` (trace + cors), `tokio`, `log`.

## Used by (internal dependents)

- [`warp`](./warp.md) — the only consumer; the app constructs the `HttpServer` singleton and passes in routers (e.g. the auth callback router).

## Related crates

- [`http_client`](./http_client.md) — the outbound counterpart (this crate is the inbound/loopback side).
- [`warp_server_auth`](./warp_server_auth.md) / the auth flow — the most likely supplier of the routers served here (login redirect handling).

## Marley relevance

**Classification: STUB / KEEP-minimal (depends on what the login stub needs).** The crate's reason for living is catching **OAuth redirect callbacks**, which is exactly the surface the **de-auth/login-stub** goal (goal 3) targets. Two paths: (a) if Marley's login stub fakes a logged-in user with no browser round-trip, the loopback callback is unnecessary — **STUB** by having `warp` pass an empty router set (or skip constructing `HttpServer` entirely), and the `Failed to start`/absent-server path already degrades cleanly; (b) keep the crate as a generic local router host if the custom panel (goal 1) or future local automation wants an HTTP control surface on `127.0.0.1` — it's a clean, dependency-light building block. It does not touch session spawn/read/write (goal 2). For the **de-Warp rebrand** (goal 4): crate name is already neutral (`http_server`), but note the `PORT_BASE = 9277` "WARP" easter-egg and the `Channel::Oss` port offset — cosmetic, low priority. Recommendation: KEEP the crate in tree; STUB its usage from `warp` so offline boot doesn't bind/serve an auth-callback listener it no longer needs.

## Notes / gotchas

- Spins up its **own private tokio runtime** (single worker thread) — intentional, with a `TODO` to fold into a shared runtime later. Be aware when accounting for thread/runtime usage.
- Bind failures are non-fatal: a port collision just logs and the server is silently absent (`_runtime` = `None`), so callers must not assume the listener exists.
- Loopback-only (`127.0.0.1`) — never externally reachable; the per-channel port scheme avoids collisions when multiple channels run side-by-side.
