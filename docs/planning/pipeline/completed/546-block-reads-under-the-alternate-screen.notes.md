# Block reads while a full-screen program shows — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-546-block-reads-under-the-alternate-screen.md
- **Pipeline spec:** 546-block-reads-under-the-alternate-screen.spec.md

## Phase 1 — Plan
- **Request:** found in #544's Plan (2026-09-26) by the anchors' sweep; part of Chad's goal.
- **Classification / tier:** bug; two Marley functions in Zed's `terminal` crate and a vendored
  addition.
- **Recall (§18.3):**
  - #544's sweep: `absolute_lines_text` (alacritty.rs) and `block_output_kept` (terminal.rs) read
    `term.grid()`, the active grid; while the alternate screen shows, its evicted count climbs with
    every scroll of the full-screen program.
  - AD-claude-544-anchors-cross-a-rewrap-as-logical-places-001: `Term::main_grid` exists.
  - Brain (consultation 5fd936c8114d4a41929660dbd0fedb1f): nothing on this seam.
- **The scenario on the unfixed build (L-claude-512):** red at "the block reads the same while
  less shows": while `less` held the alternate screen, `terminal_read` of `seq 1 3` was refused
  ("block 0's output has left the terminal's scrollback") and `terminal_blocks` listed it `kept
  False`; the alternate screen's evicted count had passed the block's lines.
- **Discovery:** `crates/terminal/src/alacritty.rs` `absolute_lines_text` (the grid, its evicted
  count and history, `bottommost_line`, the cursor, then `term.bounds_to_string`);
  `crates/terminal/src/terminal.rs` `block_output_kept` (`self.term.lock().grid().evicted_lines()`);
  `vendor/alacritty_terminal/src/term/mod.rs` `bounds_to_string` and `line_to_string` (read
  `self.grid`, `self.tabs`, `self.columns()`).

### Design
- **Vendored:** `Term::main_bounds_to_string(start, end)`, `bounds_to_string`'s loop over
  `main_grid()`, with a private `grid_line_to_string(grid, line, cols, include_wrapped_wide)`, the
  body of `line_to_string` reading the given grid (both screens share the tab stops and the width).
  Recorded in `vendor/README.md`.
- **`absolute_lines_text`:** `term.main_grid()` for the arithmetic and the cursor, and
  `term.main_bounds_to_string` for the text. On the main screen it reads exactly what it read
  before.
- **`block_output_kept`:** `term.main_grid().evicted_lines()`.
- **File manifest:** `vendor/alacritty_terminal/src/term/mod.rs`, `vendor/README.md` (vendored);
  `crates/terminal/src/alacritty.rs`, `crates/terminal/src/terminal.rs` (Zed crate, Marley
  functions; the rows widened).

### E2E plan
| REQ | Scenario and steps | Proof |
|---|---|---|
| REQ-001 | `script/e2e/546-block-reads-under-the-alternate-screen.sh`: `seq 1 3`; `less notes.txt` (a file of 200 lines, so the alternate screen scrolls); the stand-in reads `seq 1 3` before `less`, while it shows (after a page down), and after `q` | The run log: the three reads equal; shot `546-01-less-open` |
| REQ-002 | The same run, `terminal_blocks` while `less` shows | The run log: `seq 1 3` kept |

### Risks
- The copied loop can drift from upstream's `line_to_string` at a re-sync; the vendored README
  names it so the re-sync compares the two.

## Phase 2 — Code
- **Built:** vendored `Term::main_bounds_to_string` and its private `grid_line_to_string`
  (upstream's `bounds_to_string` loop and `line_to_string` body, reading the grid they are given:
  `main_grid()`), recorded in `vendor/README.md` with a note to compare the copy with upstream's
  at a re-sync; `absolute_lines_text` reads `term.main_grid()` and `term.main_bounds_to_string`;
  `block_output_kept` reads `main_grid().evicted_lines()`. The ledger rows for `alacritty.rs` and
  `terminal.rs` were widened first.
- **Deviations:** none.
- **Review:** on the main screen `main_grid()` is `grid`, so every read outside a full-screen
  program is what it was; the tab stops are one field for both screens, and both grids have the
  same width, so `self.tabs` and `self.columns()` hold for the main grid too. No entity access;
  vendored code copies Apache-2.0 alacritty inside its own crate.
- **Checks:** rustfmt clean on the Zed files (the vendored crate keeps alacritty's own style,
  outside the workspace's fmt gate); clippy on `terminal` with every target clean.

## Phase 3 — Test
- **Scenario:** `script/e2e/546-block-reads-under-the-alternate-screen.sh` (`compositor sway`),
  written at Plan and red on the unfixed build (above).
- **Checks in the run:** `check the block reads the same while less shows: pass` (`seq 1 3` read
  `1 2 3` with `less` paged down on the alternate screen), `check its output is kept while less
  shows: pass` (`block 0: 'seq 1 3', exit 0, running False, kept True`), `check the block reads
  the same after less: pass`.
- **Shots, read:** `546-01-less-open`: `less` on the alternate screen at `notes line 046` to `090`
  after a page down, the tab titled "repo — less notes.txt", no block decorations there (by
  design). `546-02-after-less`: the main screen back, `seq 1 3`'s block and the finished `less`
  block, each with its bar and pill.
- **The golden set:** the scenario joins it; all ten pass against this build.
- **Focus report:** sway, headless; Hyprland untouched.
- **Gate:** `just gate-diff`: `GATE GREEN [diff]`.
- **Verdict:** PASS.

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Fixed: agents read blocks while vim or less is open);
  `docs/marley_architecture/terminal_blocks.md` (reads on the main screen); the ledger rows for
  `alacritty.rs` and `terminal.rs`; `vendor/README.md`; the golden set gains the scenario.
- **Knowledge:** F-claude-546-block-reads-answered-from-the-alternate-screen-001 (its class is
  covered by PR-claude-compare-against-the-state-the-change-acts-on-001).
- **Brain:** consultation 5fd936c8114d4a41929660dbd0fedb1f closed with
  `decisions/marleys-block-reads-always-read-the-main-screen`, follow-up by 2026-10-10.
- **Ticket:** closed; the pair archived to `completed/`.
