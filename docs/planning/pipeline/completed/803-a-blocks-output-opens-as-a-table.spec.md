---
pipeline_id: b90b84bc-b986-4740-8e9f-5f8dff7888ce
ticket: docs/planning/tickets/closed/TICKET-803-a-blocks-output-opens-as-a-table.md
status: Phase 4 — Complete PASS
title: A block's output opens as a table
type: feature
slice: T (the block terminal)
references:
  - docs/planning/design-notes/terminal-output-as-components-2026-10-17.md
  - docs/warp_architecture/subsystems/03-terminal-session-core.md
---

## Title
A finished block whose output reads as a table gets a Table chip, and the chip or the block
menu's Open Output as Table opens Zed's table view in a tab beside the terminal.

## Scope
### In
- **Detection** (`crates/marley_terminal`, a pure function over a block's text and command, run on
  a background task when a block finishes, on at most the first 1 MB and 2,000 lines):
  - CSV and TSV: a quote-aware parse (RFC 4180's quoting, written in `marley_terminal`) choosing
    `,`, tab, `;` or `|` as the delimiter, with at least 2 columns and 3 rows and the same field count in at least
    95 percent of rows;
  - JSON lines of objects, or one JSON array of objects;
  - a markdown table: a header row and a delimiter row with the same cell count.
  - Never for a block whose output cells carry colors or text styles (what escapes leave on the
    grid; a cursor move leaves no mark there), or that ran on the alternate screen; never for a
    block still running.
- **The chip:** `Table` with the column and row counts in its tooltip, through
  `MarleyBlockChip` (`marley_workbench/src/bookmarks.rs`), after Ask the agent.
- **The menu item:** Open Output as Table in the block menu (`marley_workbench/src/blocks.rs`),
  shown when detection found a table.
- **The tab:** Zed's `TableView` (`crates/tabular_data_preview`) filled with the parsed cells
  (`TableCell::Generated`), titled with the block's command, beside the terminal's tab. When
  the command is a `cat`, `head`, `tail` or `bat` of one `.csv`, `.tsv` or `.jsonl` file in the
  block's folder, the tab is `TabularDataPreviewPane`'s preview of that file instead, so a file
  larger than what the block kept opens whole.
- When the block's first line has left the scrollback (`block_output_kept` false) and no file
  is named, no chip shows.

### Out (explicitly deferred)
- Drawing the table inside the terminal: #804.
- A program asking for a table: #805.
- Charts from numeric columns; editing the table.

## Reference (§20)
Warp, from its published docs only (§20): its blocks
(`docs/warp_architecture/subsystems/03-terminal-session-core.md` §6) carry actions, and running
`cat`, `glow` or `less` on a local markdown file offers a banner that opens a rendered viewer
while the block stays text (<https://docs.warp.dev/terminal/more-features/markdown-viewer>).
Marley offers a chip that opens a table, leaving the block's text as it is. Upstream Zed:
`tabular_data_preview`'s table view and file preview, used as they are.

### Prior art
- The code we ship: `tabular_data_preview` (CSV, TSV, PSV, SSV and JSONL parsing by extension, a
  sortable and filterable `TableView` that draws only the rows on screen); `Terminal::block_output`
  and `block_output_kept`; `MarleyBlockChip` and the block menu. At promotion: no `csv` crate in
  `Cargo.lock`, and `tabular_data_preview`'s parser is a GPL Zed function body that reads a
  `BufferSnapshot`, so §20 keeps it out of a Marley crate; the reader is Marley's own.
- Behavior maps: `docs/warp_architecture/subsystems/03-terminal-session-core.md` §6 (blocks).
- Published material: Wave's `wsh view` (CSV by MIME type, files to 1 MB), Python's
  `csv.Sniffer`, VisiData, qsv's `sniff`, RFC 4180: in the design note.

## UI proof
Scenario `script/e2e/803-a-blocks-output-opens-as-a-table.sh`, under `compositor sway` (it
clicks the chip), with a scratch repository holding `data.csv` (5 columns, 40 rows, a quoted
field with a comma), `data.jsonl`, `notes.md` with a table, and a 3 MB `big.csv`:
- `803-01-chip`: `cat data.csv`: the block shows the Table chip (REQ-001).
- `803-02-tab`: the chip clicked: a tab with the table, 5 columns, the quoted field in one cell
  (REQ-002).
- `803-03-sorted`: a column header clicked in the tab: sorted (REQ-002).
- `803-04-pipe`: `python3 -c` printing CSV to stdout: the chip, and the menu item opens it
  (REQ-001, REQ-003).
- `803-05-none`: `ls -l`, `git log --oneline` and `ls --color=always`: no chip (REQ-004).
- `803-06-big`: `head -n 5000 big.csv`: the chip opens the whole file's preview (REQ-005).
- `803-07-json` and `803-08-markdown`: `cat data.jsonl` and `cat notes.md`: each shows the chip
  and opens its table (REQ-001).

## Locked-In Decisions
- **D1 — Offer, never swap.** The block's text is what the program printed; the table is one
  click away.
- **D2 — Zed's table view**, not #799's `table` component: it sorts, filters and draws only what
  is on screen, which a large output needs.
- **D3 — The file over the text** when the command names one tabular file.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a block finishes on the main screen with output that parses as CSV, TSV, JSON lines, a JSON array of objects or a markdown table, its header shall show a Table chip | shots `803-01-chip`, `803-04-pipe`, `803-07-json`, `803-08-markdown` |
| REQ-002 | WHEN the chip is clicked, Marley shall open the parsed table in a tab beside the terminal, sortable by column | shots `803-02-tab`, `803-03-sorted` |
| REQ-003 | WHEN a block shows the chip, its menu shall offer Open Output as Table, which opens the same tab | shot `803-04-pipe` |
| REQ-004 | IF a block's output does not parse as a table, or holds escapes, THEN no chip shall show | shot `803-05-none` |
| REQ-005 | WHEN the command reads one tabular file, the tab shall show the whole file | shot `803-06-big` |

## Phase Plan
- **P1 Plan:** this spec; at promotion, the parser is Marley's own (above), detection runs on
  `smol::unblock` from the terminal's observer, as #620 reads failures, and its summary is
  cached per block.
- **P2 Code:** detection, the chip, the menu item, the tab; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test:** the visual check above.
- **P4 Complete:** CHANGELOG, `docs/marley/guide.md` (blocks), ledger, close, archive, commit.
