---
pipeline_id: dcd7d4d6-0180-4478-82f4-91b7eb9608f7
ticket: docs/planning/tickets/open/TICKET-630-block-density-and-two-line-headers.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Block density: two-line headers and gaps"
type: feature
slice: prong 1 T5
references: [docs/marley/three-prong-plan.md]
---

## Title
Stage two, third slice: two-line headers (folder and branch over the command and pill), gaps
between blocks, and `marley.block_density` (`compact`, `comfortable`); the headers on by default
(plan T5). After #629.

## Scope
### In
- #628's map takes rows that are no grid line (a header's second line, a gap), and the view
  scrolls past the grid by pixels as `block_below_cursor`'s `scroll_top` does; the mouse goes
  through the map.
- The header's first line: the block's folder (home as `~`) and the git branch from `PromptInfo`;
  the second: the command and the pill.
- `marley.block_density`: `compact` (no gap, 1.3 line height in headers) and `comfortable` (a gap
  of half a row); `marley.block_headers` turns on by default.

### Out (explicitly deferred)
- A header's duration (blocks record no time yet, #470's Out).
- Collapsing blocks.

## Reference (§20)
- **Warp (behavior):** a block's header carries its folder and branch above the command, and blocks
  stand apart (`docs/warp_architecture/subsystems/03-terminal-session-core.md` §6). The tree has no
  note that defines Warp's density; the sizes here are Marley's own, set in this spec.
- **Upstream Zed:** `terminal.line_height` (`comfortable` 1.618, `standard` 1.3); `scroll_top`.

### Prior art
- **Behavior maps:** the note above; the gpui-era #216 density notes (chrome and fonts, not
  blocks).
- **Published material:** none needed.
- **Code we already ship:** `block_below_cursor` and `scroll_top` (`terminal_view.rs`); #476's
  `bottom_shift`; `PromptInfo` (pwd, branch).

## UI proof
`script/e2e/630-block-density-and-two-line-headers.sh`: in a git repository, three blocks.
- `compact.png`: compact, the headers two lines with `~/…` and the branch, no gaps, the prompt on
  the last row;
- `comfortable.png`: comfortable, a half-row gap between blocks;
- `scrolled.png`: scrolled to the top and back, the rows in place.

## Locked-In Decisions
- D1 — Compact and comfortable only; the sizes are fixed in this spec, not user numbers.
- D2 — The headers on by default once this slice ships.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the headers are on, a block's header shall show its folder and branch over its command and pill. | Shot `compact.png` |
| REQ-002 | WHILE comfortable density is on, blocks shall stand half a row apart. | Shot `comfortable.png` |
| REQ-003 | WHILE either density is on, the live prompt shall sit on the last row. | Shots `compact.png`, `comfortable.png` |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams, the design.
- **P2 Code** — the rows past the grid, the header's two lines, the setting; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
