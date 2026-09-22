# serve-wasm

> Per-crate reference (Marley round 2) — crate dir `crates/serve-wasm`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (round 3):** `[Warp-derived/AGPL]` app code, built on `[permissive: axum/clap]` — Warp's dev static-server for the Warp-on-Web wasm bundle. Marley is **desktop-gpui only, no wasm/web build** → **REMOVE**, no counterpart. See [subsystem 07 → Marley status @ M15](../subsystems/07-app-entry-build-tooling.md#marley-status--m15).
>
> **Provenance `[permissive substance / AGPL file: standard `tower-http` `ServeDir`]`:** a dev-only static file server for the wasm bundle. Marley is native with no wasm bundle → **won't need** (and the substance is a stock `tower-http`/`clap` server anyway). See [subsystem 05 → Marley status & Provenance](../subsystems/05-cloud-auth-networking.md#provenance--licensing).

| | |
|---|---|
| Subsystem | [Cloud, Auth & Networking](../subsystems/05-cloud-auth-networking.md) |
| License | AGPL v3 (`license.workspace = true` → `AGPL-3.0-only`; no own LICENSE marker) |
| Internal deps | 0 |
| Used by | 0 |

## Purpose

A standalone developer-tooling binary: a small static webserver that serves the **Warp-on-Web (WoW) wasm bundle** plus its `index.html` and assets for local development. It is the local stand-in for the production `warp-server` web shell — run it against a built wasm directory and open `http://localhost:8000` to exercise the browser build. It is not part of any shipping product and nothing in the workspace links it (deps and dependents are both empty).

## Key types, modules & public API

Single file: `src/main.rs`. This is a `[[bin]]`-style crate (`fn main`), not a library — there is no public API surface, only a CLI.

- **`struct Args`** (`clap::Parser`): positional `directory: PathBuf` (the build dir containing the wasm bundle, `index.html`, and assets) and `--port/-p` (default `8000`).
- **`#[tokio::main] async fn main()`** — initializes `tracing_subscriber` (env filter, default `serve_wasm=info`), parses `Args`, prints `Serving Warp on http://localhost:{port}`, and calls `serve(make_router(dir), port)`.
- **`async fn serve(app: Router, port: u16)`** — binds `127.0.0.1:{port}` via `tokio::net::TcpListener` and runs `axum::serve` with a `TraceLayer`.
- **`fn make_router(build_directory: &Path) -> Router`** — the routing table:
  - `/`, `/session/{session_id}`, `/drive/{object_type}/{object_id}` → `ServeFile` of `index.html` (SPA fallback routes that mirror the client's app routes).
  - `/assets/client/wasm` → `ServeDir` of `<dir>/wasm`.
  - `/assets/client/static` → `ServeDir` of `<dir>/assets` — with a comment that this **must be kept in sync with `warp_util::path::hashed_asset_url`**.
  - Wrapped in a `tower::ServiceBuilder` + `TraceLayer` that logs method/URI per request and the response status.

## Depends on (internal)

None. External-only: `axum`, `axum-extra`, `clap` (derive), `tokio` (full), `tower`, `tower-http` (fs, trace), `tracing`, `tracing-subscriber` (env-filter).

## Used by (internal dependents)

None — standalone dev tool, not referenced by any workspace crate.

## Related crates

- [warp_web_event_bus](./warp_web_event_bus.md) — the Rust→JS event shim used by the wasm bundle this server serves.
- [warp_util](./warp_util.md) — owns `path::hashed_asset_url`, the asset-URL convention the static route must match.
- [warp](./warp.md) — the app crate that compiles to the wasm bundle being served.

## Marley relevance

**Classification: REMOVE (or KEEP-as-dev-tool, deferred).**

None of the four Marley goals touch this crate: it serves a browser build, while Marley's panel/session/auth/rebrand work is native. It has **zero dependents**, so removing it from the Marley workspace is a clean, risk-free deletion that trims the build graph.

- If Marley never ships a web target → **REMOVE** the crate entirely.
- If we keep a web build for any reason → **KEEP** but **RENAME** the user-facing string `"Serving Warp on http://localhost:..."` and the `serve_wasm=info` log target as part of the de-Warp rebrand (goal 4). These are the only Warp-isms.
- It has no auth and no Warp service coupling, so de-auth (goal 3) is a non-issue here.

Recommended: drop it from the default workspace members; keep the file around only if/when a web shell is revived.

## Notes / gotchas

- **Dev-only, hardcoded host:** binds `127.0.0.1` only (not `0.0.0.0`) — local dev, not deployable as-is.
- **No build step:** it serves a *pre-built* directory; it does not compile wasm. You must produce the `wasm/` + `assets/` + `index.html` layout separately.
- **Tight asset-path contract:** the `/assets/client/static` route is coupled to `warp_util::path::hashed_asset_url`; changing the hashing scheme in `warp_util` without updating this router (or vice-versa) yields 404s for hashed assets.
- Older `edition = "2021"` and its own `version = "0.1.0"` (most workspace crates use `edition = "2024"`).
- `panic`-on-error startup (`.unwrap()` on bind/serve) — acceptable for a dev tool, not for anything shipped.
