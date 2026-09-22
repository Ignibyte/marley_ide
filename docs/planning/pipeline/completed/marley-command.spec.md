---
pipeline_id: 18731558-06ae-4916-8c91-2c34a0713e95
ticket: forge#7 (8ce39d6e-8632-4a44-a8fe-f5255c15f913) · local docs/planning/tickets/open/TICKET-004-marley-command.md
aar_id: 5cc28178-8f9c-4401-ab29-946f690c2d17
status: Phase 5 — Complete PASS
title: marley_command — non-PTY child-spawn seam (Unix-only; Windows → #6)
type: feature
milestone: M0
references:
  - ../../../specs/SPEC-process-command.spec.md
  - ../../../specs/standards/seam-contracts.md
---

## Title

TICKET-004 — `marley_command`. The workspace's single **non-PTY** child-process spawn
seam — a thin wrapper over `std::process::Command` (blocking) and
`async_process::Command` (async) for OS parity. **Scoped Unix-only:** the Windows
console-flash-suppression half (R6 `CREATE_NO_WINDOW`, R7 `JobObject`) is deferred to
**forge#6 / TICKET-004b** (blocked on a Windows CI runner — cargo-mutants generates
surviving mutants for `#[cfg(windows)]` code on the macOS-only runner;
[[cross-platform-mutation-single-runner]]). Contract: [`SPEC-process-command.spec.md`](../../../specs/SPEC-process-command.spec.md).

`marley_command` is **not** in the PTY path (seam-contracts §4.2) — `marley_terminal`
owns interactive PTY spawn via `alacritty_terminal::tty`. This crate spawns only
non-PTY children (git, helpers, the WSL probe).

## Scope
### In (this ticket — zero `#[cfg(windows)]`)
- **R1** `blocking::Command` — builder over `std::process::Command`: `new/arg/args/
  env/envs/env_remove/env_clear/current_dir/stdin/stdout/stderr/spawn/status/output`.
- **R2** `r#async::Command` — same surface over `async_process::Command`; async
  `spawn/status/output`.
- **R3** default stdio matches std (status=inherit, output=piped). **R4** `current_dir`.
  **R5** env mutations. **R8** Unix passthrough (no added flags — trivially, no platform code).
- **R9** spawn-missing-program → `Err(io::Error)`, no panic. **R10** `status`→`ExitStatus`.
  **R11** `output`→captured stdout/stderr+`ExitStatus`. **R12** re-export `ExitStatus/Output/Stdio`.
- **R13** `wsl::is_wsl_from(reader)` + `is_wsl()`. **R14** `std::process::Command` ban + the
  single justified allow. **R15-partial** `golden_spawn_config` per-platform observable
  (the cross-OS snapshot diff is 004b/CI).

### Out (→ forge#6 / 004b — needs a Windows runner)
R6 `CREATE_NO_WINDOW`, R7 `JobObject` / `kill_on_parent_process_close`, the `windows`
module, `win32job`/`windows` deps, **all `#[cfg(windows)]`**.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — spec is the contract** for the IN requirements (R1–R5, R8–R14, R15-partial).
- **D2 — zero `#[cfg(windows)]`** (the cfg-mutation blocker). Windows half → 004b.
- **D3 — WSL seam (R13):** `is_wsl_from(reader: impl Read) -> bool` (public marker seam —
  case-insensitive "microsoft"/"wsl"); private `is_wsl_at(path) -> bool`
  (`File::open(path).map(is_wsl_from).unwrap_or(false)`) tested with **temp files**
  (marker→true, plain→false, nonexistent→false) — fully covered+mutated on macOS;
  `is_wsl()` reads the path from env `MARLEY_KERNEL_VERSION_PATH` (else `/proc/version`)
  so a `#[serial]` test drives its `→false` constant-fold mutant dead. (PR-…-injected-cwd-seam
  + AD-…-stateful-crate-serial.)
- **D4 — R14 ban:** new workspace `clippy.toml` `disallowed-types = [std::process::Command]`;
  the `blocking` module carries a module-level `#![allow(clippy::disallowed_types)]` with a
  same-line `// justification` (gate:12-clean). The `async` module wraps `async_process` → no allow.
- **D5 — async via `futures_lite::block_on`** (async-process has its own reactor; no tokio).
- **D6 — deps:** `async-process` + `futures-lite`; dev `tempfile` + `serial_test`. **Drop**
  tokio/libc/log/win32job/windows.
- **D7 — clippy.toml not in the receipt fingerprint** — flag for inspect.
- **D8 — §21:** CHANGELOG + `docs/marley_architecture/marley_command.md`.

## Acceptance Criteria (EARS)
The IN requirements (R1–R5, R8–R14, R15-partial) of
[SPEC-process-command.spec.md](../../../specs/SPEC-process-command.spec.md), verified by
the spec's named tests (minus the cfg(windows) R6/R7/R8-Windows). Plus the bar:

| # | Bar | Verify |
|---|---|---|
| AC-A | blocking + async builders spawn a real non-PTY child (echo/env/pwd) | `cargo nextest run -p marley_command` |
| AC-B | spawn-missing→Err, no panic (R9); status/output (R10/R11) | unit |
| AC-C | is_wsl_from both branches + is_wsl_at temp-files + is_wsl env-override | unit (serial for env) |
| AC-D | std::process::Command banned elsewhere; single justified allow here (R14) | clippy gate:2 + gate:12 |
| AC-E | 100% coverage + mutation MSI 100 (no cfg(windows)) | gate:4/5 |
| AC-F | deny/audit/machete clean (async-process/futures-lite) | gate:7/8/9 |
| AC-G | FULL `scripts/gates.sh` → `GATE GREEN [full]` + receipt | /commit |
| AC-H | CHANGELOG + docs/marley_architecture/marley_command.md (§21) | enforce-changelog + inspect |

## Phase Plan
- **P2 Design** — SPIKE async-process builder API + futures-lite block_on + is_wsl_at
  temp-file + the clippy disallowed-types allow; module layout (blocking/async/wsl/lib);
  the builder delegation; test manifest (one row per IN R + the mutation map; the env
  #[serial] for is_wsl); clippy.toml + deps.
- **P3 Implement** — the 3 modules + lib.rs + Cargo.toml + clippy.toml.
- **P3.5 Inspect** — critics: spawn correctness + io-error (no unwrap), WSL seam
  mutation-readiness, R14 ban soundness + the receipt-fingerprint gap, clean-room.
- **P4 Validate** — IN tests; FULL gate green (cov 100 / MSI 100).
- **P5 Complete** — CHANGELOG + arch doc; AAR; archive; close #7.
