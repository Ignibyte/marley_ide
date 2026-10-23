# Terminal blocks as the board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-916-terminal-block-look.md
- **Pipeline spec:** 916-terminal-block-look.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, item 6 (terminal blocks).
- **Classification / tier:** feature, M. `terminal_view` hunks under Marley's block hooks, Marley's
  `block_headers.rs` and `send_block.rs`.
- **Recall (§18.3):**
  - AD-claude-631-blocks-stand-apart-through-a-display-row-map-the-mouse-reads-back-001: gaps and
    extra rows are pixels inserted above viewport rows, applied at every row-to-y site and read
    back by the mouse (`marley_local`). The footer row is one more insert above the live prompt,
    through the same `RowMap`, so the mouse mapping holds.
  - AD-claude-628-a-blocks-header-takes-its-prompts-rows-one-for-one-001 and
    AD-claude-474-hover-actions-live-on-one-element-per-block-001: the header lays over the
    prompt's rows, and one element per block carries the pill and the hover actions; the progress
    line joins that element.
  - L-claude-474-an-occluding-child-ends-its-groups-hover-001, F-claude-474 (the occluding buttons
    hid under the pointer) and PR-claude-812-an-element-that-stops-a-press-over-a-view-focuses-it-001:
    the footer's buttons stop presses through `marley_keep_from_terminal`, and the footer row
    focuses the terminal on a press elsewhere in it.
  - PR-claude-631-a-gesture-belongs-to-the-element-it-was-pressed-in-001: a drag that starts on the
    footer row must not become a selection in the grid.
  - L-claude-803-a-blocks-chip-moves-with-its-output-001: compute click points from the line count.
  - F-claude-903-a-stand-in-not-on-the-terminals-path-001: the stand-in `claude` on the
    terminals' PATH through the `.bashrc`.
  - PR-claude-830-an-eased-value-asks-for-frames-where-every-form-renders-001: the line stays a
    gpui `with_animation`, not a hand-eased value.
- **Discovery:**
  - `crates/terminal_view/src/terminal_element.rs`: :1331-1360 (gutter), :1576-1660 (headers and
    `RowMap`), :2058-2155 (wash, bar, frame), :2786-2850 (`marley_wash`, `marley_bar_color`,
    `marley_pill`), :2866-2988 (`marley_block`), :1835-1910 (the chip's placement).
  - `crates/terminal_view/src/terminal_view.rs:201-263` (the hooks).
  - `crates/marley_terminal/src/` (`RowMap::new`, its `tall` inserts).
  - `crates/marley_workbench/src/block_headers.rs` (`init`, `header`, `progress_line`).
  - `crates/marley_workbench/src/send_block.rs` (`chip`, `duration`) and where the chip hook is set
    (`bookmarks.rs`'s `MarleyBlockChip`).
- **Decisions:** D1–D5 in the spec.

### Design
- Zed crate, `terminal_element.rs` (each hunk `// Marley:`):
  - `marley_bar_color` and `marley_wash`: running takes `status.warning`; the wash's bounds become
    the header's rows (the block's prompt rows from `marley_prompt_rows`), only for failed and
    running blocks.
  - The frame: `window.paint_quad(quad(rows, radius, transparent, px(1.), color, Solid))` with
    `radius = px(12.).min(rows.size.height / 2.)`; done = `colors.border`.
  - The gutter: `gutter = cell_width * 2.` when `MarleyBlockHeader`'s hook is set and native
    headers are on (a flag the hook's global carries, so the element does not read Marley's
    settings); the bar is skipped then.
  - The footer: a new `MarleyBlockFooter(Arc<dyn Fn(&Entity<TerminalView>, &Entity<Terminal>,
    usize, &App) -> Option<AnyElement>>)` asked for the newest block when the live prompt is on
    screen; with an element, the live prompt's row joins `tall` in `RowMap::new`, and the element
    is laid out over that inserted row's pixels, full width.
  - The progress: `MarleyBlockProgress(Arc<dyn Fn(usize, &App) -> Option<AnyElement>>)`; in
    `marley_block`, a running span's element gets it as an absolute child at the bottom.
- Zed crate, `terminal_view.rs`: the two globals with doc comments.
- Marley crate, `block_headers.rs`: the dot in `header` (an 8 px rounded div, its left in the
  gutter by a negative margin of the gutter's width); `progress_line` set as
  `MarleyBlockProgress` (3 px, bottom 0, `warning`); in `init`'s `observe_new` for each view, a
  `Ticker` task that, while any live block is `Running`, waits a second on the background
  executor and notifies the view, and ends when none runs (restarted by the terminal's observe).
- Marley crate, `send_block.rs`: `pub(crate) fn footer(...)` with the three buttons (the existing
  `send` and `ask_beside`), `ButtonStyle::Filled` with the accent for Ask an agent; `chip` keeps
  only the 127 case; `duration` handles `Running` from `times(index).started`.
- Ledger rows in `docs/marley/zed-touchpoints.md`: `terminal_element.rs`'s rows for the frame
  (#826), the wash and bar (#470) gain #916's changes; new text for the gutter width, the footer
  row and the progress child; `terminal_view.rs`'s hooks row gains `MarleyBlockFooter` and
  `MarleyBlockProgress`.
- File manifest: Zed crate: `terminal_element.rs`, `terminal_view.rs`. Marley crate:
  `block_headers.rs`, `send_block.rs`, `bookmarks.rs` (if the chip's composition changes),
  `marley_workbench.rs` (setting the new hooks in `init`).

### Visual check plan
- REQ-001 to REQ-003: `clear`, `git status --short`, `false`; settle; `916-01-blocks`; read the
  radius, the gap, the dots, the passed frame's colour, the failed frame and band, the command and
  output starting together, and the footer row's three labelled buttons.
- REQ-004, REQ-005: `sleep 8`; settle two seconds; `916-02-running`; settle two more;
  `916-03-ticked`; compare the times.
- REQ-003: settle until `sleep` ends; `false`; click Ask an agent (its point from the probe run's
  shot); settle; `916-04-ask`.
- All in Day: switch the theme live; repeat `false` and a short `sleep`; `916-05-day`.

### Risks
- Widening the gutter takes one column from the PTY, once, and again when `marley.block_headers`
  changes; AD-claude-631 keeps rows fixed, not columns. Programs that drew at the old width redraw
  on the resize. Recorded; if the shot shows a broken full-screen program, fall back to the dot
  over the one-cell gutter.
- The footer row inserts above the live prompt; the prompt editor (#624) docks at that prompt.
  Check the editor's position follows the row map in the shot.
- The once-a-second notify redraws the whole terminal view; with many terminals running long
  commands this is one paint a second each. Acceptable; the ticker stops when no block runs.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
