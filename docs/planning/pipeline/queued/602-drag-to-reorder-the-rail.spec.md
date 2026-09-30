---
pipeline_id: 39c99593-fefc-4cd3-9183-f49a15722434
ticket: docs/planning/tickets/open/TICKET-602-drag-to-reorder-the-rail.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Drag to reorder the rail"
type: feature
slice: workbench shell (the rail); after #600 and #601
references: [docs/planning/pipeline/completed/453-rail-keyboard-and-reorder.spec.md, docs/planning/pipeline/completed/542-rail-attention-order.spec.md, docs/planning/pipeline/queued/600-rail-menu-and-projectless-groups.spec.md]
---

## Title
Drag and drop in the rail, reorder only: project and group headers move up and down among the
headers, and terminal, Browser tab and thread rows move up and down within their own group. The
order is saved and survives a restart.

## Scope
### In
- **Headers.** A project's or a group's header drags up or down among the headers (their rows go
  with them). A drop between two headers places it there; the window's project order changes as
  Move Project Up and Down change it (#453), and a group (#600) takes its place among the projects.
- **Rows.** A terminal row (an agent CLI's included), a Browser tab row or a thread row drags up or
  down among the rows of its own kind in its own group and section (main checkout, or a worktree's
  terminals under it). A drop elsewhere (another group, another kind, outside the rail) puts
  nothing anywhere: the row stays where it was.
- **While dragging.** A preview of the row follows the pointer; a line between rows or headers
  shows where the drop lands, only where a drop is allowed. The rail's order holds still for the
  whole drag (today gpui reports no hover during a drag, which ends #542's hold).
- **A click is still a click.** A press that moves less than gpui's drag threshold opens the row as
  before.
- **Saved.** Project order through Zed's multi-workspace state, as #453's does; group order with
  #601's record; row order in Marley's own record keyed by stable ids: a terminal by its
  `MARLEY_TERMINAL_ID` (#575), a Browser tab by its workspace and item (#576), a thread by its
  `ThreadId`. Rows with no saved place (new ones) keep today's order after the placed ones.
- **With the attention order (#542).** In the default `rail_order: "attention"`, the order Chad
  set replaces the window's order as the tie-break within each attention class; in `"window"`, it
  is the order.
- **Move Project Up and Down** keep working, on the same order.

### Out (explicitly deferred)
- Moving a row to another group or project (a terminal changing workspace), or out of the rail
  into a pane, and a pane's tab into the rail.
- Dragging worktree rows, port rows or the "Needs you" entries.
- Reordering by keyboard beyond #453's menu entries.
- Tab order in the panes: a row's rail place and its tab's place stay independent.

## Reference (§20)
Upstream Zed: its pane tab bar's drag to reorder (`crates/workspace/src/pane.rs`, `DraggedTab`
with `on_drag`, `drag_over` drawing an insertion border by index, `on_drop`), and the window tab
bar's (`crates/platform_title_bar/src/system_window_tabs.rs`, `DraggedWindowTab`): a drag preview
under the pointer, a line where it lands, and the order kept. Marley matches that feel in its
rail. Warp: its vertical tab list reorders tabs by drag (the list observed in
`docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md`); Marley keeps reorder within
a group and leaves moving between groups for later.

### Prior art
- **Behavior maps.** The Warp capture; #453's spec, which deferred drag (`453-rail-keyboard-and-
  reorder.spec.md:36`); #542's spec and AD-claude-542 (the attention order and its hold).
- **Published material.** None needed.
- **Code we already ship** (Explore, 2026-09-30):
  - gpui's drag API in `crates/gpui/src/elements/div.rs`: `on_drag` (the preview is an entity),
    `drag_over`, `on_drop`, `can_drop`, `on_drag_move`; the 2 px threshold after which no click
    fires; `cx.has_active_drag`. Hover reads false during any drag (div.rs:3148-3167), the reason
    the hold must be kept by the drag itself.
  - Patterns: the pane tab bar (`pane.rs:2969-3014`, 3877), the window tabs
    (`system_window_tabs.rs:28-38, 183-223`), the project panel's drag-move target tracking
    (`project_panel.rs:6152-6292`). No helper in `crates/ui`; Zed's Threads Sidebar has no drag.
  - Project order: `MultiWorkspace::move_project_group_up/down` swap neighbours and serialize
    (`multi_workspace.rs:892-930`); there is no move-to-index API. Either repeated swaps (which
    can pass over groups the rail hides) or one additive `move_project_group_to(key, index)`,
    settled at promotion with its touchpoint row.
  - Row ids that survive a restart: `Terminal::marley_terminal_id` (#575); Browser tabs'
    `(workspace_id, item_id)` rows (#576); `ThreadId`.
  - Marley's saved-state patterns: the `KeyValueStore` scopes (`launch.rs`, `shortcut_note.rs`,
    `terminal_size.rs`) and Marley's SQLite tables keyed to `workspaces`.

## UI proof
The scenario `script/e2e/602-drag-to-reorder-the-rail.sh` (sway, since it drags) opens two scratch
projects, the second with three terminals, then: drags the second project's header above the first
and shoots mid-drag (`line.png`) and after (`projects.png`); drags the third terminal above the
first (`rows.png`); drags a terminal onto the other project and shoots that it went back
(`refused.png`); clicks a row without moving and shoots that it opened (`click.png`); quits and
starts Marley and shoots the rail (`restart.png`).

## Locked-In Decisions
- D1 — Reorder only: a drop outside the row's own group, section and kind changes nothing.
- D2 — The saved row order is Marley's own, keyed by stable ids; it never moves tabs in panes.
- D3 — The project order stays Zed's multi-workspace order, so Move Project Up and Down, the drag
  and a restart agree; groups (#600, #601) are placed in the same sequence by Marley's record.
- D4 — Under `rail_order: "attention"`, attention classes still come first; the order set by
  dragging breaks ties within a class. A drop that would cross a class boundary lands at the edge
  of its class, and the line shows only where it can land.
- D5 — The rail's order holds for the whole drag, whatever the pointer's hover says.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the user drags a header or a row, the rail shall show a preview under the pointer and a line where a drop would land, only at places a drop is allowed. | Shot `line.png` |
| REQ-002 | WHEN the user drops a project or group header between two headers, the rail shall list it there with its rows. | Shot `projects.png` |
| REQ-003 | WHEN the user drops a terminal, Browser tab or thread row between two rows of its kind in its own group and section, the rail shall list it there. | Shot `rows.png` |
| REQ-004 | WHEN the user drops a row anywhere else, the rail shall leave it where it was. | Shot `refused.png` |
| REQ-005 | WHEN the user presses and releases on a row without moving past the drag threshold, the row shall open as a click does. | Shot `click.png` |
| REQ-006 | WHEN Marley restarts, the rail shall list projects, groups and rows in the order the user left them. | Shot `restart.png` |
| REQ-007 | WHILE a drag is in progress, the rail shall not re-sort its rows. | Shot `line.png` (the rows in place under the line) and review |
| REQ-008 | WHILE `marley.rail_order` is `"attention"`, the order set by dragging shall order rows and projects within the same attention class. | Review of the diff |
| REQ-009 | WHEN the user chooses Move Project Up or Down after a drag, the project shall move one place from where the drag left it. | Review |

## Phase Plan
- **P1 Plan** — promote this pair, recall, the design in the notes: the drag types, the drop rules,
  the saved order and its keys, the Zed touch for moving a project to an index (or none).
- **P2 Code** — the drag and drop on the rail's headers and rows, the ordering in `marley_rail`,
  the saved order, the hold during a drag; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_workbench.md` and `marley_rail.md`,
  the guide page and the walkthrough (§21); the ledger; close, archive, commit.
