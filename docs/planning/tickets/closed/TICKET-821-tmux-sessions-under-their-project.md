# TICKET-821 — tmux sessions under their project, and a close prompt

- **Ticket:** LOCAL #821 (feature, the Marley layout's rail and tmux #808, #819)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** ../../pipeline/completed/821-tmux-sessions-under-their-project.spec.md
- **Source ticket:** Chad, 2026-10-20: "Can we move TMUX sessions up near Threads", "also the
  ability to close tmux sessions", then "tmux sessions should be inside the panes as a normal
  terminal session not isolated to its own thing. When exit we should have a popup "do you want
  to close the tmux session" or "hide" something along those lines". Asked where a session
  started outside every project goes: "Home group".
- **Status:** closed

## Summary
The TMUX · host sections at the foot of the rail go. Each tmux session shows as a terminal row
under the project whose folder holds the folder the session started in: a local project for this
machine's sessions, a remote project on that host for a host's sessions, and the Home group for
a session in no open project's folder. A click attaches it in a tab in that project. Closing an
attached session's tab asks Close Session (the session ends), Hide (the tab closes and the
session keeps running, its row still in the rail) or Cancel.

## Acceptance
A tmux session started in `repo` shows under `repo`; one started in `~` shows under Home; a
click attaches it in its project; closing its tab asks Close Session, Hide or Cancel, and each
does what it says.
