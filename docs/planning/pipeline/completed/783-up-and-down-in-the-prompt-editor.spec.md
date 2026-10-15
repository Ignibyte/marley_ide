---
pipeline_id: bc58f23f-95b7-4330-940b-b1a1478a81e4
ticket: docs/planning/tickets/closed/TICKET-783-up-and-down-in-the-prompt-editor.md
status: Phase 4 — Complete PASS
title: "Up and Down in the prompt editor"
type: bug
slice: prong 1 T3
references: [docs/marley/three-prong-plan.md]
---

## Title
Up and Down in the shell's prompt editor (#627) and an agent's rich input (#481) reach the
terminal behind them. Zed's `Editor::move_up` and `move_down` (`crates/editor/src/navigation.rs`)
call `cx.propagate()` when the cursor cannot move: an empty editor, the start of the first line,
the end of the last. The editor sits inside `TerminalView`, so gpui tries the next binding for the
key, the Terminal context's `["terminal::SendKeystroke", "up"]`
(`assets/keymaps/default-linux.json`), and the shell recalls its history on the grid behind the
editor. Chad found it on 2026-10-14. The shell's editor should walk the shell's history itself, as
Warp's input does.

## Scope
### In
- `crates/marley_workbench`: Up and Down in `MarleyRichInput > Editor` while no completion menu
  shows go to two new Marley actions, handled by the editor's footer, and never on to the terminal.
- In the shell's editor with one cursor: Up on the first display line shows the shell's next older
  command, Down on the last display line the next newer one and, past the newest, the text the
  editor held before the walk. Anywhere else the keys move the cursor as the editor does.
- In an agent's editor: Up and Down move the cursor as the editor does, and stop there.

### Out (explicitly deferred)
- A history of prompts sent to an agent: the agent keeps its own.
- A history search (Ctrl-R) in the editor, and a walk filtered by the typed prefix (zsh's
  `up-line-or-beginning-search`).
- The grid's own Up with `marley.prompt_editor` off: the key goes to the shell as before.
- Ctrl-Up and Ctrl-Down (the block keys) from the editor.

## Reference (§20)
- **Warp (behavior):** Up in Warp's input editor, at its top line, brings back the previous
  commands; the maps under `docs/warp_architecture/` do not describe it (checked
  `subsystems/02-editor-and-text.md`, `subsystems/03-terminal-session-core.md`), so the reference
  is the behavior Chad named and readline's `previous-history` and `next-history`: Up and Down
  step through the history, the cursor at the end of the recalled line, and Down past the newest
  gives back the line being typed.
- **Upstream Zed:** `Editor::move_up` and `move_down` keep their behavior inside the text; gpui's
  key dispatch tries the next binding for a keystroke when an action propagates
  (`crates/gpui/src/window.rs`, the `match_result.bindings` loop), which is both the bug and how
  the new actions take the key first. Action listeners stop propagation by default in the bubble
  phase (`window.rs`, "Actions stop propagation by default during the bubble phase").

### Prior art
- **Behavior maps:** #637's spec used the same fall-through for → (`marley::AcceptSuggestion`
  propagating to `editor::MoveRight`); prevention rule
  `PR-claude-an-editor-inside-a-terminal-rebinds-the-terminal-keys-it-must-keep-001` (#635) names
  this class: an editor inside a terminal view meets the Terminal context's bindings.
- **Published material:** GNU readline's `previous-history` / `next-history`; Warp's docs on its
  input editor's command history.
- **Code we already ship:** `autosuggest::history` owns the history (the session's verified
  commands, then the history file, newest first, #484 and #625). Zed's `editor` owns the inline
  input's history walk (`inline_input.rs`, `cycle_inline_input_history`), but only for an inline
  input block inside another editor, so it is the model for the walk (a draft kept, an edit ends
  the walk) and not a seam to call. `Editor::move_up`, `move_down` and `display_snapshot` are
  public. No crate owns a history walk for a free-standing editor.

## UI proof
`script/e2e/783-up-and-down-in-the-prompt-editor.sh`, under `compositor sway` (a click gives the
terminal the focus, and the editor docks): bash with `PS1='$ '` and a history file of
`git status`, `ls -la`, `echo hello world`, `ls -la`, the prompt editor at its default; then a
stand-in agent (`exec -a claude`, as #481's). Shots `783-01-up` to `783-11-agent`, and the MCP
checks of the command that ran and the line the stand-in got.

## Locked-In Decisions
- D1 — Two actions, `marley::RichInputUp` and `marley::RichInputDown`, bound to `up` and `down` in
  `MarleyRichInput > Editor && !showing_completions`, so Zed's completion menu keeps the keys while
  it shows. The footer handles them; the handler never propagates.
- D2 — The shell's editor walks only with one cursor, on the first display line for Up and the
  last for Down, so a soft-wrapped or Shift-Enter line still moves the cursor first. Everywhere
  else the handler calls `Editor::move_up` or `move_down` and stops the propagation they ask for.
- D3 — The walk's commands are `autosuggest::history` at the first Up, blank ones and repeats of a
  newer one left out, so Up never shows the same command twice in a row.
- D4 — The walk keeps the text the editor held at its first Up and gives it back on Down past the
  newest. An edit that leaves the text other than the recalled command ends the walk, as Zed's
  inline input does; the next Up starts from the newest with the edited text as the draft.
- D5 — A recalled command goes in with the cursor at its end, as readline does.
- D6 — No Zed crate changes.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Up is pressed in the shell's prompt editor with its one cursor on the first line, the system shall put the newest history command in the editor with the cursor at its end, and send nothing to the terminal | shot `783-01-up` (the grid's prompt line stays empty) |
| REQ-002 | WHEN Up is pressed again during the walk, the system shall show the next older command, leaving out a command the walk already showed | shots `783-02-older`, `783-03-oldest` |
| REQ-003 | WHEN Up is pressed at the oldest command, the system shall keep it and send nothing to the terminal | shot `783-04-stays` |
| REQ-004 | WHEN Down is pressed on the last line during the walk, the system shall show the next newer command, and past the newest the text the editor held before the walk | shot `783-05-draft` |
| REQ-005 | WHEN Down is pressed on the last line with no walk, the system shall leave the editor as it is and send nothing to the terminal | shot `783-06-down-alone` |
| REQ-006 | WHEN Enter is pressed on a recalled command, the system shall run it | shot `783-07-ran`; MCP `ran "echo hello world"` |
| REQ-007 | WHILE the cursor is below the editor's first line, Up shall move it up a line without changing the text | shot `783-08-line-up` |
| REQ-008 | WHEN a recalled command is edited, the system shall end the walk, and the next Up shall start from the newest command with the edited text kept for Down | shots `783-09-walk-again`, `783-10-edited-back` |
| REQ-009 | WHEN Up or Down is pressed in an agent's rich input, the system shall move the cursor as the editor does and send nothing to the agent | shot `783-11-agent`; MCP: the stand-in got exactly the line sent |
| REQ-010 | WHILE the completion menu shows in the editor, Up and Down shall move in the menu | review of the diff (the keymap's `!showing_completions`) |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the two actions, the keymap rows, the walk in `rich_input.rs`; a review of the
  diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
