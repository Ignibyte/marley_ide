---
pipeline_id: 6c9c862e-97b3-482b-90e7-cb261221bc20
ticket: docs/planning/tickets/open/TICKET-620-jump-to-a-failed-blocks-first-failure.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Jump to a failed block's first failure"
type: feature
slice: prong 1 T2
references: [docs/marley/three-prong-plan.md, docs/zed_architecture/subsystems/08-terminal-tasks-fusion.md]
---

## Title
Jump to First Failure: a failed block's first `path:line:col` scrolled into view and opened in
the editor at the block's folder (plan T2, second half).

## Scope
### In
- `crates/marley_terminal`: a pure, row-aware locator `first_failure(rows)` over a block's rows
  (each with its absolute line, wrapped rows joined) for the shapes rustc and cargo (` --> path:l:c`),
  the common `path:l:c: error` (gcc, clang, eslint, go), tsc (`path(l,c): error`) and Python's
  traceback (`File "path", line N`, its last frame); it returns the row and the position.
- `crates/marley_workbench`: an action `marley::JumpToFirstFailure` (the selected block, else the
  last failed block); the block menu's Jump to First Failure, shown for a failed block with a
  failure found; a click on the failed-block chip (#559's `MarleyBlockChip`) doing the same. It
  scrolls the row into view (`blocks::reveal`, generalized from a block's start to any line),
  marks the row, and opens the path against the block's `prompt.pwd` through Zed's
  `resolve_open_target`.

### Out (explicitly deferred)
- Every failure after the first (T4c lists them as diagnostics).
- Failures in a running block (#572 watches those).
- Shapes beyond the four families.

## Reference (§20)
- **Warp (behavior):** a failed block's error locations are reachable from the block
  (`docs/warp_architecture/subsystems/03-terminal-session-core.md`, 424-431).
- **Upstream Zed:** the open goes through `workspace::path_link::resolve_open_target` as a clicked
  terminal link does; the jump is Marley's.

### Prior art
- **Behavior maps:** the fusion note (`docs/zed_architecture/subsystems/08-terminal-tasks-fusion.md`
  §8, "jump-to-failure on the block's first `file:line:col`"); plan D6.
- **Published material:** the compilers' diagnostic formats (rustc's ` --> `, GNU's
  `file:line:col:`, tsc's `file(line,col)`, Python's traceback frames).
- **Code we already ship:** `running_errors.rs` (#572: failure shapes for running blocks, text
  only, no rows); `alacritty::absolute_lines_text` joins wrapped rows, so the locator needs its
  own row-keeping read; `blocks::reveal` and `scroll_to_block` (`marley_workbench/src/blocks.rs`);
  the block menu (`block_menu`); `MarleyBlockChip` (`bookmarks.rs`); Zed's `path_hyperlink_regexes`
  in `default.json` cover the same shapes for hover links. The #433 extractor was in the gpui-era
  app and is not in this tree.

## UI proof
`script/e2e/620-jump-to-a-failed-blocks-first-failure.sh` (sway): `repo/src/main.rs`; a command
printing forty lines of noise, then a rustc-shaped error with ` --> src/main.rs:2:5`, then more
noise, exiting 101; the view scrolled to the bottom.
- `menu.png`: the failed block's menu with Jump to First Failure;
- `jumped.png`: after it, the error's row in view and marked;
- `opened.png`: the editor on `src/main.rs` with the cursor at 2:5;
- `chip.png`: the failed-block chip's click doing the same from the bottom of the terminal.

## Locked-In Decisions
- D1 — The first failure is the first row, top to bottom, that a shape matches; Python's traceback
  takes its last frame, the one that raised.
- D2 — The path resolves against the block's `prompt.pwd` (#619's rule).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user chooses Jump to First Failure on a failed block, Marley shall scroll the first failure's row into view and mark it. | Shot `jumped.png` |
| REQ-002 | WHEN it jumps, Marley shall open the failure's file at its line and column, resolved against the block's folder. | Shot `opened.png` |
| REQ-003 | WHILE a block failed and a failure was found in it, its menu shall offer Jump to First Failure, and its chip shall jump. | Shots `menu.png`, `chip.png` |
| REQ-004 | The locator shall find each of the four families' shapes. | Review of the patterns |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams, the design.
- **P2 Code** — the locator, the action, the menu item, the chip; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
