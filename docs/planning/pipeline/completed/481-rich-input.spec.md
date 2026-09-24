---
pipeline_id: f6e302f0-9cd0-464e-85f1-d1d6ec50a77a
ticket: docs/planning/tickets/closed/TICKET-481-rich-input.md
status: Phase 4 — Complete PASS
title: "Rich input: a Zed editor for an agent's prompt"
type: feature
slice: prong 1 T7e
references: [docs/marley/three-prong-plan.md, docs/planning/pipeline/queued/477-agent-bar.spec.md]
---

## Title
A Zed editor docked at the bottom of a terminal running a CLI agent, whose text is sent to the
agent as its prompt.

## Scope
### In
- While a CLI agent runs in the foreground, Ctrl-G or the bar's Rich Input button opens a Zed
  `Editor` docked below the grid, above the agent bar, and moves the focus to it. The agent's
  own prompt stays as it is.
- Enter sends the text to the agent (`Terminal::paste`, bracketed while the agent asked for it,
  as Claude Code does, then a carriage return), closes the editor and gives the terminal the
  focus back. Shift-Enter adds a line. Escape closes it without sending and keeps the draft for
  the next open.
- Soft wrap, one to eight lines tall; Zed's own editing (selection with the mouse, undo, word
  motion, Vim mode when it is on).
- Without an agent in the foreground, Ctrl-G reaches the program as before.

### Out (explicitly deferred)
- `@` mentions of files and symbols (with `agent_ui`'s file search); images (Ctrl-V still
  reaches Claude Code, which reads the clipboard); slash-command menus.
- The shell's prompt editor (T3), which this editor grows into.

## Reference (§20)
- **Warp:** the rich input editor (https://docs.warp.dev/agents/cli-agents/rich-input/): Ctrl-G
  or the button opens it; "Warp hides the cursor inside the CLI agent and moves focus to the
  editor input"; the prompt is submitted to the running agent; IDE-style editing and Vim keys;
  it can dismiss itself after a submission. No Warp code.
- **Upstream Zed:** the terminal inline assistant's `PromptEditor` over a terminal
  (`agent_ui`: an `Editor` in `EditorMode::AutoHeight`, placed with
  `TerminalView::set_block_below_cursor`), and `Terminal::paste`'s bracketed paste.

### Prior art
- **Published material:** Claude Code's Ctrl-G opens the prompt in `$VISUAL` or `$EDITOR`; in
  Marley's terminals the key opens the rich input instead while an agent runs. Claude Code turns
  on bracketed paste (mode 2004) and parses the paste markers (read from its 2.1.281 bundle), so
  a multi-line prompt arrives as one paste and does not submit early.
- **Code we already ship:** the inline assistant's editor, its growth on `Resized`, and its focus
  handoff; `Terminal::paste`; the #477 footer hook, which can hold the editor above the bar.

## UI proof
UI-AFFECTING: an editor over the terminal, keys, text sent to the agent. Proven by the e2e
scenario `script/e2e/481-rich-input.sh` (CONSTITUTION §7, since #483): a HOME whose `.bashrc`
runs a stand-in `claude` that prints each line it reads (`claude got: …`) until `quit`, then
leaves the shell's `$ ` prompt. Keys go to Marley's window only, so Ctrl-G, typing, Shift-Enter,
Enter and Escape are all driven; the Rich Input button is a click, shown but not clicked.

## Locked-In Decisions
- D1 — Docked below the grid, not over the cursor as the inline assistant's block: an agent's
  prompt box can be anywhere in its screen.
- D2 — One paste, then a carriage return.
- D3 — Ctrl-G is Marley's only while an agent runs; elsewhere it is the terminal's.
- D4 — Escape keeps the draft.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE an agent runs, WHEN Ctrl-G is pressed or Rich Input clicked, the terminal shall show the editor and focus it | e2e: Ctrl-G opens it, and the text typed next lands in it |
| REQ-002 | WHEN Enter is pressed in the editor, the terminal shall be sent its text as one paste and a carriage return, and the editor shall close with the terminal focused | e2e: the stand-in prints both lines; typing after it reaches the stand-in |
| REQ-003 | WHEN Shift-Enter is pressed, the editor shall add a line and send nothing | e2e: two lines in the editor, nothing printed yet |
| REQ-004 | WHEN Escape is pressed, the editor shall close without sending and show the same text when opened again | e2e: nothing printed; Ctrl-G shows the draft |
| REQ-005 | WHILE no agent runs, Ctrl-G shall reach the program as before | e2e: after `quit`, `cat -v` prints `^G` |
| REQ-006 | The diff gate shall be green | `just gate-diff` |

## Phase Plan
- **P1 Plan** — this spec; promotion re-verifies the seams (the key context, the footer hook's
  room for the editor) and asks the brain.
- **P2 Code** — the editor, its keys, the send; ledger rows for any Zed path first.
- **P3 Test** — the e2e scenario and its shots, read; the gate.
- **P4 Complete** — docs, ledger, close, archive, commit.
