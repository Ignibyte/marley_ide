# TICKET-621 — A task's run as a block

- **Ticket:** LOCAL #621 (feature, prong 1 T4)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/completed/621-a-tasks-run-as-a-block.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`)
- **Status:** closed

## Summary
Zed's tasks run in terminals without Marley's shell hooks, so a task's output is unframed scrollback with a summary line. Marley opens a block when a task's terminal spawns, with the task's label as its command and its folder, and finishes it with the task's exit code, so a task run gets the gutter, the pill, the menu and the navigation shell blocks have.

## Acceptance
WHEN a task finishes in its terminal, Marley shall show its run as one block with the task's label, folder and exit status.
