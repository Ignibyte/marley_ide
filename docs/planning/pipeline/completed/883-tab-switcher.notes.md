# The Ctrl Tab switcher as its board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-883-switcher-ports-containers-collab.md
- **Pipeline spec:** 883-tab-switcher.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #882; split while planning: the switcher here, the port
  rows, Containers and Collab TICKET-890.
- **Classification / tier:** feature, S. One Zed crate (`tab_switcher`, a hook) and Marley's.
- **Recall (§18.3):** #854's `MarleyRecentLook` hook pattern; the ledger row first.
- **Discovery:** probes: the rail's port rows (#521) are close to the board already, with Open,
  Copy and Stop on hover; Zed's switcher draws a small icon and the tab's title, no line, no hint.
- **Decisions:** D1, D2 in the spec.

### Design
- `tab_switcher.rs`: `MarleySwitcherRow { tile, line }`, `MarleySwitcherLook(fn(&dyn
  ItemHandle, &mut Window, &mut App) -> Option<MarleySwitcherRow>)`; `render_match` puts the tile
  in the start slot and the line under the label when the global gives a row; `render_footer` the
  hint while the global is set.
- `marley_workbench`: `src/tab_tiles.rs` (new, small): `switcher_row` (D1, D2), set as the
  global in its `init`.
- **File manifest:** `crates/tab_switcher/src/tab_switcher.rs` (Zed), `tab_tiles.rs` (Marley,
  new), `marley_workbench.rs`, the ledger, the scenario.

## Phase 2 — Code
- **Built:** the ledger row for `tab_switcher.rs` first; there `MarleySwitcherRow`,
  `MarleySwitcherLook` (a function giving a row), asked in `render_match` (the tile in the start
  slot in the icon's place, the line under the label) and `render_footer` (the hint), each hunk
  `// Marley:`. `marley_workbench` depends on `tab_switcher` now; `src/tab_tiles.rs` (new):
  `switcher_row` (D1, D2), set as the global in `init`.
- **Deviation:** the hook returns the row itself, not an `Option` (clippy: the global's presence
  already decides); D1 changed in Test from the tooltip to a file's path in its project, since a
  file's tooltip is its absolute path.
- **Gate:** green on the first run, 17 of 17.
- **Review:** the close button, the indicator and the picking are Zed's, unchanged.


## Phase 3 — Test
- **Scenario:** `script/e2e/883-tab-switcher.sh` (sway): `tab switcher: toggle` from the palette.
- `883-01-switcher`: `rs` rail.rs over "crates/rail/rail.rs", `>_` repo — bash (no line: its
  tooltip is its title), "Hold Ctrl, press Tab to move · let go to open" under them. REQ-001 met.
- `883-02-day`: Marley Day, the same. REQ-001 met.
- The first run showed a file's absolute path; D1 moved to the project path.


## Phase 4 — Complete
- CHANGELOG (Changed), the ledger row checked, `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation d49512f6.
- Ticket closed; pipeline archived; TICKET-890 holds the port rows, Containers and Collab.
