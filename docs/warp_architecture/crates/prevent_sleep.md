# prevent_sleep

> Per-crate reference (Marley round 2) — crate dir `crates/prevent_sleep`. Marley is Ignibyte's fork of Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[permissive: standard deps]` — thin cross-platform keep-awake over OS APIs; standard concept, cleanly re-buildable. Gap (not yet built). See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| Field | Value |
|---|---|
| Subsystem | [platform-infra](../subsystems/06-platform-settings-infra.md) |
| License | AGPL v3 (no in-crate LICENSE marker; inherits workspace `license`) |
| Internal deps | 0 |
| Used by | 1 (`http_client`) |

## Purpose

A tiny cross-platform RAII wrapper that **keeps the OS from going to sleep** while some work is in flight (e.g. a long-running Agent Mode request or a network download). A guard object holds a system "activity" assertion; sleep is re-allowed when the guard drops. It exists to stop the machine suspending mid-operation without each caller writing per-OS power-management code.

## Key types, modules & public API

`src/lib.rs` selects a platform implementation at compile time via `#[cfg_attr]` path attributes — `mac.rs` (macos), `windows.rs` (windows), `noop.rs` (everything else) — exposed as `mod imp`. The `macos`/`noop` cfg aliases are produced by `build.rs` using `cfg_aliases`.

- `pub fn prevent_sleep(reason: &'static str) -> Guard` — begins a sleep-prevention assertion; the `reason` may surface in OS logs, so it should read as user-visible (e.g. `"Agent Mode request in-progress"`).
- `pub use imp::Guard` — opaque RAII guard. Dropping it ends the assertion.
  - macOS (`mac.rs`): wraps `NSProcessInfo::beginActivityWithOptions_reason` with `NSActivityOptions::UserInitiated` via `objc2`/`objc2-foundation`; `Drop` calls `endActivity`. Manually marked `unsafe impl Send + Sync` (only touched at create/drop).
  - Windows (`windows.rs`): uses `windows::Win32::System::Power` APIs.
  - noop (`noop.rs`): empty guard.
- `pub struct Stream<S>` — a `#[pin_project]` adapter that wraps a `futures::stream::Stream` and holds an `Option<Guard>` for as long as the stream is being polled. Built with `Stream::wrap(inner, guard)`; transparently forwards `poll_next`.

## Depends on (internal)

- *None.* This is a leaf crate (external deps only: `cfg-if`, `futures`, `pin-project`, `log`, plus per-OS `windows` / `objc2*`).

## Used by (internal dependents)

- [`http_client`](./http_client.md) — wraps long downloads/requests in a `prevent_sleep::Stream` so the machine stays awake while bytes are streaming.

## Related crates

- [`http_client`](./http_client.md) — its sole consumer.
- [`node_runtime`](./node_runtime.md) — sibling platform-infra leaf that performs the large downloads `http_client` would guard.

## Marley relevance

**Classification: KEEP (rename optional).** This is generic, vendor-neutral OS plumbing with no Warp branding, no auth, and no network coupling — none of the four Marley goals touch it. It is genuinely useful: Marley's goal (2) session spawn/write/read implies long-lived agent/terminal operations we don't want interrupted by sleep, so keeping it is the right call. No rename needed (package name is already neutral `prevent_sleep`). Leave as-is; it builds and works untouched.

## Notes / gotchas

- **Platform-specific native deps:** `objc2`/`objc2-foundation` on macOS, the `windows` crate (`Win32_System_Power` feature) on Windows. Anything else compiles to the noop backend, so CI on Linux exercises a no-op.
- The macOS guard hand-rolls `unsafe impl Send for Guard` / `Sync` with a justifying comment; respect that invariant if extending.
- Sleep prevention only lasts while the `Guard` is alive — drop it (or drop the wrapping `Stream`) promptly or the machine never sleeps.
