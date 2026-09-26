---
pipeline_id: 1420b9ec-5625-4142-9206-e86b7d0668ed
ticket: docs/planning/tickets/open/TICKET-544-blocks-survive-a-rewrap.md
status: Phase 4 — Complete PASS
title: "Blocks keep their rows when a resize rewraps the terminal"
type: bug
slice: prong 1 T0 (the block terminal's anchors, the plan's D2)
references: [docs/marley/three-prong-plan.md, docs/planning/pipeline/completed/516-secret-redaction-for-agents.notes.md]
---

## Title
A block is a range of absolute lines in the one scrollback. When a resize changes the width,
alacritty rewraps every line; the anchors must move with the rows they mark, so the margin bar,
the pill and `terminal_read` follow the block at the new width and back.

## Scope
### In
- Every anchor the blocks keep (each block's prompt line, output start and output end, the
  staged prompt's line, the prompt input's start with its column) is carried through a resize
  that changes the width as a place alacritty's rewrap keeps: logical lines from the cursor's
  logical line, and the character offset inside its own (`marley_terminal::anchored`, pure).
- The Resize arm of Zed's `Terminal` (`crates/terminal/src/terminal.rs`) reads the grid's rows
  before and after `resize` and has the blocks rewrap (a Marley hunk), through a Marley function in
  `crates/terminal/src/alacritty.rs` that builds the rows' view.
- The vendored alacritty: `Term::main_grid()` (the main screen's grid, also while a full-screen
  program shows, since the resize rewraps it too), and `shrink_columns` counting the rows it drops
  off a full history as evicted.

### Out (explicitly deferred)
- Reading blocks while a full-screen program shows (`absolute_lines_text` reads the active
  grid): #546, which uses `main_grid` from this slice.
- A shell hook positioned before a resize and applied after it, and the shell's own prompt redraw
  on SIGWINCH: both can leave the newest prompt's anchors off until the next prompt.

## Reference (§20)
N/A — Marley-specific behavior on upstream alacritty's reflow (`vendor/alacritty_terminal`,
`Grid::resize`), which Zed's terminal already runs on every resize; Warp keeps its blocks as
separate grids and has no rewrap to follow (docs/warp_architecture/, blocks).

### Prior art
- **The code we already ship.** alacritty's reflow (`vendor/alacritty_terminal/src/grid/resize.rs`)
  rewraps along `WRAPLINE` and keeps the cursor in its logical line; it returns no mapping and
  drops its own selection on a width change rather than mapping it. Zed's terminal already walks
  logical lines (`find_logical_line_start`, `logical_line_for_row` in
  `crates/terminal/src/alacritty.rs`). The grid's evicted counter (#464) keeps absolute lines
  stable across scrolling, not across a rewrap.
- **The plan's mitigation, weighed:** tagging each prompt with an OSC 8 hyperlink re-finds only the
  prompt row, needs a non-blank cell, and Zed underlines and opens hyperlinks; the logical-line
  place covers all five anchors without touching the shell.
- **Published material:** none needed; the behavior is alacritty's.

## UI proof
UI-AFFECTING. `script/e2e/544-blocks-survive-a-rewrap.sh` (`compositor sway`): three blocks, one
of long lines; the Settings window opens tiled beside the main window, narrowing the terminal
so its lines rewrap; shots before and after, and `terminal_read` of each block before, narrowed,
and widened again.

## Locked-In Decisions
- D1 — The anchors follow logical lines: a logical line is a row and the rows it wraps into.
- D2 — Pivot on the cursor's logical line, which alacritty keeps, not the grid's top, which a full
  history cuts.
- D3 — The logic is pure and lives in `marley_terminal`; the Zed hunks only read the grid and call
  it.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a resize rewraps a terminal's lines, each finished block's output read through `terminal_read` shall be the text it was before the resize. | The run log (reads before, narrowed and widened again) |
| REQ-002 | WHEN a resize rewraps a terminal's lines, each block's margin bar and pill shall sit on the block's rows. | Shots before and after |

## Phase Plan
- **P1 Plan** — this spec; the design in the notes.
- **P2 Code** — the anchor remap around the resize.
- **P3 Test** — the scenario (red on the unfixed build first); the gate.
- **P4 Complete** — docs, knowledge, close, archive, commit.
