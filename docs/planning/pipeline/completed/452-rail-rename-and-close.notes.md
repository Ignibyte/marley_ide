# Rename and close terminals from the rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-452-rail-rename-and-close.md
- **Pipeline spec:** 452-rail-rename-and-close.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] mint · [x]
  prior-art sweep · [x] spec · [x] design · [x] present (autonomous).
- **Request:** TICKET-452 (W6c), split from #442; autonomous per Chad's goal "lets continue
  working on the remaining tickets".
- **Classification / tier:** feature, small to medium; `marley_workbench` only.
- **Pre-flight:** no active pipeline, README marker present, cargo idle.
- **Recall (§18.3).** Nothing specific to rename or close in the ledger for the fork; the
  gpui-era rename (#177) is history. W3's lesson holds: a popover's menu focuses only on a
  platform frame, and menus are driven by pointer in tests
  (`L-claude-439-focus-lands-in-a-dock-panel-only-once-its-dock-is-open-001` and the #439
  notes). Brain: consultation `397d360c34ff4b088d3ebb8ab7dab401`, nothing on this seam.

### Design
- **Rows** (`rail.rs`, `render_terminal_row`): the row element is wrapped in
  `right_click_menu(("marley-rail-terminal-menu", id))` whose menu has Rename and Close; a
  hover group shows an `IconButton` (`IconName::Close`, small, muted) at the row's end,
  selector `marley-rail-terminal-close-<id>`; the row's click handler checks
  `event.click_count() == 2` for the rename.
- **`Rail::rename_terminal(workspace, view, window, cx) -> anyhow::Result<()>`:**
  `activate_terminal` (shows the project, activates and focuses the view), then
  `view.update(cx, |view, cx| view.rename_terminal(&RenameTerminal, window, cx))`.
- **`Rail::close_terminal(workspace, view, window, cx) -> anyhow::Result<()>`:** the pane
  holding the view (`workspace.pane_for(&view)`), then
  `pane.close_item_by_id(view.entity_id(), SaveIntent::Close, window, cx)`, detached with a
  prompt on error.
- **Agent rows:** `terminal_snapshot` passes the view's `custom_title()` to `agent_title`,
  which prefers it over the CLI's OSC title.
- **File manifest.** Marley only: `crates/marley_workbench/src/rail.rs`, `rail_tests.rs`;
  `menu` is already a dev-dependency (#450). Docs at Complete: `CHANGELOG.md`,
  `docs/marley_architecture/marley_workbench.md`, `docs/marley/workbench-shell.md`.

### Test plan
| REQ | Test |
|---|---|
| 001 | driven: right-click a terminal row (`simulate_mouse_down`/`up` with the right button at its bounds); `MENU_ITEM-Rename` and `MENU_ITEM-Close` are drawn |
| 002 | driven: click `MENU_ITEM-Rename`: the terminal is the active item and `is_renaming()`; `simulate_input("Build")` and `menu::Confirm`: `custom_title()` is "Build" and the row's title is "Build". Driven: a double-click on another row starts its rename |
| 003 | driven: the hover close button removes the terminal from its pane and its row from the rail; the menu's Close does the same for another |
| 004 | driven: an agent row (the #440 fake foreground) with a custom title shows the custom title |
| 005 | `script/gates.sh --diff` |

### Risks
- **Focus after rename.** Zed's rename editor takes focus in the tab; `finish_renaming`
  returns focus to the terminal. The rail's selection follows focus as before.
- **Double-click and the first click.** The first click of a double-click shows the terminal as
  a single click does; the second starts the rename.

## Phase 2 — Code (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] the row's menu · [x] the close button · [x] the
  double-click · [x] `rename_terminal` · [x] `close_terminal` · [x] agent rows' titles ·
  [x] tests · [x] review · [x] clippy at deny level · [x] coverage.
- **Built** in `crates/marley_workbench/src/rail.rs`: `render_terminal_row` wraps the row in
  `right_click_menu` (Rename, Close), swaps the bell's end slot for a close button on hover
  (`end_slot_on_hover`), and renames on the second click of a double-click;
  `Rail::rename_terminal` shows the terminal and runs Zed's `rename_terminal`;
  `Rail::close_terminal` closes the view through its pane with `SaveIntent::Close`;
  `terminal_snapshot` prefers a custom title on an agent row.
- **Deviations from the design.** `close_terminal` takes `&mut App` rather than the rail's
  context: the menu's handlers run with an `App`, and nothing in it needs the rail.
- **Tests** in `rail_tests.rs`: `a_terminal_rows_menu_renames_it_through_its_tab`,
  `a_double_click_on_a_row_renames_its_terminal`, `a_rows_close_button_closes_its_terminal`,
  `a_rows_menu_closes_its_terminal`, and in `rail::tests::agents`
  `an_agent_row_shows_a_name_the_user_gave_its_terminal`; helpers `right_click`,
  `double_click`, `title_of`, `finish_rename` and `is_open`.
- **Review against the criteria.** The close button's and the menu's handlers update the rail,
  the workspace and the pane, none leased then; the close button stops propagation so the row
  under it does not also show the terminal it closes. A close that fails reaches a prompt; a row
  whose terminal is gone logs, as the rail's other handlers do. No Warp source; Zed's rename and
  close are reused through their public API; no Zed crate changed.
- **Negative checks.** Without the double-click branch, the double-click test fails; without the
  custom-title preference, the agent-row test fails.
- **Checks.** `cargo clippy -p marley_workbench --all-targets --all-features -- -D warnings`
  clean. `cargo nextest run -p marley_workbench` under coverage: 86 passed; 1891 of 1891 lines.

## Phase 3 — Test (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] REQ-001 to REQ-004 · [x] the gate · [x] the live drive
  (not run; why below).
- **Tests** (`rail_tests.rs`):

  | REQ | Test |
  |---|---|
  | 001, 002 | `a_terminal_rows_menu_renames_it_through_its_tab`: the right-click menu lists Rename and Close; Rename shows the terminal, its tab renames, and "Build" becomes its title and its row's |
  | 002 | `a_double_click_on_a_row_renames_its_terminal` |
  | 003 | `a_rows_close_button_closes_its_terminal` (hovered first), `a_rows_menu_closes_its_terminal` |
  | 004 | `rail::tests::agents::an_agent_row_shows_a_name_the_user_gave_its_terminal` |
  | 005 | `script/gates.sh --diff` |
- **Runs.** `cargo nextest run -p marley_workbench`: 86 passed. The five new tests three times in
  a row: 5 passed each time.
- **Gate.** `script/gates.sh --diff`: 19 passed, 0 failed, `GATE GREEN [diff]`.
- **Live drive: not run.** It needs clicks. At 01:22 the desk was as it has been all night:
  stay-awake on, and his fullscreen Teams window on the workspace the headless output borrows.
  Owed with the others.
- **Pre-existing failures:** none.

## Phase 4 — Complete (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] document · [x] capture knowledge · [x] close the ticket
  · [x] archive · [x] commit.
- **Documented.** `CHANGELOG.md` (Added); `docs/marley_architecture/marley_workbench.md` (rename
  and close under the rail, the test counts, the known limits); `docs/marley/workbench-shell.md`
  (W6c shipped). No Zed path changed.
- **Knowledge appended.**
  - `AD-claude-452-the-rail-starts-zeds-own-rename-and-close-001`
  - `L-claude-452-driving-a-rows-menu-hover-and-double-click-in-a-test-001`
  - No `F-` block: Code and Test found no bug.
- **Brain.** Consultation `397d360c34ff4b088d3ebb8ab7dab401` closed with `brain decide`:
  `decisions/the-rail-renames-and-closes-terminals-through-zeds-own-tab-rename-and-close`,
  follow-up by 2026-10-07.
- **Closed and archived.** TICKET-452 in `tickets/closed/`; this pair in `pipeline/completed/`.
