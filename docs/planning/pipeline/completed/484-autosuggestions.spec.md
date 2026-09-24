---
pipeline_id: 18c75ef4-9f7a-4fb2-ac9f-267848229a73
ticket: docs/planning/tickets/closed/TICKET-484-autosuggestions.md
status: Phase 4 — Complete PASS
title: Autosuggestions from history, accepted with →
type: feature
slice: prong 1 T3a
references: [docs/marley/three-prong-plan.md]
---

## Title
As you type a command at the shell's prompt, the rest of the newest command in history that
starts with what you typed shows dimmed after the cursor, and → takes it.

## Scope
### In
- While the shell waits at its prompt (a `precmd` hook and no `preexec` since), the terminal
  reads what was typed there: the cells from where the first key after the prompt was typed up
  to the cursor, on the cursor's line, with nothing after the cursor.
- History, newest first: the commands this terminal ran (its blocks whose command the shell's
  own hook reported), then the shell's history file, which the bash and zsh integrations name
  in their `init` frame (`histfile=`), read once per file off the main thread.
- The ghost text: the rest of the newest match, drawn in a dim color from the cursor on.
- `marley::AcceptSuggestion` on → in `Terminal`: with a suggestion, it types the rest into the
  terminal; without one, the key goes on to the program.

### Out (explicitly deferred)
- A command that wraps past its first line; a prompt that moved after the first key (Ctrl-L, a
  completion list); vi mode.
- Other terminals' commands before their shells write their history.
- Accepting a word at a time; suggestions from anything but history (completions, T6).
- fish, which draws its own.

## Reference (§20)
- **Warp:** autosuggestions as Chad uses them: a command from history, as you type, in grey, and
  → takes it. The behavior maps name the terminal's ghost text
  (`docs/warp_architecture/subsystems/00-overview.md`, 03 Terminal / Blocks) and the editor's
  `Placeholder` for autosuggest text (`docs/warp_architecture/subsystems/02-editor-and-text.md`);
  the plan's T3 names "history ghost text". No Warp code.
- **Upstream Zed:** none in the terminal; the terminal element's IME marked-text painting is the
  model for text drawn at the cursor.

### Prior art
- **Published material:** fish's autosuggestions and zsh-autosuggestions: the newest history
  entry with the typed prefix, grey, → or End accepts.
- **Code we already ship:** `AnchoredBlocks` stages a prompt at `precmd` and takes it at
  `preexec`, and keeps each block's command with whether the shell's own hook reported it;
  `Terminal::input` carries every key the user types; the element's marked-text path shapes and
  paints text at `ime_cursor_bounds`; `marley::RichInput` already binds a key in `Terminal` that
  lets it through when it has nothing to do.

## UI proof
UI-AFFECTING: text drawn in the terminal, a key. Proven by `script/e2e/484-autosuggestions.sh`: a
bash with a plain prompt and a history file of its own. Shots of the ghost text as a prefix is
typed, of → taking it, of the command running, of a prefix nothing matches, and of a command run
in the session suggested from the terminal's own blocks.

## Locked-In Decisions
- D1 — The terminal reads the typed text from its own cells; no change to the user's prompt.
- D2 — Where the command starts is where the first key after the prompt was typed.
- D3 — → takes the whole suggestion; otherwise → is the program's.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the shell waits at its prompt with typed text that starts a longer history command, the terminal shall show the rest of the newest one, dimmed, after the cursor | e2e: `ech` shows `o hello world` dimmed |
| REQ-002 | WHEN → is pressed with a suggestion shown, the terminal shall type the suggestion, and without one, → shall reach the program | e2e: the line reads `echo hello world`; with no suggestion the shot is unchanged |
| REQ-003 | WHERE no history command starts with the typed text, no ghost text shall show | e2e: `zzz` |
| REQ-004 | The commands the terminal ran in the session shall be suggested before the history file's | e2e: a command run in the session is suggested over an older one from the file |
| REQ-005 | The diff gate shall be green | `just gate-diff` |

## Phase Plan
- **P1 Plan** — this spec; the design in the notes.
- **P2 Code** — `marley_terminal` (the typed-text start, the history parse and lookup, the `init`
  field), the shells' `init` frames, the terminal crate's input hunk, `terminal_view`'s hook and
  the element's painting, `marley_workbench` (the hook, the history files, the action), the
  keymap; ledger rows first.
- **P3 Test** — the e2e scenario and its shots; the gate.
- **P4 Complete** — docs, ledger, close, archive, commit.
