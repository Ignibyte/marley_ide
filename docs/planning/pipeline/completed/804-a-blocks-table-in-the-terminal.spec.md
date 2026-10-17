---
pipeline_id: 77ed2060-4dd6-45d8-9b83-75db4e460f95
ticket: docs/planning/tickets/closed/TICKET-804-a-blocks-table-in-the-terminal.md
status: Phase 4 — Complete PASS
title: A block's table drawn in the terminal
type: feature
slice: T (the block terminal)
references:
  - docs/planning/design-notes/terminal-output-as-components-2026-10-17.md
  - docs/planning/pipeline/completed/803-a-blocks-output-opens-as-a-table.spec.md
  - docs/planning/pipeline/completed/628-blocks-with-native-headers-and-ps1-hidden.spec.md
---

## Title
A block whose output #803 detected as a table can show the table in the terminal, over its own
output rows, toggled from its header; `marley.block_tables` decides whether that happens by
itself.

## Scope
### In
- **Drawing over the rows:** a `MarleyBlockBody` hook in `terminal_view` (beside
  `MarleyBlockChip`) that a host fills with an element for a block's output rows, and in
  `terminal_element.rs` the same treatment the headers get (#628): the rows' cells left out of the
  text, rects and search marks, and the element laid over the rows' extent in prepaint, painted
  with the block elements. The element is exactly as tall as the rows and draws the part of the
  table on screen, so scrolling moves it with the text, row by row.
- **The table:** a grid in the terminal's own font, one table row per terminal row, a header row
  first, columns sized to their widest cell and shrunk to the terminal's width, cells cut with an
  ellipsis, themed from the editor's colors; a row holding a search match is marked. (At
  promotion: #799's `table` component pads its rows taller than a terminal row, so it cannot keep
  a table row on its text row.)
- **Which blocks:** delimited text (#803's comma, tab, semicolon or pipe) whose every output row
  is one record: the header and the rows fill the output rows exactly, no line wrapped, within
  #803's 2,000-line sample. A wrapped line, a quoted line break or a blank line keeps the block
  to #803's tab.
- **The toggle:** an Inline / Text button in the block's header beside #803's Table chip, which
  keeps opening the tab; the block menu keeps Open Output as Table.
- **The setting** `marley.block_tables`: `offer` (the default: #803's chip, nothing in place) or
  `in_place` (a detected table opens in place when its block finishes).
- **The text stays the source:** Copy Output, a selection, Find and links read the rows' text;
  a selection drag over a table shows the text while it is held.
- No table in place on the alternate screen, under inline assist's block below the cursor, or
  while the row map is off (the cases `terminal_element.rs` turns it off).

### Out (explicitly deferred)
- Sorting or filtering in place (#803's tab does both).
- Other components in place (#805 shows them in a tab).

## Reference (§20)
Warp (its published docs and issue tracker only, §20): a block's output can be filtered without
deleting lines (<https://docs.warp.dev/terminal/blocks/block-filtering>), and tables with a raw
toggle are an open request (<https://github.com/warpdotdev/Warp/issues/9138>). Marley's toggle
keeps the block's text and draws a table over it. Marley's own #628 (block headers drawn over
their rows) is the mechanism.

### Prior art
- The code we ship: #628's header rows in `terminal_element.rs` (cells filtered from rects, text
  and search; the element prepainted over the rows); `RowMap` in
  `marley_terminal/src/display_rows.rs`; Zed's inline assist block (`BlockProperties` in
  `terminal_view.rs`), which draws an element N lines tall under the cursor and turns the row map
  off; `AnchoredBlock`'s output lines.
- Behavior maps: `docs/warp_architecture/subsystems/03-terminal-session-core.md` §6.
- Published material: DomTerm's inline HTML (`OSC 72`) and iTerm2's `File=` native views, in the
  design note.

## UI proof
Scenario `script/e2e/804-a-blocks-table-in-the-terminal.sh`, under `compositor sway`, with
#803's fixtures:
- `804-01-offer`: `marley.block_tables` at `offer`: `cat data.csv` shows text and the chip
  (REQ-004).
- `804-02-toggle`: the toggle clicked: the table drawn in the block's rows; the prompt under it
  where it was (REQ-001).
- `804-03-scrolled`: scrolled up by 5 lines: the table moved with the rows (REQ-002).
- `804-04-copy`: Copy Output, then the clipboard pasted into a scratch editor: the CSV text
  (REQ-003).
- `804-05-text`: the toggle clicked again: the text back (REQ-001).
- `804-06-in-place`: `in_place` set, `cat data.csv` run: the table shows by itself (REQ-004).
- `804-07-find`: Find for a cell's value: the match marked in the table's row (REQ-003).

## Locked-In Decisions
- **D1 — Over the rows, not in a gap.** The grid's rows stay where they are, so nothing the
  terminal knows about lines changes.
- **D2 — `offer` by default.** Content alone never changes what a block shows unless the user
  says so.
- **D3 — The Zed hunks follow #628's**, beside them in `terminal_element.rs` and `terminal_view.rs`,
  each with its touchpoints row.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a detected block's toggle is clicked, the terminal shall draw the table over the block's output rows, and WHEN clicked again, its text | shots `804-02-toggle`, `804-05-text` |
| REQ-002 | WHILE a table is drawn in place, it shall scroll with the block's rows | shot `804-03-scrolled` |
| REQ-003 | WHILE a table is drawn in place, copy and find shall read the block's text | shots `804-04-copy`, `804-07-find` |
| REQ-004 | WHERE `marley.block_tables` is `in_place`, a detected table shall open in place when its block finishes; WHERE it is `offer`, the block shall show its text and the chip | shots `804-01-offer`, `804-06-in-place` |

## Phase Plan
- **P1 Plan:** this spec; at promotion, the element is laid out in prepaint after the headers,
  over the same row map, and painted first among the block elements, so a block's bar, pill and
  chip draw over it.
- **P2 Code:** the hook, the element hunks, the toggle, the setting; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test:** the visual check above.
- **P4 Complete:** CHANGELOG, guide, ledger, close, archive, commit.
