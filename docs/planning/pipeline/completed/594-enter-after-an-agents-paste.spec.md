---
pipeline_id: 8eeb9128-3e32-4cf1-80bc-930697cc4d85
ticket: docs/planning/tickets/closed/TICKET-594-enter-after-an-agents-paste.md
status: Phase 4 — Complete PASS
title: Press an agent's Enter after its paste has landed
type: bug
slice: T-series, agent terminal writes and the prompt paths (#525, #481, #522)
references: [docs/planning/pipeline/completed/525-agent-drives-a-running-program.spec.md]
---

## Title
Marley pastes text into a terminal program and then presses Enter in three places: an agent's
`terminal_type` (#525), the rich input's Enter (#481) and review notes sent to an agent (#522).
Each sends the Enter in the same burst as the paste, and a program that reads a bracketed paste
together with whatever came with it, as Python's REPL does, takes the Enter as a new line of the
paste. Nothing runs. Keys and Enter now follow the paste after a short pause.

## Scope
### In
- `terminal_drive::paste_then`: paste now, then run what follows on the terminal after
  `AFTER_PASTE`.
- `terminal_drive::write`: the keys and Enter after the pause, the call answered after them.
- `rich_input::send` and `review_notes`: their Enter after the pause.

### Out (explicitly deferred)
- A paste with nothing after it (`send_selection`, `browser`'s line, `rich_input::insert`).
- Waiting on the program's echo instead of a time: a program may not echo a paste at all.
- Typed launch lines (`marley_agent::send_payload` before the shell's prompt), which are typed,
  not pasted.

## Reference (§20)
N/A — Marley-specific: the three paths type into a program on an agent's or the user's behalf,
which Zed's terminal does not do. The behavior they must match is the program's own: CPython's
`_pyrepl` reads a bracketed paste with `getpending()` until the end marker and inserts all of it
(`perform_bracketed_paste`, Python 3.13 and later), so an Enter must come in a later read.

### Prior art
- The code we ship: `terminal::Terminal::paste` brackets the text when the program asked for it
  and writes it in one input; `Terminal::input` and `try_keystroke` write in order on the same
  channel, so nothing in Zed's terminal separates them. gpui's background executor timer is the
  pause, as elsewhere in Marley.
- Published material: CPython's `Lib/_pyrepl/commands.py`, read on this box (3.14.7). Tools that
  drive a TUI through a pty, tmux's `send-keys` scripts among them, send the Enter as its own
  write after a pause for the same reason.
- #481's F-claude-481-the-rich-inputs-check-raced-the-echo-of-its-paste-001 saw the two writes
  arrive close together; its stand-in reads lines, so the Enter still counted there.

## UI proof
The scenario `script/e2e/525-agent-types-into-a-terminal.sh` (`compositor sway`, a Python REPL
and a stand-in agent through the plugin's bridge):
- `525-03-typed`: the agent's `print(6 * 7)` with submit has run: `42` under it.
- `525-04-no-ask`: `print('again')` ran: `again`.
- `525-10-never-ask`: each allowed line ran; the denied and unanswered ones are nowhere.
- `525-11-shell-refused`: Ctrl-D left Python, and a write at the shell's prompt is refused.
- `525-13-rich-input-open` and `525-14-rich-input-ran`: Python under an agent's name (`exec -a
  claude python3 -q`); the rich input holds `print(6 * 7 + 1)`, and its Enter runs it: `43`.
- #522's scenario, `script/e2e/522-review-notes-to-the-agent.sh`, run once: the stand-in agent
  still gets the note and its Enter.

## Locked-In Decisions
- D1 — `AFTER_PASTE` is 200 ms: long enough for a program woken by the paste to read it, short
  enough that no one waits on it.
- D2 — Only what follows a paste waits; keys alone and Enter alone go at once.
- D3 — `terminal_type` answers after the Enter is sent, so an agent's next `terminal_screen`
  sees the program's reply begin.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent's `terminal_type` pastes text with `submit` into Python's REPL, the REPL shall run the line. | Shots 525-03, 525-04 |
| REQ-002 | WHEN a `terminal_type` write pastes text and names keys, the keys shall reach the program after the paste as keys. | Review of the diff |
| REQ-003 | WHEN the rich input's Enter sends text to a program that reads bracketed pastes, the program shall run the line. | Shots 525-13, 525-14 |
| REQ-004 | WHEN review notes are sent to an idle agent, the agent shall get the notes and then Enter. | #522's scenario, its shots |
| REQ-005 | WHEN a `terminal_type` call is answered, its keys and Enter shall already be sent. | Review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `paste_then` and its three callers; a review of the diff; `script/gates.sh
  --diff` green.
- **P3 Test** — #525's scenario with the REPL's output checked and a rich-input step; #522's
  scenario once; every shot read.
- **P4 Complete** — CHANGELOG, the crate note, the knowledge blocks, close the ticket, archive,
  commit with #525's scenario and the fixture's commands.
