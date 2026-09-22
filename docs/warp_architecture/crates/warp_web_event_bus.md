# warp_web_event_bus

> Per-crate reference (Marley round 2) — crate dir `crates/warp_web_event_bus`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance `[Warp-derived/AGPL: Warp-on-Web glue — not adopted]`:** the `WarpEvent` (incl. `LoggedOut`) JS event contract for the embedded web build. Marley is native with no web build → **won't need**. See [subsystem 05 → Marley status & Provenance](../subsystems/05-cloud-auth-networking.md#provenance--licensing).

| | |
|---|---|
| Subsystem | [Cloud, Auth & Networking](../subsystems/05-cloud-auth-networking.md) |
| License | AGPL v3 (`license.workspace = true` → `AGPL-3.0-only`; no own LICENSE marker) |
| Internal deps | 0 |
| Used by | 2 |

## Purpose

A tiny WASM-only shim that lets the Rust Warp-on-Web (WoW) build push structured events **up** to the host JavaScript application that embeds it. When Warp runs in the browser it is compiled to `wasm` and mounted inside a web shell (the `warp-server` client); this crate is the one-way Rust→JS notification channel for a handful of lifecycle/UI events (logout, session joined, errors, "open this in the native app", theme background changes).

The entire crate is gated `#![cfg(target_family = "wasm")]` — it compiles to nothing on native targets, so callers must guard their use sites with the same cfg.

## Key types, modules & public API

Single file: `src/lib.rs`.

- **`pub enum WarpEvent`** — the event payload, `#[derive(Serialize)]` with `#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]`. Variants: `LoggedOut`, `SessionJoined`, `ErrorLogged { error: String }`, `OpenOnNative { url: String }`, `ThemeBackgroundChanged { color: String }`. A doc comment warns these **must stay in sync** with the `WarpEvent` TypeScript type in `warp-server/client/src/warp-client/index.ts`.
- **`pub fn emit_event(event: WarpEvent)`** — serializes via `serde_wasm_bindgen::to_value` and calls the JS FFI. If the FFI throws a `ReferenceError` (the global isn't installed → not running inside a JS host) it silently no-ops; other errors are `log::warn!`-ed.
- **`mod ffi`** (private) — `#[wasm_bindgen(js_name = "warpEmitEvent", catch)] extern "C" fn emit_event(event: JsValue)`. Binds to a JS global event bus named `warpEmitEvent`. The module comment notes this global-based approach is fragile for multi-instance/re-init embeddings and lists alternatives (native DOM `CustomEvent` via `winit`'s backing `<canvas>`, passing context into WASM directly — blocked on `wasm-bindgen` #3041/#3659).

## Depends on (internal)

None. Only external crates: `serde`, `wasm-bindgen`, `serde-wasm-bindgen`, `js-sys`, `log`.

## Used by (internal dependents)

- [warp](./warp.md) — the umbrella app crate.
- [warp_logging](./warp_logging.md) — in `src/wasm.rs` it calls `warp_web_event_bus::emit_event(WarpEvent::ErrorLogged { .. })` to surface logged errors to the web host.

## Related crates

- [serve-wasm](./serve-wasm.md) — the dev webserver that serves the WoW wasm bundle these events are emitted from.
- [warp_logging](./warp_logging.md) — primary consumer; its WASM path forwards errors here.
- [warp_util](./warp_util.md) — holds the `hashed_asset_url` convention shared by the web build.

## Marley relevance

**Classification: STUB → RENAME (low priority).**

Marley is a desktop/native fork; the four goals (custom panel, session spawn/read/write, de-auth + login stub, de-Warp rebrand) are all native-side. Because this crate is `#![cfg(target_family = "wasm")]`, it compiles out of every native build and costs us nothing — so it is safe to ignore for the initial port (effectively already a no-op for native Marley).

If/when we touch it:
- **De-auth (goal 3):** the `WarpEvent::LoggedOut` variant is part of the web auth dance; in a login-stub world we'd just never emit it. No structural change needed for native.
- **De-Warp rebrand (goal 4):** rename the type `WarpEvent` → `MarleyEvent` and the JS global `warpEmitEvent` → `marleyEmitEvent`, but **only** if we keep a web target. The contract is a hard handshake with an out-of-repo TypeScript type, so renaming requires coordinated changes in a `warp-server`-equivalent shell we don't fork. Defer unless Marley ships a web build.
- Recommended near-term: **STUB/leave** — exclude from the native workspace build path; do not invest until a web shell is in scope.

## Notes / gotchas

- **WASM-only:** the whole crate body is behind `#![cfg(target_family = "wasm")]`. On native targets it is an empty crate — `emit_event` does not exist there, so consumers (e.g. `warp_logging`) must cfg-gate their calls.
- **Out-of-repo coupling:** the `WarpEvent` enum is a wire contract with a TypeScript type living in the separate `warpdotdev/warp-server` repo, which is **not** part of this fork. Drift here breaks the web host silently.
- **Global JS state:** depends on a JS global `warpEmitEvent` being installed by the host; the FFI is `catch` and treats a missing global (`ReferenceError`) as "no host present," so absence is not an error.
- `serde-wasm-bindgen` and `js-sys` are pinned to concrete versions (`0.6.5`, `0.3.63`) rather than workspace-managed.
