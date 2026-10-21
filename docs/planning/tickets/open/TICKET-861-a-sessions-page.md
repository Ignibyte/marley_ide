# TICKET-861 — The Sessions page: every tmux session as a table

- **Ticket:** LOCAL #861 (feature, the Marley look, W1.3 follow-up)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** split from TICKET-832 while planning it (2026-10-21): the TmuxSessions board
  shows a center page beside the rail's Sessions view, which #832 leaves out.
- **Boards:** TmuxSessions
- **Size:** M
- **Status:** open

## Summary

A page in the center, opened from the rail's Sessions view: a header ("6 on 3 hosts · 2 attached
here · 4 detached", New Session…), a table of every session (session, host, windows, folder and
the project it is placed under, state, Show or Attach, Close…), the line on how a session is
placed, and the hosts to look on with their state. It reads the same `TmuxHosts` global the rail's
view does (#832).

## Acceptance

The page lists each session with its columns and state; Show, Attach, Close… and New Session…
work from it; it follows the listing as sessions come and go. Shot in Marley Night and Marley Day.
