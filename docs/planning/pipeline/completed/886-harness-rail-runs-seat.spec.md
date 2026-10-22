---
pipeline_id: 4ec9c862-5b4b-4459-84b2-7b0bdee620d6
ticket: docs/planning/tickets/open/TICKET-886-harness-rail-runs-and-seat-form.md
status: Phase 4 — Complete PASS
title: The rail's Harness section, a task's runs and the seat form as the boards draw them
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The rail's Harness header leads with a dot in its state's colour; a scheduled task's history is a
card of runs, each with its time, its outcome as a chip and its first line; and the New Harness
Seat form picks the agent in a framed toggle and ends with Cancel and a primary "Create · Enter".

## Scope
### In
- `rail.rs`: the Harness section's header dot.
- `schedule_runs.rs` `ScheduleView`: the card, the outcome chips.
- `harness_seat.rs` `NewSeatModal`: the agent toggle, the footer.

### Out (explicitly deferred)
- Each session's quota on its rail row: the board shows it, and the rows carry no quota today.

## Reference (§20)
N/A — Marley-specific: Marley's harness (#534, #691, #719); the Fleet, MoreTabs and OpenConnect
boards.

### Prior art
- The rail's harness section (`says`, `says_color`), `ScheduleView::render_run`,
  `outcome_of`, `NewSeatModal`; #870's `prompts::render_button`.
- The brain (consultation 81b2198b).

## UI proof
Scenario `script/e2e/886-harness-rail-runs-seat.sh` (sway): #719's scratch harness with a task
`checks` every minute whose first run finds nothing and second finds something, writes on.
- `886-01-rail`: the Harness header's dot (REQ-001).
- `886-02-runs`: the task's history: a card, two runs with their chips (REQ-002).
- `886-03-seat`: `marley: new harness seat`: the agent toggle and the footer (REQ-003).
- `886-04-day`: Marley Day, the seat form (REQ-001 to REQ-003).

## Locked-In Decisions
- D1 — The dot takes the header's own state colour (connected, not running, refused).
- D2 — Create stays disabled while a seat is being made, dimmed with no click.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The rail's Harness header shall lead with a dot in its state's colour. | `886-01-rail` |
| REQ-002 | A scheduled task's history shall show its runs in a card, each with its time, its outcome as a chip and its first line. | `886-02-runs` |
| REQ-003 | The New Harness Seat form shall pick its agent in a framed toggle and end with Cancel and a primary Create, in Marley Night and Marley Day. | `886-03-seat`, `886-04-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
