---
pipeline_id: 241a0bc0-d7e3-4b30-b85e-a4103627537a
ticket: docs/planning/tickets/closed/TICKET-470-blocks-drawn-in-the-terminal.md
status: Phase 4 — Complete PASS
title: Blocks drawn in the terminal, stage one
type: feature
slice: prong 1 T1a
references: [docs/marley/three-prong-plan.md, docs/marley_architecture/terminal_blocks.md]
---

## Title
Zed's terminal keeps each command as a block, and bash and zsh report them, but nothing draws
them yet. Stage one (the plan's D3) draws decorations over the terminal's own rows, with no
change to its row model: a gutter bar beside each block, a status pill at a block's top right,
and a wash over a running or failed block.

## Scope
### In
- **Where each block is on screen** (`marley_terminal`, pure). `visible_spans(blocks, top,
  screen_lines, cursor_line)`: for each block with a line among the `screen_lines` rows from the
  absolute line `top`, its index, the viewport rows it covers, whether its first line is among
  them, its state and its exit code. A block starts at its prompt's line, or its output's when
  no prompt was seen, and ends before its `output_end`, or after the cursor's line while it
  runs.
- **The viewport's top as an absolute line** (Zed's `terminal`). `Content` gains
  `marley_screen_top`, the absolute line of the screen's top row with nothing scrolled:
  evicted lines plus history. The viewport's top is that less `display_offset`.
- **The drawing** (Zed's `terminal_view::TerminalElement`). `prepaint` computes the spans after
  `sync`, none on the alternate screen, and lays out each pill; `paint` draws the washes after
  the cells' backgrounds, and the gutter bars and pills after the text.
  - Gutter bar: in the one-cell gutter left of column 0, over the block's rows; running `info`,
    exit 0 `success`, another exit code `error`, finished without one the muted border.
  - Pill: at the right end of the block's first row when that row is on screen: a check for
    exit 0, the exit code for another, `running` while it runs, none without an exit code.
  - Wash: the block's rows tinted, `info` while running and `error` after a failure, at a low
    alpha.

### Out (explicitly deferred)
- Hover copy and rerun (T1b), block navigation keys (T1c).
- A pill that stays at the top while its block's first row is scrolled away.
- Elapsed time in the pill: blocks record no time yet.
- Stage two (T5): header rows, the prompt hidden.

## Reference (§20)
- **Warp:** blocks as the unit of output, each with its status (`docs/warp_architecture/
  subsystems/03-terminal-session-core.md` §6, from observed behavior); Warp draws them as
  separate grids with headers, which is stage two here. Stage one takes the observed
  essentials, a block's extent, its status and a failed block standing out, and draws them over
  Zed's grid. No Warp code.
- **Upstream Zed:** `TerminalElement`'s layout: the one-cell gutter (`gutter = cell_width`) left
  of column 0, the paint order (background, cell rects, highlights, text, cursor, overlay
  elements) and `prepaint_as_root` for elements over the grid, as the hyperlink tooltip is.

### Prior art
- **Behavior maps:** Warp's subsystem 03 above; the gpui era's block rendering in
  `docs/marley_architecture/terminal_blocks.md` drew its own grid, which the plan's D1 set
  aside.
- **Published material:** none needed.
- **Code we already ship.**
  - `TerminalElement::prepaint` (`crates/terminal_view/src/terminal_element.rs:1189`) sizes the
    grid with a one-cell gutter and snaps its origin to device pixels; `paint` (`:1603`) paints
    `layout.rects` before the text and the hyperlink tooltip's `AnyElement` last.
  - `Terminal::blocks()` and the absolute lines of #464; `HookPosition`'s `absolute_line =
    evicted + history + cursor line` (`vendor/alacritty_terminal/src/marley_hooks.rs`), the same
    frame of reference `marley_screen_top` gives.
  - `theme::StatusColors`: `success`, `error`, `info` and their backgrounds.

## UI proof
UI-AFFECTING: the terminal's drawing.
- **Driven tests** (`terminal_view`): a real PTY whose script prints the hook frames for a
  command that succeeds and one that fails, rendered in a `TerminalView`: both pills draw on
  their blocks' first rows, at the right; after the terminal enters the alternate screen, none
  draw. Unit tests for `visible_spans` and for the rows-to-bounds geometry.
- **Live drive:** the debug `marley` on hidden workspace 9, shot by toplevel with no input. With
  no input, the blocks come from frames a scratch `HOME`'s startup file prints around real
  commands before Marley's hooks load; the capture shows the gutter bars, the pills and the
  failed block's wash. Typed commands wait for a drive with Chad away from the desk.

## Locked-In Decisions
- D1 — The spans are computed by a pure function in `marley_terminal`, and Zed's element only
  maps rows to pixels and paints.
- D2 — `Content` carries the screen's top as an absolute line, so the element needs nothing but
  the content it already reads and `blocks()`.
- D3 — Nothing is drawn on the alternate screen: its rows are not the scrollback the blocks
  anchor to.
- D4 — Colors come from the theme's status colors; the wash stays faint so the output reads as
  before.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | For each block with a line on screen, the terminal shall draw a gutter bar over its rows in its status's color | unit tests (spans, geometry); driven test; live drive |
| REQ-002 | WHEN a finished or running block's first line is on screen, the terminal shall draw its status pill at the right end of that row | driven test; live drive |
| REQ-003 | WHILE a block runs, or after it fails, the terminal shall wash its rows | unit tests; live drive |
| REQ-004 | WHILE the terminal shows the alternate screen, it shall draw no block decoration | driven test |
| REQ-005 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — `visible_spans`, the `Content` field, the element's prepaint and paint; fmt and
  clippy clean; a review of the diff.
- **P3 Test** — the unit and driven tests, with negative checks; the live drive;
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the
  ticket, archive, commit.
