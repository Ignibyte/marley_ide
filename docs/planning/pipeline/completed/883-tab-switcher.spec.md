---
pipeline_id: 145ab179-7038-432e-96b4-ffc696468def
ticket: docs/planning/tickets/open/TICKET-883-switcher-ports-containers-collab.md
status: Phase 4 — Complete PASS
title: The Ctrl Tab switcher as its board draws it
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
Each row of the Ctrl Tab switcher leads with a tile naming the tab's kind (">_" for a terminal, a
file's extension, "www" for a Browser tab), shows where the tab is under its title, and the
switcher ends with "Hold Ctrl, press Tab to move · let go to open", as the Splits board draws it.

## Scope
### In
- `crates/tab_switcher/src/tab_switcher.rs`: `MarleySwitcherLook`, a global `render_match`
  asks for each row's tile and line, and a footer while it is set (`Marley:` hunks); without it,
  Zed's rows.
- `marley_workbench`: the global, set at init.

### Out (explicitly deferred)
- The port rows, the Containers panel and Collab's card: TICKET-890.

## Reference (§20)
Upstream Zed, `tab_switcher`: the picker's rows (`render_match`); Marley adds a tile and a line
at a hook, as #854 did for Open Recent.

### Prior art
- `TabSwitcherDelegate::render_match`; `ItemHandle::tab_tooltip_text`, `project_path`,
  `act_as`; #854's `MarleyRecentLook`.
- The brain (consultation d49512f6).

## UI proof
Scenario `script/e2e/883-tab-switcher.sh` (sway): a scratch repository with `rail.rs` open
beside the terminal; `tab switcher: toggle` from the palette, which stays open.
- `883-01-switcher`: the rows with their tiles and lines, the footer (REQ-001).
- `883-02-day`: Marley Day (REQ-001).

## Locked-In Decisions
- D1 — The line is a file's path in its project, else the tab's tooltip, when it says more than
  the title; none otherwise.
- D2 — The tile is text: ">_" for a terminal, a file's extension (at most three letters), "www"
  for a Browser tab, else the title's first letter.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The tab switcher shall lead each row with a tile naming the tab's kind, show where the tab is under its title, and end with the Ctrl and Tab hint, in Marley Night and Marley Day. | `883-01-switcher`, `883-02-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
