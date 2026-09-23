# The prompt at the bottom of the terminal — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-476-prompt-at-the-bottom.md
- **Pipeline spec:** 476-prompt-at-the-bottom.spec.md

## Phase 1 — Plan (queued 2026-09-23)
- **Request:** Chad, 2026-09-23: "the terminals sessions in warp command line stays at the
  bottom which is nice. is that doable". First of the six queued that day, under the goal "lets
  continue and spec these tickets out and run them end to end".
- **Recall.** #470 moved the block decorations onto `dimensions.bounds.origin`, so they follow
  one origin; L-claude-470-the-alternate-grid-counts-its-own-evicted-lines-001 (the alternate
  screen is its own frame of reference); PR-claude-break-the-code-a-driven-test-guards-before-trusting-it-001.
- **Seams read:** `terminal_element.rs` prepaint (the origin, `should_anchor_to_bottom`, the
  padding, `TerminalBounds::new`, `set_size` then `sync`); `Terminal::set_size` (resizes only on
  a line, column or cell-size change); `alacritty.rs` `make_content` (`bottom_row_occupied`).

## Phase 1 — Plan, at promotion (2026-09-23)
- **Recall:** the ledger and the completed pipelines hold nothing on anchoring a terminal's rows;
  the brain (consultation 938f285f98d34599b02f550794fcd317) returned only unrelated follow-ups.
- **Seams re-verified.** `Row::is_clear` (vendored `grid/row.rs`) is true only when every cell is
  a space or tab with the default colours and no underline, inverse or wide-character flags, so
  a line with a coloured background counts as used. `grid[Line(i)]` counts from the live
  screen's top, as the cursor does, whatever the display offset. `paint` clips to the element's
  bounds, so rows moved below it are not drawn.

### Design
- `Content::marley_empty_bottom_rows` (Zed, `terminal.rs` struct and `Default`; set in
  `alacritty.rs` `make_content`): the live screen's rows below the lower of the cursor's line
  and the last row that is not clear, read from the grid, so it holds while the view is scrolled
  back.
- `marley_terminal::bottom_shift(empty_bottom_rows, display_offset, alt_screen)` (Marley, pure,
  in `anchored.rs` beside the other viewport helpers): 0 on the alternate screen, else the empty
  rows less the display offset, saturating.
- `TerminalElement::prepaint` (Zed): the standalone branch keeps the sub-row padding it leaves at
  the bottom; after `set_size` and `sync`, the shift is read from the synced content, the
  origin moves down by the shift in rows plus that padding, and `set_size` stores the moved
  bounds. The lines, columns and cell size are unchanged, so the PTY is not resized.
- A debug selector on the view's container (Zed, `terminal_view.rs`, test builds only) so a
  driven test can compare rows with the pane's real bottom edge.
- **Files:** `crates/marley_terminal/src/anchored.rs` and `marley_terminal.rs` (Marley);
  `crates/terminal/src/terminal.rs`, `crates/terminal/src/alacritty.rs`,
  `crates/terminal_view/src/terminal_element.rs`, `crates/terminal_view/src/terminal_view.rs`
  (Zed; rows first).

### Test plan
| REQ | Test |
|---|---|
| 001, 005 | driven: `MARLEY_TWO_BLOCKS` in a real PTY; the failed block's pill sits two and a half rows above the container's bottom edge |
| 002 | unit: `bottom_shift` cases; driven: after 200 lines and a clear, scrolling back one line leaves the pills where they were |
| 003 | unit; driven: on the alternate screen the stored bounds start at the container's top |
| 004 | driven: with mouse reporting on, a click on the shifted `oops` row reports grid line 2 |
| 006 | `just gate-diff` |

### Risks
- An inline TUI that erases and redraws its region could change the shift between frames;
  Claude Code's synchronized updates keep a redraw to one frame. The live drive with a real
  agent checks it.

## Phase 2 — Code (2026-09-23)
- **Built.** `Content::marley_empty_bottom_rows` (Zed, `terminal.rs`), set in `make_content` by
  `marley_empty_bottom_rows` (Zed, `alacritty.rs`): the live screen's rows below the lower of the
  cursor's line and the last row that is not clear, read from the grid.
  `marley_terminal::bottom_shift` (Marley, `anchored.rs`, re-exported). In
  `TerminalElement::prepaint` (Zed), the standalone branch records the padding it leaves at the
  bottom (`marley_bottom_padding`, `None` for a view that is not standalone); after `set_size`
  and `sync`, the shift from the synced content moves `dimensions.bounds.origin.y` down by the
  shift in rows plus that padding, and a second `set_size` stores it. A test-only debug selector,
  `marley-terminal-view`, on the view's container (Zed, `terminal_view.rs`).
- **Workflow:** `script/live-shot.sh` takes `OPEN`, a path to open as `marley <path>` does; the
  justfile's `shot` says so.
- **Review.** Correctness: the shift uses the content painted in the same frame; the size is
  unchanged, so no SIGWINCH; the origin is not snapped again, and the whole-row shift keeps the
  rows where Zed's own anchored layout puts them. Re-entrancy: the second `set_size` runs inside
  the terminal's own update. Upstream: three additive Zed hunks and a field, rows written first.
  No deviation from the plan.

## Phase 3 — Test (2026-09-23)
- **Tests.** `marley_terminal`:
  `the_shift_puts_the_content_on_the_bottom_edge_until_the_view_scrolls_past_it` (REQ-002,
  REQ-003). `terminal_view`, over a real PTY:
  - `marley_short_content_sits_on_the_bottom_edge` (REQ-001, REQ-005);
  - `marley_scrolled_back_the_history_shows_above_the_content` (REQ-002);
  - `marley_the_alternate_screen_is_drawn_from_the_top` (REQ-003);
  - `marley_a_click_on_shifted_content_reports_its_own_row` (REQ-004);
  - `marley_the_cursors_row_counts_as_content`, added once the review asked what keeps the
    cursor's row in view.
  All 13 of `terminal_view`'s `marley_` tests pass; clippy clean on the three crates; fmt
  applied.
- **A test that assumed too much, fixed.** The first bottom-edge test wanted the pill 2.5 rows
  above the container's edge and read 2.88. Zed counts rows with the line height rounded to
  device pixels: in the test window 1024 px give a snapped height of 1008 px, while 55 rows of
  18.2 px take 1001, so the grid's last row ends 7 px above the edge on a full screen too. The
  shift puts the last used row where Zed puts its last row, so nothing moves when the screen
  fills; the test now wants the prompt on the grid's last row, less than a row from the edge
  (L-claude-476-zeds-snapped-rows-can-leave-a-gap-below-the-grid-001).
- **Negative checks**, each file restored by sha256:
  1. no shift at all: FAIL in the bottom-edge and the scrolled-back tests;
  2. the moved origin not stored for the mouse: FAIL, the click reported at grid line 54
     (`left: … 33, 87`) instead of 2;
  3. `bottom_shift` ignoring the display offset: FAIL in its unit test and the scrolled-back
     test;
  4. `bottom_shift` ignoring the alternate screen: FAIL in its unit test and the alternate-screen
     test;
  5. the cursor's row not counted: FAIL, `left: 54`, `right: 53`.
- **Live drive** (`OPEN=<a fresh folder> just shot bottom-476` with #470's seed, the binary
  built after the change): a fresh folder's first terminal shows `echo hello`, `ls /nope` and
  the running `sleep 60` on its bottom rows, the cursor on the last one and each pill on its
  row. Zed's trust prompt for an unknown folder covered the top of the window. Two earlier shots
  opened the profile's last session, which had no terminal, and were deleted.
- **The gate's first run was red** on Zed's own
  `test_short_standalone_terminal_stays_top_anchored_on_resize`, which asserted that a short
  standalone terminal is drawn from the top (`origin.y == 0`), the behavior this ticket
  replaces. It became `test_short_standalone_terminal_sits_on_the_bottom_edge_on_resize`: the
  grid moved down, its one row in view, the empty rows past the edge. Its siblings for a full
  screen and the alternate screen pass unchanged. The ledger row records it.
- **Gate:** `just gate-diff` over `marley_terminal`, `terminal` and `terminal_view`: 20 passed,
  0 failed, `GATE GREEN [diff]`; 697 tests in the scope and 167 in `marley_terminal` pass.

## Phase 4 — Complete (2026-09-23)
- **Docs:** `CHANGELOG.md` (#476, and `just shot`'s `OPEN`); the three-prong plan (T1d shipped);
  `docs/marley_architecture/terminal_blocks.md` (`bottom_shift` and the prepaint hunk); the
  ledger rows for `alacritty.rs`, `terminal.rs`, `terminal_element.rs` and `terminal_view.rs`,
  checked against what ships.
- **Knowledge:** AD-claude-476-the-content-sits-on-the-bottom-edge-by-moving-the-grids-origin-001,
  L-claude-476-zeds-snapped-rows-can-leave-a-gap-below-the-grid-001,
  L-claude-476-a-fresh-folder-gets-a-first-terminal-for-a-capture-001. No F-block: the one red
  was the test's own expectation.
- **Brain:** consultation 938f285f98d34599b02f550794fcd317, decided at Complete.
