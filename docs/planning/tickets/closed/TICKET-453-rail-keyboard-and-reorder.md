# TICKET-453 — Keyboard navigation and project reorder in the rail

- **Ticket:** LOCAL #453 (feature, workbench shell W6d)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/453-rail-keyboard-and-reorder.spec.md
- **Source ticket:** ../../pipeline/completed/442-rail-persistence.spec.md (Out) · ../../../marley/workbench-shell.md
- **Status:** closed

## Summary
With the rail focused, the arrow keys move a cursor over its rows, Enter opens the row under
it, and left and right fold and unfold projects, through Zed's `menu::*` actions and the
`menu` key context Zed's own sidebar uses. Move Project Up and Move Project Down go into a
project header's right-click menu, through `MultiWorkspace::move_project_group_up` and
`move_project_group_down`. Split at promotion (2026-09-23): the filter is TICKET-457.

## Acceptance
With the rail focused, up and down move the highlight row by row, Enter shows what the row
holds, and left and right fold and unfold projects; Move Project Up and Down move a project's
header one place. Full EARS in the active spec.
