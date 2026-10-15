---
pipeline_id: a4c05391-a213-400e-8d8c-92d87634629a
ticket: docs/planning/tickets/closed/TICKET-784-terminal-keys-from-the-prompt-editor.md
status: Phase 4 — Complete PASS
title: "Keys only the terminal binds, pressed in the prompt editor"
type: bug
slice: prong 1 T3
references: [docs/marley/three-prong-plan.md]
---

## Title
Keys pressed in the shell's prompt editor (#627) or an agent's rich input (#481) that the editor
binds nothing for go to the next binding: the Terminal context's, which sends them to the program
behind the editor, or an unscoped one of Zed's. Reproduced on 2026-10-15 against the build of
#783: in both editors Ctrl-W, Ctrl-E, Ctrl-B, Ctrl-R, Alt-B and Alt-F reach the program,
Ctrl-Shift-V pastes the clipboard into the program, Ctrl-Shift-C copies nothing of the editor's
selection, and Ctrl-Q quits Marley at once, even with an agent at work. Chad approved the
behavior on 2026-10-15: "When you press a shell key like Ctrl-R, Ctrl-W or Alt-B while typing in
the prompt editor or an agent's input box, it works on the line you're typing (or does nothing)
instead of reaching the shell behind it."

## Scope
### In
- `crates/marley_workbench`: in `MarleyRichInput > Editor`, both editors, Ctrl-W, Ctrl-E,
  Ctrl-B, Alt-B and Alt-F act on the editor's line as readline acts on the shell's; Ctrl-R,
  Ctrl-O and Ctrl-Q do nothing; Ctrl-Shift-V pastes into the editor; Ctrl-Shift-C copies the
  editor's selection, or the grid's when the editor has none.

### Out (explicitly deferred)
- A history search on Ctrl-R in the editor, as Warp's input has.
- Readline keys Zed's editor binds its own way (Ctrl-A selects all, Ctrl-K starts a chord,
  Ctrl-U, Ctrl-D, Ctrl-L, Alt-.): none reaches the program.
- The Terminal context's keys bound to Zed's or Marley's own actions (search, Rerun Task, Split
  Right, the block keys, bookmarks, Filter, Ctrl-G, Ctrl-Shift-A): none sends the program anything.
- Ctrl-V, Zed's Paste in the editor, with an image on the clipboard.
- The grid's keys while no editor is docked.

## Reference (§20)
- **Readline (behavior):** GNU readline's emacs keys, which a shell's line answers: Ctrl-W
  `unix-word-rubout`, Ctrl-E `end-of-line`, Ctrl-B `backward-char`, Alt-B `backward-word`, Alt-F
  `forward-word`. The editor takes the same meanings through Zed's own actions.
- **Warp (behavior):** Warp's input editor keeps every key it is given; nothing typed there
  reaches the shell until the line runs. `docs/warp_architecture/subsystems/02-editor-and-text.md`
  describes the input as an editor of its own, which this follows.
- **Upstream Zed:** gpui ranks a binding with no context at the deepest depth
  (`Keymap::binding_enabled`, `crates/gpui/src/keymap.rs`), so inside a terminal view's child an
  unscoped binding (`ctrl-q` `zed::Quit`, `ctrl-o` `workspace::OpenFiles`) outranks the Terminal
  context's override; a `null` binding at the same depth and source, later in the keymap,
  suppresses it (`resolve_bindings_for_input`). An action listener that calls `cx.propagate()`
  hands the key to the next binding (`window.rs`, the `match_result.bindings` loop).

### Prior art
- **Behavior maps:** #635 met the unscoped half (Ctrl-Shift-W closed the window,
  `PR-claude-an-editor-inside-a-terminal-rebinds-the-terminal-keys-it-must-keep-001`); #783 met
  the propagating half and filed this sweep
  (`PR-claude-783-an-editor-in-a-terminal-keeps-the-keys-its-actions-pass-on-001`).
  `L-claude-491-ctrl-q-in-a-terminal-goes-to-the-shell-001` is the grid's side of Ctrl-Q.
- **Published material:** the GNU readline manual's "Commands For Moving" and "Commands For
  Killing Text".
- **Code we already ship:** Zed's editor owns every move and deletion needed
  (`editor::DeleteToPreviousWordStart`, `MoveToEndOfLine`, `MoveLeft`, `MoveToPreviousWordStart`,
  `MoveToNextWordEnd`, as Zed binds Ctrl-Backspace, End, Left, Ctrl-Left and Ctrl-Right),
  `Editor::paste` is public, and the copy runs as the editor's own `editor::Copy` action. Zed's
  terminal view owns the grid's copy and the image paste (`TerminalView::copy`, `paste`,
  `forward_ctrl_v`). Marley's keymap already uses `null` (`MarleyBrowser`'s Escape). No crate owns
  a choice between an editor's selection and a terminal's.

## UI proof
`script/e2e/784-terminal-keys-from-the-prompt-editor.sh`, under `compositor sway` (a click gives
the terminal the focus, and the editor docks). Bash with `PS1='$ '`, `stty -ixon`, and a `bind
-x` for each key that prints `LEAK <key>` on the grid when the shell gets it; then a stand-in
agent (`exec -a claude`) that reads raw bytes and prints each one. Shots `784-01-moves` to
`784-08-still-running`, and checks through Marley's MCP server (`terminal-screen`) and the
clipboard (`wl-paste`).

## Locked-In Decisions
- D1 — The keys are bound in `MarleyRichInput > Editor`, so both editors answer them alike:
  Ctrl-W `editor::DeleteToPreviousWordStart` (the arguments Zed gives Ctrl-Backspace), Ctrl-E
  `editor::MoveToEndOfLine` (End's), Ctrl-B `editor::MoveLeft`, Alt-B
  `editor::MoveToPreviousWordStart`, Alt-F `editor::MoveToNextWordEnd` (Ctrl-Left's and
  Ctrl-Right's).
- D2 — Ctrl-R, Ctrl-O and Ctrl-Q are bound to `null` there: nothing happens. Ctrl-Q no longer
  quits Marley from a terminal's editor, as it does not from the grid.
- D3 — Ctrl-Shift-V is `marley::RichInputPaste`: the clipboard's text goes into the editor at the
  cursor (`Editor::paste`). In an agent's editor an image on the clipboard goes on to the
  terminal's own paste, so the agent gets Ctrl-V and attaches it, as from the grid; in the shell's
  editor an image does nothing.
- D4 — Ctrl-Shift-C is `marley::RichInputCopy`: with text selected in the editor, the editor's
  own `editor::Copy`; with none, the key goes on to the terminal's `terminal::Copy`, which copies
  the grid's selection, so copying output at a prompt works as before.
- D5 — No Zed crate changes.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Ctrl-E, Ctrl-B, Alt-B or Alt-F is pressed in the shell's prompt editor, the system shall move the editor's cursor to the line's end, back a character, back a word or forward to a word's end, and send nothing to the shell | shot `784-01-moves` (the letters typed after each move sit where the move put the cursor); no `LEAK` on the grid |
| REQ-002 | WHEN Ctrl-W is pressed in the shell's prompt editor, the system shall delete the word before the cursor and send nothing to the shell | shot `784-02-ctrl-w`; no `LEAK` |
| REQ-003 | WHEN Ctrl-R, Ctrl-O or Ctrl-Q is pressed in the shell's prompt editor, the system shall leave the editor's text as it is, send nothing to the shell, and keep Marley running | shot `784-03-nothing`; no `LEAK`; shot `784-08-still-running` after Ctrl-Q in each editor |
| REQ-004 | WHEN Ctrl-Shift-V is pressed in the shell's prompt editor with text on the clipboard, the system shall insert the text at the editor's cursor and send nothing to the shell | shot `784-04-paste`; the grid's prompt line empty |
| REQ-005 | WHEN Ctrl-Shift-C is pressed in a prompt editor with text selected, the system shall put the selected text on the clipboard | `wl-paste` holds the selection |
| REQ-006 | WHEN Ctrl-Shift-C is pressed in a prompt editor with nothing selected, the system shall copy the grid's selection as the terminal does | review of the diff (`cx.propagate()` to `terminal::Copy`) |
| REQ-007 | WHEN any of these keys is pressed in an agent's rich input, the system shall act on the editor as in the shell's editor and send nothing to the agent | shot `784-06-agent`; the stand-in printed nothing before the line sent |
| REQ-008 | WHEN Ctrl-Shift-V is pressed in an agent's rich input with an image on the clipboard, the system shall send the agent Ctrl-V, as the terminal's paste does | shot `784-07-image`; the stand-in got `$'\026'` |
| REQ-009 | WHEN Enter is pressed in an agent's rich input after these keys, the system shall send the agent exactly the editor's text | the stand-in's bytes are the text and a carriage return |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the keymap rows, the two actions and their handlers in `rich_input.rs`; a review
  of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
