# Drag to reorder the rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-602-drag-to-reorder-the-rail.md
- **Pipeline spec:** 602-drag-to-reorder-the-rail.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30 (quoted in the ticket). His choice: reorder only, over reorder and
  move between groups.
- **Classification / tier:** feature, medium to large; `marley_rail` and `rail.rs`, perhaps one
  small Zed touch in `multi_workspace.rs`.
- **Recall (§18.3):** #453 deferred drag; AD-claude-453 (reorder through `MultiWorkspace`);
  #542 and AD-claude-542 (attention first, window order as the tie-break, the hold while the
  pointer is over the rail); BF-claude-drag-flag-leaks-on-bounds-gated-mouse-up-255 in
  `failures.md` (gpui's `on_mouse_up` fires only inside the element's bounds; a drag state kept
  by hand must be cleared by an app-level mouse-up, which gpui's own drag API does).
- **Discovery (Explore, 2026-09-30):**
  - Project order: `project_context_menu` (`rail.rs:3349-3395`), `move_project` (3040-3057) →
    `MultiWorkspace::move_project_group_up/down` (`multi_workspace.rs:892-930`), serialized in
    `serialize_now` (1443-1473) and restored by `restore_project_groups` (819-840). The rail's
    project index is not the `MultiWorkspace` index: `build_snapshot` drops groups with no pinned
    workspace (`rail.rs:5136-5140`), and the menu turns an index into a key through
    `snapshot.groups[index].key`.
  - Rows: `marley_rail::walk` (`marley_rail.rs:959-1033`) lays out header, main-checkout
    terminals, worktrees with their terminals, Browser tabs, threads, ports; `project_order`
    (843-860), `arrange_terminals` (870-884), `arrange_threads` (887-901). `build_snapshot`
    gathers terminals and Browser tabs in pane and tab order (5160-5196, `member_browsers`
    5672-5700); threads newest first (4867).
  - Attention: `refresh` copies `order` and `held` (`rail.rs:743-745`); the stable sort
    (`marley_rail.rs:765-776`); the hold set by `.on_hover` on the rail's root (`rail.rs:6261-6268`)
    as window indices, view ids and thread keys (`Held`, `marley_rail.rs:788-798`). Reordering
    projects while a hold is active changes what its indices mean: key the hold by project key.
  - Rendering: rows in a plain `v_flex().overflow_y_scroll()` (`rail.rs:6286-6307`), each through
    `row_frame` (5818-5842) or `row_card` (5974); headers in `render_project_row` (3235).
  - The rail's width handle is Zed's `DraggedSidebar`, a different type: no clash.
  - Stable ids: `marley_terminal_id` (`crates/terminal/src/terminal.rs:1947-1951`, kept by #575;
    `None` for tasks and remote terminals, which then keep today's order); Browser tabs'
    `(workspace_id, item_id)` (`browser.rs:6996-7036`); `ThreadId`.

### Design (to settle at promotion)
- Drag types: `DraggedRailHeader { key }` and `DraggedRailRow { group, section, kind, id }`, each
  its own preview view (a copy of the row's card, as `DraggedTab` renders itself).
- Drop targets: each header and row with `drag_over` drawing a top or bottom line by the pointer's
  half (`on_drag_move` with the row's bounds, as the project panel tracks its target), `can_drop`
  enforcing D1, `on_drop` writing the order.
- The order: `marley_rail` takes a `ManualOrder` (per group, per section, per kind: a list of ids)
  and sorts rows by it before the attention sort, which is stable, so D4 falls out. Projects:
  an additive `MultiWorkspace::move_project_group_to(key, index)` is simpler and exact; swaps
  would pass hidden groups. Decide with the touchpoint cost.
- Saving: rows in a Marley `KeyValueStore` scope (`marley-rail-order`) keyed by the group's key
  or group id, a JSON list of ids per section and kind; groups' place in #601's record.

### Visual check plan
- The runner's sway steps already press, move and release apart (`pointer_down`, `pointer_to`,
  `pointer_up` in `script/e2e.sh`, #487), so a mid-drag shot is `pointer_down`, `pointer_to`,
  `shot line`, `pointer_up`. Each other shot as in the spec.

### Risks
- Hover-based affordances (close buttons, hover chips) flicker during a drag since hover reads
  false; acceptable, note it.
- A terminal with no stable id (a task, a remote terminal) cannot be placed across restarts; it
  keeps today's order after placed rows.
