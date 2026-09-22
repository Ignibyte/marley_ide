---
pipeline_id: 42a54333-a396-4a36-b274-d65b0c71c39f
ticket: forge#11 (a8b226f5-a62b-4f28-a637-2e0bd6a00220) · local docs/planning/tickets/open/TICKET-006-foundation-spike.md
aar_id: eddc7ec8-b689-42c2-811e-1dc9f35365c3
status: Phase 5 — Complete PASS
title: marley_spike — gpui + alacritty_terminal + vte viability spike (throwaway)
type: spike
milestone: M0
references:
  - ../../../specs/SPEC-foundation-spike.spec.md
  - ../../../pipeline/visual-testing.spec.md
---

## Title

TICKET-006 — `marley_spike`. The **throwaway** end-to-end viability probe that proves the REUSE
stack (`gpui` window + `alacritty_terminal` PTY + `vte` parse) composes into the minimum observable
loop: open one window, spawn a shell on a PTY, stream its bytes through the parser, render one
per-command output **Block**. Retires integration risk before any product code; deleted once the M0
gate is green. Contract: [`SPEC-foundation-spike.spec.md`](../../../specs/SPEC-foundation-spike.spec.md)
(R1–R16). Built ON `marley_visual_harness` (forge #10) — its `visual_acceptance` is asserted via that
harness.

## Scope

### In (this ticket)
- **Pure decision surface** (100% cov + MSI 100, headless): `build_block` (R7); `Block`/`BlockStatus`
  (Running/Exited(i32)/Failed); `classify_read(&io::Error)->ReadOutcome{Retry,Fatal}` (R12/R13);
  `exit_code_to_status(Option<i32>)->BlockStatus` (R6); `ShutdownOutcome{Clean,SpawnFailed,ReadFatal}`
  + `exit_code_for(ShutdownOutcome)->ExitCode` (R11/R16); `SpikeConfig`/`GridSize` (R2/R4); a **pure
  grid-row extractor** (a vte `Term` grid → the body String) so R5's parse result is unit-testable
  from a synthesized grid.
- **Display/integration shim** (ACCEPTED-UNTESTABLE — `mutants::skip` + the `rust_cov`
  `--ignore-filename-regex` exclude, the marley_visual_harness precedent): `run()->ExitCode` (the gpui
  `Application::run` + the one window "Marley Spike" 800×600 rendering the Block header-above-body +
  the drive loop, R1/R2/R9/R10); the `alacritty_terminal` PTY spawn of `/bin/sh -c "echo hello-marley"`
  at 24×80 + the leader-fd read (R3/R4/R5); window-close → kill child + quit (R15). Kept thin behind
  injectable seams.
- **Tests:** pure unit (R6/R7/R11/R12/R13/R16 + build_block + the grid extractor); a **headless PTY
  integration test** (R3+R4+R5 — spawn the PTY with NO window, drain the leader fd through the parser,
  assert the grid's first row contains "hello-marley" + dims 24×80); a **headed visual test** (R2/R9/R10)
  via `marley_visual_harness` (`#[ignore]`, spawns the marley_spike bin, asserts the AX window + the
  one-Block screenshot baseline — generated + committed like the harness).
- **§21** — CHANGELOG + `docs/marley_architecture/marley_spike.md`.

### Out / deferred (per the spec's Out-of-scope)
- Multiple Blocks, block boundaries (DCS/OSC), session association, keyboard input, the PTY write path,
  theming, scrolling, IPC. R14b (no keyboard) is ACCEPTED-UNTESTABLE (absence of a code path).
- The crate is THROWAWAY (deleted once the M0 gate is green; nothing depends on it).

## Acceptance Criteria (EARS — adopt SPEC-foundation-spike R1–R16)
The authoritative clauses are R1–R16; this pipeline adopts them. The shippable subset:
- **AC-1 (R6)** — `exit_code_to_status(Some(0))==Exited(0)`, `Some(3)==Exited(3)` (test asserts the
  carried CODE, not `.success()` — the 004 trap), `None`→ the running/failed mapping per spec.
- **AC-2 (R7/R8)** — `build_block` sets all three fields; before exit, status is `Running` and holds no code.
- **AC-3 (R12/R13)** — `classify_read(WouldBlock|Interrupted)==Retry`; any other kind ==`Fatal`.
- **AC-4 (R11/R16)** — `exit_code_for(Clean)==ExitCode::SUCCESS`; every other outcome → non-zero.
- **AC-5 (R5)** — the pure grid-row extractor returns "hello-marley" from a synthesized Term grid.
- **AC-6 (R3/R4/R5 — headless integration)** — spawning `/bin/sh -c "echo hello-marley"` via
  alacritty_terminal at 24×80, draining the leader fd through the parser, yields a grid whose first row
  contains "hello-marley" and reports dims 24×80.
- **AC-7 (R2/R9/R10 — headed, via the harness)** — the launched window has AXTitle "Marley Spike",
  inner size 800×600, and the one-Block layout matches the committed screenshot baseline.
- **AC-8 (gate)** — FULL `scripts/gates.sh` → `GATE GREEN [full]` (15 gates; **gate-15 now ENFORCES** —
  runs `cargo nextest -p marley_visual_harness`); cov 100 / MSI 100 on the testable surface; the gpui/PTY
  shim ACCEPTED-UNTESTABLE.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
1. **Reuse the harness's ACCEPTED-UNTESTABLE pattern** for the gpui/PTY shim (`mutants::skip` + the
   `rust_cov` exclude for the shim file(s)), so the pure logic + the grid extractor stay 100/100.
2. **Visual_acceptance via marley_visual_harness** — `HeadedSession::launch(bin, "Marley Spike", …)` +
   AX window assert + screenshot baseline; inner header-above-body rides the screenshot (the gpui
   window-level-AX finding, AD-claude-gpui-headed-visual-testing-subprocess-001).
3. **Headless PTY integration** — spawn with NO window (the spec's R3 note: "exercised headless");
   real child → `#[serial]` (cargo-mutants threaded test).
4. Deps: `gpui`, `alacritty_terminal="0.26"`, `vte="0.15"`; dev `marley_visual_harness` (path) +
   `serial_test` + `tempfile`. alacritty/vte are Apache/MIT; gpui tree already allowlisted.

## Phase Plan
- **P2 Design** — SPIKE the alacritty_terminal 0.26 PTY API FIRST (spawn+winsize, leader-fd read,
  vte→grid, row extract — scratchpad). Then the module manifest, the pure/shim seam, the regression
  test plan + mutation map. If headless PTY-spawn-read-grid is infeasible in a test, surface it.
- **P3 Implement** — the pure surface + the thin shim + the bin + deny/rust_cov wiring.
- **P3.5 Inspect** — adversarial critics (the exit-code-status trap; classify_read; the grid parse;
  clean-room; the shim mutation-readiness).
- **P4 Validate** — write the tests; generate+commit the baseline; FULL gate green.
- **P5 Complete** — §21; knowledge; close #11; archive.
