---
pipeline_id: 39c99593-fefc-4cd3-9183-f49a15722434
ticket: docs/planning/tickets/open/TICKET-602-drag-to-reorder-the-rail.md
status: Phase 4 — Complete PASS
title: "Drag to reorder the rail"
type: feature
slice: workbench shell (the rail); after #600 and #601
references: [docs/planning/pipeline/completed/453-rail-keyboard-and-reorder.spec.md, docs/planning/pipeline/completed/542-rail-attention-order.spec.md, docs/planning/pipeline/completed/600-rail-menu-and-projectless-groups.spec.md, docs/planning/pipeline/completed/601-projectless-groups-survive-a-restart.spec.md]
---

## Title
Drag and drop in the rail, reorder only: project and group headers move up and down among the
headers, and terminal, Browser tab and thread rows move up and down within their own group. The
order is saved and survives a restart.

## Scope
### In
- **Headers.** A project's or a group's header drags up or down among the headers (their rows go
  with them). A drop on another header's block (its header or its rows) puts it in that block's
  place, as a pane's tab takes the place of the tab it is dropped on. Projects and projectless
  groups (#600) share one order, so a group can sit between two projects.
- **Rows.** A terminal row (an agent CLI's included), a Browser tab row or a thread row drags up or
  down among the rows of its own kind in its own group and section (main checkout, or a worktree's
  terminals under it). A drop elsewhere (another group, another kind, outside the rail) puts
  nothing anywhere: the row stays where it was.
- **While dragging.** A preview of the row follows the pointer; a line between rows or headers
  shows where the drop lands, only where a drop is allowed. The rail's order holds still for the
  whole drag (today gpui reports no hover during a drag, which ends #542's hold).
- **A click is still a click.** A press that moves less than gpui's drag threshold opens the row as
  before.
- **Saved.** The order is the rail's own, kept in the window's saved sidebar state beside #601's
  groups: headers by a project's folders or a group's id, rows by ids that survive a restart: a
  terminal by its `MARLEY_TERMINAL_ID` (#575), a Browser tab by its page's target id (#494), a
  thread by its `ThreadId`. Rows and headers with no saved place (new ones) keep today's order
  after the placed ones.
- **With the attention order (#542).** In the default `rail_order: "attention"`, the order Chad
  set replaces the window's order as the tie-break within each attention class; in `"window"`, it
  is the order.
- **Move Project Up and Down** keep working, on the same order: they now move a project one
  place in the rail's order, past a group as well as a project.

### Out (explicitly deferred)
- Moving a row to another group or project (a terminal changing workspace), or out of the rail
  into a pane, and a pane's tab into the rail.
- Dragging worktree rows, port rows or the "Needs you" entries.
- Reordering by keyboard beyond #453's menu entries.
- Tab order in the panes: a row's rail place and its tab's place stay independent.

## Reference (§20)
Upstream Zed, for the feel of dragging in a list: its pane tab bar's drag to reorder
(`crates/workspace/src/pane.rs`, `DraggedTab` with `on_drag`, `drag_over` drawing a 2 px border on
the side the tab lands, `on_drop` moving it to the target's index), and the window tab bar's
(`crates/platform_title_bar/src/system_window_tabs.rs`, `DraggedWindowTab`): a preview under the
pointer, a line where it lands, the dragged item taking the target's place. Marley matches that in
its rail. Warp: its vertical tab list is the rail's model (the list observed in
`docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md`); our behavior maps record no
drag in it, so nothing here comes from Warp beyond the list itself.

### Prior art
- **Behavior maps.** The Warp capture (a list, no drag recorded; `docs/warp_architecture/` has no
  reorder entry); #453's spec, which deferred drag (`453-rail-keyboard-and-reorder.spec.md:36`);
  #542's spec and AD-claude-542 (the attention order and its hold).
- **Published material.** None needed.
- **Code we already ship** (Explore and re-checked at promotion, 2026-09-30):
  - gpui's drag API in `crates/gpui/src/elements/div.rs`: `on_drag` (its constructor runs
    synchronously in the mouse-move listener, then `stop_propagation`, 2978-3005), `drag_over`
    (applied only when `can_drop` passes and the hitbox is hovered, 3471-3497), `on_drop` (takes
    the active drag, runs the listener only if `can_drop` passes, then `stop_propagation`,
    2897-2930), the 2 px `DRAG_THRESHOLD` after which no click fires. Hover reads false while any
    drag is active (3148-3167), so #542's hold ends as a drag starts unless the rail keeps it.
  - Patterns: the pane tab bar (`pane.rs`), the window tabs (`system_window_tabs.rs`). No drag
    helper in `crates/ui`; Zed's Threads Sidebar has no drag.
  - Project order: Zed's `MultiWorkspace::project_groups` never holds a folderless workspace
    (`ensure_project_group_state` and `restore_project_groups` skip an empty key,
    `multi_workspace.rs:677-693, 832-853`), and a new project goes to its top (index 0). A group
    therefore has no place in Zed's order: the rail keeps its own, and needs no Zed touch.
  - Row ids that survive a restart: `Terminal::marley_terminal_id` (`terminal.rs:1949`, #575;
    `None` for tasks and remote terminals); `BrowserView::target` (`browser.rs:4499`), which the tab
    saves and restores (#494); `ThreadId` as the row's key.
  - Saved state: the rail's fields in the window's sidebar blob (`read_rail_groups` and
    `write_rail_groups`, `rail.rs:4556-4575`, #601); `MultiWorkspace::serialize` spawns, so a rail
    listener may call it.

## UI proof
The scenario `script/e2e/602-drag-to-reorder-the-rail.sh` (sway, since it drags) opens the scratch
project `repo` with three terminals in the folders `one`, `two` and `three`, and makes the groups
Scratch and Web from the rail's empty space (#600). Then: drags Web's header onto `repo`'s and
shoots mid-drag (`line.png`: the preview, the line above `repo`'s block, the rows in place) and
after (`projects.png`: Web, `repo`, Scratch); drags the `three` terminal onto `one` (`rows-line.png` mid-drag, `rows.png`:
three, one, two); drags `one` onto Scratch's header (`refused-line.png` mid-drag: no line;
`refused.png`: nothing moved); chooses Move
Project Down on `repo` (`move.png`: Web, Scratch, `repo`); clicks the `two` row without moving
(`click.png`: `two` shown); quits and starts Marley with no path and shoots the rail
(`restart.png`: Web, Scratch, `repo`, with three, one, two). A second project is left out: after a
restart Zed brings back only the shown project's workspace (the gap #601's Test recorded), so a
group is the header that can show the order surviving.

## Locked-In Decisions
- D1 — Reorder only: a drop outside the row's own group, section and kind changes nothing.
- D2 — The saved row order is Marley's own, keyed by stable ids; it never moves tabs in panes.
- D3 — The header order is the rail's own list (projects by their folders, groups by their id),
  saved in the window's sidebar state; Move Project Up and Down move in the same list. Zed's
  project-group order is left alone: it cannot hold a group (settled at promotion; it replaces
  the queued draft's "the project order stays Zed's").
- D4 — Under `rail_order: "attention"`, attention classes still come first; the order set by
  dragging breaks ties within a class. A drop is allowed only on a row or header of the dragged
  one's class, so the line shows only where it lands.
- D5 — The rail's order holds for the whole drag, whatever the pointer's hover says.
- D6 — A drop takes the target's place, as Zed's pane tabs do: onto a row above the dragged one it
  lands before it, with the line on its top edge; onto one below, after it, with the line on its
  bottom edge. A header's target is its whole block, so the line runs above or below the block.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the user drags a header or a row, the rail shall show a preview under the pointer and a line on the edge where a drop would land, only on targets a drop is allowed on. | Shot `line.png` |
| REQ-002 | WHEN the user drops a project's or group's header on another header's block, the rail shall list it, with its rows, in that block's place. | Shot `projects.png` |
| REQ-003 | WHEN the user drops a terminal, Browser tab or thread row on a row of its kind in its own group and section, the rail shall list it in that row's place. | Shot `rows.png` (terminals); review for Browser tab and thread rows, which share the code |
| REQ-004 | WHEN the user drops a row anywhere else, the rail shall leave it where it was. | Shot `refused.png` |
| REQ-005 | WHEN the user presses and releases on a row without moving past the drag threshold, the row shall open as a click does. | Shot `click.png` |
| REQ-006 | WHEN Marley restarts, the rail shall list the headers and rows it brings back in the order the user left them. | Shot `restart.png` |
| REQ-007 | WHILE a drag is in progress, the rail shall not re-sort its rows. | Shot `line.png` (the rows in place under the line) and review |
| REQ-008 | WHILE `marley.rail_order` is `"attention"`, the order set by dragging shall order rows and projects within the same attention class. | Review of the diff |
| REQ-009 | WHEN the user chooses Move Project Up or Down after a drag, the project shall move one place from where the drag left it. | Shot `move.png` |

## Phase Plan
- **P1 Plan** — promote this pair, recall, the design in the notes: the drag types, the drop rules,
  the saved order and its keys; no Zed touch (D3).
- **P2 Code** — the drag and drop on the rail's headers and rows, the ordering in `marley_rail`,
  the saved order, the hold during a drag; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_workbench.md` and `marley_rail.md`,
  the guide page and the walkthrough (§21); the ledger; close, archive, commit.
