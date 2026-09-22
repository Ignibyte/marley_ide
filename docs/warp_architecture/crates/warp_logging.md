# warp_logging

> Per-crate reference (Marley round 2). Dir: `crates/warp_logging`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[Warp-derived/AGPL — de-auth/telemetry]` — Sentry crash/error pipeline to Warp; Marley boots offline with **no telemetry** (only a local `marley.log` path in `marley_core`). See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| Field | Value |
|---|---|
| Subsystem | [platform-infra](../subsystems/06-platform-settings-infra.md) |
| License | AGPL v3 (workspace `license = "AGPL-3.0-only"`) |
| Internal deps | 2 |
| Used by | 2 |

## Purpose

The crate that **initializes and manages logging** for the app: it sets up the `log`/`env_logger` backend, picks a destination (rotating file vs stderr), installs panic hooks, rotates and bundles log files for support, and — on WASM — wires logging into the browser. It centralizes log directory layout, file rotation policy, and optional crash reporting so callers just call `init(...)` once at startup.

## Key types, modules & public API

Root `crates/warp_logging/src/lib.rs` defines config + selects a platform `imp`:

- `pub enum LogDestination { File, Stderr }` — where output goes.
- `pub struct LogConfig { is_cli: bool, log_destination: Option<LogDestination>, max_file_size_bytes: Option<u64> }` — `is_cli` routes CLI logs to a separate subdir with a higher rotation limit so CLI runs don't evict GUI logs; `max_file_size_bytes` enables in-session rotation of `warp.log` into `warp.log.in_session.N` (ref warpdotdev/warp#10879).
- Platform impl selection: `#[cfg_attr(not(wasm), path = "native.rs")] / #[cfg_attr(wasm, path = "wasm.rs")] mod imp;` plus `mod rotation` on native.

Re-exported entry points:
- `pub use imp::init` — `fn init(config: LogConfig) -> Result<()>`, the main initializer (both targets).
- Native-only (`#[cfg(not(target_family = "wasm"))]`): `create_log_bundle_zip` (zips logs for support — uses the `zip` crate), `log_directory`, `log_file_path`, `rotate_log_files`, `init_for_crash_recovery_process`, `init_logging_for_unit_tests`, `on_crash_recovery_process_killed`, `on_parent_process_crash`.

Modules: `native.rs` (file/stderr logging, rotation, panic hooks via `log-panics`, crash-recovery process hooks), `wasm.rs` (browser logging via `console_error_panic_hook` / `web-sys` / `warp_web_event_bus`), `rotation.rs` (rotation policy, tested in `rotation_tests.rs`).

## Depends on (internal)

- [`./warp_core.md`](./warp_core.md) — shared core types/config; also gates the optional `crash_reporting` feature (`warp_core/crash_reporting`).
- [`./warp_web_event_bus.md`](./warp_web_event_bus.md) — **WASM target only**: routes browser log/panic events onto the web event bus so the JS host can surface them.

(External: `env_logger`, `log`, `chrono`, `cfg-if`, plus native `dirs`, `log-panics`, `zip`, and optional `sentry`/`sentry-log` behind `crash_reporting`.)

## Used by (internal dependents)

- [`./warp.md`](./warp.md) — the app calls `warp_logging::init(LogConfig { .. })` at startup.
- [`./onboarding.md`](./onboarding.md) — also initializes/uses logging during the onboarding flow.

## Related crates

- [`./warp_core.md`](./warp_core.md) — owns the `crash_reporting` feature this crate forwards to and shares core config.
- [`./warp_web_event_bus.md`](./warp_web_event_bus.md) — the WASM log/event sink.
- `warp_util` — sibling infra crate; logging and the path utilities are commonly initialized together at boot (not a Cargo dependency of this crate).

## Marley relevance

**Classify: KEEP / light RENAME.** Logging is goal-agnostic infrastructure and should be **kept** functionally. The one rebrand concern (**goal 4**) is cosmetic but visible: the on-disk artifacts are named `warp.log` / `warp.log.in_session.N` and live under a `warp`-named log directory (`log_directory()` / `log_file_path()` in `native.rs`). A Marley rebrand should rename these to `marley.log` and repoint the directory — a contained change inside `native.rs`/`rotation.rs`. Relevant to **goal 3 (de-auth/offline)**: the optional `crash_reporting` feature wires in Sentry (an external network reporter) — for an offline/login-stub Marley build, simply **leave `crash_reporting` off** (it is opt-in) so no telemetry leaves the box; no code change needed. Otherwise keep as-is; `init(LogConfig)` is the boot hook the app and a new Marley panel-bearing binary will call.

## Notes / gotchas

- **Dual-target build.** `init` exists on both targets, but the file-management functions (`log_file_path`, `rotate_log_files`, `create_log_bundle_zip`, crash-recovery hooks) are native-only — calling them won't compile on WASM.
- **Crash reporting is feature-gated and optional** (`crash_reporting = ["dep:sentry", "dep:sentry-log", "warp_core/crash_reporting"]`); default builds have no Sentry dependency.
- **Two rotation regimes:** per-startup (`rotate_log_files`) and in-session size-based (`max_file_size_bytes` → `.in_session.N`), the latter added per warpdotdev/warp#10879. CLI vs GUI logs are separated (`is_cli`) so they don't evict each other.
- Panic capture uses `log-panics` (with backtraces) on native and `console_error_panic_hook` on WASM; there are also dedicated crash-recovery-process entry points (`init_for_crash_recovery_process`, `on_parent_process_crash`).
- `edition = "2024"`.
