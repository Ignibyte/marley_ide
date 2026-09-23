# TICKET-458 — The rail follows a project's folders

- **Ticket:** LOCAL #458 (bug, workbench shell)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/458-rail-follows-folder-changes.spec.md
- **Source ticket:** ../../pipeline/completed/453-rail-keyboard-and-reorder.notes.md (Phase 2)
- **Status:** closed

## Summary
Removing the last folder from the displayed project leaves its row in the rail until some other
change in the window rebuilds it. Zed's `MultiWorkspace::handle_project_group_key_change`
(`crates/workspace/src/multi_workspace.rs:615-635`) returns early on an empty key without a
notify, and the workspace's own `project::Event::WorktreeRemoved` arm
(`crates/workspace/src/workspace.rs:1759-1763`) emits no `workspace::Event`, so neither of the
rail's subscriptions fires. Zed's Threads Sidebar subscribes to each project and rebuilds on
`WorktreeAdded`, `WorktreeRemoved`, `WorktreeOrderChanged` and `WorktreePathsChanged`
(`crates/sidebar/src/sidebar.rs:1005-1031`); the rail can follow the same events.

## Acceptance
With the rail open, removing a project's last folder updates the rail at once, with no other
change in the window; the driven test `enter_with_no_row_highlighted_does_nothing` then needs no
extra terminal to rebuild the rail. The EARS criteria come at promotion.
