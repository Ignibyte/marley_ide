---
status: intake
created: 2026-09-23
ticket: TICKET-713
pipeline_spec: <unassigned>
note: audit (2026-10-09): partly shipped (TICKET-577, TICKET-606, TICKET-617); the four items left are TICKET-713
---

# The rail's internals: rows it could show and costs it could shed

## What
Items carried from #438's inspect that are neither persistence nor a user-facing slice of
their own, parked here when #442 was split (2026-09-23):
- **The git branch on a project header:** `project.active_repository(cx)`, then the
  repository's `branch.name()` (as `sidebar.rs:2692-2698` reads it), refreshed from the git
  store's `RepositoryUpdated(_, HeadChanged | GitWorktreeListChanged, _)`.
- **Groups with no open workspace:** Zed keeps a group after its last workspace closes; list
  them and reopen on click (`find_or_create_workspace`).
- **Telemetry:** `open_sidebar` records a "Sidebar Toggled" event for every window opened in
  the Marley layout and every swap; upstream's silent `restore_open_sidebar` is `pub(crate)`.
- **Subscriptions per swap:** Zed's `register_sidebar` has no unregister and adds two
  subscriptions per call, so every swap leaves a dead pair for a dropped rail and each round
  trip adds a live pair on the kept Zed sidebar (#438 inspect S6). A fix needs a Zed touchpoint.
- **Refresh cost:** the rail rebuilds its snapshot on each `Wakeup` and `UpdateTab` from every
  listed terminal, two per chunk of output; refresh on the events that change a row (#438
  inspect G10).
- **Restore order:** in the Marley layout the rail opens during window creation, which
  serializes the window before `apply_restored_multiworkspace_state` restores its project
  groups; a crash in between leaves that partial state until a later serialize.
- **The restart check:** restart with center and Terminal Panel terminals both open and check
  both kinds come back, since the workspace and the Terminal Panel each clean the shared
  `terminals` table with only their own item ids (`workspace.rs:7961-7982`,
  `terminal_panel.rs:359-378`).

## Why
Each is real but small, or needs a Zed touchpoint, or is a check rather than a change. Batched,
they make a cleanup slice once the W6 features have landed.

## Notes
The sources are #438's notes (`docs/planning/pipeline/completed/438-marley-layout-and-rail.notes.md`)
and #442's queued notes, in git history before its promotion.

## Promotion
This is NOT an active pipeline doc — it is a candidate. Promote it via
`/pipeline:plan` when ready: it becomes a ticket (`docs/planning/tickets/open/`) + an active
pipeline doc pair (`docs/planning/pipeline/active/`). On promotion, set
`status: promoted` and fill `ticket:` + `pipeline_spec:`.
