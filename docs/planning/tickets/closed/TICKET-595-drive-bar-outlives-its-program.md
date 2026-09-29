# TICKET-595 — End the drive bar with the program an agent typed into

- **Ticket:** LOCAL #595 (bug, T-series agent terminal writes, after #525)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/595-drive-bar-outlives-its-program.spec.md
- **Source ticket:** found in #525's visual check, after the fact (#594's run, shot 525-11-shell-refused)
- **Status:** closed

## Summary
Once an agent has typed into a program (#525), a bar under the terminal says what it typed, with
Take Over. When that program exits, the bar stays under the shell: the footer shows the stored
last write without asking whether its program still runs, and the drive state catches up only
when something next reads it. Ctrl-I is not affected: the take-over reads the drive through
`drive()`, which drops the last write once the program has changed, so the key reaches the shell.
(As filed, this ticket said Ctrl-I would take over at the shell's prompt; reading the code showed
it does not.)

## Acceptance
When the program an agent typed into is no longer in the foreground, the bar is gone.
