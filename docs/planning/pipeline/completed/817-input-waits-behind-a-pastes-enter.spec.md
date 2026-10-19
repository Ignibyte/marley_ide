---
pipeline_id: 6ee15e18-a099-4ff4-8ee9-43b55f068421
ticket: docs/planning/tickets/closed/TICKET-817-keys-typed-after-enter-reach-an-agent-before-it.md
status: Phase 4 — Complete PASS
title: Input waits behind a paste's Enter
type: bug
slice: the block terminal's prompt editor (#624, #627) and Marley's terminal writes (#594)
references: []
---

## Title
`terminal_drive::paste_then` pastes, waits 200 ms (`AFTER_PASTE`, which an agent CLI or Python's
REPL needs, F-claude-594), then sends its Enter or keys. Meanwhile the terminal takes input as
usual, so keys typed after Enter in an agent's prompt editor, or queued in the window behind the
Return, reach the agent ahead of that Enter and join the prompt just sent. While a paste's Enter
is owed, the terminal's other input should wait behind it.

## Scope
### In
- `terminal::Terminal` (a Zed crate) gains a queue for its input: while a hold Marley set is
  active, `input` keeps the bytes; the owed write releases it, writing itself first and then what
  waited. The hold also ends at a deadline, so a write that never comes frees the terminal.
- `terminal_drive::paste_then` sets the hold after its paste and releases it with its `after`,
  for every caller: an agent's editor (`rich_input::send`), `terminal_type`, review notes,
  delegation, resume, limits, send selection.

### Out (explicitly deferred)
- Focus and mouse reports, which Zed writes to the pty directly, not typed input.
- Two sends within one 200 ms window: the second paste waits in the queue, behind the first's
  Enter, and its own Enter comes 200 ms after it was asked for, which may be sooner than 200 ms
  after the queue lets the paste out.

## Reference (§20)
Warp (observed behavior, `docs/warp_architecture/subsystems/00-overview.md` §3, "SUBMIT → WRITE
TO THE SESSION"): writes to the session go through one queue (`PtyWrite`), in the order they
were asked for, so a submitted command and its newline come before what is typed next. Marley
keeps that order for its delayed Enter by queuing the terminal's input behind it. Zed's terminal
writes input straight to the pty's channel and has no delayed write of its own.

### Prior art
- `crates/terminal/src/terminal.rs`: `input` (Marley's `marley_input_held` refusal for a remote
  link that is down, #641, then `write_input`); `paste` and `try_keystroke` reach the pty through
  `input`; focus reports (`\e[I`, `\e[O`) and mouse reports call `write_to_pty` directly.
- `crates/marley_workbench/src/terminal_drive.rs`: `paste_then` and `AFTER_PASTE`; its callers
  detach or await the task, so its release runs unless the terminal is gone.
- #816: the shell's editor writes its paste and Enter together and needs no hold.
- F-claude-594 and PR-claude-keys-after-a-paste-go-in-a-later-write-001: the wait itself stays.

## UI proof
`script/e2e/817-input-waits-behind-a-pastes-enter.sh`, under sway: a stand-in `claude` on the
terminal's PATH logs each line it reads (`got: <line>`).
- `817-01-agent`: Ctrl-G opens the agent's editor; one burst types `first`, Return, `second`,
  Return. The stand-in shows and logs `got: first`, then `got: second` (REQ-001, REQ-002).
- The same scenario on the installed build (no fix) logs `got: firstsecond` and an empty line.
- #816's scenario still passes (the shell's editor, REQ-003).

## Locked-In Decisions
- D1 — The queue lives in the terminal, where every typed key and Marley's own writes pass
  (`input`), not in each Marley path that could type.
- D2 — The hold ends at its release or at a deadline, whichever comes first: a dropped task or a
  lost wakeup cannot leave a terminal that types nothing. The deadline is ten times
  `AFTER_PASTE`.
- D3 — The release writes the owed bytes before the queued ones, in one update.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user sends a prompt from an agent's editor and types the next line at once, the agent shall read the prompt as a line of its own. | Shot `817-01-agent`; `expect` on the stand-in's log |
| REQ-002 | WHEN keys are typed while the prompt's Enter is owed, the agent shall read them after it, in the order typed. | Shot `817-01-agent`; `expect` |
| REQ-003 | WHEN the shell's editor sends a command, it shall run as #816 made it, the next command alone after it. | `just e2e` of #816's scenario |
| REQ-004 | WHEN a hold is never released, the terminal shall take input again once its deadline passes. | The review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the terminal's queue (a ledger row), `paste_then`'s hold and release; a review of
  the diff; `script/gates.sh --diff` green.
- **P3 Test** — the scenario on the debug build and on the installed build without the fix;
  #816's scenario.
- **P4 Complete** — CHANGELOG, the crate note, the ledger, close, archive, commit.
