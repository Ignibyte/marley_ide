# TICKET-808 — tmux sessions in the rail

- **Ticket:** LOCAL #808 (feature, the rail and remote terminals)
- **Owner:** b6416b37-a6bf-4172-91e4-8d05bc38cda5
- **Pipeline doc:** ../../pipeline/completed/808-tmux-sessions-in-the-rail.spec.md
- **Source ticket:** Chad, 2026-10-17: "I would like to try to have this work but by attaching to
  existing tmux agents or an ssh server that we can connect to has tmux it starts a new session",
  "maybe ssh servers we know about that could actually just open them up. This would be settings
  im guessing?"; he approved the sentence below ("ok lets build 3 and 4").
- **Status:** closed

## Summary
For each host in `ssh_connections` that a setting names (and this machine), the rail lists its
tmux sessions under a host heading; a click attaches that session in a terminal tab, sessions
running Claude Code or Codex show as agents, and a + starts a new session there. #543's remote
terminal already makes a session of Marley's own on a host and reattaches it after a drop; this
reads the sessions already there, on the host's own tmux server.

## Acceptance
With a host named in the setting, the rail shows that host's tmux sessions, a click attaches one
in a tab, an agent's session is marked as one, and the host's + opens a new session.
