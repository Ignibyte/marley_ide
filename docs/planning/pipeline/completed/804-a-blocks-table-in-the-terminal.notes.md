# A block's table drawn in the terminal — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-804-a-blocks-table-in-the-terminal.md
- **Pipeline spec:** 804-a-blocks-table-in-the-terminal.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17 (see #803): "formatted automatically in a table".
- **Discovery (drafted by /spec; re-verified at promotion):** the row map
  (`crates/marley_terminal/src/display_rows.rs`, `RowMap::new`); its build and the cases that turn
  it off (`crates/terminal_view/src/terminal_element.rs`, prepaint); the header rows' element and
  the filtering of their cells; `marley_block` (the header); inline assist's `BlockProperties`
  (`crates/terminal_view/src/terminal_view.rs`). A tall pixel gap was ruled out: the map adds
  uniform space above rows only, and a gap pushes grid rows out of view.
- **Recall (§18.3):** #628's header rows (cells filtered from rects, text runs, block-element
  rects and search marks; the element prepainted over the rows' extent through the row map);
  #629 (a search mark in a hidden row would draw over the element); #631 (the row map is off on
  the alternate screen, under inline assist, with a clip from above); PR-claude-631 (a gesture
  stays with the element it was pressed in: a table shown during a drag would take it, so a held
  selection shows the text); L-claude-628 (a copy is checked by pasting into the prompt editor
  with `copy_on_select`); L-claude-803 (a chip's point follows the output's line count).
- **Promoted 2026-10-17**; seams re-read on the tree #803 left: `marley_prompt_rows` and
  `marley_terminal::prompt_rows`, `marley_rows_extent`, `marley_maps_rows`, the paint loop over
  `marley_blocks`, `Terminal::matches` (grid lines from the live screen's top), `selection_started`.

### Design
- **`marley_terminal::anchored::output_rows` (Marley):** for each finished block, its output's
  viewport rows and how many of its lines are above the viewport, as `prompt_rows` does for
  prompts.
- **`terminal_view` (Zed, additive):** `MarleyBlockBody`, a global hook beside `MarleyBlockChip`:
  given the view, the terminal, a block, its first output line on screen, the rows and the line
  height, an element or none. In `terminal_element.rs`, only while `marley_maps_rows`, each
  block's element is laid out over its output rows' extent as the headers are, its rows join
  `marley_hidden_rows` (so their cells and search marks go), and the elements go first in
  `marley_blocks`.
- **`settings_content/src/marley.rs` (Zed, additive file rows):** `block_tables: Option<
  MarleyBlockTables>`, `offer` (default) or `in_place`; `MarleySettings::block_tables`.
- **`block_tables.rs` (Marley):** `Found` also keeps the sample's shape (a file block too) and
  `lines`, the output's row count when it fits (rows + header = lines, not cut); `in_place:
  Option<Arc<Table>>`; the toggle parses the output off the main thread and stores it, or drops
  it; `in_place` mode does that as a block is read; the body hook draws the grid for the rows on
  screen while the block's row count still equals `lines` and no selection is being dragged,
  marking rows that hold one of `Terminal::matches`.

### File manifest
| File | Crate | Change |
|---|---|---|
| `crates/marley_terminal/src/anchored.rs`, `marley_terminal.rs` | Marley | `output_rows` |
| `crates/terminal_view/src/terminal_view.rs`, `terminal_element.rs` | Zed | the hook and its layout (rows extended) |
| `crates/settings_content/src/marley.rs` | Zed | the setting (row extended) |
| `crates/marley_workbench/src/marley_workbench.rs`, `block_tables.rs` | Marley | the setting's value, the toggle, the body |
| `docs/marley/zed-touchpoints.md` | docs | three rows |

### Visual check plan
| REQ | Set-up and action | Shot |
|---|---|---|
| REQ-004 | `offer`: `cat data.csv` | `804-01-offer` (text, Table and Inline) |
| REQ-001 | Inline clicked | `804-02-toggle` |
| REQ-002 | scrolled up 5 lines | `804-03-scrolled` |
| REQ-003 | block menu Copy Output, Ctrl+V in the prompt editor | `804-04-copy` |
| REQ-001 | Text clicked | `804-05-text` |
| REQ-004 | `in_place` set, `cat data.csv` again | `804-06-in-place` |
| REQ-003 | Ctrl+F `user07` | `804-07-find` |

### Risks and decisions
- One element per block on screen, prepainted every frame; it lays out only the rows on screen.
- A rewrap (a width change) changes the block's row count, and the table hides until the width
  fits again: the grid's rows are the truth.

## Phase 2 — Code
- **Built:** as the design, and the setting also on the Marley settings page (Terminal → Block
  Tables), with its dropdown and its default in `default.json` (three more ledger clauses).
  `MarleySettings::from_settings` reads the two block settings through `block_settings`, which
  keeps it under clippy's 100 lines.
- **Review of the diff:** the body hook is called in prepaint with `&App` and reads only the
  global and the terminal; it returns none for a block not drawn in place, so a terminal without
  tables pays one map lookup per block on screen. The element is opaque, so a selection left on
  its rows after a drag stays under it, and the text shows while a drag is held
  (`selection_started`). The table hides when the block's row count changes (a rewrap), so it
  never sits over the wrong rows.
- **Clippy:** `too_long_first_doc_paragraph`, `clone_on_ref_ptr`, `too_many_lines`
  (`from_settings`), each fixed at the source.
- **Gate (`804-gate-1.log`)**, on the tree after the row fix and the final scenario: **GATE
  GREEN [diff]**.

## Phase 3 — Test
- **Scenario:** `script/e2e/804-a-blocks-table-in-the-terminal.sh` (`compositor sway`), 60
  records so the block is taller than the screen.
- **Run 1:** every step worked, and the shots found a real bug: each table row was 19 px against
  the grid's 19.5 px, so the table drifted up, about 20 px by the screen's bottom (user60 at
  y 871, its text row at 893). Fixed: each row is placed at its own grid row's top. The menu's
  Copy Output measured at (750, 738).
- **Run 2 (final).** The shots:
  - `804-01-offer` (REQ-004): `offer`: the block's CSV text, and on its last row (its first is
    off screen) Table and Inline, Inline's tooltip "Draw the table over the block's output".
  - `804-02-toggle` (REQ-001): the same rows as a table (Name, Team, Commits, Lines, Notes in
    padded columns, every other row shaded), user60 on y 892 where its text was, the prompt under
    it unmoved; the toggle reads Text.
  - `804-03-scrolled` (REQ-002): scrolled up to the block's start: the folder and the command,
    then the table's bold header row on the first output row, user01 to user41, row 7's Notes
    "fixed the parser, again" in one column.
  - `804-04a-menu`, `804-04-copy` (REQ-003): the Block menu over the table; after Copy Output,
    Ctrl+V filled the prompt editor with the CSV lines, commas and all.
  - `804-05-text` (REQ-001): Text clicked: the CSV text back, the toggle reads Inline.
  - `804-06-in-place` (REQ-004): `in_place` set, `clear`, `cat data.csv`: the table drawn by
    itself, the toggle reading Text.
  - `804-07-find` (REQ-003): a terminal search for `user50` (1/1): that table row marked in the
    search color.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` ("A block's table drawn in the terminal"); `docs/marley/guide.md`
  (The block terminal → Blocks); `docs/marley_architecture/terminal_blocks.md` ("Since #804");
  `docs/marley/zed-touchpoints.md` (the `terminal_element.rs`, `terminal_view.rs`,
  `settings_content/src/marley.rs`, `settings_ui.rs`, `marley_page.rs` and `default.json` rows).
- **Knowledge:** F-claude-804-table-rows-of-a-rounded-height-drifted-off-the-grid-001,
  L-claude-804-place-each-row-at-its-grid-rows-top-001.
- **Brain:** no decision recorded: the spec's D1 to D3 stand; the promotion's choices (Marley's
  own grid, the toggle beside the chip, one record per row) are in the spec.
