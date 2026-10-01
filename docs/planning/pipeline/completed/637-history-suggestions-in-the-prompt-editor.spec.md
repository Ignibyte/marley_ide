---
pipeline_id: 454f648f-6194-45cd-8c42-3deeb8ca5477
ticket: docs/planning/tickets/closed/TICKET-637-history-suggestions-in-the-prompt-editor.md
status: Phase 4 — Complete PASS
title: "History suggestions in the prompt editor"
type: feature
slice: prong 1 T3
references: [docs/marley/three-prong-plan.md]
---

## Title
#484's history suggestion in the shell's prompt editor. Since #627 the editor holds the line at
every prompt by default, and it draws no suggestion, so at the default one never shows. #627 left
the editor's version to a follow-up that was never minted; #635's golden run found the gap.

## Scope
### In
- `crates/marley_workbench`: the shell's prompt editor draws the rest of the newest history command
  its text starts (#484's lookup and source: the session's verified commands, then the history
  file) after its text, as the inlay #573's hint already uses, while the cursor is at the end of
  the text with nothing selected.
- → at the end of the editor's text takes the suggestion; anywhere else, or with none shown, → is
  the editor's own cursor move.
- The history suggestion wins over #557's hint and #573's reading, as on the grid (#557).

### Out (explicitly deferred)
- The grid's suggestion with `marley.prompt_editor` off: unchanged (#484).
- Taking a suggestion one word at a time (fish's Alt-→).
- Suggestions in an agent's prompt editor: the history is the shell's.

## Reference (§20)
- **Warp (behavior):** autosuggest text drawn as the input editor's placeholder after the typed
  text (`docs/warp_architecture/subsystems/02-editor-and-text.md`, line 180), as #484 cited.
- **Upstream Zed:** the editor's edit-prediction inlay (`Inlay::edit_prediction`, `splice_inlays`)
  for the dimmed text, and gpui's key dispatch, which tries the next binding for a keystroke when
  an action handler propagates (`crates/gpui/src/window.rs`, the `match_result.bindings` loop), so
  → falls through to `editor::MoveRight`.

### Prior art
- **Behavior maps:** #484's spec and AD-claude-484 (the lookup, the history order); #573's design
  (the editor's hint inlay with a reserved id, `paint_hint`); #627's Scope Out naming this ticket.
- **Published material:** fish's autosuggestions (shown while the cursor is at the end of the line,
  → takes them), the behavior #484 follows.
- **Code we already ship:** `marley_terminal::suggestion` and `autosuggest::history` own the
  lookup; Zed's `editor` owns the inlay and `Editor::insert`; gpui owns the fall-through. Nothing
  new is needed beyond wiring them into the shell's editor.

## UI proof
`script/e2e/637-history-suggestions-in-the-prompt-editor.sh`, under `compositor sway` (a click
gives the terminal the focus, and the editor docks): bash with `PS1='$ '` and a history file of
`git status`, `echo hello world`, `ls -la`, the prompt editor at its default. Shots `637-01-ghost`,
`637-02-taken`, `637-03-ran`, `637-04-nothing`, `637-05-left`, `637-06-right`,
`637-07-session-first`, and the MCP check of the commands that ran.

## Locked-In Decisions
- D1 — The suggestion is the same lookup as the grid's (`marley_terminal::suggestion` over
  `autosuggest::history`), fed the editor's whole text.
- D2 — It shows only while the editor has one cursor, at the end of its text, with nothing
  selected, as #484 shows one only while nothing follows the cursor.
- D3 — One inlay: the editor's hint inlay (#573's reserved id) carries the history suggestion when
  there is one, else #557's hint or #573's reading.
- D4 — → is `marley::AcceptSuggestion` in `MarleyShellInput > Editor`; the editor's footer handles
  it, and without a suggestion it propagates so the next binding, `editor::MoveRight`, runs. The
  workspace's grid handler passes the action on while the shell's editor holds the line, so it
  never types into the grid behind the editor.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the shell's prompt editor holds a prefix of a history command with the cursor at its end, the system shall show the rest of the newest such command dimmed after the text | shot `637-01-ghost` |
| REQ-002 | WHEN → is pressed while that suggestion shows, the system shall add the rest to the editor's text | shot `637-02-taken` |
| REQ-003 | WHEN Enter is pressed after the suggestion was taken, the system shall run the completed command | shot `637-03-ran`; MCP `ran "echo hello world"` |
| REQ-004 | WHILE no history command starts with the editor's text, the system shall show no suggestion | shot `637-04-nothing` |
| REQ-005 | WHILE the cursor is before the end of the editor's text, the system shall show no suggestion, and → shall move the cursor without taking one | shots `637-05-left`, `637-06-right` |
| REQ-006 | WHEN → is pressed in the editor with no suggestion, the system shall move the cursor as the editor does and send nothing to the terminal | shots `637-04-nothing`, `637-05-left`, `637-06-right` (the grid line stays empty); review |
| REQ-007 | WHEN a command run in the session and a history file command both start with the text, the system shall suggest the session's | shot `637-07-session-first` |
| REQ-008 | WHERE both a history suggestion and #557's hint apply, the system shall show the history suggestion | review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `rich_input.rs`'s hint takes the history suggestion first and repaints on cursor
  moves; the footer's AcceptSuggestion; the grid handler's guard; the keymap; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
