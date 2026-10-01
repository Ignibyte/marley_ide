---
pipeline_id: 619db794-cdae-44bc-bc85-0e5b41e84d31
ticket: docs/planning/tickets/open/TICKET-628-blocks-with-native-headers-and-ps1-hidden.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Blocks with native headers, PS1 hidden"
type: feature
slice: prong 1 T5
references: [docs/marley/three-prong-plan.md]
---

## Title
Stage two, first slice: a display-row map lets a block's prompt rows (`prompt_line..output_start`)
be drawn as one native header row with the command and the pill; the live prompt stays the shell's
(plan T5). A setting, off by default until #630.

## Scope
### In
- `crates/marley_terminal`: a pure display-row map from the visible grid lines and the blocks:
  each display row is a grid line or a block's header; a header replaces its block's prompt rows
  one for one or fewer, so `display_offset` keeps counting grid lines.
- `crates/terminal_view/src/terminal_element.rs` (Zed crate, additive `// Marley:` hunks): the
  cells, the wash and the gutter are laid out through the map; a header row draws the command and
  #470's pill; `grid_point` maps the mouse through it, and a header row starts no selection.
- A setting `marley.block_headers` (off by default).

### Out (explicitly deferred)
- Navigation, the pinned header, bookmark ticks, search and the scrollbar in display rows (#629).
- Density, gaps and a two-line header with the folder and branch (#630).
- Remote and alternate-screen blocks (drawn as today).

## Reference (§20)
- **Warp (behavior):** a block's header is its own grid apart from its output; the prompt can be
  hidden (`docs/warp_architecture/subsystems/03-terminal-session-core.md` §6, 231-262, `header_grid`,
  `honor_ps1` at 64 and 104).
- **Upstream Zed:** `terminal_element.rs`'s painting and Zed's own rows outside the grid
  (`block_below_cursor` with `scroll_top`, the inline assist's precedent).

### Prior art
- **Behavior maps:** the note above; #470's and #529's deferred lists (stage two).
- **Published material:** none needed.
- **Code we already ship:** `visible_spans`, `block_lines`, `AnchoredBlock`'s `prompt_line` and
  `output_start` (`anchored.rs`); `marley_block_spans`, `marley_rows_bounds` and the pill painting
  (`terminal_element.rs`); `grid_point_and_side` (`terminal/src/mappings/mouse.rs`); #476's
  `bottom_shift`. The hooks send DCS frames only, no OSC 133, so nothing marks where PS1 ends:
  the whole `prompt_line..output_start` range is hidden.

## UI proof
`script/e2e/628-blocks-with-native-headers-and-ps1-hidden.sh`: bash with a two-line PS1, the
setting on.
- `headers.png`: after `echo hi` and `ls`, each block one header row (`echo hi` with its pill) over
  its output, no PS1 text above them, the live two-line prompt at the bottom;
- `selected.png`: a drag over `hi`, the selection on `hi` alone, and the copy pasted at the prompt
  reads `hi`;
- `off.png`: the setting off, the blocks as today.

## Locked-In Decisions
- D1 — A header never adds rows in this slice, so scrolling keeps its grid-line arithmetic.
- D2 — The whole prompt range hides, since the hooks do not mark PS1's end.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the setting is on, each finished block shall show one header row with its command and pill in place of its prompt rows. | Shot `headers.png` |
| REQ-002 | WHILE the shell is at a prompt, its live prompt shall be drawn as the shell drew it. | Shot `headers.png` |
| REQ-003 | WHEN the user drags over a block's output, the selection and the copy shall hold that output only. | Shot `selected.png` |
| REQ-004 | WHILE the setting is off, blocks shall draw as before. | Shot `off.png` |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams, the design.
- **P2 Code** — the map, the painting and mouse hunks and their rows, the setting; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
