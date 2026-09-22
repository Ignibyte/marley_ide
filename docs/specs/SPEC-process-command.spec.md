---
spec_id: process-command
component: marley_command
bucket: REIMPLEMENT
milestone: M0
status: draft
title: Non-PTY child-process spawn seam with Windows CREATE_NO_WINDOW + JobObject parity
goal: Give the workspace one process-spawning wrapper that is a transparent passthrough on Unix and silences the console-window flash (and binds child lifetime to the parent) on Windows, so every NON-PTY child process — git, helper subprocesses, the WSL probe — spawns identically on every OS.
reuses: [tokio, async-process, futures-lite, libc, win32job, windows, log]
spec_source: "behavior-only — observable cross-OS child-spawn parity: spawn a non-PTY child with builder-controlled program/args/cwd/env/stdio and read its exit status and captured streams; on Windows suppress the console-window flash and bind the child's lifetime to the parent; detect whether the host is WSL. No fork module/type/static names; no fork file paths. The behavioral contract is pinned in standards/seam-contracts.md §4.2 + §5."
clean_room: "behavior-derived from a fork-reference doc; IP-counsel sign-off pending"
browser_testable: no
visual_acceptance: N/A
mutation_testing: Enabled
references:
  - ./standards/quality-bar.spec.md
  - ./standards/seam-contracts.md
---

## Purpose
`marley_command` is the workspace's single **non-PTY** child-process spawn seam: a thin, drop-in
wrapper over `std::process::Command` (blocking) and `async_process::Command` (async). Its reason
to exist is OS parity. On Windows, spawning a child without `CREATE_NO_WINDOW` briefly flashes a
console window and a killed/closed parent can orphan its children; this crate forces the
`CREATE_NO_WINDOW` creation flag and (on request) assigns the child to a `JobObject` that
terminates when the parent process handle closes. On Unix it is an effectively transparent
passthrough. It is foundational substrate for the non-interactive subprocesses the rest of the
workspace runs — `git`, helper binaries, and the WSL-detection probe.

**PTY ownership (seam-contracts §4.2, binding):** `marley_command` is **not** in the PTY path at
M1. `marley_terminal` owns interactive terminal/PTY spawn end to end via `alacritty_terminal::tty`
(its own openpty + fork + exec) and does **not** route through this crate. This crate makes no
claim on the shell-into-PTY path; it spawns only non-PTY children.

The `blocking` / `r#async` module split mirrors the `std::process` / `async_process` taxonomy it
wraps (a std-mirroring naming choice, not a fork-derived layout): consumers can swap a
`std::process::Command` for `marley_command::blocking::Command` with no behavioral change beyond
the documented Windows parity.

## Public surface (the contract)
- `marley_command::blocking::Command` — drop-in replacement for `std::process::Command`. Builder
  surface: `new(program)`, `arg`, `args`, `env`, `envs`, `env_remove`, `env_clear`, `current_dir`,
  `stdin`, `stdout`, `stderr`, `spawn() -> io::Result<Child>`, `status() -> io::Result<ExitStatus>`,
  `output() -> io::Result<Output>`. On Windows it additionally exposes
  `kill_on_parent_process_close(bool) -> &mut Self`.
- `marley_command::r#async::Command` — drop-in replacement for `async_process::Command` with the
  same builder surface and `spawn`/`status`/`output` returning futures; runtime-agnostic (drivable
  from a Tokio runtime via `async-process`'s own reactor).
- `marley_command::unix` (cfg unix) and `marley_command::windows` (cfg windows; owns `JobObject`) —
  platform helper modules.
- `marley_command::wsl::is_wsl() -> bool` — production WSL-detection entry; reads the host's real
  kernel-version source.
- `marley_command::wsl::is_wsl_from(version_reader: impl Read) -> bool` — **injection seam** for
  WSL detection. Takes any reader yielding the kernel-version text and returns whether it carries a
  WSL marker, so both branches are testable on any host. `is_wsl()` delegates to `is_wsl_from` over
  the real source.
- Re-exports: `marley_command::{ExitStatus, Output, Stdio}` (from `std::process`).

## EARS Requirements
R1. The system shall expose `blocking::Command` providing the `std::process::Command` builder surface (`new`, `arg`, `args`, `env`, `envs`, `env_remove`, `env_clear`, `current_dir`, `stdin`, `stdout`, `stderr`, `spawn`, `status`, `output`).

R2. The system shall expose `r#async::Command` providing the same builder surface as `blocking::Command` with `spawn`, `status`, and `output` returning futures. *(Delivery note, TICKET-004: `spawn` is **synchronous** (`io::Result<Child>`), mirroring `async_process::Command::spawn`, which is itself sync — fork/exec returns immediately; `status`/`output` are the futures. A future-wrapped `spawn` would break drop-in parity.)*

R3. WHEN `spawn`, `status`, or `output` is invoked on a `blocking::Command` with no stdio explicitly set, the system shall use the same default stdio disposition as `std::process::Command` (inherit for `status`, piped-stdout/stderr for `output`).

R4. WHEN `current_dir(path)` is set and the child is spawned, the system shall start the child with its working directory equal to `path`.

R5. WHEN `env(key, value)`, `envs`, `env_remove`, or `env_clear` is configured and the child is spawned, the system shall apply exactly those environment mutations to the child's environment relative to the inherited parent environment.

R6. WHILE the target OS is Windows, WHEN a child is spawned, the system shall set the `CREATE_NO_WINDOW` process creation flag on that child.

R7. WHILE the target OS is Windows, WHEN a child is spawned with `kill_on_parent_process_close(true)`, the system shall assign the child to a `JobObject` configured to terminate the child when the parent process handle closes.

R8. WHERE the target OS is Unix, the system shall spawn the child as a transparent passthrough that adds no creation flags and no job object beyond what `std::process::Command` would set.

R9. IF `spawn` fails (for example the program is not found or is not executable), THEN the system shall return `Err(io::Error)` and shall not panic.

R10. WHEN `status` is invoked, the system shall run the child to completion and return its `ExitStatus`.

R11. WHEN `output` is invoked, the system shall run the child to completion and return an `Output` carrying the captured stdout, captured stderr, and `ExitStatus`.

R12. The system shall re-export `ExitStatus`, `Output`, and `Stdio` from `std::process` at the crate root.

R13. WHEN `wsl::is_wsl_from(version_reader)` is given a reader whose contents contain a WSL kernel marker, the system shall return `true`, and WHEN the contents contain no WSL marker the system shall return `false`; `wsl::is_wsl()` shall delegate to `is_wsl_from` over the host's real kernel-version source.

R14. The system shall confine all direct use of `std::process::Command` to this crate, each occurrence carrying a `// justification` for its `#[allow(clippy::disallowed_types)]`, so that the workspace no-suppressions gate (quality-bar gate 12) passes with the ban on `std::process::Command` enforced everywhere else.

R15. WHEN the shared golden builder configuration is spawned on a given platform runner, the system shall produce the asserted observable child behavior (same program, args, cwd, env, exit status) for that platform, and the normalized builder-config snapshot emitted by each platform runner shall be identical across runners except for the Windows-only `CREATE_NO_WINDOW` flag and optional `JobObject`.

## Acceptance Criteria
| # | Criterion (maps to R#) | Status |
|---|---|---|
| 1 | `blocking::Command` exposes every named builder method and compiles as a drop-in for `std::process::Command` (R1) | planned |
| 2 | `r#async::Command` exposes the same surface with future-returning spawn/status/output (R2) | planned |
| 3 | Default stdio matches std for `status` (inherit) and `output` (piped) (R3) | planned |
| 4 | A child spawned with `current_dir(tmp)` reports `tmp` as its cwd (R4) | planned |
| 5 | `env`/`envs`/`env_remove`/`env_clear` are reflected in the child's observed environment (R5) | planned |
| 6 | On Windows, spawned children carry `CREATE_NO_WINDOW` (R6) | planned |
| 7 | On Windows, `kill_on_parent_process_close(true)` children die when the parent handle closes (R7) | planned |
| 8 | On Unix, the spawn adds no extra flags/job object vs std (R8) | planned |
| 9 | `spawn` of a missing program returns `Err(io::Error)`, no panic (R9) | planned |
| 10 | `status` returns the child's `ExitStatus` (R10) | planned |
| 11 | `output` returns captured stdout/stderr + `ExitStatus` (R11) | planned |
| 12 | `ExitStatus`, `Output`, `Stdio` re-exported at crate root (R12) | planned |
| 13 | `is_wsl_from` returns the correct boolean for both a WSL-marker and a non-WSL reader on any host; `is_wsl()` delegates to it (R13) | planned |
| 14 | The only `std::process::Command` allow lives in this crate with a justification; the no-suppressions gate stays green (R14) | planned |
| 15 | Shared golden builder-config fixture: per-runner observable child matches the golden expectation, and the cross-runner normalized snapshots diff to identity modulo the Windows-only no-window/job (R15) | planned |

## Visual / Behavioral Acceptance
N/A — headless infrastructure crate; no UI surface.

## Test Plan
- **Unit:**
  - `R1_blocking_builder_surface_present` — construct via every builder method; assert the configured command spawns a real non-PTY child (e.g. echo) and round-trips.
  - `R2_async_builder_surface_present` — same, awaiting the async spawn/status/output futures under a Tokio test runtime.
  - `R3_default_stdio_matches_std` — `status` inherits; `output` captures piped stdout.
  - `R4_current_dir_applied` — spawn a `pwd`/cwd-printing child; assert reported dir equals the set tmp dir.
  - `R5_env_mutations_applied` — set/remove/clear envs; spawn an env-printing child; assert exact deltas.
  - `R6_windows_create_no_window_flag` (cfg windows) — assert the creation-flags carry `CREATE_NO_WINDOW`.
  - `R7_windows_job_object_kills_on_parent_close` (cfg windows) — spawn long-lived child with `kill_on_parent_process_close(true)`; drop/close parent handle; assert child terminates.
  - `R8_unix_transparent_passthrough` (cfg unix) — assert no added creation flags / no job object handle present beyond std.
  - `R9_spawn_missing_program_errs` — spawn a non-existent program; assert `Err(io::Error)`, no panic.
  - `R10_status_returns_exit_status` — child with known exit code; assert `ExitStatus` code.
  - `R11_output_captures_streams` — child writing to stdout+stderr; assert both captured plus `ExitStatus`.
  - `R12_reexports_present` — name-resolution test referencing `marley_command::{ExitStatus, Output, Stdio}`.
  - `R13_is_wsl_from_both_branches` — call `is_wsl_from` with an injected reader containing a WSL kernel marker (expect `true`) and a reader without it (expect `false`); both run on any host. A thin `R13_is_wsl_delegates` asserts `is_wsl()` reads the real source through `is_wsl_from`.
  - 100% coverage on this crate's touched lines per compiled platform.
- **Integration (non-PTY seam):**
  - `git_subprocess_spawn_seam` — a non-PTY consumer (git/helper) builds its child process through `blocking::Command` (and the async twin through `r#async::Command`), runs it in a fixture cwd with a configured env, and reads its captured output + exit status (exercises R1/R2/R4/R5/R10/R11 across the real spawn seam). No PTY is allocated; this crate is exercised exactly as git/helper spawns use it.
- **Cross-platform parity (R15):**
  - `R15_golden_builder_config_parity` — a single shared fixture `golden_spawn_config` (program/args/cwd/env) is materialized on each platform runner. Per-runner assertion: the spawned child's observable behavior matches the golden expectation. A CI step then diffs the normalized builder-config snapshot each runner emits and requires identity except for the Windows-only `CREATE_NO_WINDOW` flag and optional `JobObject`. This maps R15 (previously unmapped) to the shared golden fixture diffed across the Unix and Windows runners.
- **Visual:** N/A.
- **Regression:** the no-suppressions gate (quality-bar gate 12) stays green — the only `#[allow(clippy::disallowed_types)]` lives in this crate (R14); the workspace ban on `std::process::Command` elsewhere keeps compiling.

## Mutation Targets
`cargo-mutants` (MSI 100% on the testable surface) must kill mutants that:
- flip the `CREATE_NO_WINDOW` flag set/clear and the `kill_on_parent_process_close` boolean (Windows runner).
- swap the Unix passthrough into adding flags (Unix runner).
- short-circuit `spawn` error propagation into a panic/`unwrap` or an `Ok` default (R9).
- replace the `status`/`output` return with a default `ExitStatus`/empty `Output` (R10/R11).
- negate or constant-fold the WSL-marker test inside `is_wsl_from`, or make `is_wsl()` bypass `is_wsl_from` (R13).
- drop any `env`/`current_dir` application (R4/R5).
- alter the normalized golden-config snapshot so a cross-runner divergence would slip past (R15).

ACCEPTED-UNTESTABLE:
- The `cfg(windows)` creation-flag and `JobObject` lines (R6/R7) are exercised only on a Windows CI runner; on macOS/Linux they are `cfg`-compiled-out and therefore not touched lines on those hosts. The closed decision: their coverage and mutation kills are **required on the Windows runner job**, not waived — no host pretends to cover code it did not compile.
- R15 cross-OS *equivalence* cannot be observed from a single host (no run sees both OSes at once). It is verified by the shared `golden_spawn_config` fixture: each platform runner asserts the per-platform observable child and emits a normalized config snapshot; the CI parity step diffs the two runners' snapshots for identity modulo the documented Windows-only flag/job. This is the named verification — a closed decision, not a gap.

## Dependencies
- REUSE (permissive, all MIT/Apache — map 1:1 to the `cargo deny` license allowlist): `tokio` (test runtime + async integration), `async-process` + `futures-lite` (async spawn substrate), `libc` (Unix), `win32job` + `windows` (Windows `JobObject` / `CREATE_NO_WINDOW` creation flag), `log`.
- Marley components: none — zero internal dependencies. This is foundational substrate for the workspace's **non-PTY** child spawns (git, helper subprocesses, the WSL probe). `marley_terminal` does **not** depend on this crate for PTY spawn (it uses `alacritty_terminal::tty`; seam-contracts §4.2).

## Delivery status

- **TICKET-004 (forge #7)** shipped the cross-OS-portable surface **Unix-only** (R1–R5,
  R8–R14, R15-partial); GATE GREEN [full], coverage 100 / MSI 100. No `#[cfg(windows)]`.
- **TICKET-004b (forge #6)** — the Windows half (R6 `CREATE_NO_WINDOW`, R7 `JobObject` /
  `kill_on_parent_process_close`, the `windows` helper module, `win32job`/`windows` deps)
  is **deferred / blocked on a Windows CI runner**: on a single-OS runner cargo-mutants
  reports the cfg'd-out Windows mutants as MISSED → MSI < 100, and §0 bars exclusions.
- The `marley_command::unix` helper module is **not needed** at M1 (the Unix passthrough
  adds nothing over `std::process::Command` — R8 is satisfied structurally).

## Out of scope / deferred
- **PTY allocation, shell selection, and the interactive terminal grid** — owned entirely by `marley_terminal` via `alacritty_terminal::tty` (seam-contracts §4.2). This crate spawns no PTY and provides no shell-into-PTY path at M1.
- **WASM target** — the wasm build is out of scope at M1 (the async spawn substrate is non-wasm, and the local-first charter has no wasm subprocess consumer). Picked up only if a wasm consumer is introduced; no `WHERE target is wasm` behavior is specified here.
- Package renaming churn (`command` → `marley_command` across dependents) is mechanical and tracked separately; this spec fixes the crate name as `marley_command` for the seam contract.
- The empty `test-util` feature surface — deferred until a consumer needs spawn fakes.

## Clean-room provenance
Behavior-derived from a fork-reference doc, IP-counsel sign-off pending (`clean_room` frontmatter).
Spec'd from the behavior-only description in `spec_source` — observable I/O only (cross-OS non-PTY
child spawn with builder-controlled program/args/cwd/env/stdio; Windows console-window suppression
+ parent-bound child lifetime; WSL detection), with **no** fork module/type/static names and **no**
fork file paths. The `blocking`/`r#async` module split is a std-mirroring naming choice tracking
`std::process` / `async_process`, not a transcription of any fork layout. The fresh Rust
implementation REUSES permissive crates (`tokio`, `async-process`, `futures-lite`, `libc`,
`win32job`, `windows`, `log` — all MIT/Apache) and is written from this spec, not translated from
any AGPL source.
