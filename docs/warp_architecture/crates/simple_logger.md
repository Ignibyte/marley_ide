# simple_logger

> Per-crate reference (Marley round 2) — crate dir `crates/simple_logger`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[permissive: standard deps]` — generic file logger; standard. Marley has a logfile path (`marley_core::marley_logfile_path`) but no ported logger yet. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| | |
|---|---|
| Subsystem | [platform-settings-infra](../subsystems/06-platform-settings-infra.md) |
| License | AGPL v3 (workspace `AGPL-3.0-only`; no own LICENSE marker) |
| Internal deps | 2 (`warp_core`, `warpui_core`) |
| Used by | 3 (`lsp`, `mcp`, `warp`) |

## Purpose

A small async, file-based logger for capturing **server / subprocess stderr+stdout streams** to disk (notably MCP server output and LSP server output). It is *not* the app's structured `tracing`/`log` facade — it is a sink that writes timestamped raw lines to per-namespace files, with optional size-based rotation. It exists because long-lived chatty child processes (e.g. an MCP server over a multi-day session) would otherwise grow a single log file unboundedly (`warpdotdev/warp#7723`), and because the app needs a single owner that guarantees at-most-one writer per log path.

## Key types, modules & public API

- `SimpleLogger` (`src/lib.rs`) — cheaply `Clone`able handle wrapping `Arc<LogFileWriter>`. Methods: `log(&self, message: String)` (non-blocking `try_send` onto an unbounded `async_channel`), `close(&self)` (explicit stream shutdown before all clones drop). A background `BackgroundTask` (spawned on `warpui_core::r#async::executor::Background`) drains the channel, prepends `chrono::Local` timestamps, flushes per line, and performs rotation.
- `RotationConfig` (`src/lib.rs`) — `new(max_file_size_bytes: u64, max_rotation: usize) -> Option<Self>` (zero → disabled). Drives `.1 … .N` suffix rotation via `perform_rotation` / `path_with_suffix` (raw `OsString` append, not `set_extension`).
- `manager::LogManager` (`src/manager.rs`) — the **singleton** (`impl SingletonEntity for LogManager`) that owns all loggers. `register_namespace(name, purge_on_startup)`, `register(namespace, relative_path, executor)`, and `register_with_rotation(.., rotation)`. Enforces one live writer per resolved path (returns `LogManagerError::LoggerAlreadyActive`); reclaims stale registrations when all clones drop *or* the channel was explicitly `close()`d.
- `manager::resolve_log_path(namespace, relative_path) -> PathBuf` — read-only path resolution (for reading logs back for display). Base dir = `warp_core::paths::secure_state_dir()` (falling back to `state_dir()`), Windows nests under `WARP_LOGS_DIR`.
- `LogManagerError` (`thiserror`) with `safe_message()` for release-channel error reporting (omits file paths).

## Depends on (internal)

- [warp_core](./warp_core.md) — `paths::secure_state_dir`/`state_dir`/`WARP_LOGS_DIR` for log directory resolution.
- [warpui_core](./warpui_core.md) — `r#async::executor::Background`/`BackgroundTask` for the drain task, and `Entity`/`SingletonEntity` traits to register `LogManager` as a singleton.

## Used by (internal dependents)

- [lsp](./lsp.md) — capture language-server stderr.
- [mcp](./mcp.md) — capture MCP server stderr/stdout (primary rotation user).
- [warp](./warp.md) — app wires up `LogManager` namespaces at startup.

## Related crates

- [warpui_core](./warpui_core.md) — the async executor and entity/singleton machinery this builds on.
- [warp_core](./warp_core.md) — `paths` module that decides where logs live.
- The app's `tracing`/`log` setup (separate) — `simple_logger` is the raw-stream sink, not the structured logging facade.

## Marley relevance

**KEEP (light rebrand).** This is generic, well-factored subprocess log plumbing that directly serves Marley goal (2) **session spawn/write/read**: capturing child-process output (Marley will spawn shell/agent sessions and needs their stderr on disk for the new UI panel, goal (1)). Reuse as-is. The only Warp-specific surface is the Windows `WARP_LOGS_DIR` segment and the secure-state-dir layout, which come from `warp_core::paths` — handle those in the rebrand of `warp_core`, not here. No auth, no network, nothing to stub. If we rename the singleton's on-disk namespace layout for goal (4) de-Warp rebrand, do it via `warp_core::paths`. Low effort, high reuse — leave the crate name as `simple_logger` (it isn't Warp-branded).

## Notes / gotchas

- Per-line `flush()` makes logs immediately visible but is comparatively expensive — fine for low/medium-volume server streams, not for hot paths.
- The active file may exceed `max_file_size_bytes` by up to one log line (rotation happens *after* the crossing write so lines are never split).
- Rotation drops the active file handle before renaming so Windows (which forbids renaming open files) succeeds; on rotation failure it reopens in **append** mode to preserve data and re-seeds the byte counter from `metadata().len()`.
- A live `Arc` is *not* sufficient to keep a path reserved — `LogManager` also treats an explicitly `close()`d channel as stale, enabling eager path reuse on retry.
- `register_namespace(_, purge_on_startup=true)` does `remove_dir_all` on the namespace dir at first registration — destructive; be deliberate about which namespaces purge.
