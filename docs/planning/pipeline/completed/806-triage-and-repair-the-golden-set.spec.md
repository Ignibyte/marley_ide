---
pipeline_id: 9af27f68-8129-49aa-b197-44e687f2b225
ticket: docs/planning/tickets/closed/TICKET-806-triage-and-repair-the-golden-set.md
status: Phase 4 — Complete PASS
title: Triage and repair the golden set
type: chore
slice: the regression suite (#517)
references:
  - docs/planning/pipeline/completed/797-merge-upstream-zed.spec.md
---

## Title
Sort each failing golden-set scenario into a stale scenario (repaired here) or a Marley bug
(filed as its own ticket), from its run's log and shots.

## Scope
### In
- A run of the whole golden set on the current tree (head `b0fb8af84a`, the debug build).
- For each failure: the failing check, the shot at that step, the scenario's steps, and the code
  or the ticket that changed what the step expects; a verdict, stale or bug, with that evidence.
- Stale scenarios repaired (click points, steps, expectations that follow a deliberate change)
  and run again until they pass.
- Each Marley bug filed as a bug ticket with the scenario, the check and the evidence.

### Out (explicitly deferred)
- Fixing the bugs found: each is its own ticket.
- New scenarios, and any scenario not in the golden set.

## Reference (§20)
N/A — Marley-specific: the suite tests Marley's own features; nothing outside Marley is matched.

### Prior art
- The code we ship: `script/regress` (#517), `script/e2e.sh` and its helpers, the scenarios.
- #695's golden repair (7 left failing) and #797's comparison of the 29 on the pre-merge build.
- Behavior maps, published material: none on this seam.

## UI proof
Each repaired scenario's own run under `script/regress`, its checks passing and its shots read;
the final run of the whole golden set.

## Locked-In Decisions
- **D1 — A check is never loosened to pass.** A scenario changes only where Marley changed on
  purpose (a ticket says so); otherwise the failure is a bug.
- **D2 — Bugs are filed, not fixed here**, so this ticket's diff stays scenarios only.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | Each failing scenario shall have a verdict, stale or bug, with its evidence in the notes | the notes' triage table |
| REQ-002 | WHEN a scenario is stale, its repaired run shall pass | `script/regress` lines |
| REQ-003 | WHEN a failure is Marley's, a bug ticket shall record it | `tickets/open/` |

## Phase Plan
- **P1 Plan:** this spec.
- **P2 Code:** the run, the triage, the repairs; `script/gates.sh --diff --fast` (no Rust).
- **P3 Test:** each repaired scenario run; the whole set run again.
- **P4 Complete:** ledger, close, archive, commit.
