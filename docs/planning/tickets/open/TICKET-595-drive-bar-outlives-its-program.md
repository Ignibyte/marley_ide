# TICKET-595 — End the drive bar and Ctrl-I's take-over with the program an agent typed into

- **Ticket:** LOCAL #595 (bug, T-series agent terminal writes, after #525)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** (none yet)
- **Source ticket:** found in #525's visual check, after the fact (#594's run, shot 525-11-shell-refused)
- **Status:** open

## Summary
Once an agent has typed into a program (#525), a bar under the terminal says so, with Take Over,
and Ctrl-I takes over. When that program exits, the bar stays under the shell: the footer shows
the stored last write without asking whether its program still runs, and `toggle_control` takes
over while any last write is stored. So Ctrl-I, which is Tab, at the shell's prompt would take
over instead of completing, where #525's REQ-012 says it reaches the program. The drive state
catches up only when an agent next reads the terminal.

## Acceptance
When the program an agent typed into is no longer in the foreground, the bar is gone and Ctrl-I
reaches the terminal's program as before.
