# TICKET-594 — Press an agent's Enter after its paste has landed

- **Ticket:** LOCAL #594 (bug, T-series agent terminal writes, after #525)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** (none yet)
- **Source ticket:** found in #525's visual check, after the fact (#593's run, shot 525-03-typed)
- **Status:** open

## Summary
`terminal_type` (#525) pastes an agent's text and sends its keys and Enter in the same burst.
Python's REPL (3.13 and later) reads a bracketed paste up to its end marker together with
whatever else is waiting, so the Enter lands inside the pasted text as a new line: the REPL
shows `...` and runs nothing, while the tool answers that it typed the bytes. Every submitted
line in #525's scenario sat unrun. Keys after a paste, such as Escape, are taken the same way.

## Acceptance
An agent's `terminal_type` with text and `submit` runs the line in Python's REPL (`print(6 * 7)`
prints 42), and keys sent after a paste reach the program as keys.
