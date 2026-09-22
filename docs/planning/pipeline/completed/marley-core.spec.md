---
pipeline_id: 766f41dd-9af6-4f96-81e1-0bd383c079e2
ticket: forge#5 (644bd932-eb18-4e13-9883-03983c2dbaae) · local docs/planning/tickets/open/TICKET-003-marley-core.md
aar_id: 5bb453d9-7fc1-40d4-ad83-af3ffdf1aeaf
status: Phase 5 — Complete PASS
title: marley_core — app/runtime core (SessionId, paths, Config, feature flags)
type: feature
milestone: M0
references:
  - ../../../specs/SPEC-marley_core.spec.md
  - ../../../specs/standards/seam-contracts.md
  - ../../../specs/standards/quality-bar.spec.md
---

## Title

TICKET-003 — `marley_core`. The offline-boot app/runtime core: `SessionId`
(seam-contracts §2 owner), the `~/.marley` filesystem layout, a single lazily-built
`Config` (one channel; no cloud/server/auth), and the process feature-flag registry.
Composes the M0 leaves; `marley_terminal` is the downstream `SessionId` consumer.

**Contract authoritative in [`SPEC-marley_core.spec.md`](../../../specs/SPEC-marley_core.spec.md)
(R1–R18).** This pipeline doc adopts it verbatim.

## Scope
### In (4 modules)
- `session_id` — `SessionId(u64)` transparent newtype; `next()` process-unique
  monotonic (atomic); `as_u64`; `From<u64>` / `From<SessionId>`; `Copy/Clone/Eq/Hash/Debug`.
- `paths` — `marley_home_dir()`=`<home>/.marley`; `config|data|themes|skills` children;
  `mcp_config_file_path`=`config/mcp.json`; `logfile_path`=`data/<logfile_name>`;
  `PathError::HomeDirUnresolved` (no panic on unresolved home).
- `config` — `AppId` (as_str), `Config::marley()` lazy pointer-stable (once_cell),
  `app_id()`/`logfile_name()`; **no server/cloud/auth field** (single channel).
- `features` — `FeatureFlag` enum; `is_enabled` = override → user-preference →
  baseline → false; `set_enabled`/`set_user_preference`; `override_enabled`→
  `OverrideGuard` (RAII, thread-local, `test-util`); `apply_default_flags`;
  `mark_initialized`; `DEFAULT_FLAGS`; R17 debug-panic if read before init.
- Deps: `directories`, `once_cell`, `serde`, `enum-iterator` (0BSD); dev `serial_test`.

### Out (per spec)
Multi-channel matrix, server/RTC/IAP/telemetry/MCP-OAuth, autoupdate, OS-info,
flag-preference disk persistence (M1 settings), `ShellSessionId` (owned by marley_terminal).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — spec is the contract.** R1–R18 = the AC + test list 1:1.
- **D2 — testable home seam (R5/R6).** private `home_dir_from(Option<PathBuf>) ->
  Result<PathBuf,PathError>` (Some→`join(".marley")`, None→`Err(HomeDirUnresolved)`);
  public `marley_home_dir()` = `home_dir_from(directories home)`. **No `$HOME` env in
  tests** (races under the threaded mutation runner). Applies PR-claude-absolutize-via-injected-cwd-seam-001.
- **D3 — flag-state determinism under threaded mutation.** `mutation_g` uses `cargo
  test` (threaded, one process); global baseline/user-pref/init-guard contaminate
  across threads → cargo-mutants baseline would fail closed. So the global-state tests
  (R12/R13/R16/R17) use **`serial_test::#[serial]` + a `#[cfg(any(test,feature="test-util"))]`
  reset seam** clearing baseline+user-pref+init between them. The override layer
  (R14/R15) is thread-local → already isolated. (Confirmed in design via a spike.)
- **D4 — R9 & R18 ACCEPTED-UNTESTABLE compile-time** (spec-closed): `Config` has no
  network/auth field and flag-storage tracks the variant count — verified by a
  compile-time assertion (e.g. `const _` / `static_assertions`), **no runtime branch →
  no viable mutant** (excluded by construction, NOT an exclusion file).
- **D5 — R17 debug-panic** per PR-claude-debug-panic-release-defined-001: structure so
  the debug-coverage build covers the panic line (`#[should_panic]`), no equivalent mutant.
- **D6 — enum-iterator 0BSD** → `deny.toml` `[licenses] allow += "0BSD"` with a
  justification comment (gate-defining file; a later `.rs` commit re-verifies the receipt).
- **D7 — §21 doc phase:** CHANGELOG entry + `docs/marley_architecture/marley_core.md` at complete.

## Acceptance Criteria (EARS)
All of **R1–R18** in [SPEC-marley_core.spec.md](../../../specs/SPEC-marley_core.spec.md),
verified by the spec's named tests. Plus the bar:

| # | Bar | Verify |
|---|---|---|
| AC-A | All R1–R18 tests pass (serial where global) | `cargo nextest run -p marley_core` |
| AC-B | R9/R18 compile-time structural assertions hold | build (a violation fails to compile) |
| AC-C | 100% line coverage on the testable surface | gate:4 |
| AC-D | Mutation MSI 100 on the runtime surface (R9/R18 carry no mutant) | gate:5 |
| AC-E | deny/audit/machete clean incl. 0BSD allowance | gate:7/8/9 |
| AC-F | FULL `scripts/gates.sh` → `GATE GREEN [full]` + receipt | /commit |
| AC-G | CHANGELOG + docs/marley_architecture/marley_core.md (§21) | enforce-changelog + inspect |

## Phase Plan
- **P2 Design** — module layout; the flag state machine (storage sized by variant
  count; the 3-layer resolution; OverrideGuard Drop; the reset seam) + a **spike**
  confirming determinism under `cargo test` threads; the home seam; the once_cell
  Config; the R9/R18 compile-time assertions; deps + deny 0BSD; test manifest (one
  row per R + the serial/threaded-isolation plan + the mutation map).
- **P3 Implement** — `session_id.rs`/`paths.rs`/`config.rs`/`features.rs`/`lib.rs` + Cargo.toml + deny.toml.
- **P3.5 Inspect** — critics: flag-resolution correctness + thread-safety, mutation/coverage readiness (the global-state determinism!), clean-room, no-network-field.
- **P4 Validate** — R1–R18 tests; FULL gate green (cov 100 / MSI 100).
- **P5 Complete** — CHANGELOG + arch doc; AAR; archive; close #5.
