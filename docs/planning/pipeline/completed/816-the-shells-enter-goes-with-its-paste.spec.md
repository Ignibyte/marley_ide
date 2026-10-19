---
pipeline_id: c1dd46f0-344a-4a7b-b0d5-324c42cd6102
ticket: docs/planning/tickets/closed/TICKET-816-keys-typed-after-enter-reach-the-shell-before-it.md
status: Phase 4 — Complete PASS
title: The shell's Enter goes with its paste
type: bug
slice: the block terminal's prompt editor (#624, #627)
references: []
---

## Title
Return in a shell's prompt editor pastes the line, sends the `\r` 200 ms later through
`terminal_drive::paste_then`, and gives the terminal the keys at once. Whatever is typed in those
200 ms, or waits in the window's queue behind the Return, reaches the shell before its Enter, so
`cd x`, Enter, `ls` typed fast runs `cd xls`. The shell's Enter should go in the same write as
the paste, ahead of anything typed after it.

## Scope
### In
- `rich_input::send` for `Target::Shell`: the paste and the `\r` in one terminal update.
- Found in Code: keys typed while that command runs reach the shell, which may echo them only
  after the editor reopened at its next prompt, empty, so the next Enter met an empty editor and
  only closed it (zsh, `816-02-zsh` of the first run: `$ here` left unrun). The editor now
  records whether it took the shell's line when it opened (`Prompt::took_line`); when it did not,
  Enter sends the line the shell shows first, then the editor's text.

### Out (explicitly deferred)
- `Target::Agent` and the other `paste_then` callers (`terminal_type`, review notes, delegation,
  resume, limits, send selection): an agent CLI may read a bracketed paste with the input that
  waits behind it (F-claude-594), so their Enter keeps its 200 ms. Keys typed in that window can
  still reach an agent before its Enter; filed as #817.

## Reference (§20)
Warp (observed behavior, `docs/warp_architecture/subsystems/00-overview.md` §3, "SUBMIT → WRITE
TO THE SESSION"): on Enter the input editor's text goes to `PtyController::write_command` and
through one queue of pty writes, so the command and its newline leave together and nothing typed
after can come between them. Marley matches that for the shell's editor: the paste and its `\r`
in one terminal update. Zed's terminal has no editor in front of the prompt.

### Prior art
- F-claude-594-an-enter-sent-with-a-paste-was-read-as-part-of-it-001 and
  PR-claude-keys-after-a-paste-go-in-a-later-write-001: the 200 ms exists for Python's REPL (3.13
  and later), which drains waiting input with a bracketed paste. The shell editor only opens at a
  prompt Marley's own bash and zsh hooks report (#624), and both shells' readers (readline's
  bracketed-paste-begin, zle's bracketed-paste widget) read a paste only up to its end marker
  `\e[201~`, so a `\r` after it is the next key, accept-line.
- `Terminal::paste` (`crates/terminal/src/terminal.rs`) brackets the text when the program asked
  (`Modes::BRACKETED_PASTE`), else turns line breaks into `\r`; `Terminal::input` writes in
  order to the pty's channel, so one update's paste and `\r` keep their order.
- bash 5.3 and zsh 5.9 on the box turn bracketed paste on by default.

## UI proof
`script/e2e/816-the-shells-enter-goes-with-its-paste.sh`, under sway: one quick burst of keys
types `cd sub`, Enter, `here`, Enter into the shell's prompt editor, `here` logging the folder it
runs in; first in bash, then in a zsh terminal.
- `816-01-bash`: two blocks, `cd sub` then `here` with `…/repo/sub`; the log's line names `sub`
  (REQ-001).
- `816-02-zsh`: the same in zsh with `sub2` (REQ-002).
- 561-browser-env-opener, the golden scenario that found it, passes (REQ-003).

## Locked-In Decisions
- D1 — Only the shell's editor drops the wait: a shell at its prompt reads a paste to its marker,
  and nothing then sits between the line and its Enter.
- D2 — No input hold at the terminal: a hold that a dropped task never releases would freeze the
  terminal, and the agents' case is its own ticket.
- D3 — A line the shell shows at Enter counts as typed ahead only when the editor opened without
  taking one: an editor that took the line holds it, and the person may have edited it there.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user sends a command from the shell's prompt editor in bash and types the next one at once, each shall run alone, in order. | Shot `816-01-bash`; `expect` on the log |
| REQ-002 | WHEN the same is done in zsh, each command shall run alone, in order. | Shot `816-02-zsh`; `expect` on the log |
| REQ-003 | WHEN 561-browser-env-opener runs, its `cd` and the command after it shall run as two commands. | `script/regress` line for 561 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `send`'s shell branch; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the scenario on the debug build and on the installed build without the fix; 561.
- **P4 Complete** — CHANGELOG, the crate note, the ledger (a PR beside PR-claude-keys-after-a-
  paste…), close, archive, commit; #817 filed.
