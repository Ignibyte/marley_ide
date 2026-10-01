---
pipeline_id: 76d045a0-4bff-495c-b3f9-3d2c978202e3
ticket: docs/planning/tickets/open/TICKET-629-block-navigation-in-display-rows.md
status: Phase 4 — Complete PASS
title: "Block navigation in display rows"
type: feature
slice: prong 1 T5
references: [docs/marley/three-prong-plan.md]
---

## Title
Stage two, second slice: with #628's headers on, moving between blocks lands on a header, the
pinned header reads as the header it stands for, the bookmark ticks keep their place, and a
search match inside a hidden prompt is not drawn over the header (plan T5). After #628.

## Scope
### In
- The pinned header (#529) shows the command as the native header does, with no `$ `, while
  `marley.block_headers` is on.
- `terminal_element.rs` (Zed crate, in #628's hunk): the search matches (not the selection) whose
  first row is a hidden prompt row are left out of the highlighted ranges.
- The block keys, `reveal` and the bookmark ticks are checked, unchanged: #628 moved no row.

### Out (explicitly deferred)
- Density, gaps and a second header line (#630).
- A selection that crosses a hidden prompt (the copy still holds the prompt's text).

## Reference (§20)
- **Warp (behavior):** moving between blocks brings a block's header to the top; the header stays
  pinned while its output scrolls (`docs/warp_architecture/subsystems/03-terminal-session-core.md`
  §6).
- **Upstream Zed:** the terminal's search and scrollbar, kept, read through the map.

### Prior art
- **Behavior maps:** the note above; #529's deferred "nested headers in stage two".
- **Published material:** none needed.
- **Code we already ship:** `block_scroll`, `sticky_block` (`anchored.rs`); `blocks::step`,
  `reveal`, `scroll_to_block` (`marley_workbench/src/blocks.rs`); `sticky_header.rs`; the bookmark
  ticks (`bookmarks.rs`); #628's headers.
- **Re-verified at promotion (2026-10-01), against #628 as shipped:** #628's header takes exactly
  its prompt's rows, so there is no display-row map: a block's first line, where the block keys
  and `reveal` put the top (`block_scroll`), is its header's row; the bookmark ticks place a block
  by `scrollback_fraction` of absolute lines, and the scrollbar counts grid lines, both unchanged
  by rows that keep their place. Two things still read the shell's prompt: the pinned header
  (#529) prefixes the command with `$ `, unlike the native header, and Zed's search highlights a
  match inside a hidden prompt (`relative_highlighted_ranges`, search matches before the
  selection), where the header now shows the command at another column.

## UI proof
`script/e2e/629-block-navigation-in-display-rows.sh`: bash with a two-line PS1, the headers on;
three blocks of 40 lines each.
- `top.png`: Ctrl+Up to the middle block: its header on the top row;
- `pinned.png`: scrolled into its output: the pinned header reads the command with no `$ `;
- `ticks.png`: a bookmark on the first block: its tick at the right edge, where the block sits in
  the scrollback;
- `search.png`: a search for `seq`: the matches in the output rows, none drawn over a header.

## Locked-In Decisions
- D1 — #628's rows are the grid's, so nothing needs a map; this slice changes only what still
  shows the shell's prompt.
- D2 — Only search matches are dropped in a hidden prompt; a selection is the user's own and is
  drawn wherever it goes.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user moves to a block, its header row shall land on the top row. | Shot `top.png` |
| REQ-002 | WHILE a block is scrolled back, its pinned header shall show the command as its header does. | Shot `pinned.png` |
| REQ-003 | WHILE the headers are on, a bookmark's tick shall sit at its block's height. | Shot `ticks.png` |
| REQ-004 | WHILE the headers are on, a search match inside a hidden prompt shall not be drawn. | Shot `search.png` |

## Phase Plan
- **P1 Plan** — promote, re-verify against #628, the design.
- **P2 Code** — the pinned header's text, the search filter and its row; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
