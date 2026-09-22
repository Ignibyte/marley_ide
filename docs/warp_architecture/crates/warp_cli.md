# warp_cli

> Per-crate reference (Marley round 2). Crate dir: `crates/warp_cli`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (round 3):** `[Warp-derived/AGPL]` — Warp's clap CLI/worker-dispatch layer (the multi-personality `warp`/`oz` binary). Marley ships a **single un-channelled `marley` binary** with no CLI/worker personality (`src/bin/marley.rs` → `marley_app::run()`), so there is no counterpart; a future control CLI would be `[Marley-original]`. See [subsystem 07 → Marley status @ M15](../subsystems/07-app-entry-build-tooling.md#marley-status--m15).

| | |
|---|---|
| Subsystem | [terminal-session-core](../subsystems/03-terminal-session-core.md) |
| License | AGPL v3 (inherits workspace `license = "AGPL-3.0-only"`; no own LICENSE marker) |
| Internal-deps count | 3 |
| Used-by count | 5 |

## Purpose

`warp_cli` is the **command-line argument-parsing and dispatch layer** for the Warp binary (`description = "CLI argument parsing for Warp"`, `edition = "2024"`). The single Warp/`oz` executable is multi-personality: it can launch the GUI app, run hidden worker subprocesses (terminal server, plugin host, ripgrep search, remote-dev daemon, minidump server), or act as a scriptable CLI/agent tool. `warp_cli` defines the entire `clap` command tree, parses `argv` once, and hands the typed result to `app` (the `warp` package) to route on.

It also owns the **`local_control` ("warpctrl") CLI** — a separate parser, selected *before* the main parser, that lets an external invocation drive an already-running Warp instance (windows/tabs/panes/sessions/input/surfaces).

## Key types, modules & public API

Top-level in `src/lib.rs`:

- `pub struct Args` (`#[derive(Parser)]`, `#[command(name = "oz")]`) — the root parser. Entry points:
  - `Args::from_env() -> Self` — parses `std::env::args()`; **feature-flag-gates subcommands at runtime** (e.g. hides/disables `environment`, `provider`, `integration`, `schedule`, `secret`, `federate`, `artifact`, `api-key` based on `warp_core::features::FeatureFlag`). Returns `Args::default()` on wasm.
  - `Args::clap_command() -> clap::Command` — builds the customized command (version string, per-channel binary name via `ChannelState`, `--help` examples). Use this, not `CommandFactory::command()`.
  - Accessors: `command()`, `app_args()`, `into_app_args()`, `global_options()`, `api_key()`, `output_format()`, `debug()`, `server_root_url()`, `ws_server_url()`, `session_sharing_server_url()`.
- `pub enum Command` — the dispatch root: `Worker(WorkerCommand)`, `CommandLine(Box<CliCommand>)`, `Completions { shell }`, `DumpDebugInfo`, `PrintTelemetryEvents`. `Command::prints_to_stdout()` distinguishes GUI vs. text modes.
- `pub enum WorkerCommand` — bundled worker subprocesses: `TerminalServer(TerminalServerArgs)` (unix), `PluginHost` (feature `plugin_host`), `MinidumpServer`, `RemoteServerProxy`/`RemoteServerDaemon` (`RemoteServerIdentityArgs`), `RipgrepSearch { pattern, paths, ... }`.
- `pub enum CliCommand` — the user-facing CLI: `Agent`, `Environment`, `MCP`, `Run` (alias `task`), `Model`, `Login`, `Logout`, `Whoami`, `Provider`, `Integration`, `Schedule`, `Secret`, `Federate`, `HarnessSupport`, `Artifact`, `ApiKey`. `CliCommand::as_str_for_tracing()` maps each to a stable tracing label.
- Config structs: `GlobalOptions` (`--api-key`/`WARP_API_KEY`, `--output-format`/`WARP_OUTPUT_FORMAT`), `ParentOpts` (`--parent-pid`, Windows `--parent-handle`), `RemoteServerIdentityArgs`, `AppArgs` (`--finish-update`, crash-recovery, `urls`), `RecoveryMechanism`.
- Helper fns: `terminal_server_subcommand()`, `ripgrep_search_subcommand()`, `installation_detection_server_subcommand()`, `finish_update_flag()`, `dump_debug_info_flag()`, `parent_flag()`, `binary_name()`, `version_string()`.
- Env-var constants: `OZ_RUN_ID_ENV`, `OZ_PARENT_RUN_ID_ENV`, `OZ_CLI_ENV`, `OZ_HARNESS_ENV`, `SERVER_ROOT_URL_OVERRIDE_ENV`, `WS_SERVER_URL_OVERRIDE_ENV`, `SESSION_SHARING_SERVER_URL_OVERRIDE_ENV`.

### Notable submodules (`pub mod`)

- `local_control` (`src/local_control/mod.rs`) — the `--warpctrl` control CLI. `CONTROL_MODE_FLAG = "--warpctrl"`; `pub struct ControlArgs` with `ControlArgs::from_env()` / `from_control_mode_env()` / `try_parse_control_mode_from(...)`. Large command surface: `ControlCommand`, `WindowCommand`, `TabCommand`, `PaneCommand`, `SessionCommand`, `InputCommand`, `SurfaceCommand`, `ThemeCommand`, `SettingCommand`, `KeybindingCommand`, `TargetArgs`, etc. Submodules `commands.rs`, `selectors.rs`, `output.rs`, `completions.rs` use the [`local_control`](#depends-on-internal) crate's `protocol`/`discovery`/`selection` APIs (e.g. `local_control::protocol::ControlError`, `discovery::InstanceRecord`, `selection::select_instance`).
- `agent` — `RunAgentArgs`, `AgentCommand`, `Prompt`, `Harness`, `OutputFormat`. The agent-run surface (cloud + local harnesses).
- `json_filter` — `JqFilter` (wraps a compiled `jaq_all` filter), `JsonOutput`, `parse_jq_filter(src) -> Result<JqFilter, String>`: lets CLI output be filtered with embedded `jq`.
- Other command modules: `mcp`, `model`, `task`, `provider`, `integration`, `schedule`, `secret`, `federate`, `artifact`, `api_key`, `share`, `skill`, `scope`, `completions`, `config_file`, `environment`, `harness_support`, `sort_order` (`SortOrderArg`).

## Depends on (internal)

- [`local_control`](./local_control.md) — the actual app-control protocol (instance discovery, request/response `protocol`, `selection`). `warp_cli::local_control` is the *CLI front-end*; the `local_control` crate is the transport/types it speaks. **Core to session/window/pane control (Marley goal 2).**
- [`warp_core`](./warp_core.md) — feature flags (`warp_core::features::FeatureFlag`) that gate subcommands, and channel/version metadata (`warp_core::channel::ChannelState` for `cli_command_name()`, `app_version()`).
- [`warp_util`](./warp_util.md) — platform helpers, e.g. `warp_util::windows::attach_to_parent_console()` so help/errors print when the CLI runs under Windows.

## Used by (internal dependents)

5 dependents:

- [`warp`](./warp.md) (the `app` crate) — the primary consumer. `app/src/lib.rs` calls `warp_cli::Args::from_env()` and matches on `warp_cli::Command` / `WorkerCommand` / `CliCommand` to route the whole process (GUI vs workers vs CLI). `app/src/debug_dump.rs` handles `Command::DumpDebugInfo`.
- [`integration`](./integration.md) — integration-test harness reuses the arg types.
- [`cloud_object_models`](./cloud_object_models.md) — shares CLI-defined data shapes.
- [`warp_completer`](./warp_completer.md) — completions tooling.
- [`warp_ripgrep`](./warp_ripgrep.md) — pairs with the `RipgrepSearch` worker subcommand.

## Related crates

- [`local_control`](./local_control.md) — read together; this crate is unusable to understand in isolation from it.
- [`warp_core`](./warp_core.md) — the feature-flag and channel source of truth that shapes which subcommands exist.
- [`ipc`](./ipc.md) — the *other* local cross-process mechanism (app↔plugin-host), distinct from `local_control` (CLI↔running-app).

## Marley relevance

**Classify: EXTEND (and RENAME later) — this is a primary Marley work surface.**

This crate sits on three of the four goals at once:

1. **Session spawn/write/read (goal 2):** `WorkerCommand::TerminalServer`, plus the entire `local_control` CLI (`SessionCommand`, `PaneCommand`, `InputCommand`, `WindowCommand`, `TabCommand`) is exactly how Marley would script "open a session / write input / read output." **EXTEND** here: add Marley control verbs and/or a clean programmatic API rather than only-argv parsing.
2. **De-auth + login stub (goal 3):** `CliCommand::{Login, Logout, Whoami}` and `GlobalOptions::api_key` (`--api-key`/`WARP_API_KEY`, plus `SERVER_ROOT_URL_OVERRIDE_ENV`, `WARP_SESSION_SHARING_SERVER_URL`) are the CLI entry points to Warp's cloud auth. **STUB** these: make `Login`/`Whoami` no-ops returning a synthetic local user, and ignore/short-circuit `--api-key` so an offline Marley boots without Warp's backend. The many `FeatureFlag`-gated cloud subcommands (`environment`, `provider`, `secret`, `federate`, `artifact`, `api-key`, `schedule`) can simply be left flag-disabled — `Args::from_env()` already hides them — which is the cheapest de-cloud path.
3. **De-Warp rebrand (goal 4):** heavy. `#[command(name = "oz")]`, `display_name = "Oz"`, the "Oz CLI" about-text, `https://docs.warp.dev/...` help links, `WARP_*` env-var names, and `ChannelState::cli_command_name()` all surface "Warp"/"Oz" to users. Rebrand to Marley here. Also rename the package `warp_cli` → `marley_cli`, but note **5 dependents** import it by name, so do the rename in one mechanical pass or defer until later in the fork.

The crate is large but mostly declarative `clap` structs, so edits are low-risk and high-leverage for the rebrand and de-auth goals.

## Notes / gotchas

- **`edition = "2024"`** (newer than its peers) — needs a recent Rust toolchain.
- **Runtime subcommand gating is two-layered:** `from_env()` *both* pre-scans `argv[1]` to emit "unrecognized subcommand" errors *and* `clap_command()` calls `mut_subcommand(..).hide(true)`. If you add/rename a subcommand you must update both places or help/visibility will desync.
- **`ServiceId`-style identity:** the binary's behavior depends on **how it was invoked** (`binary_name()` reads `argv[0]`; an `oz` symlink vs `warp` changes display). Rebrand must account for symlink/alias names, not just strings.
- **Embedded jq:** `json_filter` pulls in `jaq-all`/`jaq_json` to run real `jq` expressions on CLI output — non-trivial dependency surface for an arg-parsing crate.
- **Windows specifics:** `process_handle` module, `--parent-handle` arg, and `attach_to_parent_console()` only compile/behave on Windows (`cfg(windows)`).
- **Feature flags:** `plugin_host`, `integration_tests`, `api_key_authentication` (all default-off); `cfg_aliases` in `build.rs` defines `enable_crash_recovery` used by `AppArgs`.
- **wasm:** `Args::from_env()` short-circuits to `Args::default()` and the lib is `#![cfg_attr(target_family = "wasm", allow(dead_code))]` — the CLI is effectively native-only.
