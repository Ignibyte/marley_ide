---
pipeline_id: d08054e9-781b-47eb-b2f1-70b698b7ca51
ticket: docs/planning/tickets/closed/TICKET-862-the-panel-buttons-become-a-labeled-group.md
status: Phase 4 — Complete PASS
title: The panel buttons become one labeled group
type: feature
slice: The Marley look, W1.4 follow-up (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
In the Marley layout, the bottom row's right end shows every dock's panels as one group, each
with its icon and its name (Files, Changes, Outline, Knowledge, Fleet, Collab, Agent, …), the one
showing filled, and an accent dot on a panel open as a tab; Zed's three unlabeled groups step
aside. The Zed layout keeps Zed's.

## Scope
### In
- `crates/workspace/src/dock.rs` (Zed crate): `Dock::panel_handles`, the dock's panels; and
  `MarleyPanelButtons`, a global whose answer hides `PanelButtons`.
- `panel_group.rs` (new): `PanelGroup`, the status item.
- `status_buttons.rs`: adds it after Home.

### Out (explicitly deferred)
- A panel's right-click menu (dock position, width, hide) from the group: Zed's menu stays on its
  own buttons, which the Zed layout shows.

## Reference (§20)
Upstream Zed's `workspace::dock::PanelButtons`, which the group draws again with names.

### Prior art
- `PanelButtons::render`: a panel's icon, its tooltip, its toggle action, and the dock's toggle
  for the active one.
- `panel_tabs::PanelTab` (#747) draws a moved panel as a center tab.
- `status_buttons.rs`: Home at the row's right.
- The brain (consultation 40c98317): nothing on this seam.

## UI proof
Scenario `script/e2e/862-panel-buttons-as-a-labeled-group.sh` (sway).
- `862-01-group`: the bottom row's right: Home and the panels, named, no other panel buttons
  (REQ-001).
- `862-02-open`: Changes opened (a tab): its dot (REQ-002).
- `862-03-day`: Marley Day (REQ-001).

## Locked-In Decisions
- D1 — A panel's name is the board's word where it has one (Files, Changes, Outline, Knowledge,
  Fleet, Containers, Collab, Agent, Terminal, Debug), else its tooltip.
- D2 — Names show while the window is at least 1200 px wide; narrower, icons only, each with its
  tooltip.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | In the Marley layout, the bottom row's right end shall show every dock's panels as one group, each with its icon and name, the showing one filled, in Marley Night and Marley Day, and no other panel buttons. | `862-01-group`, `862-03-day` |
| REQ-002 | WHILE a panel is open as a tab, its button shall carry an accent dot. | `862-02-open` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
