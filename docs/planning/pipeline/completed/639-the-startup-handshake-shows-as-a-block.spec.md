---
pipeline_id: 6b743019-530d-4412-ab8f-90feb35bbcc2
ticket: docs/planning/tickets/closed/TICKET-639-the-startup-handshake-shows-as-a-block.md
status: Phase 4 — Complete PASS
title: "Blocks follow Zed's screen clears, and the startup check is no block"
type: bug
slice: prong 1 blocks (T1/T5)
references: [docs/marley/three-prong-plan.md]
---

## Title
Before Marley types a command into a new or restored terminal (an agent CLI, a resumed session),
Zed runs a startup check, `printf '%s%s%s\n' __zed_init_command_ready_ 1 __`, then clears the
screen and the scrollback, moving the prompt's line to the top row (`clear_saved_screen`). Zed's
Clear (Ctrl-Shift-L) clears the same way. Marley's blocks are anchored to absolute lines and are
told of neither: the check shows as a block, header and all, with the agent's output under it,
and after a Clear every old block's header stays drawn over the cleared rows, the next command's
output read as an old block's. Found in #540's dry run; the calibration run of 2026-10-01 showed
both.

## Scope
### In
- `marley_terminal::AnchoredBlocks`: a `Preexec` whose command holds Zed's startup marker opens
  no block (the shell's state moves as for any command); `screen_cleared(cursor_was, cursor_now)`
  moves every anchor across a clear: a block wholly above the cursor's old line is emptied (it is
  gone from the screen and the history), a running one goes on from the cursor's new line, and
  the staged prompt and the typed input's start move with the cursor's line.
- `terminal::Terminal`: both places that call `clear_saved_screen` (the Clear event and
  `clear_for_init_command`) tell the blocks, the cursor's absolute line taken before and after.

### Out (explicitly deferred)
- A program's own clear (`clear`, Ctrl-L in the shell: the erase sequences alacritty handles).
- Restoring cleared blocks on undo; Zed's Clear has no undo.

## Reference (§20)
- **Upstream Zed:** `terminal::Terminal`'s Clear and `write_init_command_after_startup`, which
  clear through `clear_saved_screen`; kept as they are, with Marley's anchors told of the move.
- **Warp (behavior):** a cleared terminal shows no old block (`docs/warp_architecture/subsystems/03-terminal-session-core.md`,
  the block list a clear empties).

### Prior art
- **Behavior maps:** AD-claude-470 (blocks anchored to absolute lines), #544's `rewrap` (every
  anchor carried across a change to the grid: the same shape for a clear).
- **Published material:** none needed.
- **Code we already ship:** `alacritty_terminal::marley_hooks::HookPosition` (the cursor's
  absolute line, as `Terminal::input` takes it, #484); `clear_saved_screen` (`alacritty.rs`), which
  counts the cleared history as evicted and moves the cursor's line to row 0; Zed's
  `INIT_COMMAND_STARTUP_MARKER_PREFIX`.

## UI proof
`script/e2e/639-the-startup-handshake-shows-as-a-block.sh`, `compositor sway`: bash with
`PS1='$ '`, a stand-in `claude` first on the PATH. Shots: `639-01-cleared`, `639-02-after-clear`,
`639-03-agent`.

## Locked-In Decisions
- D1 — The startup check is recognized by Zed's marker in the command (`__zed_init_command_ready_`),
  which only Zed types; Marley keeps a copy of the prefix, and the touchpoint row says to follow
  upstream if it changes.
- D2 — A clear empties the blocks it wiped rather than removing them: block indices are ids
  across Marley (bookmarks, the rail, agents' blocks), so none moves.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the terminal is cleared with Zed's Clear, the system shall draw no block over the cleared rows | shot `639-01-cleared` |
| REQ-002 | WHEN a command runs after the clear, its block shall have its own header and output, and no earlier block shall show | shot `639-02-after-clear` |
| REQ-003 | WHEN Marley starts an agent CLI in a new terminal, the terminal's first block shall be the agent's command, with no block for Zed's startup check | shot `639-03-agent` |
| REQ-004 | WHEN a restored terminal resumes a Claude Code session (#540), the same shall hold | review of the diff (the same `write_init_command_after_startup` path as REQ-003) |
| REQ-005 | WHILE a command runs, a clear shall leave its block going on from the top row | review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the touchpoint row; `AnchoredBlocks`; the two call sites; a review; the gate.
- **P3 Test** — the visual check.
- **P4 Complete** — CHANGELOG, `terminal_blocks.md`, ledger, close, archive, commit.
