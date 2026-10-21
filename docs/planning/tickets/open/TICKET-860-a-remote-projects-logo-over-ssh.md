# TICKET-860 — A remote project's logo, read once over SSH

- **Ticket:** LOCAL #860 (feature, the Marley look, W1.2 follow-up)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** split from TICKET-831 while planning it (2026-10-20): the ProjectLogo board
  says a remote project's file is read once over SSH and kept.
- **Boards:** ProjectLogo
- **Size:** M
- **Status:** open

## Summary

#564's search, and #831's shared cache on top of it, read the local disk with `std::fs`, so an SSH
project has only its letter. Read a remote project's `logo.png` or `logo.svg` once through the
project's own `Fs` (the remote worktree), keep the bytes in Marley's data folder keyed by host and
path, and use them until the file changes in the worktree.

## Acceptance

A remote project with `logo.png` shows it in the rail, the strip, Home and the title bar after the
first connection, and still shows it when the connection is down.
