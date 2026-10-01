# Move a terminal to another project in the rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-613-move-a-terminal-to-another-project.md
- **Pipeline spec:** 613-move-a-terminal-to-another-project.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones".
- **Batch:** wave 2 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - L-claude-602-nested-drop-targets-need-a-drag-type-each-001: a drop goes to the deepest `on_drop` of its type, so the drop on another project sits on its header frame.
  - L-claude-493 and F-claude-493: a move between panes is a remove then an add; `pane_for` lags until the events drain.
  - AD-claude-575: `added_to_workspace` moves the terminal database row and Marley's id, except for task terminals.
  - The view's `workspace` and `project` are read by `close_guard.rs`, `agent_events.rs`, `system_one.rs`, `push.rs` and Zed's link opening, so they must follow the move.
- **Discovery:** one Explore sweep for the wave (2026-09-30) over the rail, the ports scan, the
  thread store, terminal views and tooltips; the spec's Prior art cites what applies here, and the
  Plan phase re-verifies each seam at promotion.

### Promotion (Opus, 2026-09-30)
- The pair moved to `active/`; the BACKLOG row removed; the ticket in-progress.
- **Recall, again:** as queued, plus F-claude-575-the-terminal-panels-cleanup-deleted-the-center-
  terminals-rows-001 (a workspace's cleanup can delete terminal rows it no longer holds), which a
  restart check now covers (REQ-005). The brain (consultation fb231e1348544270b6463008070ce628):
  nothing on this seam.
- **Seams re-verified:**
  - `workspace::move_item` (`workspace.rs:12289-12330`): `remove_item_and_focus_on_pane` on the
    source, `add_item_inner` on the destination, which runs the item's `added_to_pane` and so
    `added_to_workspace` with the new workspace.
  - `TerminalView::added_to_workspace` (`terminal_view.rs:2216-2240`) already moves the terminal
    database row and Marley's id; the view's `workspace`, `project` (300-303) and
    `_terminal_subscriptions`, set by `subscribe_for_terminal_events(&terminal, workspace, …)`
    (1419), are never updated after `new`.
  - `ContextMenu::submenu(label, builder)` (`ui/src/components/context_menu.rs:799`).
  - `rail_order.rs`: `DraggedRailRow` (186-216), `header_block`'s `on_drop::<DraggedRailHeader>`
    (272-300); L-claude-602: a drop goes to the deepest element with an `on_drop` of its type,
    and a successful drop stops propagation, so the handler ends the drag.
  - #606's `hand_off` opens more projects in the running Marley's window.

### Design
- **Approach.**
  - *The Zed hunk* (`crates/terminal_view/src/terminal_view.rs`, its row updated first): in
    `added_to_workspace`, when `workspace.weak_handle()` is not the view's `workspace`, set
    `workspace` and `project` (`workspace.project().downgrade()`) and replace
    `_terminal_subscriptions` with `subscribe_for_terminal_events(&terminal, workspace, window,
    cx)`, under a `// Marley:` comment saying why (#613). The spec's `marley_rehome` became this:
    the move already calls `added_to_workspace`, so no new public function is needed.
  - *The move* (`rail.rs`): `Rail::move_terminal(terminal_id, target_group, window, cx)` finds the
    `TerminalEntry` (its workspace and view), the source pane (`pane_for`), and the target
    group's workspace's active pane; then, deferred through `window.defer` so no pane or
    workspace is updated inside the rail's update, `workspace::move_item(source, destination,
    view_id, destination.items_len(), true, window, cx)`, and the target workspace activated in
    the MultiWorkspace. A closed target or a missing entry does nothing.
  - *The drop* (`rail_order.rs`): the header's frame (not its block) gets `can_drop` for a
    `DraggedRailRow` that is a terminal of another project, a `drag_over` style that marks the
    header, and `on_drop`, which ends the drag and moves the terminal.
  - *The menu* (`rail.rs` `terminal_context_menu`): Move to Project, a `submenu` of the window's
    other open projects by name, each moving the terminal there.
- **File manifest:**
  - Zed: `crates/terminal_view/src/terminal_view.rs` (`added_to_workspace`), its row in
    `docs/marley/zed-touchpoints.md` first.
  - Marley: `crates/marley_workbench/src/rail.rs`, `crates/marley_workbench/src/rail_order.rs`.
- **Visual check plan** (`script/e2e/613-move-a-terminal-to-another-project.sh`, sway): `repo`
  opened, `repo-b` handed off into the same window; in `repo`'s terminal, `echo marker-613` then
  `sleep 913`.

  | REQ | Set up and do | Shot |
  |---|---|---|
  | REQ-001, REQ-002 | Drag the terminal's row onto `repo-b`'s header | `dragged.png` (the row under `repo-b`, the terminal showing `marker-613`), `pid.txt` (`sleep 913` alive) |
  | REQ-003 | Right-click the row, Move to Project, `repo` | `menu.png`, `moved.png` |
  | REQ-005 | Move it to `repo-b` again, quit, start Marley | `restored.png` |
  | REQ-004 | Review | the hunk and the readers named in the recall |

- **Risks and decisions:**
  - The old project's weak list of terminals keeps the moved terminal until it is released;
    nothing kills a terminal through it.
  - A task terminal's database row is not moved (Zed skips it in `added_to_workspace`); moving
    one works for this session only.

## Phase 2 — Code (2026-09-30)
- **Built,** as designed:
  - `terminal_view.rs` (Zed, its row first): `added_to_workspace` re-points `workspace` and
    `project` and resubscribes to the terminal's events when the view joins a different workspace.
  - `rail.rs`: `move_targets` (the window's other open groups, by place and name),
    `move_terminal` (deferred through `window.defer`: `pane_for`, the target's active pane,
    `workspace::move_item` to its end, then the target workspace shown), the terminal menu's
    Move to Project submenu (targets computed at render, so the menu reads no entity when it
    opens), and the header made a drop target.
  - `rail_order.rs`: `DraggedRailRow::moves_to` and `Rail::takes_terminals` (`can_drop`, a
    `drop_target_background` while over, and an `on_drop` that ends the drag and moves).
- **Deviations:** projectless groups (#600) are targets too: they are headers with a workspace
  of their own, and nothing makes a move there different.
- **Review of the diff:**
  - REQ-001: a terminal row dropped on another open header moves; a drop on its own header or on
    a closed one is refused by `can_drop`.
  - REQ-002: the same view moves, so its `Entity<Terminal>` and the shell stay.
  - REQ-003: the submenu lists the other open groups and moves on a choice.
  - REQ-004: after the move, `workspace` and `project` are the target's and the terminal's
    events go to it; `marley_workspace()` (read by `close_guard.rs`, `agent_events.rs`,
    `system_one.rs`, `push.rs`) returns the new one.
  - REQ-005 rests on `added_to_workspace`'s database move, which the scenario checks.
  - Re-entrancy: the move is deferred, so no pane or workspace is updated inside the rail's
    update; the menu's targets are computed in render.
- **Gate:** `just gate-diff` 17 PASS, 0 FAIL (`GATE GREEN [diff]`).

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/613-move-a-terminal-to-another-project.sh` under sway: `repo` opened,
  its terminal running `echo marker-613` then `sleep 913`; `repo-b` handed to the same window
  (#606's `hand_off`). The positions were measured from the first runs' shots (`layout.png`, then
  `menu.png`) and are named in the scenario.
- **Shots, each read** (the last run, which ran every step):
  - `layout.png`: `repo-b` shown and first, `repo` under it with its row "repo — sleep 913,
    sleep 913 · running".
  - `over.png` (REQ-001): the row dragged over `repo-b`'s header, which takes the drop target's
    background, the drag's card "repo — sleep 913" under the pointer.
  - `dragged.png` (REQ-001, REQ-002): the row under `repo-b`, after "repo-b — bash", with its folder
    line (its shell's folder is outside `repo-b`); its tab in front in `repo-b`'s pane, the
    terminal showing `$ echo marker-613`, `marker-613` and `$ sleep 913` `running`. `pid.txt`:
    "sleep 913 runs: pid 1064617".
  - `menu.png` (REQ-003): the row's menu with Rename, Move to Project ▸ and Close, the submenu
    open on `repo`.
  - `moved.png` (REQ-003): after choosing `repo`, the row back under `repo`, `repo` shown with the
    same terminal and scrollback; `pid-moved.txt`: the same pid, 1064617.
  - `restored.png` (REQ-005): after a second drag to `repo-b`, a quit and a launch, `repo-b` open
    with "repo-b — bash" and the moved terminal as "repo — bash" in its folder, `repo` listed
    closed and dimmed.
  - REQ-004 is the review's (Phase 2).
- **Reds:** none. The first runs measured positions; no source changed in this phase.
- **Focus:** sway stopped with the run's Marley each time; Hyprland had no Marley windows
  before or after.

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md` (Added: move a terminal to another project);
  `workbench-shell.md` (#613 in the shell's list); `marley_workbench.md` (a Move to another
  project bullet in the rail section); the guide and the guide page (the drag text, which said a
  row dropped on a header goes back) and the walkthrough (2.11's step 3 and its check, which said
  the same). The `terminal_view.rs` touchpoint row was written before the code and describes the
  hunk.
- **Knowledge:** AD-claude-613-a-terminal-moves-as-its-own-view-001,
  L-claude-613-zeds-added-to-workspace-is-the-hook-for-an-item-that-changes-workspace-001.
- **Brain:** consultation fb231e1348544270b6463008070ce628 closed with `brain decide`
  (`decisions/marley-moves-a-terminal-to-another-project-as-its-own-view`, follow-up by
  2026-10-14).

