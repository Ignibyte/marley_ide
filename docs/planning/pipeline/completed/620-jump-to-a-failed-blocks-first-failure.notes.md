# Jump to a failed block's first failure — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-620-jump-to-a-failed-blocks-first-failure.md
- **Pipeline spec:** 620-jump-to-a-failed-blocks-first-failure.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, first batch (#619 to #623): T2 and the rest of T4.
- **Recall (§18.3):**
  - #572's `running_errors` knows failure shapes but keeps only text, not where a line sits; `absolute_lines_text` joins wrapped rows.
  - `blocks::reveal` scrolls to an absolute line and is the jump's base.
  - #433's `file:line:col` extractor belonged to the gpui-era app; nothing in this tree extracts positions.
- **Discovery:** one Explore sweep for T2 and T4 (2026-09-30) over the block model, the
  terminal's links, the failure shapes, the task spawn path and the diagnostics store; the spec's
  Prior art cites what applies here, and the Plan phase re-verifies each seam at promotion.

## Phase 1 — Plan (promoted 2026-09-30)
- **Promoted;** seams re-verified: `blocks::reveal(terminal, index)` scrolls to a block's first
  line (`blocks.rs:148`) and `block_menu` builds the Block section (277); the element's chip hook
  is `bookmarks::chip`, which shows `send_block::chip`'s Ask the agent on the newest failed block;
  `Terminal::block_output(block)` reads a finished block's text with wrapped rows joined, and
  `alacritty::marley_rows_view` (`pub(super)`) has each row's wrap flag from the absolute line
  `first`; `apply_shell_hook` notifies the terminal after every hook; `open_path_like_target` is
  `pub(super)` in `terminal_view`, so the open follows `browser.rs`'s `open_abs_path` then
  `go_to_singleton_buffer_point`.
- **Recall:** the queued notes stand. The brain (`rusty-cli brain ask`, consultation
  ce7c6ab1d2fa421bbbd67ed5f599f2c7): nothing on this seam.

### Design
- **`crates/marley_terminal/src/failures.rs`** (new, pure): `Failure { row, path, line, column,
  severity, message }` and `failures(text, wraps, first_row) -> Vec<Failure>`: the text's logical
  lines walked with the rows they start on (a row continues while its wrap flag is set), each
  line matched by hand-written parsers (no regex): rustc's ` --> path:l:c` (its row the
  `error`/`warning` header above when there is one), `path:l:c: [error|warning]…` (gcc, clang,
  go), tsc's `path(l,c): error TS…`, and Python's `File "path", line N` (the last frame of a
  traceback, with the exception line as its message). `first_failure` takes the first error, else
  the first failure.
- **`crates/terminal/src/terminal.rs`** (Zed crate, `// Marley:` hunk): `marley_rows()`, the
  main screen's `RowsView` under the lock, beside `block_output`.
- **`crates/marley_workbench/src/failures.rs`** (new): a `BlockFailures` global; an observer on
  each terminal view's terminal reads each newly finished, failed block once (its text and the
  rows), and keeps its failures by (terminal, block index); a closed terminal's go. `jump(view,
  index)`: selects the block, scrolls the failure's row to the top (`blocks::reveal_line`, the
  generalized `reveal`), and opens the path against the block's `prompt.pwd` at its line and
  column; a file that does not open shows a toast. `JumpToFirstFailure`, an action in the crate's
  list, on the focused terminal's newest failed block. The block menu's Jump to First Failure and
  a Jump chip in `bookmarks::chip`, for a failed block with failures.
- **Manifest:** the two new modules (Marley), `blocks.rs`, `bookmarks.rs`, the action list in
  `marley_workbench.rs`, `terminal.rs` (Zed) and its ledger clause, the scenario.

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-003 | a failing command printing forty lines, a rustc error with ` --> src/main.rs:2:5`, forty more, exit 101 | `chip.png` (the chip at the bottom), `menu.png` (the block menu) |
| REQ-001 | Jump to First Failure from the menu | `jumped.png`: the `error` line at the top of the view |
| REQ-002 | the same | `opened.png`: `src/main.rs` at 2:5 |
| REQ-004 | review of the four parsers | — |

### Risks
- A wide terminal joins nothing; a narrow one wraps a long error line, which the row walk counts.

## Phase 2 — Code (2026-09-30)
- **Built:** `marley_terminal::failures` (the four parsers, the row walk, `first_failure`);
  `Terminal::marley_rows()` (Zed hunk) and its ledger clause; `marley_workbench::failures`
  (`BlockFailures`, the observer, `first`, `jump`, `jump_focused`, `chip`); `blocks::select`
  shared and `blocks::reveal_line` split out of `reveal`; the block menu's Jump to First Failure
  above Send to Agent; the Jump chip in `bookmarks::chip` before Ask the agent; the action
  `JumpToFirstFailure`.
- **Review found:** `jump_focused` runs inside the workspace's action listener, and `jump` updates
  that workspace to open the file, an update of an entity already being updated, which panics in
  gpui: the action now defers the jump (`window.defer`). The observer reads only blocks with an
  index at or past the last read one, newest first, so the per-output check costs a comparison.
- **Clippy found:** a long first doc paragraph, unused imports, `TaskExt` and
  `InteractiveElement` not in scope, `Button::icon` (now `end_icon`), `nonminimal_bool`, and
  `needless_pass_by_ref_mut` once the jump deferred.
- **Gate:** `just gate-diff` GREEN, 17 PASS (the scenario written before it; one measuring run
  before the gate, its shots read in Test).

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/620-jump-to-a-failed-blocks-first-failure.sh` (sway).
- **Run 1 (measuring, before the gate):** the chip, the menu and the open all showed, but the
  error sat at the top of the view before the jump: the 45 lines from the error to the end filled
  the screen exactly. The script now prints 80 lines after the error.
- **Run 2:** `jumped.png` had the error's row at the top, under the pinned command (#529), which
  hid it; only ` --> src/main.rs:2:13` showed. Fixed in `jump` (the row above the report goes to
  the top); the gate GREEN again, rebuilt.
- **Run 3, the shots read:**
  - `chip.png` (REQ-003): the failed block's output from line 36 to 80 in view, the error off
    screen above, and at the right of the block's last row the red Jump to Failure chip with its
    arrow; the rail's row says `bash fail.sh · exit 101`.
  - `menu.png` (REQ-003): the right-click menu's Block section headed by Jump to First Failure,
    above Send to Agent.
  - `opened.png` (REQ-002): after the chip's click, `src/main.rs` in the editor, the cursor at
    2:13 (the status bar), on `x`.
  - `jumped.png` (REQ-001): the terminal again: the pinned `$ bash fail.sh` with `exit 101`, then
    `error[E0425]: cannot find value \`x\` in this scope`, ` --> src/main.rs:2:13` and the
    rest; the block outlined as selected.
- **REQ-004** (the four families): reviewed in `failures.rs`; the scenario drives rustc's.
- **Not driven:** the action from the palette (it calls the same `jump`, deferred).

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md`; the guide (Blocks); `terminal_blocks.md`; the plan's T2 row;
  the touchpoints clause (Code).
- **Knowledge:** F-claude-620-an-action-jump-would-have-updated-its-own-workspace-001,
  F-claude-620-the-pinned-command-covered-the-jumped-to-error-001,
  L-claude-620-a-scroll-shot-needs-the-target-off-screen-first-001.
- **Brain:** consultation ce7c6ab1d2fa421bbbd67ed5f599f2c7 closed with `brain decide`.

