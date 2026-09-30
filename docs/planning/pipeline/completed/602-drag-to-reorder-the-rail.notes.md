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

### Promotion (Opus, 2026-09-30, back-to-back run)
- **Pre-flight:** cargo 1.98.1, gate, e2e and shear present, hooks wired, no active pipeline,
  README marker present, cargo idle, branch level with its origin.
- **Recall (§18.3):**
  - PR-claude-drag-flag-needs-buttonless-move-selfheal-001: a hand-kept drag flag must not rely
    on a bounds-gated mouse-up alone. Here: the flag clears on the rail's mouse-up in and out,
    in the drop handlers, and on the first hover-true (which gpui sends only once no drag is
    active).
  - AD-claude-542: the hold keeps window indices; a header drop changes what they mean, so a drop
    releases the hold, and the next hover takes it again from the new order.
  - #601's Test: after a restart Zed brings back only the shown project's workspace, so the
    restart shot uses groups (which Marley reopens) as the headers that show the order.
  - Brain (`rusty-cli brain ask`, consultation 2325a54a): nothing on this seam; only unrelated
    follow-ups came back.
- **Seams re-verified:** see the spec's Prior art. Two corrections to the queued draft: Zed's
  project-group order cannot hold a projectless group (so D3 changed), and a Browser tab's id that
  survives a restart is its page target, not `(workspace_id, item_id)` (the item id is the view's
  entity id, new on every start).

### Design
- **Approach.**
  - *Pure model (`marley_rail`, no gpui):*
    - `Run`, the set a dragged row or header may drop among, and `run(snapshot, &Row)`.
      Headers carry their class under the attention order. Terminals carry their project,
      worktree section and class. Browser tabs carry their project; the attention order
      never sorts them. Threads carry their project and class. Worktree and port rows get `None`
      and never drag.
    - `place(items, places, place_of)`: a stable sort by position in the saved list, with
      unplaced items after, in their order (the `held_position` idiom).
    - `move_to(order, moved, target, before)`.
  - *Saved order (`rail.rs`):* `SavedOrder { groups, headers, rows }` replaces the rail's
    `saved_groups` field. `groups` keeps #601's projectless-group order as it was.
    `headers` holds header ids: `project:<folders, one per line>` or `group:<uuid>`.
    `rows` maps a header id to its row ids: `terminal:<MARLEY_TERMINAL_ID>`,
    `browser:<target>`, `thread:<key>`, and `view:<entity id>` for a row with no stable id,
    which then keeps its place for the session only. It is kept in the sidebar blob as
    `marley_order` (`read_rail_order`, `write_rail_order`), beside `marley_groups`.
  - *Applying it:*
    - `rail_groups` sorts the listed groups, projects and projectless groups together, by
      `headers`. So the snapshot's project indices, `note_focus` and `GroupEntry` all follow one
      order, and #542's attention sort, being stable, uses it as its tie-break (D4, REQ-008).
    - `build_snapshot` records each row's place id in `Snapshot::places`. `arrange` then sorts
      each project's terminals, Browser tabs and threads by `rows[header]`.
  - *Dragging:*
    - `DraggedRailHeader { place, position, class, name }` and `DraggedRailRow { selection,
      project, run, position, title }`. Each renders its own preview: a small raised card with
      the title, as `DraggedTab` renders itself.
    - `on_drag`'s constructor sets `Rail::dragging` through the rail's weak handle; it runs in
      the mouse-move listener, outside any rail update.
    - Rows: the row card takes `on_drag`, a `can_drop` (same run, not itself), `drag_over` with
      `border_0` and then a 2 px top or bottom border in `drop_target_border` (D6: top when the
      target sits above), and `on_drop`.
    - Headers: `render` now wraps each project's header and rows in one block, a `v_flex` with
      the same gap. The block is the header's drop target, with the line above or below the block.
  - *Drops:*
    - `drop_header` rebuilds `headers` from the rail's current order with the move applied.
    - `drop_row` rebuilds `rows[header]` from the group's current terminals, then Browser tabs,
      then threads, with the dragged kind moved.
    - Both keep only current headers, clear the keyboard's header cursor on a header drop, call
      `MultiWorkspace::serialize` (it spawns, so it is safe from a rail listener), end the drag
      and refresh.
  - *The hold (D5):*
    - The root's `on_hover` ignores `false` while `dragging`.
    - `end_drag` (the root's left mouse-up and mouse-up-out, the drop handlers, and a
      hover-true self-heal) clears `dragging` and the hold, and refreshes. The next pointer move
      over the rail takes the hold again from the new order.
  - *Move Project Up and Down:* `move_project` swaps the project with its neighbour in the rail's
    header order and saves it (D3, REQ-009). The menu's first and last already count every header.
  - *Click:* every row opens `on_click`, which gpui drops once a drag starts past its 2 px
    threshold (REQ-005).
- **File manifest.**
  - `crates/marley_rail/src/marley_rail.rs` (Marley): `Run`, `run`, `place`, `move_to`.
  - `crates/marley_workbench/src/rail.rs` (Marley): `SavedOrder`, the blob helpers,
    `Snapshot::places`, `arrange`, header ids, the drag types and previews, the row and block
    targets, `drop_header`, `drop_row`, `end_drag`, the hold guard, `move_project` on the rail's
    order, and the block grouping in `render`.
  - No Zed crate changes; no `zed-touchpoints.md` row.
  - `script/e2e/602-drag-to-reorder-the-rail.sh` (Test).
- **Visual check plan** (sway; `pointer_down`, `pointer_to` in steps, `pointer_up`):

| REQ | Scenario step | Shot |
|---|---|---|
| 001, 007 | Press Web's header, move over `repo`'s block, hold | `line.png`: preview under the pointer, line above `repo`'s block, rows unmoved |
| 002 | Release there | `projects.png`: Web, `repo`, Scratch |
| 003 | Drag the `three` terminal onto `one` | `rows.png`: three, one, two |
| 004 | Drag `one` onto Scratch's header, release | `refused.png`: order unchanged |
| 009 | Right-click `repo`, Move Project Down | `move.png`: Web, Scratch, `repo` |
| 005 | Click `two` without moving | `click.png`: `two` selected and shown |
| 006 | Quit, start with no path | `restart.png`: Web, Scratch, `repo`; three, one, two |
| 008 | Not reachable by a shot: the scenario's shells are idle, all one class | Review: the stable attention sort over the placed order, and `can_drop`'s class check |

  Browser tab and thread rows share the row helper; the review covers them (REQ-003).

### Risks
- **Hover-based affordances.** Close buttons and hover chips disappear during a drag, since
  hover reads false. This is accepted.
- **Rows with no stable id.** A task terminal or a remote terminal keeps its place only for the
  session.
- **Collisions.** Two projects with the same folders on different hosts share one header id and
  sort together.
- **Zed's own sidebar.** Zed's layout keeps Zed's project order; the rail's order is its own (D3).
- **New projects after a drag.** A project added after the user has placed the headers lands at
  the bottom (unplaced goes last). Before any drag, the order is as today.
- **Pruning.** A drop saves only the headers listed now. A project hidden after a restart (the #601
  gap) loses its place at the next header drop.

## Phase 2 — Code (2026-09-30)
- **Built:**
  - `marley_rail`: `Run` and `run`, `place` (a stable cached-key sort, unplaced last) and `move_to`.
  - `rail_order.rs`, a new child module of `rail.rs`, as the switcher is:
    - `SavedOrder` and `Hold`;
    - the blob helpers, and the header and row places;
    - `arrange`;
    - the two drag types with their previews, and the drop line;
    - `draggable_header`, `header_block` and `draggable_row`;
    - `drop_header`, `drop_row`, `order_changed`, `start_drag` and `end_drag`.
  - `rail.rs`:
    - `saved_groups` became `saved_order`, and `held_order` became `hold`;
    - `Snapshot::places`;
    - `rail_groups` sorts by the saved headers;
    - `move_project` moves in the rail's order;
    - `render_blocks` and `render_row`, with `render` now building on them;
    - the hover guard, and the left mouse-up in and out;
    - `render_project_row` became a method that takes the header's drag.
- **Deviations from the plan:**
  - The drag flag lives in `Hold`, beside #542's held order, not in a bool on `Rail`. Clippy's
    `struct_excessive_bools` refused a fourth bool, and one field for "what holds the order"
    reads better.
  - `render_terminal_row`'s right-click menu moved into `terminal_context_menu`. The drag
    parameter took the function to 101 lines, one over `too_many_lines`.
  - `render_project_row` reads `last` and `filtering` from `self`, which keeps it at seven
    parameters.
- **Review of the diff (per criterion):**
  - REQ-001: `can_drop`, then `drag_over`, only for the drag's own type, so a header's line never
    shows on a row, nor a row's on a header block.
  - REQ-004:
    - A row dropped on another group's row: the row's listener takes the drag, `can_drop` is
      false, and nothing is called; the root's mouse-up ends the drag.
    - A row dropped on a header: no listener of its type, and the root's mouse-up ends it.
  - REQ-005: rows and headers open `on_click`, which gpui drops once the drag has started.
  - REQ-007: the hover guard, and `end_drag` from the root's left mouse-up in and out, from each
    drop handler and from a hover-true (the self-heal of PR-claude-drag-flag-needs-buttonless-
    move-selfheal-001).
  - REQ-008: the attention sorts in `marley_rail` are stable over the placed order, and a drop
    needs the dragged row's class.
  - REQ-009: `move_to` with the neighbour in the rail's order.
  - Re-entrancy:
    - `on_drag`'s constructor updates the rail through a weak handle from the mouse-move
      listener, outside any update.
    - The drop listeners call `MultiWorkspace::serialize`, which only spawns.
  - Provenance: no Warp source. The drag wiring follows gpui's documented API and the pane's
    pattern; no Zed function body was carried over.
  - Upstream: no Zed file touched.
  - **Found and fixed in review:** a drag that began after the hold was released (a Move Project
    from the menu, with no hover since) held no order. `start_drag` now takes the hold if none
    is held.
- **Gate:** `just gate-diff`: 17 PASS, 0 FAIL, receipt written (`GATE GREEN [diff]`).

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/602-drag-to-reorder-the-rail.sh`, under sway. It uses `drag_to`
  (press, six moves, hold) before each mid-drag shot. Built with `just build`; run with `just e2e`,
  shots in the scratchpad.
- **First run: measuring.**
  - Its guessed coordinates missed Web's header. Its second drag was Scratch's header over
    `repo`'s block instead, and it already showed a header's preview, the line above the block and
    the drop.
  - Its Move Project Down moved `repo` past the Web group.
  - Its restart listed Scratch, Web, `repo`.
  - `layout.png` gave the positions: `repo` 96; `one`, `two` and `three` at 142, 200 and 258;
    Scratch 317; Web 364.
- **Second run: every shot read.**
  - `layout.png`: `repo` with one, two, three; Scratch; Web.
  - `line.png` (REQ-001, REQ-007): the "Web" preview card by the pointer, and a 2 px line across
    the top of `repo`'s block (confirmed on a 3x crop). The rows and headers are where they were,
    Web still last: nothing re-sorted mid-drag.
  - `projects.png` (REQ-002): Web, `repo` with its three rows, Scratch.
  - `rows-line.png` (REQ-001): the "three — bash" preview, and a line on the top edge of `one`'s
    row; the rows are in place.
  - `rows.png` (REQ-003): three, one, two under `repo`.
  - `refused-line.png` (REQ-001, REQ-004): the "one — bash" preview over Scratch's header, and no
    line on the header or on any row.
  - `refused.png` (REQ-004): still three, one, two. Scratch is only hovered, not opened (the title
    bar still reads `repo`).
  - `move.png` (REQ-009): Web, Scratch, `repo`. Move Project Down moved `repo` one place from
    where the drag left it, past a group.
  - `click.png` (REQ-005): `two` selected, its tab active, its prompt at `cd two`.
  - `restart.png` (REQ-006): after a quit and a start with no path, Web, Scratch, `repo`, with
    three, one, two, each back in its folder. The pane's tabs stay one, two, three, as D2 says.
- **Not reachable by a shot:**
  - REQ-008: the scenario's shells are idle, all one attention class. It is covered by the
    review: stable sorts over the placed order, and `can_drop` on the class.
  - Browser tab and thread rows (REQ-003): they share `draggable_row` with terminals; covered by
    the review.
- **Focus:** the run was in its own headless sway; Hyprland had 0 Marley windows before and after,
  and no rule was added.
- **Fixes:** none needed; the gate's green from Phase 2 stands.
- **Pre-existing, not in scope:** none new.

## Phase 4 — Complete (2026-09-30)
- **Documented:**
  - `CHANGELOG.md` (Unreleased, Added).
  - `docs/marley_architecture/marley_workbench.md`: a new section, "Dragging to reorder the
    rail". The #453 and #600 paragraphs now say where the order comes from.
  - `docs/marley_architecture/marley_rail.md`: "Dragging".
  - `docs/marley/workbench-shell.md`: the slice line.
  - `docs/marley/guide.md`: "Drag to reorder", and Move Project's new reach.
  - `docs/marley/walkthrough.md`: stop 2.11.
  - The guide page: the new article `rail-drag` with its contents line, and the attention-order
    and project-menu articles updated. It was assembled from the scratch parts, which rebuilt the
    shipped page byte for byte before the edit.
  - No Zed path touched, so no touchpoint row.
- **Knowledge:**
  - F-claude-602-a-drag-begun-after-the-hold-was-let-go-held-nothing-001;
  - L-claude-602-nested-drop-targets-need-a-drag-type-each-001;
  - L-claude-602-zeds-project-group-list-never-holds-a-folderless-workspace-001;
  - AD-claude-602-the-rail-owns-its-order-001.
  - Brain: `brain decide` closed consultation 2325a54a as
    `decisions/marleys-rail-owns-its-drag-order`.
