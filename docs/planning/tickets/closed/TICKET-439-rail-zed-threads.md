# TICKET-439 — Zed agent threads in the rail

- **Ticket:** LOCAL #439 (feature, workbench shell W3)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/439-rail-zed-threads.spec.md
- **Source ticket:** ../../design-notes/workbench-shell-shelf.md · ../../../marley/workbench-shell.md
- **Status:** closed

## Summary
Chad wants each project to list its Zed agent threads next to its terminals, so the Marley layout loses nothing the Threads Sidebar offered for day-to-day agent work. Thread rows come from Zed's thread metadata store, their live status from the Agent Panel. A click opens the thread in the Agent Panel on the right, and the project's + gains New Agent Thread for the Zed Agent and every configured ACP agent.

## Acceptance
Existing threads appear under their project with live status; clicking one opens it in the right-hand Agent Panel; New Agent Thread starts one that appears as a row. Full EARS in the spec.

## Resolution
Shipped 2026-09-22. Each project lists its Zed agent threads, newest first, with live status and
an attention dot for a run that ended unseen. A thread row opens its thread in the right-hand
Agent Panel, and the project's `+` starts a thread for the Zed Agent or any configured agent.
The rail keeps `is_threads_list_view_active` false (AD-claude-439-the-rail-does-not-claim-zeds-threads-list-001).
`script/gates.sh --diff` is green at 100% line coverage. The rows have not been seen live yet:
the drive would have moved Chad's windows off his monitor.
