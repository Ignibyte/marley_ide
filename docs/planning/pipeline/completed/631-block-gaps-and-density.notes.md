# Gaps between blocks and a header density setting — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-631-block-gaps-and-density.md
- **Pipeline spec:** 631-block-gaps-and-density.spec.md

## Phase 1 — Plan (promoted 2026-10-01)
- **Request:** Chad, 2026-10-01: "lets do 637, 631, 540", run autonomously (a session goal). The
  ticket was Deliberate, waiting for Chad to weigh the look against the diff in Zed's terminal
  crates; naming it is that call.
- **Classification / tier:** feature, prong 1 T5, the largest Zed touch of the slice: two files of
  `terminal_view`, one of `terminal`, the settings files.
- **Checklist:** pick ✓, pre-flight ✓, recall ✓, mint ✓, prior art ✓, spec ✓, design ✓.
- **Recall (§18.3):**
  - PR-claude-render-transform-and-inverse-hit-test-change-in-lockstep-001: a change to how rows
    are placed and the hit-test that reads them back move together; every consumer of the
    inverse is listed below and changed in the same diff.
  - AD-claude-476 and F-claude-557: the bottom shift moves the grid's origin and stores it with
    `set_size`, so the mouse maps through it; it waits while a block sits below the cursor.
  - AD-claude-628: the thirteen row-to-y sites and the four pixel-to-row ones, and the header
    drawn over the prompt's own rows.
  - Brain (`rusty-cli brain ask`, consultation 8643aa6839e1448ea85d71a5f6989759): nothing on this
    seam.
- **Discovery (an Explore map, 2026-10-01):** row to y in `terminal_element.rs`: text runs (167),
  block glyphs (224), backgrounds (265), the cursor (684, fed to the IME bounds and the
  autosuggestion), selection and search ranges (2337-2352, stepped one line height a row), the
  block below the cursor (1659-1675), headers (1716), block elements (1791), the last-row chip
  (1823), the pinned header (1843), `marley_rows_bounds` (2502: wash, selected outline, scope
  outline) and `marley_gutter_bounds` (2523), the bottom shift (1456). Pixel to row:
  `grid_point_and_side` (mouse.rs 202), `content_index_for_mouse` (terminal.rs 3968), the clip
  culling (1567, inline mode only) and `marley_block_at` (2453). Every window position enters
  `terminal.rs` at one of eight `position - terminal_bounds.bounds.origin` lines (2302, 2927,
  3004, 3011, 3028, 3093, 3176, 3264); the internal events (`UpdateSelection`, `FindHyperlink`)
  and vi motion carry grid-space positions from there.

### Design
- **`marley_terminal::display_rows` (new, pure):** `RowMap { inserts: Vec<Insert { row, gap,
  lead }>, shift }` in pixels (f32, the crate has no gpui). `RowMap::new(starts, tall_headers,
  gap, line_height, anchor)` where `starts` are viewport rows where a block starts, `tall_headers`
  the rows whose header takes a line more, `Anchor::Bottom` (shift = minus everything inserted)
  or `Anchor::Top` (shift 0). `offset(row)`: shift plus every insert at or above `row`.
  `lead(row)`: the header's extra line above `row`. `grid_y(y, line_height)`: the inverse: a y
  inside an insert maps to its row's top (D4). `is_identity()`.
- **`terminal_view` (Zed):**
  - `MarleyBlockSpacing { gap_rows: f32, tall_headers: bool }`, a global beside the other hooks.
  - `prepaint`: after the bottom shift, while `content_mode` is scrollable, no block sits below
    the cursor and the screen is not the alternate one, the spans and prompt rows give the map
    (`Anchor::Bottom` at `display_offset == 0`, else `Top`); a one-row prompt's header is tall
    when the density says so; the map goes to `terminal.marley_set_row_map`. The header hook is
    called with the display rows; headers, block elements and the chip are laid out at
    `offset(row) - lead(row)`; the cursor's and IME's y gain `offset(row)`.
  - `paint`: text runs, glyph rects and background rects are painted at the origin moved by their
    row's offset; highlighted ranges are split at each insert inside them; the wash, gutter and
    outlines take the map in `marley_rows_bounds`/`marley_gutter_bounds`.
  - `marley_block_at`: the y through `grid_y` first.
- **`terminal` (Zed):** `marley_row_map: RowMap` on `Terminal`, `marley_set_row_map`, and
  `marley_local(position)`: the window position less the origin, its y through `grid_y`, used at
  the eight lines.
- **`marley_workbench`:** `BlockDensity` (`Comfortable`, `Compact`) on `MarleySettings`, and the
  global set at init and on every settings change (comfortable: half a row, tall headers;
  compact: none).
- **Settings:** `block_density` in `MarleySettingsContent`, `"block_density": "comfortable"` in
  `default.json` with its comment, a dropdown on the Marley page.
- **File manifest:**
  - Marley: `crates/marley_terminal/src/display_rows.rs` (new) and the crate root;
    `crates/marley_workbench/src/marley_workbench.rs`; `script/e2e/631-block-gaps-and-density.sh`.
  - Zed (each with its touchpoint row first): `crates/terminal_view/src/terminal_element.rs`,
    `crates/terminal_view/src/terminal_view.rs` (the global), `crates/terminal/src/terminal.rs`,
    `crates/settings_content/src/marley.rs`, `assets/settings/default.json`,
    `crates/settings_ui/src/marley_page.rs`.

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001..003 | sway; trust; click the terminal; `echo one`, `printf 'two\nthree\n'`, `git status --short` | `631-01-comfortable`: half-row gaps, two-line headers, the prompt on the last row |
| REQ-004 | Escape (keys to the grid); double-click `three` | `631-02-word`: the highlight on `three` |
| REQ-005 | drag from `one`'s output to `two` | `631-03-drag`: highlight on those rows, the gap clear |
| REQ-006 | right-click `two`'s row; shot; Escape | `631-04-menu`: the menu, the outline around `printf`'s block |
| REQ-003 | `seq 1 60` | `631-05-full`: the prompt on the last row of a full screen |
| REQ-007 | wheel up 10 | `631-06-scrolled`: rows from the top edge, the pinned header |
| REQ-007 | wheel down 20 | `631-07-back`: the live prompt on the last row |
| REQ-008 | `profile_setting marley.block_density '"compact"'`; settle | `631-08-compact`: no gaps, one-row headers |
| REQ-009 | review | the map's guards |
Not reachable by a shot: the inline terminal and Inline Assist's block (review only).

### Risks
- A site left out shows as a row drawn at its grid place under a gap: the review walks the
  Explore list one by one.
- Highlight ranges split per insert: the selection and search highlight code steps one line
  height per row from a start y, so a range spanning a gap needs a range per segment.
- Scroll feel: the anchor change between the live screen and one line back moves rows by the
  inserted space. Accepted (Out); pixel scrolling would need Zed's scroll model.

## Phase 2 — Code (2026-10-01)
- **Built:**
  - `marley_terminal::display_rows` (`RowMap`, `Anchor`): `new`, `offset`, `lead`, `breaks`,
    `grid_y`, `is_identity`; `AnchoredBlocks::staged_line`.
  - `terminal_view`: `MarleyBlockSpacing` (`terminal_view.rs`); in `terminal_element.rs` the map
    built in `prepaint` before the cursor (spans, the live prompt's row, the header hook asked for
    a row more over a one-row prompt), the cursor and IME bounds, headers, block elements and the
    last-row chip through it, the map on `LayoutState` and handed to the terminal at the end of
    `prepaint`; at paint, background rects, text runs and glyph rects by row
    (`marley_row_origin`), highlighted ranges split (`marley_split_range`), the wash, gutter and
    both outlines (`marley_mapped_bounds` around the unchanged `marley_rows_bounds` and
    `marley_gutter_bounds`); `marley_block_at` through `marley_local`.
  - `terminal.rs`: `marley_row_map`, `marley_set_row_map`, `marley_local` at the eight origin
    subtractions, `marley_left_pressed`.
  - `marley_workbench`: `block_density` on `MarleySettings`; `block_headers.rs` sets
    `MarleyBlockSpacing` at init and on each settings change.
  - Settings: `MarleyBlockDensity` and `block_density` (`settings_content`), the default in
    `default.json`, the dropdown renderer (`settings_ui.rs`) and Block Density on the Marley page's
    Terminal section.
  - The scenario, calibrated against a dry run (below).
- **Deviations:**
  - The global is `MarleyBlockSpacing` (the setting's enum took `MarleyBlockDensity`, as the other
    Marley enums are named).
  - The live prompt is staged, not a block, so it was not among the spans: the first dry run drew
    no gap above it. `staged_line` gives its row.
  - The map is on whenever nothing clips the view from above, not only when the grid's bounds
    equal what is visible: the bottom shift (#476) moves the bounds past the bottom edge, which
    would have turned the map off for every terminal not yet full.
  - `settings_ui.rs` (the dropdown renderer) was not in the manifest; its row is written.
  - **A bug found on the way (F-…):** a drag in the grid at a prompt selected nothing, and the
    release never reached the terminal. Since #627 the prompt editor takes the focus right after
    a press in the grid, and the element sends a drag and a left release to the terminal only
    while the terminal's own focus handle is focused. Fixed: `marley_left_pressed` (a left press
    in this terminal not yet released) lets both through.
- **Dry runs (calibration, before the gate):** the first run's shots showed the gaps, the two-line
  headers and the double click on `three`, and showed the drag selecting nothing (the bug above)
  and no gap above the live prompt; both were fixed and the second run showed both right.
- **Review:**
  - Lockstep (PR-claude-render-transform-and-inverse-hit-test-change-in-lockstep-001): every
    row-to-y site in the Explore list goes through the map or is off when it is off (the block
    below the cursor, the pinned header at row 0, the bottom shift applied before it); every
    window position entering `terminal.rs` goes through `marley_local`; `marley_block_at` too. The
    internal events and vi motion stay in grid space.
  - REQ-009: no map in an inline terminal (`is_scrollable`), on the alternate screen, with a block
    below the cursor, or with the view clipped from above (the clip culling counts rows by pixel).
  - A row moved up past the top edge is clipped by the element's content mask, as any row is.
  - Re-entrancy: the hook calls read the terminal; the one update (`marley_set_row_map`) runs after
    the content borrow ends, outside any other update of the terminal.
  - Upstream discipline: each hunk carries `// Marley:`; `marley_rows_bounds`, `marley_gutter_bounds`
    and their tests are untouched; the rows were widened before the first edit.
- **Clippy found:** `missing_const_for_fn` on `RowMap::is_identity` (now `const`).
- **Gate:** `script/gates.sh --diff` GATE GREEN (17 of 17), the second run; the first was red on
  the clippy lint above (and its receipt, for the fix made while it ran). Both at `nice 19`.

## Phase 3 — Test (2026-10-01)
- **Build:** `cargo build -p zed --bin marley` at `nice 19` after the gate's fix.
- **Scenario:** `script/e2e.sh script/e2e/631-block-gaps-and-density.sh`, a headless sway, exit 0.
  Focus report: no Hyprland window before or after, no rule added; sway stopped with the run's
  Marley. The plan's rows were reordered once in the dry runs: compact is switched on while the
  three blocks are on screen and switched back before the full screen, so its shot shows the
  headers.
- **Shots, each read (the block area cropped):**
  - `631-01-comfortable` (REQ-001..003): three blocks, each with a two-line header
    (`…/work.…/repo · main` muted, then the command) and its output, about half a row of
    background between each block's last row and the next header; the gutter bars and pills sit
    on the blocks as drawn; `$` on the last row, half a row below `four`, the prompt editor under
    it.
  - `631-02-word` (REQ-004): `three`, under the second gap, highlighted alone; the second block's
    hover actions show where the pointer is.
  - `631-03-drag` (REQ-005): the highlight runs from `one` to the line's end, stops, leaves the gap
    and the second header's folder line clear, and goes on over the second block's command row and
    the start of `two`.
  - `631-04-menu` (REQ-006): the terminal menu with its Block section; the outline is around the
    `printf` block, from its folder line to `three`, as drawn.
  - `631-05-compact` (REQ-008): after `marley.block_density` became `compact` with Marley running:
    no gaps, each header one row (`echo one …/work.…/repo · main`), `$` right under `four`.
  - `631-06-full` (REQ-003): `seq 1 60` filled the screen; `60`, half a row of gap, then `$` on the
    last row; the oldest rows went up past the top edge (`18` cut at the top).
  - `631-07-scrolled` (REQ-007): scrolled back, the pinned header `seq 1 60` over row 0 and the rows
    from `7` down, from the top edge; the scrollbar shows the place.
  - `631-08-back` (REQ-007): the wheel back down: the live screen as in `631-06`, `$` on the last
    row.
- **Fixes in Test:** none (the two found in the dry runs are in Phase 2).
- **REQ-009** rests on the review (no scenario reaches an inline terminal or Inline Assist's block).
- Pre-existing: none seen.

## Phase 4 — Complete (2026-10-01)
- **Docs (§21):** `CHANGELOG.md` (Added: the gaps and the setting; Fixed: the drag at a prompt);
  `docs/marley/three-prong-plan.md` T5 (#631 shipped); `docs/marley_architecture/terminal_blocks.md`
  (`RowMap`, `staged_line`) and `marley_workbench.md` (the spacing global). Touchpoint rows checked
  against what shipped: `terminal_element.rs`, `terminal_view.rs`, `terminal.rs`,
  `settings_content/src/marley.rs`, `settings_ui/src/marley_page.rs`, `settings_ui.rs`,
  `default.json`, each naming #631's hunk.
- **Ledger:** F-claude-631-a-drag-at-a-prompt-selected-nothing-001,
  PR-claude-631-a-gesture-belongs-to-the-element-it-was-pressed-in-001,
  L-claude-631-the-bottom-shift-makes-every-short-terminal-look-clipped-001,
  AD-claude-631-blocks-stand-apart-through-a-display-row-map-the-mouse-reads-back-001.
- **Brain:** `brain decide` on consultation 8643aa6839e1448ea85d71a5f6989759
  (`decisions/marleys-blocks-stand-apart-through-a-display-row-map-the-mouse-reads-back`).
- **Ticket:** closed; archived to `completed/`; committed on `marley/workbench-shell`.
