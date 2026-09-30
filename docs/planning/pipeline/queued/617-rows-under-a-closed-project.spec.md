---
pipeline_id: c0e31709-3763-4329-be8f-f64ae9e8fcf8
ticket: docs/planning/tickets/open/TICKET-617-rows-under-a-closed-project.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Threads and ports under a closed project"
type: feature
slice: workbench shell, the rail; after #606
references: [docs/planning/pipeline/completed/606-closed-projects-in-the-rail.spec.md, docs/planning/pipeline/completed/521-ports-per-project.spec.md]
---

## Title
A closed project's header can fold open again, and lists the project's agent threads and the ports
listening in its folders, as an open project does.

## Scope
### In
- **Threads:** from `ThreadMetadataStore` by the group's folders and host, with the agent's icon and
  name found without a project where they can be (the native agent needs none; others by name).
  A thread row opens the project (#606's `open_closed_project`, awaited) and then the thread.
- **Ports:** the ports scan takes a closed local group's folders from its key's path list, so its
  listeners are attributed to it; Copy and Stop work as under an open project; Open opens the
  project first, then the Browser tab.
- **The header** gets its chevron back once the project has rows, and folds as others do; it
  stays dimmed while the project is closed.

### Out (explicitly deferred)
- Terminals, browser tabs and worktree rows of a closed project: they live in its workspace.
- Live statuses of a closed project's threads (nothing runs them while it is closed).
- Linked worktrees' ports of a closed project (their folders are not in the key).

## Reference (§20)
Upstream Zed: the Threads Sidebar lists a project group's threads from `ThreadMetadataStore` by
its path list whether a workspace is open or not (`crates/sidebar/src/sidebar.rs`), and opens a
group's workspace with `MultiWorkspace::find_or_create_workspace` (`multi_workspace.rs:1105-1121`).

### Prior art
- **Behavior maps:** #606 (AD-claude-606) and #521 (AD-claude-521).
- **Published material:** none needed.
- **Code we already ship:**
  - `ThreadMetadataStore::entries_for_path` and `entries_for_main_worktree_path`, which need only
    a path list and a host.
  - The rail's `group_threads` (`rail.rs:5596-5661`), `push_closed` (6068-6090),
    `open_closed_project` (3175-3216), `open_thread` (2338-2375).
  - `ports::project_folders` (`ports.rs:128-157`) and `attribute` (163-208).
  - `agents::thread_icon` and `thread_agent_name` (`agents.rs:112, 130`), which take a project.

## UI proof
The scenario `script/e2e/617-rows-under-a-closed-project.sh` (sway) opens two projects, seeds a
thread for the second and starts a server listening in its folder, restarts Marley so the second
is closed (as #606's scenario does), then:
- `closed.png`: the second project's dimmed header with its thread and port rows;
- clicks the thread (`thread.png`: the project open and the thread in its Agent Panel);
- restarts again, double-clicks the port row (`port.png`: the project open and the Browser tab on
  the port).

## Locked-In Decisions
- D1 — Only rows that need no workspace to be listed are shown; opening one opens the project.
- D2 — A closed group's port folders come from its key's path list.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a project is closed, the rail shall list its threads and the ports listening in its folders under its header. | Shot `closed.png` |
| REQ-002 | WHEN the user opens a closed project's thread row, Marley shall open the project and then the thread. | Shot `thread.png` |
| REQ-003 | WHEN the user opens a closed project's port row, Marley shall open the project and then the port in a Browser tab. | Shot `port.png` |
| REQ-004 | WHILE a closed project has rows, its header shall fold and unfold them. | Shot `closed.png`; review |

## Phase Plan
- **P1 Plan** — promote, recall, the design (threads without a workspace, the ports' folders).
- **P2 Code** — the rows, the openers, the header; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
