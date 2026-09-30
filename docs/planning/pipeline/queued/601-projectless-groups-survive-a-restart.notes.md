# Projectless groups survive a restart — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-601-projectless-groups-survive-a-restart.md
- **Pipeline spec:** 601-projectless-groups-survive-a-restart.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** the restore half of Chad's projectless groups (#600).
- **Classification / tier:** feature; `marley_workbench`, maybe one Zed hook.
- **Recall (§18.3):** #575 (terminal ids across restore), #576 (Browser tabs keyed by workspace
  and item), #577 (a restored terminal in its folder: Zed's terminal panel once deleted other
  terminals' saved rows), #494 (Browser tabs restored), #486 (the size of a launch's first
  terminals).
- **Discovery (Explore, 2026-09-30):** see the spec's Prior art; the restore order in
  `crates/zed/src/main.rs:1447+` (`read_serialized_multi_workspaces` then
  `restore_multiworkspace` then `apply_restored_multiworkspace_state`). An upstream commit,
  `1b467a0b1c` ("Restore saved local session members by workspace identity"), is not in this
  tree; read it at promotion, since it may restore more than the active workspace and change
  what Marley has to do.

### Design (to settle at promotion)
- The record: #600's `MarleyGroupsDb` rows plus an order column, or a list in the rail's blob in
  `sidebar_state` (per window, written when the `MultiWorkspace` serializes). The blob is per
  window, which is what the restore needs; the table survives the blob.
- The reopen: after `restore_multiworkspace`, for each recorded group whose workspace is not the
  active one, `open_workspace_by_id` (or `Workspace::new_local` with that id) with
  `OpenMode::Add` into the window, then mark it as a group again (#600's hook).

### Visual check plan
- The scenario quits and restarts Marley on its scratch profile; the runner supports a restart
  (see `script/e2e/494-browser-restore.sh` and `577-*` for the pattern). Shots as in the spec.

### Risks
- The active group is restored by Zed and must not be reopened a second time.
- A window restored with no folder at all (only groups) must not trip Zed's "empty window gets
  the Launchpad" path.
