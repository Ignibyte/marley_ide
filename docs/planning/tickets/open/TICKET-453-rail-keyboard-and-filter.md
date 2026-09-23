# TICKET-453 — Keyboard navigation, a filter and project reorder in the rail

- **Ticket:** LOCAL #453 (feature, workbench shell W6d)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet (split from #442 at its promotion)
- **Source ticket:** ../../pipeline/completed/442-rail-persistence.spec.md (Out) · ../../../marley/workbench-shell.md
- **Status:** open

## Summary
The rail's key context (`MarleyRail`) binds up, down, enter, left and right to `menu::*` for
selection, confirm, collapse and expand, and `ctrl-f` to a filter field under the header that
narrows rows by fuzzy match on titles and project names (Zed's `fuzzy` crate). Filtered rows
keep their item-id keys (`PR-claude-live-refresh-selection-identity-key-must-be-unique-001`).
Move Project Up and Move Project Down join the project header menu, through
`MultiWorkspace::move_project_group_up` and `move_project_group_down`. The bindings go into the
Marley keymap (#450) after a shadow sweep of each chord.

## Acceptance
With the rail focused, the arrow keys move the selection, enter opens the selected row, and left
and right fold and unfold projects; typing in the filter leaves only matching rows; Move Project
Up and Down move the project's header one place. The EARS criteria come at promotion (the
draft is REQ-005, REQ-006 and REQ-008 of #442's queued spec, in git history).
