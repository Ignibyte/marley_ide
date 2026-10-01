---
pipeline_id: e1815e25-2984-402d-95ba-ff9c88b40879
ticket: docs/planning/tickets/open/TICKET-635-the-golden-regression-green.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "The golden regression run green"
type: chore
slice: wave 5, the test pass (after #634)
references: [docs/planning/design-notes/remaining-work-2026-09-30.md]
---

## Title
`just regress` runs the golden set's scenarios (`script/e2e/golden`, 56 lines) each in a headless
sway of its own, and each checks itself. It has not run since the per-ticket regression stopped on
2026-09-29. This ticket runs it, fixes what regressed, brings stale scenarios to the shipped UI, and
runs it again green.

## Scope
### In
- The run: `just regress` on the debug build (the recipe's own `idle` and `REGRESS_TIMEOUT`), its
  per-scenario PASS and FAIL lines and run folders kept.
- Triage of every failure: **stale** (a ticket since changed the UI or its default on purpose, as
  #627's prompt editor or #630's headers did: the scenario follows it, citing the ticket),
  **regression** (the code is fixed, an `F-…` block at Complete), or **flaky** (timing: the
  scenario's waits are made to wait on what it checks, never longer sleeps alone).
- The run again, green.

### Out (explicitly deferred)
- Adding scenarios to the golden set (each ticket's own scenario stays outside it unless Chad
  asks); unit tests (#634); mutation (#636).

## Reference (§20)
N/A — Marley-specific: the regression suite of Marley's own scenarios (#517).

### Prior art
- **Code we already ship:** `script/regress` (#517: a headless sway per scenario, `E2E_BINARY`,
  `REGRESS_DIR`, `REGRESS_TIMEOUT`), `script/e2e.sh` and its helpers, `script/e2e/golden`.
- **Knowledge:** AD-claude-517-the-golden-set-gates-the-install-001;
  L-claude-628-check-a-copy-through-the-prompt-editor-with-copy-on-select-001 (since #627 a
  scenario that types at a prompt types into the prompt editor); the e2e lessons of each ticket
  since #517.

## UI proof
N/A — no UI delta: the change is scenarios and any code a regression exposes. Its proof is the
`just regress` log of the first and the green run; each fixed regression is seen in its scenario's
shots, which the notes read.

## Locked-In Decisions
- D1 — The shipped behavior is the reference, as in #634.
- D2 — A flaky scenario waits on its condition, not on a longer sleep.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `just regress` runs, every golden scenario shall pass. | The green run's log |
| REQ-002 | WHEN a scenario failed in the first run, the notes shall say whether it was stale (with its ticket), a regression (with its fix) or flaky (with what it now waits on). | The notes' triage table |

## Phase Plan
- **P1 Plan** — the set as it stands; the build to run.
- **P2 Code** — the first run, the triage, the fixes; the gate green.
- **P3 Test** — the green run; the fixed scenarios' shots read.
- **P4 Complete** — knowledge; close, archive, commit.
