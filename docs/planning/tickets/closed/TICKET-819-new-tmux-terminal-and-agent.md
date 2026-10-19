# TICKET-819 — New tmux Terminal and New Agent in tmux

- **Ticket:** LOCAL #819 (feature, the Marley layout's rail and tmux #808)
- **Owner:** 6bc788ef-5565-4849-9e73-e341a6348e9f
- **Pipeline doc:** ../../pipeline/completed/819-new-tmux-terminal-and-agent.spec.md
- **Source ticket:** Chad, 2026-10-19: "on the + on the left panel if tmux is on the box auto
  detected. THen lets add a New Terminal Agent." Asked what the + should do: "new tmux agent and
  new tmux terminal session basically".
- **Status:** closed

## Summary
When tmux is installed, Marley notices it with no setting: a project's + (and, by #820, its
header's right-click) offers New tmux Terminal, a shell in a new tmux session in the project's
folder, and New Agent in tmux, an installed agent CLI started in a new tmux session there. Both
open in a terminal tab attached to the session, survive closing Marley, and the session is listed
under TMUX · this machine in the rail (#808), which an unset `marley.tmux_hosts` now shows by
itself when tmux is installed.

## Acceptance
With tmux installed and no `tmux_hosts` set, the + offers both entries; each starts a tmux session
in the project's folder, attached in a tab, listed under TMUX · this machine.
