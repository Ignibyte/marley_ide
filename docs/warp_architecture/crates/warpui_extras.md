# warpui_extras

> Per-crate reference (Marley round 2) — crate dir `crates/warpui_extras`. Marley is forked from Warp (warpdotdev/warp).
>
> Provenance: **[Warp-derived — MIT-intended, AGPL-in-manifest]** (`license.workspace = true` ⇒ AGPL-3.0-only; the MIT label is intent only — see gotchas) · Marley status: **not adopted** — Marley links neither this nor `warp_server_auth`; its settings/secrets live in Marley-original crates. See the subsystem doc's *Marley status @ M15*.

| | |
|---|---|
| Subsystem | [UI Framework & Rendering](../subsystems/01-ui-framework-rendering.md) |
| License | **MIT** (per Marley license rule) — ⚠️ but `Cargo.toml` uses `license.workspace = true`, which resolves to **AGPL-3.0-only**; see gotchas |
| Internal deps | 1 |
| Used by | 7 |

## Purpose

`warpui_extras` is a small companion crate to `warpui_core` providing **optional, OS-specific facilities that need a secret/credential store and a persistent user-preferences store**. It is deliberately split out of the core framework so that platform credential backends (macOS Keychain via `security-framework`, Linux Secret Service / D-Bus, Windows DPAPI/registry, browser `localStorage`) don't bloat or platform-lock the core. The two facilities register themselves as models into a `warpui_core::AppContext`.

## Key types, modules & public API

`src/lib.rs` is feature-gated and tiny:

```rust
#[cfg(feature = "secure_storage")]
pub mod secure_storage;
#[cfg(feature = "user_preferences")]
pub mod user_preferences;
```

- **`secure_storage`** (`src/secure_storage/mod.rs`) — credential store abstraction:
  - `pub trait SecureStorage` (`mod.rs:101`) and `pub type Model = Box<dyn SecureStorage>`.
  - Registration entry points into the app: `register(service_name, ctx)`, `register_noop(...)`, `register_unavailable(ctx)`, `register_with_fallback(...)`, `register_with_dir(...)` — all take `&mut warpui_core::AppContext`.
  - `pub trait AppContextExt` (`mod.rs:190`) extends `AppContext` with storage accessors; `pub enum Error`.
  - Backends: `mac.rs` (Keychain), `linux.rs` (Secret Service), `windows.rs` (DPAPI), `noop.rs`, `unavailable.rs`.
- **`user_preferences`** (`src/user_preferences/mod.rs`) — persisted prefs:
  - `pub trait UserPreferences` (`mod.rs:18`), `pub type Model = Box<dyn UserPreferences>`, `pub enum Error`.
  - Backends: `file_backed.rs`, `toml_backed.rs`, `registry_backed.rs` (Windows), `local_storage.rs` (wasm), `user_defaults.rs` (macOS `NSUserDefaults`), `in_memory.rs`.

Default features: `secure_storage`, `user_preferences`, `user_preferences-file`.

## Depends on (internal)

- [`warpui_core`](./warpui_core.md) — registers its `Model`s into `warpui_core::AppContext` and uses its `Error`/context types.

## Used by (internal dependents)

7 crates: [`warp`](./warp.md), [`warp_core`](./warp_core.md), [`warp_server_auth`](./warp_server_auth.md), [`ai`](./ai.md), [`settings`](./settings.md), [`integration`](./integration.md), [`mcp`](./mcp.md).

## Related crates

- [`warpui_core`](./warpui_core.md) — parent framework; extras plug into its context.
- [`warp_server_auth`](./warp_server_auth.md) — **the auth crate**; it reads/writes credentials through `secure_storage`, making this crate central to Marley's de-auth goal.
- [`settings`](./settings.md) — consumes `user_preferences`.

## Marley relevance

**Classification: STUB (secure_storage path) / KEEP (user_preferences).** This crate is the **hinge for goal (3): de-auth + login stub.**
- Warp stores auth tokens/credentials via `secure_storage`. For an **offline Marley boot** with no login, the cleanest move is to `register_noop` / `register_unavailable` (already-provided entry points) instead of the Keychain/Secret-Service backends, so the app never blocks on a credential store and `warp_server_auth` gets a benign empty store. This is a **stub via existing API — no source surgery required.**
- `user_preferences` is auth-independent and worth keeping (a new Marley panel and settings persistence rely on it). **KEEP.**
- **Rebrand (goal 4):** `service_name` passed to `register(...)` is the Keychain/secret-service service label (currently Warp's) — change it to a Marley identifier so credentials namespace under Marley.
- **Dependency removal:** stubbing secure_storage lets Marley drop the heavy `security-framework` / `secret-service` / `ouroboros` / `ring` deps on minimal builds by disabling the `secure_storage` feature.

## Notes / gotchas

- **License discrepancy:** the Marley brief lists this crate as MIT, but its `Cargo.toml` declares `license.workspace = true`, and `[workspace.package].license = "AGPL-3.0-only"`. There is **no per-crate `LICENSE` file** to override it. Treat the MIT designation as the *intended* Marley policy but flag this as a real inconsistency to reconcile (`warpui`/`warpui_core` hard-code `license = "MIT"`; this crate does not).
- Linux secure storage pulls `ouroboros` (self-referential structs), `ring`, and `secret-service` with `rt-async-io-crypto-rust` — D-Bus at runtime.
- macOS uses `objc2-foundation` `NSUserDefaults`/`NSString`; Windows uses `windows` + `windows-registry` + `Win32_Security_Cryptography`.
- `user_preferences-toml` (off by default) swaps the file backend to `toml_edit`.
