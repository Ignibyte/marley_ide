---
pipeline_id: 453e3180-c7cd-4271-9254-e150d15d1b13
ticket: docs/planning/tickets/closed/TICKET-866-tabs-as-pills-and-themed-menus.md
status: Phase 4 — Complete PASS
title: Tabs with their kind and state, the + menu's keys, and the split menu drawn
type: feature
slice: The Marley look, W2.1 follow-up (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
In the Marley layout a center tab leads with its kind's tile (as #883's switcher draws it) and
carries a dot for its state (a running command, a failed one, an agent waiting); a project's +
menu shows each entry's key; the split menu draws each split as a small picture. The tmux close
prompt is #870's card already.

## Scope
### In
- `crates/workspace/src/pane.rs` (Zed crate): `MarleyTabLook`, a global; with it set a tab's icon
  is the look's tile and the look's dot follows the title; `MarleySplitPictures`, a global; with it
  set the split menu's rows lead with its picture.
- `tab_tiles.rs`: the tab look (tile, dot) and the split pictures.
- `rail.rs`: the + menu's entries name their actions, so the menu shows their keys.

### Out (explicitly deferred)
- The tab's pill shape and "+3 more": Zed's `Tab` component draws the frame and the bar scrolls;
  changing either is a change to every Zed tab, not a hook.
- Tiles in the + menu: Zed's menu rows take icons, not drawn tiles.

## Reference (§20)
Upstream Zed's `workspace::pane` tab bar and its split menu.

### Prior art
- A probe: Zed's tabs (icon and title), the + menu without keys, the split menu with keys and no
  pictures.
- `tab_tiles::switcher_row` (#883) draws a tab's kind as a tile.
- `notifications::unread`, the terminal's blocks (`marley_anchored`) for state.
- The brain (consultation 52246918): nothing on this seam.

## UI proof
Scenario `script/e2e/866-tab-pills-and-themed-menus.sh` (sway): a scratch repository, a terminal
with a failed command, a file, a tmux session.
- `866-01-tabs`: the tab bar: each tab's tile and the terminal's failed dot (REQ-001).
- `866-02-plus`: the project's + menu with keys (REQ-002).
- `866-03-split`: the split menu's pictures (REQ-003).
- `866-04-day`: Marley Day, the tabs (REQ-001).

## Locked-In Decisions
- D1 — The dot: the error colour for a terminal whose last command failed, the accent while one
  runs, the warning colour for a terminal that needs you; none otherwise.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | In the Marley layout, each center tab shall lead with its kind's tile and carry its state's dot, in Marley Night and Marley Day. | `866-01-tabs`, `866-04-day` |
| REQ-002 | A project's + menu shall show each entry's key where it has one. | `866-02-plus` |
| REQ-003 | The split menu shall draw each split as a small picture of the panes. | `866-03-split` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
