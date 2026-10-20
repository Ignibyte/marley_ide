# Any row goes anywhere in its group — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-823-mix-row-kinds-in-a-group.md
- **Pipeline spec:** 823-mix-row-kinds-in-a-group.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-20, under /goal "finish the remaining tickets we discussed". Told a
  row reorders only among its own kind (`DraggedRailRow::takes` needs the same `Run`), he chose
  "mix any order".
- **Classification / tier:** feature; the rail's pure model and its drag.
- **Recall (§18.3):**
  - L-claude-602: a drop stops the event, so the drag ends in the drop handler (`end_drag`).
  - #713: while the window's restore runs, the rows of groups not back yet keep their places;
    `order_changed` already guards it.
  - PR-claude-drag-flag-needs-buttonless-move-selfheal-001: the drag flag clears off the element
    too; `Rail`'s `on_mouse_up_out` already does it.
  - Rusty's brain (consultation `10a03b1250b64980a112cf8bed689f1c`): nothing on this seam.

### Design
- **`crates/marley_rail/src/marley_rail.rs`** (Marley crate):
  - `ProjectSnapshot.order: Vec<Selection>`: the rows the user placed, in their order.
  - `Run::Rows { project }`, the window order's run for a project's main-checkout terminals,
    Browser tabs and threads.
  - `run`: under `RailOrder::Window`, a terminal with no worktree, a Browser tab and a thread give
    `Run::Rows`; a worktree's terminal keeps `Run::Terminals`; the attention order is unchanged.
  - `pub fn movable_rows(project) -> Vec<Selection>`: main-checkout terminals, Browser tabs and
    threads in today's kind order, stable-sorted by their position in `project.order` (unplaced
    last). Used by `walk` and by `drop_row`.
  - `walk`: under the window order, the shown rows of `movable_rows` replace the three sections,
    after Home's page and before the worktrees, then the other tabs, the Files row and the ports.
    The attention order walks as before.
- **`crates/marley_workbench/src/rail_order.rs`** (Marley crate):
  - `arrange` fills `project.order` from the saved places: a terminal's or Browser tab's place
    looked up through `snapshot.places`, a thread's through its key.
  - `drop_row` builds its list from `marley_rail::movable_rows` instead of chaining the kinds.
- **`crates/marley_workbench/src/rail.rs`** and the rail's tests: `order: Vec::new()` where a
  `ProjectSnapshot` is built.

### File manifest
| File | Crate | Change |
|---|---|---|
| `crates/marley_rail/src/marley_rail.rs` | Marley | order, `Run::Rows`, `movable_rows`, walk |
| `crates/marley_workbench/src/rail_order.rs` | Marley | arrange fills the order; drop uses it |
| `crates/marley_workbench/src/rail.rs` | Marley | the new field in two constructors |
| `script/e2e/823-mix-row-kinds-in-a-group.sh` | — | the visual check |

### Visual check plan
| REQ | The scenario | Shot |
|---|---|---|
| REQ-003 | `repo` with a terminal, a Browser tab (palette) and a Marley thread. | `823-01-sections` |
| REQ-001 | The thread's row dragged onto the terminal's, then dropped. | `823-02-line`, `823-03-thread-first` |
| REQ-001 | The Browser tab's row dropped on the terminal's from below. | `823-04-browser-between` |
| REQ-002 | Quit and launch on the same profile. | `823-05-restart` |

### Risks and decisions
- The keyboard order changes with the rail's: it reads the same walk, which is the point.
- A thread with no tab under a project is still a row and still placed by its key.

## Phase 2 — Code
- **Built:** to the manifest. `marley_rail`: `ProjectSnapshot::order`, `Run::Rows`, `run` gives
  it under the window order to a main-checkout terminal, a Browser tab and a thread,
  `movable_rows`, `mixed_rows`, and `walk` lists the run after Home's page and before the
  worktrees. `rail_order.rs`: `arrange` resolves the saved places to `project.order`; `drop_row`
  takes `movable_rows` and appends the worktrees' terminals, so a drop within a worktree still
  finds both rows. `rail.rs`: `order: Vec::new()` in both constructors.
- **Deviations:** clippy's line limit moved `walk`'s Files block into `files_rows` and had
  `build_snapshot` give `matched` inline (before `name`, which the next field moves). No
  behavior change in either.
- **Review:** a drop under the attention order still reads `movable_rows`, a mixed list; the
  per-kind `place` in `arrange` then sorts each kind by it, so the order within a kind is what the
  user left, as before. The restore hold (#713) is untouched: `order_changed` keeps unlisted
  groups' rows. Keys, selection and the switcher read `walk`, so they follow the run.
- **Gate:** `script/gates.sh --diff` GATE GREEN [diff], 17 passed.

## Phase 3 — Test
- **Scenario:** `script/e2e/823-mix-row-kinds-in-a-group.sh`, `compositor sway`, `rail_order`
  set to `"window"` in the run's settings; one run, every check passed, no fix needed.
- **Shots:**
  - `823-01-sections` (REQ-003): under `repo`, `repo — bash`, `about:blank`, then `New Agent
    Thread`: today's order before any drag.
  - `823-02-line` (REQ-001): the thread's card follows the pointer over the terminal's row and the
    drop line sits on the terminal's top edge.
  - `823-03-thread-first` (REQ-001): the thread first, then the terminal, then the Browser tab.
  - `823-04-browser-between` (REQ-001): the Browser tab between the thread and the terminal.
  - `823-05-restart` (REQ-002): after a quit and a launch with no path (the Marley log shows two
    starts, 11:14:57 and 11:15:54), the same order: thread, Browser tab, terminal. The pointer sits
    where the last drop left it, so the Browser tab's row shows its hover as in `04`.
- **Not reached:** the attention order's sections (unchanged code path) and a worktree's
  terminals (no worktree in the scenario), read in the diff.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added), `docs/marley/guide.md` (Drag to reorder),
  `docs/marley_architecture/marley_rail.md` (One run under the window order). No Zed path.
- **Knowledge:** nothing new beyond L-claude-822's menu lesson, which this run's scenario used
  (it clicks the + menu's entries by point). No F block.
- **Brain:** `decisions/the-rails-rows-of-every-kind-share-one-order-under-the-window-order`
  (consultation `10a03b1250b64980a112cf8bed689f1c`, follow-up 2026-11-03).

