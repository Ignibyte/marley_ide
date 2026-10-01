# TICKET-617 — Threads and ports under a closed project

- **Ticket:** LOCAL #617 (feature, workbench shell, the rail (after #606))
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/completed/617-rows-under-a-closed-project.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 2 of `docs/planning/design-notes/remaining-work-2026-09-30.md`)
- **Status:** closed

## Summary
#606 lists a closed project as a dimmed header alone. Its agent threads are in Zed's metadata store and its ports may still listen, so the rail lists them under the closed header: a thread row opens the project and the thread, a port row works as it does under an open project.

## Acceptance
WHILE a project is closed, the rail shall list its threads and its listening ports under its header.
