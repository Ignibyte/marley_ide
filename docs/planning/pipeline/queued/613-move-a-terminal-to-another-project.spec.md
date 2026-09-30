---
pipeline_id: 5895bb90-feff-4c37-b803-bb6fa6bcc748
ticket: docs/planning/tickets/open/TICKET-613-move-a-terminal-to-another-project.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Move a terminal to another project in the rail"
type: feature
slice: workbench shell, the rail; after #602
references: [docs/planning/pipeline/completed/602-drag-to-reorder-the-rail.spec.md, docs/planning/pipeline/completed/575-terminal-id-across-restore.spec.md]
---

## Title
A terminal row dragged onto another project of the window, or moved there with its menu's Move to
Project, joins that project's workspace with its shell still running, its scrollback and its
Marley state (id, agent events, turns) kept.

## Scope
### In
- **Drag:** a terminal row dropped on another project's header (the header's frame, not its block,
  per L-claude-602-nested-drop-targets) moves the terminal there, at the end of that project's
  terminals. The header shows the drop target while a terminal row is over it.
- **Menu:** a terminal row's right-click menu gains Move to Project, a submenu of the window's
  other open projects.
- **The move:** the same `TerminalView` entity moves from its pane to the target workspace's
  active center pane through `workspace::move_item`, so every Marley global keyed by the view's
  entity id stays; the view's workspace and project are re-pointed at the target (a small
  additive Zed hunk in `terminal_view`, `marley_rehome`, with its touchpoint row), and its
  terminal database row and Marley id move as `added_to_workspace` already does (AD-claude-575).
- **After the move** the rail lists the row under its new project, the rail's saved order puts
  it last there, and the target project shows.

### Out (explicitly deferred)
- Moving to a closed project (#606's): it has no workspace until opened; the menu leaves closed
  projects out, and a drop on a closed header does nothing.
- Moving between windows, into or out of the rail, or a pane's tab into the rail.
- Moving browser rows, thread rows, port rows or worktree rows.
- The shell's working folder: it stays where it is; only the project the row belongs to changes.

## Reference (§20)
Upstream Zed: a tab moves between panes through `workspace::move_item` (`crates/workspace/src/
workspace.rs:12289`), as `Pane::handle_tab_drop` does for Zed's own tab drag (`pane.rs:3877`);
the item keeps its entity. Zed has no move across workspaces in one window, since a MultiWorkspace
shows one at a time; the rail is where both projects show.

### Prior art
- **Behavior maps:** #602's drag (`rail_order.rs` `DraggedRailRow`, `draggable_row`,
  `drop_row`) and its lesson on nested drop targets; Zed's `DraggedTab` and `handle_tab_drop`.
- **Published material:** none needed.
- **Code we already ship:**
  - `workspace::move_item(source, destination, item_id, index, activate, window, cx)`
    (`workspace.rs:12289`), a free function with no workspace check.
  - `TerminalView::new(terminal, workspace, workspace_id, project, window, cx)` (`terminal_view.rs:
    403`) and its fields `workspace`, `project`, `workspace_id` (300-330); `added_to_workspace`
    (2216-2240) moving the terminal database row and `MarleyTerminalIdentity.moved`.
  - `impl Drop for Terminal` kills the shell (`terminal.rs:3800`): any strong handle keeps it.
  - The rail's `TerminalEntry { workspace, view }` (`rail.rs:374`) and `build_snapshot`, which
    lists a terminal under whichever workspace holds it.
  - L-claude-493: a move between panes is a remove then an add.

## UI proof
The scenario `script/e2e/613-move-a-terminal-to-another-project.sh` (sway) opens two projects in
one window, starts `sleep 900` in a terminal of the first (its pid written to a file), then:
- drags the terminal's row onto the second project's header (`dragged.png`: the row under the
  second project, the terminal in front, the same prompt and scrollback); `pid.txt` shows the
  `sleep` still running;
- moves it back with its menu's Move to Project (`menu.png` with the submenu, `moved.png` after).

## Locked-In Decisions
- D1 — The terminal's own `TerminalView` entity moves; nothing is recreated.
- D2 — The Zed hunk is additive: a `marley_rehome` on `TerminalView` that re-points `workspace`,
  `project` and `workspace_id` and resubscribes, used only by the rail.
- D3 — Only open projects of the same window are targets.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a terminal row is dropped on another open project's header, the rail shall list it under that project. | Shot `dragged.png` |
| REQ-002 | WHEN a terminal moves to another project, its shell and what it runs shall keep running, with its scrollback. | `pid.txt`; shot `dragged.png` |
| REQ-003 | WHEN the user chooses Move to Project and a project in a terminal row's menu, the terminal shall move there. | Shots `menu.png`, `moved.png` |
| REQ-004 | WHERE a terminal moved, its links, context menu and Marley's tools shall act in the new project. | Review of `marley_rehome` and its callers |

## Phase Plan
- **P1 Plan** — promote, recall, the design (the drop target, the rehome hunk, the menu).
- **P2 Code** — the drop, the menu, the move, the Zed hunk and its row; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
