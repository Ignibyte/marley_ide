---
pipeline_id: ab29863b-8334-462f-8e22-72501ed643ae
ticket: docs/planning/tickets/open/TICKET-636-a-mutation-run-of-the-pure-cores.md
status: Phase 4 — Complete PASS
title: "A mutation run of the Marley crates' pure cores"
type: chore
slice: wave 5, the test pass (after #634, whose green suites the run needs)
references: [docs/planning/design-notes/remaining-work-2026-09-30.md]
---

## Title
`cargo-mutants` (27.1.0, installed) runs over the Marley crates' pure cores against the suites
#634 made green. Its survivors are read one by one and reported in a findings doc; a survivor that
shows a real bug gets the bug fixed; killing the rest with new tests waits for Chad.

## Scope
### In
- The run: `cargo mutants` over the nine crates with no gpui (`marley_agent` 685 mutants,
  `marley_dcs` 43, `marley_fleet` 38, `marley_mcp` 412, `marley_rail` 212, `marley_remote` 43,
  `marley_sdk` 100, `marley_system_one` 113, `marley_terminal` 1,012: 2,658), in the topology
  AD-claude-443 settled for a full run: copy mode, two workers, each copy building in its own
  target (`CARGO_TARGET_DIR` unset), the copies under `~/.cache/marley-mutants`, `--no-config`
  (no exclusions), nextest, the output outside the repository.
- `docs/planning/design-notes/mutation-run-2026-10.md`: per crate the mutants, caught, missed,
  timeouts and unviable; each missed mutant with its place and a reading: **untested** (no test
  holds that behavior), **equivalent** (the mutant behaves the same), or **bug** (the mutant
  shows code that is wrong now).
- Each **bug** fixed in code, an `F-…` block at Complete.
- `intake/kill-mutation-survivors.md`: the untested survivors worth a test, for Chad to decide
  against the no-new-tests rule.

### Out (explicitly deferred)
- New tests (CONSTITUTION §0, Chad's 2026-09-23 directive); the gpui crates (`marley_workbench`'s
  views), which the suites barely reach.

## Reference (§20)
N/A — Marley-specific: Marley's own mutation audit, as rustal keeps one outside its gate.

### Prior art
- **Code we already ship:** `cargo-mutants` 27.1.0; the retired `script/mutation.sh` (git history)
  and its decisions: AD-claude-the-workflow-is-four-phases-and-mutation-waits-for-the-end-001,
  AD-claude-443-mutation-topology-and-no-masks-001,
  F-claude-443-a-full-mutation-workers-shared-one-target-001 (workers sharing one target dir
  collide), BF-claude-mutation-survivors-loopbound-childbin-serial-001.

## UI proof
N/A — no UI delta: a report and any bug a survivor exposes. Its proof is the run's `mutants.out`
summaries quoted in the findings doc; a fixed bug in UI code is seen in its own scenario.

## Locked-In Decisions
- D1 — No test is written to kill a survivor in this ticket; the intake asks Chad.
- D2 — Copy mode, two workers, a target per copy (AD-claude-443, F-claude-443): no worker writes
  the shared target or another's binaries, and the working tree is never mutated. The copies and
  their targets go under `~/.cache` on `/home`, since `/mnt/fast` has 37G free.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the run ends, the findings doc shall list each crate's mutants, caught, missed, timeouts and unviable. | The findings doc |
| REQ-002 | WHEN a mutant is missed, the findings doc shall mark it untested, equivalent or bug, with its place. | The findings doc |
| REQ-003 | WHERE a survivor shows a bug, the code shall be fixed. | The diff; `F-…` blocks |
| REQ-004 | The diff shall add no test function. | Review of the diff |

## Phase Plan
- **P1 Plan** — the crates; the run's options; the time it takes.
- **P2 Code** — the run, the reading, the fixes; the gate green.
- **P3 Test** — the fixed crates' suites green; `just shot`.
- **P4 Complete** — the findings doc, the intake, knowledge; close, archive, commit.
