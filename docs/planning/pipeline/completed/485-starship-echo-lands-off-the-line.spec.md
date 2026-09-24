---
pipeline_id: 4d66b060-0492-4286-a8ce-7ee81c6ce248
ticket: docs/planning/tickets/closed/TICKET-485-starship-echo-lands-off-the-line.md
status: Phase 4 — Complete PASS
title: A new terminal opens at the last terminal's size
type: bug
slice: prong 1 T0
references: [docs/planning/pipeline/completed/483-e2e-visualization-tests.notes.md]
---

## Title
Typed text after the first character went missing on a long multi-line prompt, because the shell
started at Zed's 100 × 6 debug size: a new terminal now opens at the size the last terminal view
laid out.

## Scope
### In
- `crates/terminal/src/terminal.rs`: the size `set_size` last gave a PTY terminal, remembered for
  the process; `TerminalBuilder::new` opens the grid, the PTY and the content at it, and at the
  debug size only before any terminal has one.
- `script/e2e/485-long-prompt.sh`.

### Out (explicitly deferred)
- The first terminals of a launch, which open before any view has a size: TICKET-486 keeps the
  last size across launches.
- Readline itself: its multi-line redraw after a resize restores the layout it made for the old
  width.

## Reference (§20)
- **Upstream Zed:** `TerminalBuilder::new` opens every PTY at `TerminalBounds::default()`
  (`DEBUG_TERMINAL_WIDTH` 500 px over `DEBUG_CELL_WIDTH` 5 px is 100 columns, 30 px over 5 px is
  6 lines) and the view resizes it at its first layout. Marley keeps that resize and changes only
  the size it opens at.

### Prior art
- **Published material:** readline's `_rl_redisplay_after_sigwinch` redraws only the last line
  of a multi-line prompt through `redraw_prompt`, which expands that line, draws, and then
  `rl_restore_prompt`s the layout made when the line began; the next keystrokes are drawn with
  it.
- **Code we already ship:** `Terminal::set_size` already knows each real size; `TerminalBounds`
  is `Copy`; the crate depends on `parking_lot`.

## UI proof
UI-AFFECTING: what a terminal shows as you type. Proven by `script/e2e/485-long-prompt.sh`: a
bash whose `.bashrc` runs `bind` and sets a two-line prompt whose second line is 124 colored
columns and a branch, as starship draws one. Shots of a line typed in the first terminal of the
launch and in a terminal opened from the command palette, before the fix and after it.

## Locked-In Decisions
- D1 — Open at the last real size; keep the view's first resize, which corrects the rest.
- D2 — Only PTY terminals set it: a display-only terminal is an agent's card, not a pane.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a terminal opens while another has been laid out, its shell shall start at that terminal's size | e2e: the second terminal echoes a typed line whole, cursor after it |
| REQ-002 | The first terminal of a launch shall start as before, at the debug size | e2e: shot 01, the known limit (TICKET-486) |
| REQ-003 | The diff gate shall be green | `just gate-diff` |

## Phase Plan
- **P1 Plan** — the investigation in the notes: the probes, the pty log, the root cause.
- **P2 Code** — the terminal crate's hunks, with the ledger row.
- **P3 Test** — the scenario before and after the fix; the gate.
- **P4 Complete** — CHANGELOG, knowledge, close, archive, commit.
