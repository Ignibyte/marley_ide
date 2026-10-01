---
pipeline_id: 76d045a0-4bff-495c-b3f9-3d2c978202e3
ticket: docs/planning/tickets/open/TICKET-629-block-navigation-in-display-rows.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Block navigation in display rows"
type: feature
slice: prong 1 T5
references: [docs/marley/three-prong-plan.md]
---

## Title
Stage two, second slice: block navigation, the pinned header, bookmark ticks, search highlights
and the scrollbar work in display rows while the headers are on (plan T5). After #628.

## Scope
### In
- `block_scroll`, `scroll_to_block` and `reveal` (#473, #554) put a block's header row on the top
  row; #529's pinned header becomes the header row pinned while the block is scrolled back;
  #559's bookmark ticks and the scrollback fraction, Zed's search highlights, and the scrollbar
  read through #628's map.

### Out (explicitly deferred)
- Density and gaps (#630).

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
  ticks (`bookmarks.rs`); #628's map.

## UI proof
`script/e2e/629-block-navigation-in-display-rows.sh`: the headers on; three blocks with long
output.
- `top.png`: after Alt+Up to the middle block, its header on the top row;
- `pinned.png`: scrolled into its output, the pinned header matching that header;
- `ticks.png`: a bookmark on the first block, its tick at the right height.

## Locked-In Decisions
- D1 — One map, #628's, for every grid-to-screen conversion.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user moves to a block, its header row shall land on the top row. | Shot `top.png` |
| REQ-002 | WHILE a block is scrolled back, its pinned header shall match its header row. | Shot `pinned.png` |
| REQ-003 | WHILE the headers are on, a bookmark's tick shall sit at its block's height. | Shot `ticks.png` |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams, the design.
- **P2 Code** — the conversions through the map; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
