# TICKET-708 — A restored group starts with the keys

- **Ticket:** LOCAL #708 (bug, the Marley layout)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [708-a-restored-group-starts-with-the-keys.spec.md](../../pipeline/completed/708-a-restored-group-starts-with-the-keys.spec.md)
- **Source ticket:** found in #700's Test phase (L-claude-700-a-restored-folderless-workspace-starts-unfocused-001); #702 fixed the related case of a group made at start
- **Status:** closed

## Summary
After a relaunch that restores a folderless workspace as the window's shown one (Home or Rusty,
or Zed's start workspace), nothing in the window has focus: keys reach nothing until a click, so
the palette, a quit and typing all do nothing. #702 found that `Workspace::new` focuses a new
workspace's pane and its setup moves focus again, which is how a window ends up focused on
something it doesn't draw. Find where focus goes during a restore and give the shown workspace
its active item's focus once the window is restored.

## Acceptance
After a quit and relaunch that restores Home as the shown workspace, the palette opens on its key
with no click first.
