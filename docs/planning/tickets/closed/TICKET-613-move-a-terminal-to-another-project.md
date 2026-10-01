# TICKET-613 — Move a terminal to another project in the rail

- **Ticket:** LOCAL #613 (feature, workbench shell, the rail (after #602))
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/completed/613-move-a-terminal-to-another-project.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 2 of `docs/planning/design-notes/remaining-work-2026-09-30.md`)
- **Status:** closed

## Summary
A terminal row belongs to the project whose workspace holds it. #602 deferred moving one to another project. Dragging a terminal row onto another project in the same window, or its menu's Move to Project, re-homes the terminal into that project's workspace with its shell still running and its scrollback kept, so the rail lists it there.

## Acceptance
WHEN a terminal row is dropped on another project of the window, or moved there from its menu, the rail shall list it under that project, and its shell shall still run.
