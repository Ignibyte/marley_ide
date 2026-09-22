# TICKET-234 — Launcher / landing page (open-a-workspace home; supersedes #202)

- **Forge ticket:** #234 (9a011201-2545-4a97-ac46-10671cb53415) (feature, M13 sprint #26)
- **Owner:** session c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** a6d85f41-c4b5-43c1-a165-84f7ae1687ee
- **Pipeline doc:** ../../pipeline/active/workspace-launcher.spec.md
- **Source ticket:** M13 "The Workspace Cockpit" sprint #26 (chad: closing everything → a landing page)
- **Status:** closed

## Summary
The PhpStorm-style landing page shown when no workspace is open. Relaxes the M10 never-empties guards
(dropping `close_project`'s `LastProject` + guarding the `adjust_active` underflow) so a zero-workspace
state is reachable, then the top-level render branches to a launcher (title + recent workspaces + Open
Folder) instead of the blocked/blank screen — keeping the ~90 `workspace()` sites safe by only rendering
the shell when ≥1 workspace. Opening a workspace (a recent or the folder picker) records its root in a
persisted recents list and returns to the shell. Supersedes #202. Deps #233.

## Acceptance
0 workspaces → the launcher renders (not a panic); closing the last workspace is allowed; the launcher
opens a workspace via Open Folder or a recent row; opening records the root in a persisted, dedup'd,
capped recents list; the pure `should_show_launcher`/`push_recent` + the `adjust_active(0)` fix are
cov/MSI 100; and #202 is closed as superseded. Full EARS in the pipeline spec. Driven-validated
(control granted): close all → launcher → open → clears.
