# marley_command — Notes

- **Forge ticket:** #7 `8ce39d6e-8632-4a44-a8fe-f5255c15f913` (feature, M0, Unix-only), claimed `dc7df9b5-…`.
- **AAR:** `5cc28178-8f9c-4401-ab29-946f690c2d17`.
- **Local ticket doc:** `docs/planning/tickets/open/TICKET-004-marley-command.md`.
- **Pipeline spec:** `marley-command.spec.md` (pipeline_id `18731558-…`).
- **Branch:** `ticket-004-marley-command` (stacked on 003).
- **Windows half:** forge#6 / TICKET-004b (BLOCKED on a Windows CI runner).

## Phase 1 — Plan

- **Request:** `marley_command` non-PTY spawn seam, **Unix-only** (chad's decision); the
  Windows console-flash/JobObject half (R6/R7) → 004b, because cargo-mutants generates
  surviving mutants for `#[cfg(windows)]` code on the macOS-only runner ([[cross-platform-mutation-single-runner]]).
- **Classification / tier:** work pipeline, **feature** (adapter crate; process-spawn
  confined here per §14).
- **Forge recall (§18.3):** the cfg-fn-mutant failure (`BF-cfg-gated-fn-unkillable-mutant-001`)
  is the reason for the Unix-only scope. Apply PR-…-injected-cwd-seam (is_wsl_at temp-file),
  AD-…-stateful-crate-serial (env #[serial] for is_wsl), PR-…-assert-no-field (N/A here).

### Confirmed pre-flight facts
- **No `clippy.toml`** → add one (workspace root) with `disallowed-types = std::process::Command`.
  gate:13 bans only transmute/unsafe; the std-Command ban rides on clippy gate:2 (-D warnings).
  No other crate uses `std::process::Command`. The single allow lives in the `blocking` module
  (module-level `#![allow(clippy::disallowed_types)] // justification`, gate:12-clean).
- **`clippy.toml` not in the receipt fingerprint** (`gate_state_hash` covers scripts/*.sh,
  .claude/hooks/**, deny.toml, .gitleaks.toml, Cargo manifests+lockfile — NOT clippy.toml).
  → flag for inspect: consider folding clippy.toml into the fingerprint.

### Carry to Design
- **WSL (R13)** — three-layer seam: `is_wsl_from(reader)` (marker, public; both branches via
  injected readers) → `is_wsl_at(path)` (open+delegate, private; tested with **tempfile** —
  marker/plain/nonexistent → covers the map-closure + unwrap_or + true/false on macOS) →
  `is_wsl()` reads `MARLEY_KERNEL_VERSION_PATH` env (else `/proc/version`) so a `#[serial]`
  test drives `is_wsl()`'s `→false` constant-fold mutant dead (on macOS it's else-always-false).
- **blocking::Command** holds a `std::process::Command`; each builder method delegates +
  returns `&mut self`; spawn/status/output forward (R3 default stdio = std's). R9: forward
  the io::Result (no unwrap).
- **r#async::Command** holds an `async_process::Command`; async spawn/status/output forward.
  Test via `futures_lite::future::block_on`. SPIKE the async-process builder API + reactor.
- **R15-partial** — a `golden_spawn_config` (program/args/cwd/env) materialized + the
  per-platform observable asserted on macOS; the cross-runner snapshot diff is 004b/CI.
- **Deps:** `async-process` + `futures-lite`; dev `tempfile` + `serial_test`. Verify each
  USED (machete); drop tokio/libc/log/win32job/windows.

### §21 reminder
`.rs` ticket → enforce-changelog needs a staged CHANGELOG entry; complete adds
`docs/marley_architecture/marley_command.md`.

**Phase 1 status:** PASS (autonomous-through-commit per session goal). → Phase 2 Design.

## Phase 2 — Design

### Spikes (scratchpad) — confirmed
- **async-process 2.5 + futures-lite 2.6:** `futures_lite::future::block_on(async {
  async_process::Command::new(..).arg().current_dir().env().output().await })` works with
  **no tokio** (own reactor); full builder surface (args/envs/env_remove/env_clear/stdin…).
- **clippy disallowed-types:** `clippy.toml disallowed-types=[{path="std::process::Command",
  reason=…}]` makes `clippy -D warnings` **error** "use of a disallowed type"; a module-level
  `#![allow(clippy::disallowed_types)] // justification` clears it AND passes the gate:12 grep
  (same-line justification, `disallowed_types` not a blanket group).
- **is_wsl_at** (proven pattern): `File::open(path).map(is_wsl_from).unwrap_or(false)` with temp
  files covers all branches on macOS; env-override kills `is_wsl()`'s `→false`.

### Architecture — modules (process-spawn confined here, §14)
- **`blocking.rs`** — `#![allow(clippy::disallowed_types)] // marley_command is the OS-parity
  spawn seam (seam-contracts §4.2); std::process::Command is banned elsewhere`. `pub struct
  Command { inner: std::process::Command }`; `new(program)`; builder methods delegate to
  `inner` + `-> &mut Self` (`arg/args/env/envs/env_remove/env_clear/current_dir/stdin/stdout/
  stderr`); `spawn(&mut self) -> io::Result<std::process::Child>`, `status -> io::Result<ExitStatus>`,
  `output -> io::Result<Output>` forward (R9 no unwrap). No stdio set by default (R3 = std's). No
  platform code (R8 passthrough trivially).
- **`r#async.rs`** — `pub struct Command { inner: async_process::Command }`; same builder; async
  `spawn/status/output` forward the futures. (async_process ≠ std → no clippy allow.)
- **`wsl.rs`** — `pub fn is_wsl_from(mut r: impl Read) -> bool` (read_to_string; lossy-lowercase
  `contains("microsoft")`); `fn is_wsl_at(path: &Path) -> bool`
  (`File::open(path).map(is_wsl_from).unwrap_or(false)`); `pub fn is_wsl() -> bool` (reads env
  `MARLEY_KERNEL_VERSION_PATH` else `/proc/version`, then `is_wsl_at`).
- **`lib.rs`** — `#![deny(missing_docs)]`; `pub mod blocking; pub mod r#async; pub mod wsl;`
  `pub use std::process::{ExitStatus, Output, Stdio};` (R12); crate docs.

### File manifest
| File | Change |
|---|---|
| `crates/marley_command/src/lib.rs` | re-exports + mod decls + docs |
| `crates/marley_command/src/blocking.rs` | blocking::Command (std wrapper + the allow) |
| `crates/marley_command/src/async.rs` (as `r#async`) | async::Command (async_process wrapper) |
| `crates/marley_command/src/wsl.rs` | is_wsl_from / is_wsl_at / is_wsl |
| `crates/marley_command/Cargo.toml` | async-process, futures-lite; dev tempfile, serial_test |
| `clippy.toml` (workspace root) | `disallowed-types = [std::process::Command]` |

### Regression test plan (one row per IN R; spawn REAL children to observe delegation)
| Test | R | kills |
|---|---|---|
| `r1_blocking_builder_spawns` (every builder method; `/bin/echo` arg) | R1 | builder delegation deletes |
| `r2_async_builder_spawns` (block_on) | R2 | async forward |
| `r3_default_stdio` (status inherits; output captures piped stdout) | R3 | stdio default |
| `r4_current_dir` (`sh -c pwd` in a tempdir → stdout == dir) | R4 | **current_dir delete** |
| `r5_env` (`sh -c 'printf %s "$FOO"'` env(FOO,bar)→"bar"; +remove/clear) | R5 | **env delete** |
| `r9_spawn_missing_errs` (blocking + async; bogus program → `Err`, no panic) | R9 | **spawn error→Ok/panic** |
| `r10_status_code` (`sh -c 'exit 3'` → code 3) | R10 | **status→default** |
| `r11_output_streams` (`sh -c 'echo o; echo e >&2'` → stdout "o", stderr "e") | R11 | **output→empty** |
| `r12_reexports` (name-resolution `marley_command::{ExitStatus,Output,Stdio}`) | R12 | — |
| `r13_is_wsl_from_both` (reader "…Microsoft…"→true; "Linux"→false) | R13 | **contains negate/const** |
| `r13_is_wsl_at_tempfiles` (marker-file→true; plain→false; nonexistent→false) | R13 | map/unwrap_or |
| `r13_is_wsl_env` (**#[serial]**: env→marker tempfile→true; env-unset→false) | R13 | **is_wsl →false / →true** |
| `r15_golden_spawn_config` (program/args/cwd/env observable on macOS) | R15-p | parity (per-platform) |

- **R14** has no unit test — verified by the FULL gate: clippy (gate:2) passes WITH the workspace
  ban configured AND the single justified module allow; gate:12 stays green.
- **R8** (Unix passthrough) is structural (no platform code → the wrapper *is* std) — covered by R1/R10/R11.

### Risks
- **R1 — `std::env::set_var` edition.** If the workspace is edition 2024, `set_var` is `unsafe`
  → the `r13_is_wsl_env` test needs `unsafe { … } // SAFETY: test-only, #[serial]` (gate:13 allows
  unsafe WITH a SAFETY: comment). **Check the edition at implement**; if 2021, it's safe.
- **R2 — clippy.toml not in the receipt fingerprint** — flag for inspect (fold into `gate_state_hash`).
- **R3 — module name `async`** is a keyword → file `async.rs` + `pub mod r#async;`.

**Phase 2 status:** PASS. → Phase 3 Implement.

## Phase 3 — Implement

Built `crates/marley_command/src/{lib,blocking,async,wsl}.rs` + `Cargo.toml` + the new
workspace `clippy.toml`. Clean: `cargo check --workspace`, `clippy --workspace
--all-targets -D warnings` (the disallowed-types ban is live workspace-wide; the
`blocking` module's justified allow silences it; **no other crate breaks**), `fmt`,
`machete` (no unused), `deny licenses` (async-process tree all permissive).

### Deviations / notes
1. **`r#async::Command::spawn` is SYNC** (`-> io::Result<Child>`), not async. The spec
   says "spawn/status/output returning futures", but `async_process::Command::spawn` is
   itself synchronous (fork/exec returns immediately; the async work is *waiting*). A
   faithful drop-in mirrors that: `spawn` sync, `status`/`output` async. (If a future-
   returning spawn is wanted, it's a trivial wrap — flagged for inspect.)
2. **Edition 2021** → `std::env::set_var` is **safe** (not unsafe) — the validate-phase
   `is_wsl` env test needs no `unsafe`/SAFETY.
3. **`futures-lite` is a dev-dep** (only tests use `block_on`); `async-process` is the
   one runtime dep. dev: `tempfile`, `serial_test`, `futures-lite`.
4. **`clippy.toml`** (workspace root) bans `std::process::Command`; the single allow is
   `blocking.rs`'s module-level `#![allow(clippy::disallowed_types)] // …§4.2…`.
   (clippy.toml is NOT in the receipt fingerprint — carried for inspect.)

### As-built
- `blocking::Command { inner: std::process::Command }` + the full builder; spawn/status/
  output forward `io::Result` (no unwrap). No default stdio (R3 = std's). No platform code.
- `r#async::Command { inner: async_process::Command }`; async status/output via
  `futures_lite::block_on` in tests; `std::process::Stdio` converts into the async stdio.
- `wsl`: `is_wsl_from(reader)` (lossy-lowercase `contains("microsoft")`), private
  `is_wsl_at(path)` (`File::open.map(is_wsl_from).unwrap_or(false)`), `is_wsl()` (env
  `MARLEY_KERNEL_VERSION_PATH` else `/proc/version`).
- `lib`: re-exports `ExitStatus/Output/Stdio` (R12). No `unsafe`, no `#[cfg(windows)]`.

**Phase 3 status:** PASS. → Phase 3.5 Inspect.

## Inspect (Phase 3.5)

2 critics (mutation/coverage · clean-room/R14/receipt-gap), both VERIFYING empirically
(`cargo mutants --list` real run; `cargo clippy --workspace`; `cargo deny`; the
gate_state_hash sensitivity probe).

| # | Sev | Finding | Verdict | Action |
|---|---|---|---|---|
| 1 | **MED** | `clippy.toml` (the SOLE R14 enforcement) is gate-defining but NOT in the commit-receipt fingerprint → a config-swap-around-the-green exploit | **REAL** (this ticket makes clippy.toml gate-defining) | **FIXED**: added `clippy.toml` to both `ls-files` pathspecs in `gate_state_hash` (lib-hook-helpers.sh) + the header; verified an edit now changes the hash (`7fd4…`→`b618…`). forge `BF-clippy-toml-not-in-receipt-fingerprint-001` + `PR-claude-gate-defining-files-in-receipt-fingerprint-001`. |
| 2 | **HIGH (validate)** | `status`'s `Ok(Default::default())` mutant = `ExitStatus::default()` = **success/code 0** → a `success()`/exit-0 assertion can't kill it | **REAL** (empirically MISSED) | Carry to validate: assert a **non-zero `code()`** (`sh -c 'exit 3'`→`Some(3)`) for status, in **both** blocking AND async (separate awaits). |
| 3 | INFO | Only **8 viable mutants** (status×2, wsl×6); all builder/spawn/output mutants UNVIABLE (`Command/Child/Output: !Default`) | **TRUE** | Spawn-and-observe tests still needed — for **line coverage** (execute every body), not for kills. |
| 4 | LOW | async `spawn` is SYNC, not future-returning (R2 literal) | **CORRECT as-is** | `async_process::Command::spawn` is itself sync (fork/exec is immediate; status/output are the async ops) — a future-wrapped spawn would break drop-in parity. **Adjudication: keep sync; the spec's R2 wording should be amended** to "spawn → io::Result<Child> (sync, mirroring async_process); status/output → futures". Note in the arch doc. |
| 5 | LOW | `async_process::Command` not in the ban (async half unenforced) | **REAL but zero-risk today** | No other crate deps async-process; hardening beyond R14's text → defer (add when async-process broadens). |
| 6 | LOW | spec public surface lists `unix`/`windows` helper modules this impl omits | **EXPECTED** | Unix passthrough needs no `unix` module (R8 structural); `windows` = 004b. Note in arch doc. |

**Verified PASS:** R14 ban (clippy green with ban+allow; no other crate uses
std::process::Command incl tests; allow justified+specific, gate:12-clean); clean-room
(all Marley-original/std-mirroring; `MARLEY_KERNEL_VERSION_PATH` original; no fork
taxonomy); licenses (async-process/futures-lite/tempfile/serial_test all MIT/Apache, no
copyleft). Zero `#[cfg(windows)]`/platform-cfg confirmed (no surviving-mutant problem).

### Carry to VALIDATE (the precise kill/coverage map)
- **status (both wrappers, the HIGH):** spawn `sh -c 'exit 3'`, assert `status.code() ==
  Some(3)` — NOT `success()`. Async: its own `block_on` of a non-zero-exit `status().await`.
- **wsl (6 mutants), both polarities each:** `is_wsl_from` marker-reader→true + non-marker→false
  (use a CAPITAL-"Microsoft" reader for fidelity); `is_wsl_at` marker-tempfile→true +
  nonexistent→false (private → **in-crate** test); `is_wsl` env→marker-tempfile→true +
  **env-unset**→false. The env tests are **`#[serial]`** (env races under the threaded
  mutation runner) and must `remove_var` (the unset arm `remove_var` first). The env-unset
  arm also covers the `/proc/version` fallback closure + kills `is_wsl→true`.
- **Coverage:** call **every** builder method (incl plural `args`/`envs`) in **both**
  blocking AND async; in async, the `let cfg: Stdio = cfg.into()` (stdin/stdout/stderr)
  lines + `status` AND `output` each awaited separately.
- R1/R3/R4/R5/R9-R12 spawn real children (`/bin/echo`, `sh -c pwd`, `sh -c 'printf %s "$FOO"'`).

Post-fix: shellcheck (lib-hook-helpers.sh) CLEAN; clippy/fmt/deny unaffected.

**Phase 3.5 status:** PASS. → Phase 4 Validate.

## Phase 4 — Validate

**Tests:** 16 in-crate `#[cfg(test)]` tests (blocking 8, async 3, wsl 4, lib 1) per the
inspect kill map — **16 pass** (nextest), 0 doctests.
- blocking: r1 (every builder), r3 (stdio), r4 (current_dir), r5 (env/remove/clear via
  HOME), r9 (spawn-missing→Err), **r10 (`status.code()==Some(3)` — the non-zero kill)**,
  r11 (streams), r15 (golden config).
- async: r2 (every builder via `block_on`), **r2_async_status (`code()==Some(3)`)**, r2 spawn-missing.
- wsl: is_wsl_from (both readers, capital-M), is_wsl_at (temp files, both + absent),
  is_wsl env (`#[serial]`: marker tempfile→true, unset→false, `remove_var` cleanup).
- lib: r12 re-exports resolve.

**Two reds fixed at source (not the gate):**
1. **gate:14 rustdoc** — `[`r#async`]` parsed as a broken link to `r` (the `#`) →
   rewrote as plain "the `r#async` module".
2. **gate:4 coverage** — the r12 test's `fn _accepts_stdio(_: Stdio) {}` was an uncalled
   fn body (uncovered) → replaced with `let _stdio: Option<crate::Stdio> = None;` (type
   resolution, no body).

**FULL gate:** `scripts/gates.sh` → **GATE GREEN [full], 16/16.**
- gate:4 coverage **100% lines**; gate:5 mutation **MSI 100%** (8 viable mutants —
  status×2 + wsl×6 — all killed; the builder/spawn/output mutants unviable: `!Default`).
- gate:2 clippy green **with the workspace `std::process::Command` ban active** (R14).
- gate:6 miri N/A (no unsafe); gate:15 visual N/A. Receipt written.

**Pre-existing failures:** none. **Adjudications (for complete/spec):** async `spawn` is
sync (mirrors `async_process`; R2 wording to be amended); the spec's `unix`/`windows`
helper modules are N/A here (R8 structural; Windows = 004b).

**Phase 4 status:** PASS — GATE GREEN [full]. → Phase 5 Complete.
