# TICKET-258 — Persist split-file panes across restart (c=<path> grid codec)

- **Forge ticket:** #258 (a293b7ac-9714-4fb7-8805-c9f9c2c10ccd) (feature, M15)
- **Owner:** (autonomous /goal /work 249 to 258)
- **AAR:** 2fdf7580-0f75-45d7-9989-bebb6d09eea8
- **Pipeline doc:** ../../pipeline/completed/split-pane-persist.spec.md
- **Source ticket:** M15 — The Editable Editor (sprint #28, fa328492); the #242 decomposition (#248–258); #246
  follow-on
- **Status:** closed

## Summary
The #246 split-right file pane is a read-only `CodeView` pane whose file path is LOST on save — `serialize_leaf`
emits a bare `c` (path-less), so the pane is dropped on restore (#154). Extend the grid codec to `c=<path>`
(mirroring the #205 `t=<cwd>` terminal-cwd codec, framing-guarded) + a restore arm that re-reads the file into a
CodeView pane (mirroring the #243 editor-tab paths restore), so a split-file pane survives a relaunch. Creating a
split-file pane triggers a layout persist (the #243 persist-TRIGGER discipline). This is the **(b) persistence
half** of #258; the **(a) editable split pane** is split out to a fast-follow **#259** (a `PaneContent` model
change rewiring the 16+ `active_tab().editor()` sites — too large for the codec seam).

## Acceptance
A split-right file pane persists across quit→relaunch: it serializes as `c=<path>` (framing-safe), restores by
re-reading the file (unreadable / legacy bare `c` → dropped, no crash, both restore paths), and its creation
triggers the save. The pure codec fns are cov/MSI 100. Full EARS criteria in the pipeline spec.
