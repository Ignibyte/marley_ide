---
pipeline_id: 619db794-cdae-44bc-bc85-0e5b41e84d31
ticket: docs/planning/tickets/open/TICKET-628-blocks-with-native-headers-and-ps1-hidden.md
status: Phase 4 — Complete PASS
title: "Blocks with native headers, PS1 hidden"
type: feature
slice: prong 1 T5
references: [docs/marley/three-prong-plan.md]
---

## Title
Stage two, first slice: a block's prompt rows (`prompt_line..output_start`) are drawn as a native
header, the command in Marley's own type with #470's pill, in place of the shell's PS1; the live
prompt stays the shell's (plan T5). A header takes exactly the rows its prompt took, so no row
moves. A setting, off by default until #630.

## Scope
### In
- `crates/marley_terminal`: `prompt_rows(blocks, top, screen_lines)`, pure: for each running or
  finished block whose command the nonce verified, the viewport rows of its prompt
  (`prompt_line..output_start`) that are on screen.
- `crates/terminal_view/src/terminal_view.rs` (Zed crate, additive): a hook global,
  `MarleyBlockHeader`, the header element for a block's index, or none (the setting off).
- `crates/terminal_view/src/terminal_element.rs` (Zed crate, additive `// Marley:` hunks): for each
  such block whose header the hook gives, the cells of its prompt rows (text runs, backgrounds,
  block glyphs) are left out, and the header is laid out over those rows before the block's pill
  and actions, which draw over it.
- `crates/marley_workbench`: the hook: the command's first line in the buffer font (`…` when it
  has more), over the first prompt row; a press on it starts no selection.
- A setting `marley.block_headers` (off by default), with its toggle on the Marley page.

### Out (explicitly deferred)
- Navigation, the pinned header, bookmark ticks, search and the scrollbar in display rows (#629).
- Density, gaps and a two-line header with the folder and branch (#630).
- Remote and alternate-screen blocks (drawn as today), and a block whose command the nonce did not
  verify, so no forged frame can hide rows.
- A selection or a search match crossing a hidden prompt still covers its hidden text (#629).

## Reference (§20)
- **Warp (behavior):** a block's header is its own grid apart from its output; the prompt can be
  hidden (`docs/warp_architecture/subsystems/03-terminal-session-core.md` §6, 231-262, `header_grid`,
  `honor_ps1` at 64 and 104).
- **Upstream Zed:** `terminal_element.rs`'s painting and Zed's own rows outside the grid
  (`block_below_cursor` with `scroll_top`, the inline assist's precedent).

### Prior art
- **Behavior maps:** the note above; #470's and #529's deferred lists (stage two).
- **Published material:** none needed.
- **Re-verified at promotion (2026-10-01)**, an Explore map of `terminal_element.rs`: thirteen
  sites turn a viewport row into a y (text runs, backgrounds, block glyphs, the clipped layout's
  culling, the cursor and IME, `block_below_cursor`, selection and search ranges, the wash and
  outlines, the gutter, the block elements, the last-row chip, the pinned header, the bottom
  shift) and four turn a pixel back into a row (`grid_point_and_side`, `content_index_for_mouse`,
  `marley_block_at`, vi motion). A header that takes exactly its prompt's rows leaves all of them
  as they are; only the cells of those rows go, after `marley_layout_grid`, whose runs and rects
  carry the viewport row. The pinned header (#529, `MarleyStickyHeader`) is the precedent for a
  hook that gives the element a row's element and stops the press.
- **Code we already ship:** `visible_spans`, `block_lines`, `AnchoredBlock`'s `prompt_line` and
  `output_start` (`anchored.rs`); `marley_block_spans`, `marley_rows_bounds` and the pill painting
  (`terminal_element.rs`); `grid_point_and_side` (`terminal/src/mappings/mouse.rs`); #476's
  `bottom_shift`. The hooks send DCS frames only, no OSC 133, so nothing marks where PS1 ends:
  the whole `prompt_line..output_start` range is hidden.

## UI proof
`script/e2e/628-blocks-with-native-headers-and-ps1-hidden.sh`: bash with a two-line PS1, the
setting on.
- `headers.png`: after `echo hi` and `ls`, each block's header (`echo hi` with its pill) on its
  first prompt row and the second left empty, no PS1 text, the live two-line prompt at the bottom;
- `selected.png`: a drag over `hi`, the selection on `hi` alone;
- `pasted.png`: the selection, copied as it was made (`terminal.copy_on_select`), pasted into the
  prompt editor, reads `hi`;
- `off.png`: the setting off, the blocks as today.

## Locked-In Decisions
- D1 — A header takes exactly its prompt's rows: no row moves, so scrolling, the cursor, the
  selection and the mouse keep their arithmetic. A prompt of two rows leaves its second row empty
  until #630 puts the folder and branch there.
- D2 — The whole prompt range hides, since the hooks do not mark PS1's end.
- D3 — Only a block whose command the nonce verified hides its prompt, so output that prints a
  frame cannot hide rows (PR-claude-474).
- D4 — The header is a hook's element, as the pinned header is: the workbench draws it and reads
  the setting; the element only asks and lays it out.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the setting is on, each running or finished block shall show a header with its command and pill in place of its prompt rows. | Shot `headers.png` |
| REQ-002 | WHILE the shell is at a prompt, its live prompt shall be drawn as the shell drew it. | Shot `headers.png` |
| REQ-003 | WHEN the user drags over a block's output, the selection and the copy shall hold that output only. | Shots `selected.png`, `pasted.png` |
| REQ-004 | WHILE the setting is off, blocks shall draw as before. | Shot `off.png` |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams, the design.
- **P2 Code** — the map, the painting and mouse hunks and their rows, the setting; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
