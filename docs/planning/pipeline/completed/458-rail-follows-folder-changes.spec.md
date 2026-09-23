---
pipeline_id: c8b08d95-7995-4f72-9163-ff440921f036
ticket: docs/planning/tickets/closed/TICKET-458-rail-follows-folder-changes.md
status: Phase 4 — Complete PASS
title: The rail follows a project's folders
type: bug
slice: workbench shell
references: [docs/planning/pipeline/completed/453-rail-keyboard-and-reorder.notes.md, docs/planning/knowledge/failures.md]
---

## Title
When the displayed project lost its last folder, its row stayed in the rail until some other
change rebuilt it (`F-claude-453-the-rail-missed-a-projects-last-folder-going-001`). The rail
now follows each project's folder events, as Zed's Threads Sidebar does, and rebuilds on them.

## Scope
### In
- **The subscriptions** (`marley_workbench::rail`). The rail follows each listed workspace's
  project, as it follows the workspace, its terminals, its Agent Panel and its agent servers.
  It rebuilds on `project::Event::WorktreeAdded`, `WorktreeRemoved`, `WorktreeOrderChanged`
  and `WorktreePathsChanged`, and ignores every other project event. The rebuild waits until
  the events already queued have been delivered.
- **The workaround goes.** #453's `enter_with_no_row_highlighted_does_nothing` added a terminal
  to make the rail rebuild after the removal; it now relies on the removal itself.

### Out (explicitly deferred)
- Any other project event: diagnostics, language servers and the rest do not change a row.

## Reference (§20)
- **Upstream Zed:** the Threads Sidebar subscribes to each workspace's project and rebuilds on
  the same four worktree events (`crates/sidebar/src/sidebar.rs:1005-1031`). Marley keeps the
  four events and the rebuild.
- **Warp:** N/A. This is the rail's view of Zed's projects.

### Prior art
- **Behavior maps:** none on this seam.
- **Published material:** none.
- **Code we already ship.**
  - Zed's sidebar subscription above.
  - The workspace's own `project::Event::WorktreeRemoved` arm emits no `workspace::Event`
    (`crates/workspace/src/workspace.rs:1763-1767`).
  - `MultiWorkspace::handle_project_group_key_change` returns early on an empty key without a
    notify (`crates/workspace/src/multi_workspace.rs:615-628`), which is why the rail's two
    existing subscriptions miss the last folder going.
  - The rail's own `resubscribe` and `Watched` keep one subscription per listed entity.

## UI proof
UI-AFFECTING.
- **Driven tests:** removing the displayed project's last folder takes its row out of the rail
  with no other change in the window; adding a folder to a project and removing it again renames
  its row each time, and the switcher keeps the order in which its terminals were shown.
- **Live drive:** remove a project's last folder from the project panel and see the rail
  update; screenshot. It needs clicks, so it runs only while Chad is away from the desk;
  otherwise the Test phase records why.

## Locked-In Decisions
- D1 — The rail follows the project's four worktree events, as Zed's sidebar does, and no
  others, so a busy project does not rebuild the rail.
- D2 — The rebuild is deferred (`defer_in`), as Zed's sidebar schedules its own update. A
  project emits `WorktreeAdded` or `WorktreeRemoved` before `WorktreePathsChanged`, and the
  `MultiWorkspace` rekeys the project's group only on the second. A rebuild on the first finds
  the workspace in no group, drops the project's rows for that rebuild, and so forgets their
  recency in the switcher and the attention dots of their threads. Decided in the Code phase.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a listed project's folders are added, removed or reordered, the rail shall rebuild once those events have been delivered, with no other change in the window | driven tests |
| REQ-002 | WHEN a folder is added to or removed from a project, the rail shall keep that project's rows' recency and attention | driven test |
| REQ-003 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — the project subscriptions in `rail.rs`; fmt and clippy clean; a review of the
  diff.
- **P3 Test** — the workaround removed, the tests run, with a negative check; the live drive;
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the
  ticket, archive, commit.
