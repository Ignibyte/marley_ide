---
pipeline_id: e4454870-e790-4dfb-b6b4-685f45d87dc7
ticket: docs/planning/tickets/open/TICKET-902-rail-header-icons-float-left.md
status: Phase 4 — Complete PASS
title: The rail header's icons all float to the left
type: bug
slice: The Marley look, follow-up to #873 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
The rail header's six icons (Projects, Threads, Sessions, Files, Add Project, Shrink) sit together
from the left; the empty space after them keeps its right-click menu.

## Scope
### In
- `rail.rs` `render_header`: the spacer after the buttons instead of between them.

### Out (explicitly deferred)
- Nothing.

## Reference (§20)
N/A — Marley-specific: the rail header (#745, #873).

### Prior art
- `render_header`, `render_view_switch` (#873), `render_files_button` (#868), `render_add_project`.
- The brain (consultation 21f4f052): nothing on this seam.

## UI proof
Scenario `script/e2e/902-rail-header-icons-float-left.sh` (sway): a scratch project.
- `902-01-night`: the header's six icons from the left (REQ-001).
- `902-02-day`: the same in Marley Day (REQ-001).

## Locked-In Decisions
- D1 — Chad's order is kept: the view switch, then Files, Add Project, Shrink.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The rail header shall draw its icons together from its left edge, the empty space after them, in Marley Night and Marley Day. | `902-01-night`, `902-02-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
