# TICKET-171 — sweep the vestigial right-dock plumbing

- **Forge ticket:** #171 `be697f04-ce9a-4857-9c12-6b5ea7d6422d` (chore; sprint #22)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **Pipeline doc:** ../../pipeline/active/dock-sweep.spec.md
- **Status:** closed

## Summary
Remove DockRight/persist_dock_right/AppliedSettings.right/toggle_dock() (all dead post-#153); keep the
general DockSide/region_widths primitive with docks[1] a documented Closed slot. Zero behavior change.
