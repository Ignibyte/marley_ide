# A block's output opens as a table — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-803-a-blocks-output-opens-as-a-table.md
- **Pipeline spec:** 803-a-blocks-output-opens-as-a-table.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17 (see the ticket).
- **Research:** `docs/planning/design-notes/terminal-output-as-components-2026-10-17.md`.
- **Discovery (drafted by /spec; re-verified at promotion):** `AnchoredBlock` and
  `AnchoredBlocks` (`crates/marley_terminal/src/anchored.rs`); `Terminal::block_output`,
  `block_output_kept` (`crates/terminal/src/terminal.rs`); `absolute_lines_text`
  (`crates/terminal/src/alacritty.rs`); `MarleyBlockChip` (`crates/terminal_view/src/terminal_view.rs`,
  set in `crates/marley_workbench/src/bookmarks.rs`); the block menu
  (`crates/marley_workbench/src/blocks.rs`); `TableView::new`, `set_contents`,
  `TableCell::Generated`, `TabularDataPreviewPane::open_preview_to_the_side_of_pane`
  (`crates/tabular_data_preview`). `marley_workbench` does not depend on `tabular_data_preview`
  yet.
- **Recall (§18.3):** #620's `failures.rs` (an observer on each view's terminal reading each
  block as it finishes, from `next` and `restored_count`, a cache by `(terminal, index)`, a chip
  in `bookmarks::chip`, a menu item); F-claude-733 and its rule: a click that adds a tab hands
  the open to `window.defer`; #764: a restored block is history, read once; #546: blocks are on
  the main grid, so a full-screen program's output never reaches `block_output`.
- **Promoted 2026-10-17**; seams re-read on the tree #802 left. No `csv` crate in `Cargo.lock`;
  `TableView::new`, `set_contents`, `TableLikeContent`, `TableCell::Generated`, `LineNumber`
  and `ui::table_row::TableRow` are public; `TabularDataPreviewPane::open_preview_in_pane` takes
  an `Entity<Editor>`; `tabular_data_preview::init` runs in Zed's `main`.

### Design
- **`marley_terminal/src/tables.rs` (Marley, new):** `detect(text) -> Option<TableShape>` over
  the first 1 MB and 2,000 lines (kind, columns, rows); `parse(text) -> Option<Table>` (headers
  and rows of strings, the same rules, the whole text); `tabular_file(command, folder)`: a
  `cat`, `head`, `tail`, `bat` or `batcat` of one `.csv`, `.tsv`, `.psv`, `.ssv` or `.jsonl`
  file, with no pipe, redirect or substitution. The reader: RFC 4180 quoting (quoted fields,
  `""`, line breaks inside quotes); per delimiter `,`, tab, `;`, `|`, the commonest field count
  `n >= 2` held by at least 95 percent of at least 3 records; JSON: an array of at least 2
  objects, or at least 2 lines each an object, columns in first-seen key order, values as text;
  markdown: a header row, a delimiter row of the same cell count, and at least one row.
- **`terminal` (Zed, additive):** `alacritty::absolute_lines_styled` and
  `Terminal::block_output_styled(block)`: whether any cell of the block's first 2,000 output
  lines has a color or a style, read under the lock as `block_output` reads text.
- **`marley_workbench/src/block_tables.rs` (Marley, new):** an observer per terminal view (as
  #620's) that reads each newly finished block: a tabular file named by its command (checked
  with `Fs::metadata`), else its output unless styled or not kept, detected off the main thread;
  the result cached by `(terminal, index)` and the view notified. `chip` (Table, tooltip with the
  counts or the file name) and `open` (deferred): for a file, a buffer opened through the
  project, an `Editor::for_buffer`, and Zed's preview in the terminal's pane; for text, a
  `BlockTable` item holding a `TableView` filled from `parse`, titled with the command. A second
  open brings the tab forward.
- **`bookmarks.rs`, `blocks.rs` (Marley):** the chip after Ask the agent; Open Output as Table
  after Send to Agent.

### File manifest
| File | Crate | Change |
|---|---|---|
| `crates/marley_terminal/src/tables.rs`, `marley_terminal.rs`, `Cargo.toml` | Marley | new module; `serde_json` |
| `crates/terminal/src/alacritty.rs`, `terminal.rs` | Zed | the styled read (ledger rows extended) |
| `crates/marley_workbench/src/block_tables.rs`, `marley_workbench.rs`, `Cargo.toml` | Marley | new module; `tabular_data_preview` |
| `crates/marley_workbench/src/bookmarks.rs`, `blocks.rs` | Marley | chip and menu item |
| `docs/marley/zed-touchpoints.md` | docs | the two rows |

### Visual check plan
| REQ | Set-up and action | Shot |
|---|---|---|
| REQ-001 | `cat data.csv` (5 columns, 40 rows, a quoted comma) | `803-01-chip` |
| REQ-002 | the chip clicked; a header's sort clicked | `803-02-tab`, `803-03-sorted` |
| REQ-001, REQ-003 | `python3 -c` printing CSV; the block menu's Open Output as Table | `803-04-pipe` |
| REQ-004 | `ls -l`, `git log --oneline`, `ls --color=always` | `803-05-none` |
| REQ-005 | `head -n 5000 big.csv` (3 MB), its chip | `803-06-big` |
| REQ-001 | `cat data.jsonl`, `cat notes.md`, each chip | `803-07-json`, `803-08-markdown` |

### Risks and decisions
- False positives: prose with commas fails the 95 percent rule; `ls -l` and `git log` have no
  delimiter in common. A wrong chip costs a click; the text is never changed (D1).
- Reading the output and the styles takes the terminal's lock once per finished block, over at
  most 2,000 lines.

## Phase 2 — Code
- **Built:** as the design. The menu item lives in `block_tables::with_menu_item`, which
  `block_menu` calls in one line, so the menu builder stays under clippy's 100-line limit.
  `sample` reads a block's first 2,000 output lines by handing `block_output` a copy of the block
  whose end is cut there, so no new read was needed for the sample.
- **Review of the diff:** the chip's `found` takes the terminal's lock once (`block_output_kept`)
  for an output source; the terminal element calls the hook outside any lock (it reads through
  short `Terminal` calls), so this cannot deadlock as F-claude-798 did. Every open is deferred
  (`window.defer`, F-claude-733). A block read is never read twice (`next`, `restored_count`).
  Rows reach `TableRow::try_from_vec` already padded to the header's width, so no row panics.
  No Zed function body was copied: the reader is written fresh against RFC 4180.
- **Clippy (the three crates):** `too_long_first_doc_paragraph`, a `match` arm typed `bool`
  (`add_item_to_center`), `needless_pass_by_ref_mut` twice, `too_many_lines` on `block_menu`;
  each fixed at the source.
- **Gate (`803-gate-1.log`)**, on the tree after the final scenario: **GATE GREEN [diff]**.

## Phase 3 — Test
- **Scenario:** `script/e2e/803-a-blocks-output-opens-as-a-table.sh` (`compositor sway`).
- **Run 1 (measuring):** the chip showed on `cat data.csv`'s first row at (1538, 72); the click
  guess missed. After a `clear` blocks stack from the bottom, so a chip's y follows the output's
  line count (`chip_y`); a block whose first row is off screen draws its chip at its last row
  (1565, 891). The right-click guess hit empty space above the blocks.
- **Run 2:** every open worked; the menu's Open Output as Table measured at (778, 653); the
  scrollbar drag on the big preview did nothing.
- **Run 3:** the sort buttons (on hover at a header cell's right end) and the menu item clicked;
  the big file proven by sorting `id` Z-A instead of scrolling.
- **Run 4 (final), with a piped step added for Marley's own reader.** The shots:
  - `803-01-chip` (REQ-001): `cat data.csv`'s block with the Table chip right of the hover
    buttons and before the exit pill; tooltip "Open data.csv as a table".
  - `803-02-tab` (REQ-002): a tab "Preview data.csv" beside the terminal's tab, columns Name,
    Team, Commits, Lines, Notes, 40 rows; row 7's Notes "fixed the parser, again" in one cell.
  - `803-02b-parsed` (REQ-002): `cat data.csv | cat` (a pipe, so the text path): a tab
    "cat data.csv | cat", the same five columns, row 7's Notes in one cell without its quotes.
  - `803-03-sorted` (REQ-002): Commits sorted A-Z (the ↓ button filled): 12, 13, 17, 18, …
    in Zed's text order (6 after 59); row numbers follow their file lines.
  - `803-04a-menu`, `803-04-pipe` (REQ-001, REQ-003): the `python3 -c` block's chip, its menu's
    Block section with Open Output as Table after Send to Agent, and the tab it opened: city,
    temp, rain, four rows.
  - `803-05-none` (REQ-004): `ls -l`, `git log --oneline` (colored hashes) and
    `ls --color=always`: each block with its exit pill, no Table chip.
  - `803-06a-big-chip`, `803-06b-big-top`, `803-06-big` (REQ-005): `head -n 5000 big.csv`, its
    chip on the last row ("Open big.csv as a table"), a tab "Preview big.csv"; sorted by id Z-A
    its top rows are file lines 10,001 and 10,000, past the 5,000 the block printed: the whole
    file.
  - `803-07-json` (REQ-001): `cat data.jsonl`'s chip opened "Preview data.jsonl": service, state,
    p95_ms, four rows (Zed's preview shows JSON strings with their quotes).
  - `803-08-markdown` (REQ-001): `cat notes.md`'s chip opened "cat notes.md": Area, Change,
    Owner and the three rows, the prose around the table left out.
- **Pre-existing, not in scope:** Zed's table sorts numbers as text, and its JSONL preview quotes
  strings.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` ("A block's output opens as a table"); `docs/marley/guide.md`
  (The block terminal → Blocks); `docs/marley_architecture/terminal_blocks.md` ("Since #803");
  `docs/marley/zed-touchpoints.md` (the `alacritty.rs` and `terminal.rs` rows, written before the
  hunks).
- **Knowledge:** L-claude-803-a-blocks-chip-moves-with-its-output-001,
  L-claude-803-zeds-table-view-sorts-text-001. No bug was found in Code or Test.
- **Brain:** no decision recorded: the spec's decisions D1 to D3 stand, and the parser choice is
  recorded in the spec.
