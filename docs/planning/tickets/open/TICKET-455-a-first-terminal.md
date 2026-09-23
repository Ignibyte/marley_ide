# TICKET-455 — A first terminal for a project shown with none

- **Ticket:** LOCAL #455 (feature, workbench shell W6f)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet (split from #451 at its promotion)
- **Source ticket:** ../../pipeline/completed/451-marley-layout-presets.spec.md (Out) · ../../../marley/workbench-shell.md
- **Status:** open

## Summary
In the Marley layout the terminal is the main surface, so a project first shown with no center
terminal should get one at its root. A restored workspace adds its saved terminals after the
rail first sees it, and seeding before that would double them. `Workspace::is_restoring` is
the obvious gate, but `load_workspace` sets it inside the task it spawns
(`crates/workspace/src/workspace.rs:7857-7862`), a turn after the workspace exists, so a rail
that reads it at `WorkspaceAdded` can see `false` for a workspace about to restore. Planning
starts with that: find a signal that a workspace's items are in (a restore's end, or a fresh
workspace with no serialized items), or seed only for a project opened fresh.

## Acceptance
A project opened in the Marley layout with no terminal shows one at its root, focused only if
nothing else took focus; a restored project, with or without saved terminals, gains no extra
one. The EARS criteria come at promotion.
