# TICKET-439 — Zed agent threads in the rail

- **Ticket:** LOCAL #439 (feature, workbench shell W3)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/439-rail-zed-threads.spec.md
- **Source ticket:** ../../design-notes/workbench-shell-shelf.md · ../../../marley/workbench-shell.md
- **Status:** open

## Summary
Chad wants each project to list its Zed agent threads next to its terminals, so the Marley layout loses nothing the Threads Sidebar offered for day-to-day agent work. Thread rows come from Zed's thread metadata store, their live status from the Agent Panel. A click opens the thread in the Agent Panel on the right, and the project's + gains New Agent Thread for the Zed Agent and every configured ACP agent.

## Acceptance
Existing threads appear under their project with live status; clicking one opens it in the right-hand Agent Panel; New Agent Thread starts one that appears as a row. Full EARS in the queued spec.
