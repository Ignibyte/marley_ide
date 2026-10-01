---
pipeline_id: f1626f53-313f-4c6f-97d6-a245dfaf4a30
ticket: docs/planning/tickets/open/TICKET-634-the-test-suites-green.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "The test suites run and green"
type: chore
slice: wave 5, the test pass (after #475)
references: [docs/planning/design-notes/remaining-work-2026-09-30.md]
---

## Title
The tests in the tree have not run since #483 (2026-09-23). This ticket runs the Marley crates'
suites and the Marley tests inside the Zed crates Marley changed, and makes them pass without
adding a test: a stale test follows the behavior its ticket shipped; a real bug is fixed in code.

## Scope
### In
- The run: `cargo nextest run` (installed, 0.9.143) over every `crates/marley_*` crate and the Zed
  crates holding Marley tests (`terminal`, `terminal_view`, `settings_ui`, `settings_content`,
  `workspace` where Marley added tests, found at promotion), one cargo command at a time, its log
  kept.
- Triage of every failure: **stale** (a ticket since 2026-09-23 changed the behavior on purpose:
  the test's expectation follows the shipped behavior, and the notes name the ticket) or **bug**
  (the code is fixed, an `F-…` block at Complete); a test that cannot run on this box (a missing
  program, the display) is named with why.
- The run again, green.

### Out (explicitly deferred)
- New tests (CONSTITUTION §0); coverage; the golden e2e set (#635); mutation (#636).
- Zed's own tests in crates Marley never touched.

## Reference (§20)
N/A — Marley-specific: the test debt of Marley's own workflow; Zed's tests in untouched crates
stay Zed's.

### Prior art
- **Code we already ship:** the suites in `crates/marley_*/src/*_tests.rs` and `#[cfg(test)]`
  modules; `cargo-nextest` 0.9.143; gate:2, which builds every target, so the tests compile today.
- **Knowledge:** AD-claude-483-e2e-visualization-tests-replace-unit-tests-001 (the tests kept,
  run by no gate); L-claude-465-zeds-clippy-bans-std-process-in-tests-too-001;
  F-claude-465-gate4-counted-lines-from-a-stale-executable-001.

## UI proof
N/A — no UI delta: the change is tests and any code a failing test exposes. Its proof is the
suites' log, the first run's failures and the green run; a fixed bug in UI code is checked by its
own scenario, and §7's visual check is `just shot`.

## Locked-In Decisions
- D1 — No new test; a test changes only to follow behavior a ticket shipped, with the ticket named.
- D2 — The shipped behavior is the reference: a test is never made to pass by changing code that
  works as its ticket meant.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the suites run, every test shall pass, or be named in the notes with why it cannot run on this box. | The green run's log |
| REQ-002 | WHEN a test failed in the first run, the notes shall say whether it was stale (with its ticket) or a bug (with its fix). | The notes' triage table |
| REQ-003 | The diff shall add no test function. | Review of the diff |

## Phase Plan
- **P1 Plan** — the crate list; the run's command.
- **P2 Code** — the first run, the triage, the fixes; the gate green.
- **P3 Test** — the green run; `just shot`.
- **P4 Complete** — knowledge (each bug an `F-…`); close, archive, commit.
