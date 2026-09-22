# command

> Per-crate reference (Marley round 2) — crate dir `crates/command`. Marley is forked from Warp ([warpdotdev/warp](https://github.com/warpdotdev/warp)).
>
> **Provenance:** `[Warp-derived: AGPL-3.0]` thin process-spawn wrapper; the mechanism (`std::process::Command` + Windows `CREATE_NO_WINDOW`) is `[public: OS/std]`. Marley reimplements it as **`marley_command` `[Marley-original]`** (the **non-PTY** spawn seam — the PTY path goes through `terminal_blocks`/`alacritty_terminal::tty` directly). See subsystem [Provenance & licensing](../subsystems/03-terminal-session-core.md#provenance--licensing).

| Field | Value |
|-------|-------|
| Subsystem | [03 — Terminal & Session Core](../subsystems/03-terminal-session-core.md) |
| License | AGPL v3 (`license.workspace = true` → `AGPL-3.0-only`); no own LICENSE marker. |
| Internal deps | 0 |
| Used by | **17** |

## Purpose

`command` is a thin, **drop-in process-spawning wrapper** over `std::process::Command` and
`async_process::Command`. Its sole reason to exist: on **Windows**, spawning a child without the
`CREATE_NO_WINDOW` flag briefly flashes a console window — this crate forces that flag (and a job
object for kill-on-parent-close) so the app never flashes a console. On Unix it is an effectively
transparent passthrough. It is the substrate that builds the shell process feeding the PTY
(`spawn_command_in_pty` uses `command::blocking::Command`).

## Key types, modules & public API

`lib.rs` exposes:

- **`blocking::Command`** (`src/blocking.rs`) — drop-in replacement for `std::process::Command`.
  `pub struct Command { inner: std::process::Command, … }` with the full builder surface (`new`,
  `arg`/`args`, `env`/`envs`, `current_dir`, `stdin`/`stdout`/`stderr`, `spawn`, `status`, `output`).
  On Windows it tracks `kill_on_parent_process_close` and a `JobObject`; tracks `stdin/stdout/stderr
  _is_default` to mirror std's pipe-vs-inherit defaults.
- **`async` (`r#async::Command`)** (`src/async.rs`, non-wasm) — drop-in replacement for
  `async_process::Command`.
- **`unix`** (`src/unix.rs`, cfg unix) and **`windows`** (`src/windows.rs`, cfg windows — `JobObject`)
  platform helpers.
- **`wsl`** (`src/wsl.rs`) — WSL detection/handling helpers.
- Re-exports `std::process::{ExitStatus, Output, Stdio}` for convenience.

The intended usage is mechanical: replace `std::process::Command` imports with `command::blocking`
and `async_process::Command` with `command::async` throughout the workspace.

## Depends on (internal)

None — `command` has **zero internal dependencies** (only `log`, plus platform crates
`win32job`/`windows` on Windows, `libc` on Unix, `async-process`/`futures-lite` off-wasm).

## Used by (internal dependents)

17 crates. Notable: [`local_control`](./local_control.md), [`warp`](./warp.md),
[`node_runtime`](./node_runtime.md), [`lsp`](./lsp.md), [`computer_use`](./computer_use.md),
[`remote_server`](./remote_server.md), [`integration`](./integration.md),
[`app-installation-detection`](./app-installation-detection.md),
[`command-signatures-v2`](./command-signatures-v2.md), [`warp_ripgrep`](./warp_ripgrep.md),
[`warp_util`](./warp_util.md), [`warp_completer`](./warp_completer.md),
[`warp_server_client`](./warp_server_client.md), [`warp_channel_config`](./warp_channel_config.md),
[`warp_isolation_platform`](./warp_isolation_platform.md), [`warpui`](./warpui.md),
[`warpui_core`](./warpui_core.md).

## Related crates

- [`local_control`](./local_control.md) — uses it for Windows helper-process spawning.
- The `app/src/terminal/local_tty/unix.rs` engine (`spawn_command_in_pty`) — the prime consumer on
  the PTY spawn path (architecture §3).

## Marley relevance

**Classify: KEEP (rename-deferred).** Pure infrastructure with **no Warp branding and no auth** — it
just needs to keep working so goal (2) spawn/write/read functions on every OS. It is *upstream* of
the PTY spawn path, so don't remove or stub it. Two caveats:
- It has **17 dependents**, so renaming the package (`command` → `marley_command`) is high-churn and
  low-value — **defer or skip**; the generic name `command` isn't a brand liability.
- The Windows `CREATE_NO_WINDOW` / `JobObject` behavior is load-bearing for a polished Windows build;
  keep it if Marley targets Windows, otherwise it is inert on macOS/Linux.

## Notes / gotchas

- The crate **intentionally uses `std::process::Command`** and `#![allow(clippy::disallowed_types)]`
  in `blocking.rs` — the workspace bans `std::process::Command` elsewhere precisely so everyone routes
  through this wrapper. Keep that lint discipline.
- `edition = 2024`. Windows pulls `win32job` (2.0.2) + `windows`; Unix pulls `libc`; off-wasm pulls
  `async-process` + `futures-lite`. WASM gets only the `blocking` surface stubs.
- `wsl.rs` ships tests (`wsl_tests.rs`); `test-util` feature exists but is empty today.
- No async runtime is imposed — `async::Command` is `async_process`-based (smol/futures-lite), not Tokio.
