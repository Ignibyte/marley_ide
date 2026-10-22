---
pipeline_id: c0f65580-46d8-4fd9-bd0b-cb31ec629438
ticket: docs/planning/tickets/open/TICKET-849-components.md
status: Phase 4 — Complete PASS
title: The Component Library as its board draws it
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The Component Library gains the Components board's navigation column: a "Find a component" field
over each group's component names; typing filters the names and the cards, and a name clicked
narrows the page to that component.

## Scope
### In
- `component_library.rs`: the column (find field, group headings, names), the filter on the
  cards; Try it and the templates as they are.

### Out (explicitly deferred)
- A Component tab from `show card.json` (#799's, already drawn by the same renderer).

## Reference (§20)
N/A — Marley-specific: the Component Library (#799, #801); the Components board.

### Prior art
- `component_library.rs` (`Group::ALL`, `entry_card`), the Threads page's search field (#836).
- The brain (consultation 0b49be4f): nothing on this seam.

## UI proof
Scenario `script/e2e/849-components.sh` (sway): a scratch repository.
- `849-01-library`: the column with Find a component, the groups and names; the cards (REQ-001).
- `849-02-found`: "stat" typed: the column and the cards narrowed to it (REQ-002).
- `849-03-day`: Marley Day (REQ-001).

## Locked-In Decisions
- D1 — The filter matches a component's name or description, case aside; a group with no match
  hides its heading.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Component Library shall show a column with a find field over its groups' component names, in Marley Night and Marley Day. | `849-01-library`, `849-03-day` |
| REQ-002 | WHEN the user types in the find field or clicks a name, the column and the cards shall narrow to what matches. | `849-02-found` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
