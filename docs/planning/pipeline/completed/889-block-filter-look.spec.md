---
pipeline_id: be161aff-3f5b-46f9-a090-70033b5ebd7a
ticket: docs/planning/tickets/open/TICKET-889-launch-menu-remote-and-block-tools.md
status: Phase 4 — Complete PASS
title: The block filter as the Workflows board draws it
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The block filter's field row reads as the Workflows board draws it: the query, then "N of M
lines" beside it, then Aa, .* and Invert in one framed group, then Context.

## Scope
### In
- `block_filter.rs`: `header` (the title and Close), `controls` (the count beside the
  field, the framed toggles).

### Out (explicitly deferred)
- Nothing else is open on this ticket: the Launch section of the + menu already draws as the
  board does (seen in `889-01-menu`); the remote terminal menu lists hosts from settings and
  needs none here; "Keys on a selected block" names no place in the window, a question for Chad.

## Reference (§20)
N/A — Marley-specific: Marley's block filter (#528); the Workflows board.

### Prior art
- `FilterPanel::header`, `controls`; the walkthrough's line naming Invert (kept).
- The brain (consultation 8b41d4e0).

## UI proof
Scenario `script/e2e/889-menu-and-filter.sh` (sway): a launch config pair, a forty-line log.
- `889-01-menu`: the project's + with its Launch section, as it is (REQ-001).
- `889-02-filter`: "FAILED": the field, "3 of 40 lines" beside it, the framed toggles (REQ-002).
- `889-03-day`: Marley Day (REQ-002).

## Locked-In Decisions
- D1 — Invert keeps its name: the walkthrough uses it.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | A project's + menu shall list its launch configs under Launch. | `889-01-menu` |
| REQ-002 | The block filter shall show its count beside the field and its toggles in one framed group, in Marley Night and Marley Day. | `889-02-filter`, `889-03-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
