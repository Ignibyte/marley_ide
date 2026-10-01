---
pipeline_id: 288a61ef-41f1-48e0-ac15-94ab462c2d61
ticket: docs/planning/tickets/open/TICKET-618-port-row-lines-and-header-tooltip.md
status: Phase 4 — Complete PASS
title: "A port row's clipped lines, and a header tooltip over its menu"
type: bug
slice: workbench shell, the rail; polish after #603 and #606
references: [docs/planning/pipeline/completed/603-service-aware-port-rows.notes.md, docs/planning/pipeline/completed/606-closed-projects-in-the-rail.notes.md]
---

## Title
A port row's URL and unit read in full at the default rail width, or end in an ellipsis with the
whole in the row's tooltip; and a header's tooltip closes when its right-click menu opens.

## Scope
### In
- **The port row:** its hover buttons (Open, Copy, Stop) stop reserving their width while hidden:
  they show over the row's end on hover, on a background that covers the text under them. The
  URL drops its scheme's `http://` when it is `127.0.0.1` or `localhost` (the host and port are
  the part that matters), truncates in the middle when it still does not fit, and the row's
  tooltip starts with the full URL. The unit line truncates at its start (`…603.service`), keeping
  the name's end.
- **The header tooltip:** a closed header's "Not open. Click to open it." is built inside its
  `right_click_menu` trigger and only while the menu is closed (`is_menu_active`), as Zed's dock
  buttons do and as #615 did for the port row, so the menu opening drops it. The rail's
  empty-space menu has no tooltip on its trigger, so nothing changes there.

### Out (explicitly deferred)
- Other rows' layouts, and the rail's width.
- The tooltips of the chips at a header's end (changed lines, pull request): they show only while
  the pointer is on the chip itself.

## Reference (§20)
Upstream Zed: a trigger's tooltip is withheld while its menu is open (`crates/workspace/src/
dock.rs:1533-1553`, `ui::PopoverMenu::trigger_with_tooltip` in `popover_menu.rs:197-217`), and gpui
drops a tooltip whose builder is gone (`gpui/src/elements/div.rs:2365-2372`).

### Prior art
- **Behavior maps:** #603's Test note and F-claude-603-a-long-trailing-state; #606's Test note.
- **Published material:** none needed.
- **Code we already ship:**
  - `render_port_row` (`rail.rs:4669-4764`), `row_card` and `RowLine` (6809-6891), the row's tooltip
    (4818-4823, set at 4746-4749); `visible_on_hover` keeps its layout (`ui/src/traits/
    visible_on_hover.rs:13-16`).
  - `Label::truncate_start` and `truncate_middle` (`ui/src/components/label/label.rs:74, 80`).
  - `right_click_menu`'s trigger receives `is_menu_active` (`ui/src/components/right_click_menu.rs:
    32-41, 161`); the rail's header menu (`rail.rs:4312`) and closed header tooltip (4301); #615's
    port row already builds its tooltip in the trigger (`rail.rs:5458`).
  - `Hsla::blend` (`gpui/src/color.rs:580`) gives the opaque shade the overlaid buttons sit on:
    the rail's `panel_background` under the row's hover or selected fill.

## UI proof
The scenario `script/e2e/618-port-row-lines-and-header-tooltip.sh` (sway) starts a user unit with
a long name listening in the scratch project (as #603's scenario does), then:
- `row.png`: the port row at the default rail width, its host and port and unit readable;
- `hover.png`: the pointer on the row, the buttons over its end;
- `tooltip.png`: the row's tooltip starting with the full URL;
- restarts Marley so a project is closed, points at its header until its tooltip shows,
  right-clicks it (`menu.png`: the menu open with no tooltip over its first entry).

## Locked-In Decisions
- D1 — Hover buttons overlay the row's end instead of reserving room.
- D2 — Header tooltips follow Zed's rule: none while their menu is open.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a port row shows at the default rail width, its host and port and its unit's name shall be readable, the rest in its tooltip. | Shots `row.png`, `tooltip.png` |
| REQ-002 | WHILE the pointer is on a port row, its buttons shall show over its end without moving its text. | Shot `hover.png` |
| REQ-003 | WHEN a header's right-click menu opens, the header's tooltip shall close. | Shot `menu.png` |

## Phase Plan
- **P1 Plan** — promote, recall, the design (the overlay, the truncations, the trigger).
- **P2 Code** — the row and the header; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
