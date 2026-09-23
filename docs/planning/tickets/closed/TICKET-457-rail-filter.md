# TICKET-457 — A filter for the rail

- **Ticket:** LOCAL #457 (feature, workbench shell W6h)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/457-rail-filter.spec.md
- **Source ticket:** ../../pipeline/completed/453-rail-keyboard-and-reorder.spec.md (Out) · ../../../marley/workbench-shell.md
- **Status:** closed

## Summary
A filter field under the rail's header narrows the rows to the projects, terminals and threads
whose name or title contains what is typed, with the matched characters highlighted, as Zed's
Threads Sidebar filter does (`crates/sidebar`: its `filter_editor`, matched by
`agent_ui::threads_archive_view::fuzzy_match_positions`, a substring match that ignores ASCII
case). Filtered rows keep their item-id keys
(`PR-claude-live-refresh-selection-identity-key-must-be-unique-001`), a project shows when it
or any of its rows match, and the keyboard navigation of #453 moves over the filtered rows.
`ctrl-f` (`cmd-f` on macOS) in the rail's context focuses the filter, through the Marley keymap
after a shadow sweep. Corrected at promotion (2026-09-23): the first draft said fuzzy match and
cited Warp's "Search tabs", but Zed's filter is a substring match and the behavior maps describe
no Warp session filter.

## Acceptance
Typing in the filter leaves only the matching rows and their projects; clearing it brings every
row back; the arrow keys and Enter work over what is shown. Full EARS in the active spec.
