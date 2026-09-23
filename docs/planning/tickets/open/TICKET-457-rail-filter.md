# TICKET-457 — A filter for the rail

- **Ticket:** LOCAL #457 (feature, workbench shell W6h)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet (split from #453 at its promotion)
- **Source ticket:** ../../pipeline/completed/453-rail-keyboard-and-reorder.spec.md (Out) · ../../../marley/workbench-shell.md
- **Status:** open

## Summary
A filter field under the rail's header narrows the rows by fuzzy match on terminal and thread
titles and project names, as Warp's "Search tabs" and Zed's Threads Sidebar filter do
(`crates/sidebar`, its `filter_editor`, reached by `ctrl-f` there). Filtered rows keep their
item-id keys (`PR-claude-live-refresh-selection-identity-key-must-be-unique-001`), a project
shows when it or any of its rows match, and the keyboard navigation of #453 moves over the
filtered rows. `ctrl-f` in the rail's context focuses the filter, through the Marley keymap
after a shadow sweep.

## Acceptance
Typing in the filter leaves only the matching rows and their projects; clearing it brings every
row back; the arrow keys and Enter work over what is shown. The EARS criteria come at
promotion.
