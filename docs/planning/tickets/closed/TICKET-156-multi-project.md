# TICKET-156 — M9 seq-7: multi-project (open/switch projects in a workspace)

- **Forge ticket:** #156 `6ce20570-4560-43a9-819d-940b88da91a7` (feature, M9; sprint #20)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `671c025b-f3ba-41a2-bd9d-da95e06ce9b8`
- **Pipeline doc:** ../../pipeline/active/multi-project.spec.md
- **Status:** closed

## Summary
A workspace holds multiple projects: an open-project action (native folder picker) adds a Project; the rail
switches projects; project_root/project_files/file_tree/branch/git all follow the ACTIVE project via
sync_active_project. Shim wiring over seq-1's tested algebra. Deps #150-155.

## Acceptance
Open a 2nd project → the rail lists 2; switch → Files + titlebar + branch change; FULL gate GREEN. Full EARS in
the spec.
