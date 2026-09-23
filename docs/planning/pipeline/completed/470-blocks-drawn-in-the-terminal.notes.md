# Blocks drawn in the terminal, stage one — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-470-blocks-drawn-in-the-terminal.md
- **Pipeline spec:** 470-blocks-drawn-in-the-terminal.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] mint · [x]
  prior-art sweep · [x] spec · [x] design · [x] present (Chad's goal: "lets continue on those",
  T1 and zsh; autonomous).
- **Classification / tier:** feature, medium; `marley_terminal`, and small additive hunks in
  Zed's `terminal` and `terminal_view`. T1 split: T1a here, hover actions (T1b) and navigation
  keys (T1c) after.
- **Pre-flight:** #469 committed (`9f5028bd02`); no other active pipeline; README marker present;
  cargo idle.
- **Recall (§18.3).**
  - The plan's D3 (stage one draws over the grid; stage two changes the row model) and D2
    (blocks anchored by absolute line).
  - #464's `AnchoredBlock` lines and `HookPosition`'s absolute line (evicted + history + cursor
    line).
  - The gpui era's gutter decisions (AD-claude-434) drew a grid of Marley's own; not this
    substrate.
  - Brain: consultation `d992610eafc04776a8c75aa76b3c30df`, nothing on this seam.
- **Discovery.**
  - `TerminalElement::prepaint`: `gutter = cell_width`, the grid's origin moved right by it and
    snapped to device pixels; `terminal.sync` fills `last_content`; `hyperlink_tooltip` is an
    `AnyElement` placed with `prepaint_as_root`.
  - `paint`: `origin = dimensions.bounds.origin - (0, scroll_top)`; `layout.rects` (cell
    backgrounds), then highlights, then text, then the cursor and the overlay elements.
  - `make_content` (`crates/terminal/src/alacritty.rs:884`) builds `Content`; its other
    literals are `Default` and a test helper in `terminal.rs`.
  - The kernel refuses `TIOCSTI` (`dev.tty.legacy_tiocsti = 0`), so no startup file can type
    into its own terminal; the live drive prints the frames instead.
  - The ledger's `alacritty.rs` row predates #464 and does not name `absolute_lines_text`; it is
    brought up to date with this ticket's hunk.

### Design
- **`marley_terminal::anchored`**: `BlockSpan { index, rows: Range<usize>, starts_in_view,
  state, exit_code }` and `visible_spans(blocks, top, screen_lines, cursor_line) ->
  Vec<BlockSpan>`, with `try_from` for the narrowing casts.
- **`terminal`**: `Content::marley_screen_top: u64` in its three literals; `make_content` sets
  it to `grid.evicted_lines() + history_size`.
- **`terminal_view::terminal_element`**:
  - `LayoutState` gains `marley_spans: Vec<BlockSpan>` and `marley_pills: Vec<AnyElement>`.
  - In `prepaint`, after the content is read: no spans on the alternate screen; else `top =
    marley_screen_top - display_offset`, `cursor_line = marley_screen_top + cursor line`, and
    for each span that starts in view a pill, a row-wide `div` justified to the end holding a
    small rounded label, `prepaint_as_root` at the row's origin; `debug_selector`
    `marley-block-pill-{index}`.
  - `marley_rows_bounds(rows, origin, dimensions)`: the pixel bounds of a span's rows over the
    grid's width, and `marley_gutter_bounds(rows, element_left, origin, dimensions)`: a bar two
    pixels wide centered in the gutter cell. Pure, unit-tested.
  - In `paint`, washes after `layout.rects`; bars and pills after the text runs.
- **File manifest.**
  - Marley: `crates/marley_terminal/src/anchored.rs` (and its re-export).
  - Zed: `crates/terminal/src/alacritty.rs`, `crates/terminal/src/terminal.rs`,
    `crates/terminal_view/Cargo.toml` (`marley_terminal`), `crates/terminal_view/src/
    terminal_element.rs`. Ledger rows first.

### Test plan
| REQ | Test |
|---|---|
| 001 | unit: `visible_spans` clamps a block that starts above or ends below the viewport, skips one outside it, ends a running block after the cursor's line, and starts one at its output when no prompt was seen; unit: the gutter bounds sit in the gutter cell over the rows |
| 002 | driven: a PTY prints the frames for `true` and `false`; both pills draw, each at its block's first row and at the grid's right end |
| 003 | unit: the wash color is chosen for running and failed blocks and none for a success |
| 004 | driven: the same terminal after `\e[?1049h` draws no pill |
| 005 | `script/gates.sh --diff` |

### Risks
- **Reflow on resize** moves lines under their anchors (the plan's D2 risk); stage one draws
  what the anchors say. Reflow is its own slice.
- **The pill covers text** at a row's right end, a right-aligned prompt's included.

## Phase 2 — Code (2026-09-23)
- **Checklist:** [x] recall · [x] marker present · [x] ledger rows first · [x] `anchored.rs` · [x]
  `alacritty.rs` and `terminal.rs` · [x] `terminal_view/Cargo.toml` · [x]
  `terminal_element.rs` · [x] fmt and clippy · [x] review.
- **Built.**
  - `marley_terminal::anchored`: `BlockSpan`, `visible_spans`, re-exported.
  - `terminal`: `Content::marley_screen_top` in the struct and its `Default`; `make_content`
    sets it. The test helper's literal takes it from `..Default::default()`.
  - `terminal_view`: `marley_terminal` as a dependency; in `terminal_element.rs`, the two
    `LayoutState` fields, the prepaint block, the three paint calls, and `marley_block_spans`,
    `marley_rows_bounds`, `marley_gutter_bounds`, `marley_wash`, `marley_bar_color` and
    `marley_pill`.
- **Deviations.**
  - **`marley_block_spans`** holds the alternate-screen guard and the scroll arithmetic, where the
    design had them inline in `prepaint`, so a unit test can break them (Phase 3).
  - **The wash spans the gutter and the grid**, from the element's left edge, so a failed block
    reads as one band with its bar.
- **Review.**
  - Borrows: the prepaint block reads the terminal after `sync` and after every other read of
    `last_content`, and takes nothing mutable.
  - `u64::try_from` for the cursor's line, which is never negative on screen; `saturating_sub`
    for a scroll larger than the screen's top.
  - The pills are taken out of the layout before the paint closure, as the tooltip and the
    block below the cursor are.
  - Upstream: each hunk marked `Marley:`; the ledger rows were written first, the stale
    `alacritty.rs` row among them.

## Phase 3 — Test (2026-09-23)
- **Checklist:** [x] REQ-001 · [x] REQ-002 · [x] REQ-003 · [x] REQ-004 · [x] negative checks ·
  [x] live drive · [x] gate.
- **Tests.**
  - `marley_terminal`: `a_viewport_shows_the_rows_of_each_block_it_holds` and
    `a_block_with_no_line_in_the_viewport_is_left_out` (REQ-001).
  - `terminal_view::terminal_element`: `marley_a_blocks_decorations_cover_its_rows` (REQ-001),
    `marley_a_blocks_colors_follow_its_status` (REQ-001, REQ-003),
    `marley_the_blocks_on_screen_follow_the_scroll_and_not_the_alternate_screen` (REQ-004).
  - `terminal_view`: `marley_blocks_draw_their_pills_on_their_first_rows` (REQ-002), a real PTY
    printing `seq 1 200` and then the frames of `true` and `false`: both pills draw, centered one
    row apart (within the pixel the layout snaps to), at the grid's right end;
    `marley_the_alternate_screen_draws_no_block` (REQ-004).
- **Negative checks,** each restored by checksum:
  - `marley_screen_top` forced to 0: the pill test fails (the blocks map off screen).
  - A running block ending at its output's start: the viewport test fails.
  - The alternate-screen guard disabled: the driven alternate-screen test still passed.
    Entering the alternate screen clears it, which scrolls the alternate grid, and its own
    evicted-line count put the viewport's top at 49, past the blocks, so they fell off screen
    either way. The guard moved into `marley_block_spans`, whose unit test fails without it.
  - The scroll ignored in `marley_block_spans`: its unit test fails.
- **Clippy** on `marley_terminal`, `terminal` and `terminal_view`, all targets: clean.
- **Live drive.** The debug `marley` on a copy of Chad's profile with `terminal.env.HOME` at a
  scratch home, on hidden workspace 9, shot by toplevel with no input. The scratch `.bashrc`
  prints the hook frames around three real commands before Marley's hooks load (the kernel
  refuses `TIOCSTI`, and nothing else can type on a hidden workspace): `echo hello`, `ls /nope`,
  `sleep 60`. The capture shows a green bar beside `echo hello` with a check pill; a red bar and
  a faint red wash over `ls /nope` with an `exit 2` pill; a blue bar and wash from `sleep 60`
  down to the cursor with a `running` pill; each pill at the right end of its block's first
  row.
- **Gate.** Two reds before the green:
  - gate:16: `crates/terminal_view/src/terminal_view.rs` changed with no ledger row. The driven
    tests went in through a `python3` patch, which `enforce-zed-ledger.sh` (Write and Edit only)
    never saw; the row was added
    (L-claude-470-a-script-edit-skips-the-ledger-hook-001).
  - gate:3: `marley_blocks_draw_their_pills_on_their_first_rows` gave up after 32 seconds with
    fewer than two finished blocks, under the gate's parallel load; alone it takes 0.2 s. Three
    runs of the three crates' suites together failed it once. The script printed its frames and
    exited at once, and on the child's exit the event loop drains with one `pty_read`, whose
    first read error ends it (`vendor/alacritty_terminal/src/event_loop.rs:266-277`), so under
    load the last frames can go unread. The helper now ends every script with `sleep 60`; four
    runs under the same load passed. #464's `build_shell_hook_terminal` has the same exposure
    and has not failed; it is noted for the next ticket that touches those tests
    (L-claude-470-a-pty-test-child-that-exits-can-lose-its-last-bytes-001).
  - Then `GATE GREEN [diff]`, 20 passed, the receipt matching the tree.

## Phase 4 — Complete (2026-09-23)
- **Docs (§21).** `CHANGELOG.md` (Added); `docs/marley_architecture/terminal_blocks.md`
  (`visible_spans` and the element's drawing); `docs/marley/three-prong-plan.md` (T1a shipped,
  T1b and T1c named); `docs/marley/zed-touchpoints.md`: the `alacritty.rs` row brought up to date
  (#464's `absolute_lines_text` was missing) and extended, the `terminal.rs` row extended, and new
  rows for `terminal_view`'s `Cargo.toml`, `terminal_element.rs` and `terminal_view.rs`.
- **Ledger (§19).** `AD-claude-470-stage-one-draws-blocks-over-zeds-rows-001`,
  `L-claude-470-the-alternate-grid-counts-its-own-evicted-lines-001`,
  `L-claude-470-a-script-edit-skips-the-ledger-hook-001`,
  `L-claude-470-a-pty-test-child-that-exits-can-lose-its-last-bytes-001`. No F block: nothing
  shipped broken; the flake was the new test's own.
- **Brain.** Consultation `d992610eafc04776a8c75aa76b3c30df` closed by
  `decisions/stage-one-draws-blocks-over-zeds-own-terminal-rows`, follow-up 2026-10-07.
- **Ticket** closed; the pipeline archived; one commit.
