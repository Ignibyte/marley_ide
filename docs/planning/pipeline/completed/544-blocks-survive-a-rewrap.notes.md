# Blocks keep their rows when a resize rewraps the terminal — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-544-blocks-survive-a-rewrap.md
- **Pipeline spec:** 544-blocks-survive-a-rewrap.spec.md

## Phase 1 — Plan
- **Request:** found in #516's Test (2026-09-25): the Settings window, tiled beside the main
  window by sway, narrowed the terminal from about 120 columns to 30, its lines rewrapped, and
  every block lost its rows; part of Chad's goal.
- **Classification / tier:** bug; the block terminal. A Zed crate (`terminal`: the resize arm and
  its grid helpers in `alacritty.rs`) and a Marley crate (`marley_terminal::anchored`).
- **Recall (§18.3):**
  - F-claude-516-a-rewrap-moves-every-blocks-rows-001: the bug as #516's first run met it.
  - L-claude-516-a-second-window-in-sway-narrows-the-terminal-under-test-001: the Settings
    window, tiled beside the main window, is a repeatable narrowing in a sway scenario.
  - The plan's D2 ("Reflow on resize is the known weak spot"), with the OSC 8 tag as the
    mitigation to evaluate; AD on #464 (the grid counts evicted lines; "a reflowing resize is not
    covered").
  - Brain (consultation 9650b3ab5250446d8eac2ce796da37eb): nothing on this seam.
- **The scenario on the unfixed build (L-claude-512):** red at "the blocks read the same after
  the rewrap": after the narrowing, `long` read one line and a half (`long line 2: word01 word02
  wor`) and `echo done` read `7 word08 word09 word10 word11`, rows of `long`'s output.
- **Discovery** (an Explore sweep, then the seams re-read):
  - The anchors: each block's `prompt_line`, `output_start`, `output_end` (exclusive); the
    blocks' `staged` prompt line; `input_start` (a line and a column)
    (`crates/marley_terminal/src/anchored.rs` 33-79). A hook's absolute line is the grid's
    evicted count plus its history plus the cursor's row (`vendor/.../marley_hooks.rs` 63-65).
  - One resize path: `set_size` queues `InternalEvent::Resize`, whose arm
    (`crates/terminal/src/terminal.rs` 1885-1911) calls `resize(term, new_bounds)` (1901,
    `term.resize`) and touches no block. `Term::resize` reflows the main grid, and while the
    alternate screen shows it reflows the private `inactive_grid`, which is the main screen then
    (`vendor/.../term/mod.rs` 677-678). The height changes move rows through `scroll_up`, which
    counts evictions; a width change reflows along `WRAPLINE`, keeps the cursor inside its logical
    line (`grid/resize.rs` 101-388), and, with a full history, drops rows off the top at 367-368
    without counting them. alacritty returns no mapping.
  - The consumers: the element's spans, wash, bar, pill and Copy
    (`terminal_view/src/terminal_element.rs` 1619-1650, 1773-1826, 2182-2380), the block keys
    (`marley_workbench/src/blocks.rs`), suggestions (`autosuggest.rs` 76-87, `input_start`), and
    MCP (`mcp.rs`: `block_output`, `block_output_kept`). All read the anchors, so fixing the
    anchors fixes them all.
  - Found beside it: `absolute_lines_text` and `block_output_kept` read the active grid, which
    is the alternate screen's while vim or less shows, so `terminal_read` answers wrong rows then.
    Ticketed as #546; not this slice.

### Design
- **The key a rewrap keeps.** alacritty rewraps logical lines (a row and the rows its last
  cell's `WRAPLINE` continues into) and keeps the cursor's logical line. So an anchor is carried
  across a width change as a *place*: how many logical lines it sits above the cursor's (negative
  below), and its character offset inside its logical line (row offset times the columns, plus
  its column). Pivoting on the cursor, not the top, is what survives rows dropped off the top of
  a full history and padding cut at the bottom.
- **Pure, in `marley_terminal::anchored`:** `RowsView { first, cursor, wraps, columns }` (the
  absolute line of the grid's first row, the cursor's row index, each row's `WRAPLINE`, the
  width); `rewrap_line(before, after, line, column) -> (u64, usize)`: an anchor above `before`'s
  first row is left as it is (already evicted); one whose logical line is above `after`'s first
  row comes back as the line before it, so it reads as evicted; otherwise the row of the target
  logical line and the offset's row and column at the new width, clamped to the logical line's
  last row. `AnchoredBlocks::rewrap(&before, &after)` applies it to all five anchors.
- **In `crates/terminal/src/alacritty.rs` (Zed crate, a Marley function):**
  `marley_rows_view(term)` builds the view from `term.main_grid()`, rows from the top of the
  history to the screen's bottom.
- **In `crates/terminal/src/terminal.rs` (Zed crate):** in the Resize arm, when the columns change,
  the view before `resize(term, new_bounds)`, the view after, then `self.blocks.rewrap(..)`.
- **Vendored (`vendor/alacritty_terminal`, Marley-owned):** `Term::main_grid()`, the primary
  screen's grid whichever screen shows; `shrink_columns` adds the rows its `truncate` drops to
  `evicted_lines`. Both recorded in `vendor/README.md`.
- **File manifest:** `crates/marley_terminal/src/anchored.rs` (Marley); `crates/terminal/src/alacritty.rs`,
  `crates/terminal/src/terminal.rs` (Zed; rows widened first); `vendor/alacritty_terminal/src/term/mod.rs`,
  `vendor/alacritty_terminal/src/grid/resize.rs`, `vendor/README.md` (vendored).

### E2E plan
| REQ | Scenario and steps | Proof |
|---|---|---|
| REQ-001 | `script/e2e/544-blocks-survive-a-rewrap.sh`: `seq 1 3`, `long` (five lines of about 110 characters), `echo done`; read each block through the stand-in agent at the full width, after the Settings window narrows the terminal to about 30 columns, and after it closes | The run log: the three reads equal; red on the unfixed build (above) |
| REQ-002 | The same run | Shots `544-01-wide`, `544-02-narrowed`, `544-03-wide-again`: the bars and pills on each block's rows |

### Risks
- A shell hook positioned before a resize but applied after it (the IO thread positions hooks;
  the main thread applies them after a short batch) carries an old-width line: rare, a block
  misplaced until the next one. Accepted, and noted in the code.
- The shell redraws its prompt on SIGWINCH at the old width's row count and can overwrite rows
  above it; the staged prompt line and `input_start` can be off until the next prompt.
- A wide character at a wrap point can put a restored column one cell off.

## Phase 2 — Code
- **Built:**
  - `marley_terminal::anchored` (pure): `RowsView { first, cursor, wraps, columns }`, re-exported
    at the crate root; private `LogicalLines` (each row's logical line, each line's first row) and
    `Place` (`Distance::Above` or `Below` the cursor's logical line, and the character offset inside
    its own); `AnchoredBlocks::rewrap(before, after)`, which carries each block's prompt line,
    output start and output end, the staged prompt's line and the input start with its column. An
    anchor above `before`'s first row stays as it was; one whose logical line is above `after`'s
    first row comes back as the line before it, so it reads as evicted; one below the cursor is
    clamped to the last logical line; a restored row is clamped to its logical line's last row.
  - `crates/terminal/src/alacritty.rs`: `marley_rows_view(term)`, from `term.main_grid()`: each row's
    last-cell `WRAPLINE` from the top of the history to the screen's bottom, the cursor's row, and
    `evicted_lines` as the first row's absolute line (`topmost_line` is `evicted` in the
    `absolute_lines_text` arithmetic).
  - `crates/terminal/src/terminal.rs`: the Resize arm, only when the columns change, reads the view,
    resizes, reads it again and calls `self.blocks.rewrap`, with the hook race noted on the hunk.
  - Vendored: `Term::main_grid`; `shrink_columns` adds the rows its `truncate` cuts to
    `evicted_lines`; both in `vendor/README.md`. The ledger rows for `terminal.rs` and
    `alacritty.rs` were widened first.
- **Deviations:** none from the design.
- **Review:** the arithmetic uses `checked_sub`, `saturating_*` and `get` throughout, so no input
  can panic it; nothing reads or updates an entity (the arm runs inside `Terminal`'s own update,
  on `term` and `self.blocks`); nothing is derived from Warp, and the logical-line walk is written
  anew for the pure view (Zed's `find_logical_line_start` walks the grid itself, one row at a
  time, and stays where it is).
- **Checks:** `cargo check -p marley_terminal -p terminal` clean; rustfmt clean; clippy on both with
  every target, warnings as errors: the first run wanted `RowsView`'s first doc paragraph shorter
  (and `rewrap`'s was split too); the second is clean.

## Phase 3 — Test
- **Scenario:** `script/e2e/544-blocks-survive-a-rewrap.sh` (`compositor sway`), written at Plan
  and red on the unfixed build (above).
- **Found in Test, fixed:** the first run on the fixed build failed exactly as the unfixed one
  did. A temporary log line in the Resize arm never printed: the arm's `columns_changed` compares
  `last_content.terminal_bounds` with the new bounds, but `set_size` stores the new bounds there
  before it queues the event, so the test is always false (upstream's `reset_cwd_history` on a
  width change never runs either; left as it is). The hunk now reads the rows' view before and
  after `resize` and rewraps when the grid's own width changed. The log line was removed.
- **Checks in the final run:** `check the blocks read the same after the rewrap: pass` (the three
  blocks' reads at about 30 columns equal the full-width reads) and `check the blocks read the
  same at the width they started at: pass` (after the Settings window closes).
- **Shots, read:**
  - `544-01-wide`: `seq 1 3`, `long` (five one-row lines) and `echo done`, each with its bar and
    its pill on its own first row.
  - `544-02-narrowed`: the terminal beside the Settings window, about 30 columns wide; each long
    line wraps into three rows, the green bar runs down `long`'s wrapped rows, and `echo done`'s
    pill sits on `$ echo done` (on the unfixed build the bar and the pills vanished).
  - `544-03-wide-again`: back at the full width, row for row as `544-01`, all three pills on
    their prompts.
- **The golden set** against this build: all eight pass (the anchors feed 484's suggestions, 491's
  and 516's reads). `544-blocks-survive-a-rewrap` joins the set, and the #516 scenario's comment
  no longer names the rewrap as a live bug.
- **Focus report:** sway, headless; Hyprland untouched.
- **Gate:** `just gate-diff`: `GATE GREEN [diff]`, 16 passed, the vendored alacritty's own build
  among them.
- **Verdict:** PASS.

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Fixed: blocks survive a resize); `docs/marley_architecture/terminal_blocks.md`
  (the rewrap); `docs/marley/three-prong-plan.md` (D2 and its risk settled by #544);
  `docs/marley/zed-touchpoints.md` (the `terminal.rs` and `alacritty.rs` rows); `vendor/README.md`
  (the two vendored hunks); the guide (the troubleshooting entry and the planned row gone); the
  golden set gains the scenario.
- **Knowledge:** F-claude-544-the-resize-arms-columns-changed-is-always-false-001,
  F-claude-544-a-rewrap-cut-history-rows-without-counting-them-001,
  PR-claude-compare-against-the-state-the-change-acts-on-001,
  L-claude-544-when-a-fix-fails-exactly-like-the-bug-instrument-001,
  AD-claude-544-anchors-cross-a-rewrap-as-logical-places-001.
- **Brain:** consultation 9650b3ab5250446d8eac2ce796da37eb closed with
  `decisions/marleys-block-anchors-cross-a-rewrap-as-logical-places-pivoting-on-the-cursor`,
  follow-up by 2026-10-10.
- **Found here and ticketed:** TICKET-546 (block reads while a full-screen program shows), with a
  Queue row.
- **Ticket:** closed; the pair archived to `completed/`.
