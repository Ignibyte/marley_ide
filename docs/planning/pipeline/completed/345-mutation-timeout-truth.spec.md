---
pipeline_id: 3ee9f0c7-9b0d-4cb0-97c5-d2b965da5854
ticket: forge#345 (8af264da-17ea-4974-8dd0-bceb389d6730) · local docs/planning/tickets/open/TICKET-345-mutation-timeout-truth.md
aar_id: 7a4476ce-188c-41dc-ab1c-d72133650546
status: Phase 5 — Complete PASS
title: gate:5 truth — mutants run under nextest, and a Timeout must prove it was a hang
type: chore
milestone: M22
references: [gates.sh:257-259 (the Timeout∈caught jq, verbatim), gates.sh:236 (margs --jobs 2 --no-times), gates.sh:247-251 (exit 0|2|3 accepted), cargo-mutants 27.1.0 --test-tool <cargo|nextest> (verified via --help), outcomes.json rows carry {scenario, log_path, summary, phase_results} (verified on the live mutants.out), doctests MEASURED ZERO runnable workspace-wide, the #337 evidence (3 Timeouts, all caught-by-assert mislabels), #348 (the .config/nextest.toml — ADDITIVE synergy, NOT a dependency; #345 runs first — see D3), #334 (the load-flaky baseline sibling)]
---

## Title
`scripts/gates.sh:258-259` counts a timed-out mutant as CAUGHT. Sound when the timeout means "the
mutant caused a hang"; NOT sound when it means "the suite is slow today" — a genuinely undetected
mutant that happens to time out is laundered into MSI 100, the one thing the floor exists to make
impossible. #337's run had 3 Timeouts; reading their logs showed all 3 were killed DECISIVELY by six
failing asserts — mislabeled only because plain `cargo test` doesn't fail-fast and the wall clock blew
cargo-mutants' deadline. MSI 100 was honest **by luck of inspection**. This ticket makes the classifier
tell the truth: run mutants under nextest (fail-fast, process-per-test), and audit every remaining
Timeout's log so a mislabel is VISIBLE instead of silently absorbed.

## Scope
### In
- **`--test-tool=nextest`** added to gate:5's `margs` (gates.sh:236) for BOTH modes (full + `--in-diff`).
  Verified in `cargo mutants --help` @ 27.1.0. Effects: (a) the process-per-test model the suite is
  already tuned for (gate:3 runs nextest); (b) fail-fast — a killed mutant is detected in seconds
  instead of riding out 626 tests to a deadline; (c) the load-flaky baseline (#334's family, exit 4
  under load) gets the more robust runner.
- **The Timeout truth audit** (belt-and-braces, additive): after the run, for each outcome with
  `summary=="Timeout"`, grep its `log_path` (field verified present in `mutants.out/outcomes.json`)
  for failing tests (`FAILED` / `assertion`); print
  `mutation: N timeout(s); M mislabeled (log shows failing tests — caught-by-assert, perf not detection)`.
  The Timeout∈caught CONVENTION stays (a genuine hang IS detection — Infection/Stryker), but the
  mislabel is now reported, never silent. The gate comment at :257 is rewritten to say all of this.
- **The #348 synergy (why the order is hard):** with `.config/nextest.toml`'s terminate ceiling in
  place, a mutant that induces a per-test hang gets KILLED by nextest → the suite FAILS → cargo-mutants
  records an honest `CaughtMutant`, no convention needed. Timeouts shrink toward the one residue —
  whole-run overrun — which the audit then covers.
- **The doctest question, measured shut:** nextest does not run doctests, so switching the mutant
  runner could drop a kill signal — IF any existed. Verified 2026-07-18: `cargo test --doc` runs
  **0 tests** in both fence-bearing crates (marley_lsp, marley_editor — `running 0 tests` each); the
  4 grep hits are mid-prose backticks (hover.rs:46/:87) and a ```` ```text ```` fence (comment.rs:84).
  Nothing is dropped. (If a future doctest becomes a sole killer, the mutant goes MISSED and the gate
  fails CLOSED — the §0-correct direction; fix = add a unit test.)
- Exit-code contract re-checked: the 0|2|3 case (gates.sh:247-251) is runner-independent; unchanged.
### Out (explicitly)
- Lowering `MUT_MSI_MIN` or excluding slow tests (§0 — the ticket forbids both; the floor is right,
  the classifier was weak). Changing the Timeout∈caught convention itself. cargo-mutants version bump.
  #335's temp-tree leak. The nextest config file itself (#348 owns `.config/nextest.toml`).

## Reference (§20)
N/A — Marley-specific gate integrity; no reference-app analog.

### Prior art
1. **Published material** — cargo-mutants documents `--test-tool nextest` as the supported runner
   swap; the Timeout∈caught stance is the Infection/Stryker MSI convention (the gate comment already
   cites it — the convention survives, gains an audit).
2. **OUR OWN CODE** — gate:3 has run nextest since inception (gates.sh:74-83: "nextest does not run
   doctests, so cargo test --doc is run alongside") — the doctest split is already the house model;
   gate:5 joining nextest CONVERGES the runners rather than adding one. The #337 log-reading that
   found the mislabels is the audit, done by hand once; this ticket automates it.
3. Checked gpui/ropey/regex — no owner (not their seam).

## Locked-In Decisions
- **D1-NEXTEST-RUNNER** — `--test-tool=nextest` in margs, both modes; no other margs change.
- **D2-AUDIT-NOT-RECLASSIFY** — a mislabeled Timeout still COUNTS caught (it was caught — by assert);
  the audit only makes the mislabel visible. No floor, no threshold, no new failure mode from the
  audit itself (§0: this is visibility, not a baseline).
- **D3-SYNERGY-NOT-DEPENDENCY** — *(downgraded at Phase 1 promotion — was D3-RIDES-348 "HARD order,
  #348 first"; that claim is FALSE, and the goal runs #345 BEFORE #348.)* `--test-tool=nextest` needs
  **no** `.config/nextest.toml`: it is ABSENT today (verified on `d90ff9c`) yet gate:3 (`tests_g`,
  gates.sh:80-82) runs `cargo nextest run --workspace` on every gate and passes — nextest requires none
  of #348's deliverable. So #345's runner swap + Timeout audit stand ALONE. #348's terminate ceiling is
  ADDITIVE synergy: once it lands, a mutant that induces a per-test hang gets KILLED by nextest → the
  suite FAILS → cargo-mutants records an honest `CaughtMutant`, shrinking the Timeout residue toward the
  one whole-run-overrun case the audit covers. #348's validate re-proves the pair together — but #345
  does not wait on it.
- **D4-FAIL-CLOSED-ON-DOCTEST-DRIFT** — no doctest shim, no allowance; a future sole-killer doctest
  surfaces as a MISSED mutant and is fixed by writing a unit test.
- **D5-GATE-IS-TEST** — no `.rs` changes; verification is exit codes + smokes per §7's provision.

## Acceptance Criteria (EARS)
| # | The system shall | Verify |
|---|---|---|
| REQ-001 | run gate:5's mutants under nextest in both full and diff modes | read the margs; a diff-mode run's cargo-mutants log names the nextest runner |
| REQ-002 | re-classify the #337 trio honestly: a synthetic `--in-diff` over font_zoom.rs yields its full set CaughtMutant with 0 Timeouts, faster than the recorded #337 run | run gate:5 diff-scoped at font_zoom.rs (the exact laundering evidence, now clean) |
| REQ-003 | report `N timeout(s); M mislabeled` after any run with Timeouts, sourced from each outcome's log_path | inject a synthetic outcomes.json + log fixture through the audit path (shell-level smoke); zero-timeout runs print nothing extra |
| REQ-004 | still fail the gate on a genuinely MISSED mutant under the new runner | negative smoke: neuter one font_zoom assert → the mutant goes MISSED → gate:5 exits red; restore, clean |
| REQ-005 | keep the 0/2/3 exit acceptance and the fail-closed 1/4 handling byte-identical | diff review + the REQ-002/004 runs exercising exit 0 and 2 |
| REQ-006 | leave MUT_MSI_MIN and the MSI arithmetic untouched | diff review (the jq caught/missed lines change only by the audit's ADDITIONS) |

## Phase Plan
P2 confirm #348 landed (hard gate); confirm the nextest runner's baseline behavior under `--jobs 2`
(two parallel mutants × nextest's own parallelism — decide whether to pin nextest threads for the
baseline the way the load-flake lesson suggests, WITH evidence, not preemptively). P3 the margs line +
the audit block (pure shell, jq over outcomes.json + grep over log_path), shellcheck-clean. P3.5
critics on: the audit's quoting/paths (log_path is relative to mutants.out?), a Timeout with a MISSING
log file (must not crash the gate), diff-mode's early-return paths skipping the audit cleanly. P4 the
four smokes above + a full `--diff` gate green. P5 docs (testing-standards: what Timeout now means) +
AAR; record the "measured, not assumed" doctest check as the reusable lesson. Standing traps:
[m22-editing-bar.md](../../design-notes/m22-editing-bar.md); shellcheck gates this file; **heavy runs
`CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0`**.
