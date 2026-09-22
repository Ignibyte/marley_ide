# app-installation-detection

> Per-crate reference (Marley round 2) — crate dir `crates/app-installation-detection`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (round 3):** `[Warp-derived/AGPL]` — Warp's loopback `/install_detection` probe with hardcoded warp.dev CORS (a website→desktop bridge). Marley has no such web bridge → **REMOVE**, no counterpart. See [subsystem 07 → Marley status @ M15](../subsystems/07-app-entry-build-tooling.md#marley-status--m15).
>
> **Provenance (06 re-review):** `[Warp-derived/AGPL — rebrand/de-auth]` — axum endpoint with hardcoded `warp.dev` CORS origins; not ported (no marketing-site install probe). N/A. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| | |
|---|---|
| Subsystem | [platform-settings-infra](../subsystems/06-platform-settings-infra.md) |
| License | AGPL v3 (workspace `AGPL-3.0-only`; no own LICENSE marker) |
| Internal deps | 1 (`command`) |
| Used by | 2 (`integration`, `warp`) |

## Purpose

A tiny local **HTTP probe** that lets the warp.dev website (and localhost dev origins) detect whether the desktop app is installed/running on the user's machine. The app runs this `axum` router on a loopback port; the website fetches `GET /install_detection` cross-origin and, if it gets `ok`, knows the native app is present (so it can deep-link / change its CTA instead of prompting a download). It is a web-to-desktop bridge, not a settings store.

## Key types, modules & public API

- `make_router() -> axum::Router` (`src/lib.rs`) — the entire public surface. Builds a `Router` with one route, `route_service("/install_detection", get(detect_installation))`, wrapped in a `tower_http::trace::TraceLayer` (custom `http-request` span with method/uri, status logged on response) and a `CorsLayer`.
- `detect_installation()` (private, `async`) — handler returning the `&'static str` `"ok"`.
- CORS policy is the security-relevant part: `AllowOrigin::predicate` permits only `http://localhost:8080`, `http://localhost:8082`, `https://warp.dev`, and any `*.warp.dev` subdomain; methods restricted to `GET`.
- Cargo declares `command`, `nix`, and (on Windows) `win32job` as deps that the current `lib.rs` doesn't reference — they back the process/job-control plumbing used when the binary owns the listener lifecycle (see Notes).

## Depends on (internal)

- [command](./command.md) — subprocess/process utilities (used by the binary/lifecycle wiring around the detection server rather than the `make_router` surface).

## Used by (internal dependents)

- [warp](./warp.md) — the GUI app starts the detection server so the website can find a running install.
- [integration](./integration.md) — integration tests exercise the detection endpoint/router.

## Related crates

- [command](./command.md) — process control sibling.
- [simple_logger](./simple_logger.md) — another small "infra glue" crate in this subsystem batch.

## Marley relevance

**REMOVE (or STUB to a no-op).** The crate's entire reason for existing is the warp.dev marketing site detecting an installed Warp app — pure Warp-web coupling with hardcoded `warp.dev` / `*.warp.dev` CORS origins. Marley has no such website and this directly conflicts with goal (4) **de-Warp rebrand** (the origin allowlist literally names Warp's domains). It contributes nothing to goals (1)/(2)/(3). Cleanest: delete the crate and drop the startup call in `warp`; if removing the call site is disruptive, **stub** `make_router()` to return an empty `Router` (or bind nothing) so no loopback listener with `warp.dev` CORS ships. Either way, do not leave the `warp.dev` origin predicate in a rebranded build.

## Notes / gotchas

- The `warp.dev`/`*.warp.dev` origin allowlist is hardcoded in `make_router()` — a literal de-Warp grep target.
- Declared deps `nix` and the Windows-only `win32job` (plus `command`) are unused in `lib.rs` as checked out — they support a process/job-object lifecycle (e.g. tying the detection listener to the parent process so it dies with the app) that lives in the binary/consumer, not the exported router. Don't assume `lib.rs` is the whole story when removing deps.
- The handler returns a bare `"ok"` with no body/JSON contract; the website only checks reachability + CORS success, so behavior is "responds 200 from an allowed origin" rather than any payload.
