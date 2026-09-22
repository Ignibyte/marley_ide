# `marley_command` — non-PTY child-process spawn seam

> Per-crate architecture note (CONSTITUTION §21). Authoritative behavior contract:
> [`docs/specs/SPEC-process-command.spec.md`](../specs/SPEC-process-command.spec.md).
> Delivered **Unix-only** by TICKET-004 (forge #7); the Windows half is TICKET-004b
> (forge #6). Grew the **system URL-opener** in M12.2 (#196). GATE GREEN [full],
> coverage 100 / MSI 100. **Verified current @ M15** (re-read against the code 2026-07-12).
> **Provenance (§20):** `[Marley-original]` — the spawn seam + the opener are Marley's own
> code; the Warp `command` crate is the `[Warp-derived: AGPL]` *design* reimplemented
> clean-room (std-mirroring, §20), never carried as source.

## Purpose & ownership

The workspace's single **non-PTY** child-process spawn seam — a thin OS-parity wrapper
over `std::process::Command` (blocking) and `async_process::Command` (async). The
workspace **bans `std::process::Command` everywhere else** (clippy `disallowed-types`,
gate:2) and routes non-PTY spawns — `git`, helper subprocesses, the WSL probe —
through here, so the (future) Windows console-window suppression is applied uniformly.

**Not** the PTY path (seam-contracts §4.2): `marley_terminal` owns interactive
terminal/PTY spawn via `alacritty_terminal::tty` and does not route through this crate.

The crate root also owns the workspace **system URL-opener** (M12.2 #196) — `open_url` /
`open_url_with` / `is_openable_url` — so a clickable link in terminal output opens in the
browser through the same audited spawn seam. It is consumed by the render's
`marley_app::open_link_target` (a URL → `open_url`; a `file:line:col` → the code view).

## Modules

| Module | Surface |
|---|---|
| `blocking` | `Command { inner: std::process::Command }` — full builder (`new/arg/args/env/envs/env_remove/env_clear/current_dir/stdin/stdout/stderr`) + `spawn/status/output`. Carries the single `#[allow(clippy::disallowed_types)]` (R14). |
| `r#async` | `Command { inner: async_process::Command }` — same builder; **async** `status`/`output` (driven by any executor; tests use `futures_lite::block_on`). |
| `wsl` | `is_wsl_from(reader)` (the marker seam) · private `is_wsl_at(path)` · `is_wsl()`. |
| (crate root) | re-exports `ExitStatus` / `Output` / `Stdio`; the **URL-opener** (#196) — `is_openable_url` (http/https allowlist), `open_url` (via the platform `default_opener`: `open` on macOS, `xdg-open` elsewhere), and the testable `open_url_with(program, url)` seam. |

## Key design decisions (and why)

- **Std-mirroring builders.** The `blocking`/`r#async` split + method names mirror
  `std::process` / `async_process` so a consumer swaps the type with no behavioral
  change beyond the (future) Windows parity — a std-mirroring naming choice, not a fork
  layout (clean-room §20).
- **`async` `spawn` is synchronous** (`-> io::Result<Child>`), `status`/`output` async —
  because `async_process::Command::spawn` is itself sync (fork/exec returns immediately;
  the async work is *waiting*). A future-wrapped spawn would break drop-in parity. (The
  spec's R2 "spawn returning futures" is amended to this in SPEC-process-command.)
- **WSL detection via injected seams.** `is_wsl_from(reader: impl Read)` is the testable
  marker seam (both branches via injected readers); `is_wsl_at(path)` is tested with
  temp files; `is_wsl()` reads `/proc/version`, overridable via
  `MARLEY_KERNEL_VERSION_PATH` so its constant-fold-`false` mutant is killable on a
  non-WSL host. (PR-claude-absolutize-via-injected-cwd-seam-001; the env tests are
  `#[serial]`.)
- **R14 ban + the receipt fix.** `clippy.toml` `disallowed-types = [std::process::Command]`
  enforces the seam; `blocking` carries the one justified module-level allow. Because
  `clippy.toml` is now a **gate-defining** file, TICKET-004 also folded it into the
  commit-receipt fingerprint (`gate_state_hash`) so the ban can't be weakened around a
  green receipt (`PR-claude-gate-defining-files-in-receipt-fingerprint-001`).
- **URL-opener: scheme-guarded, shell-free (#196).** `open_url_with` rejects any scheme
  outside `http`/`https` (`is_openable_url` — so a crafted `file:` / `javascript:` / `data:` /
  schemeless link can't reach the opener) and returns `Err` **without spawning** on reject;
  a valid URL is passed as a **single argument** to the opener program — no shell, so there is
  nothing to inject (the #42 split-marker lesson: one arg through the spawn seam, never a shell
  string). `default_opener()` is `#[cfg]`-selected (not `cfg!`) so the other-OS arm compiles
  OUT and the live arm reaches 100% coverage; tests drive `open_url_with("true", …)` so no real
  browser launches.

## Unix-only scope (and the Windows deferral)

This crate ships **zero `#[cfg(windows)]` code**. The Windows half — R6 `CREATE_NO_WINDOW`,
R7 `JobObject` / `kill_on_parent_process_close`, the `windows` helper module — is
TICKET-004b, **blocked on a Windows CI runner**: on the macOS-only runner, cargo-mutants
generates mutants for cfg'd-out `#[cfg(windows)]` code that are reported MISSED (no-op →
survives) → MSI < 100, with exclusions barred by §0. See
[[cross-platform-mutation-single-runner]]. The spec's `unix`/`windows` helper modules are
N/A here (the Unix passthrough needs no platform module — R8 is structural).

## Dependencies

`async-process` (runtime dep); dev `futures-lite` (`block_on`), `tempfile`, `serial_test`.
All MIT/Apache. **No `unsafe`** (gate-6 miri N/A). No UI surface (gate-15 N/A).

## See also

- [`warp_architecture/subsystems/03-terminal-session-core.md`](../warp_architecture/subsystems/03-terminal-session-core.md) — the Warp `command` crate (the `[Warp-derived: AGPL]` design this reimplements) + the *Provenance & licensing* tags; its §7 traces `spawn_command_in_pty`, the PTY path that `marley_terminal` (not this crate) owns.
- [terminal_blocks.md](terminal_blocks.md) — the PTY spawn seam + the OSC 8 hyperlink DATA feeding the `open_url` consumer.
- [app_shell.md](app_shell.md) — `marley_app::open_link_target` / `marley_app::links`, the render-side caller of `open_url` (#196/#214).
