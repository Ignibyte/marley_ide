# TICKET-601 — Projectless groups survive a restart

- **Ticket:** LOCAL #601 (feature, workbench shell: the rail)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/queued/601-projectless-groups-survive-a-restart.spec.md
- **Source ticket:** Chad, 2026-09-30 (the projectless groups of #600); split from #600 because Zed restores only the active folderless workspace of a window.
- **Status:** open

## Summary
#600's groups live for the session: Zed saves no project group for a folderless workspace and, at
launch, reopens only a window's active workspace, so every other group and its terminals would be
gone after a restart. Marley records each window's groups (id, name, workspace, order) and, when
the window comes back, reopens each group's workspace by its id, so the group returns with its
name, in its place, with its terminals in their folders and its Browser tabs, as a project's do.

## Acceptance
With two groups, one with two terminals and one with a Browser tab, quit Marley and start it:
both groups are back with their names and order, the terminals in the folders they were in, and
the Browser tab on its page.
