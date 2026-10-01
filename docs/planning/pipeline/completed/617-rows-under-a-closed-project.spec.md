---
pipeline_id: c0e31709-3763-4329-be8f-f64ae9e8fcf8
ticket: docs/planning/tickets/open/TICKET-617-rows-under-a-closed-project.md
status: Phase 4 — Complete PASS
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
  A thread row opens the thread's own folders under the group's key, as Zed's Threads Sidebar
  opens a closed group's thread, and then the thread in that workspace's Agent Panel, loading the
  panel when the new workspace has none yet.
- **Ports:** the ports scan takes a closed local group's folders from its key's path list, so its
  listeners are attributed to it; Copy, Stop and Restart work as under an open project; Open and
  Show Logs open the project first, then the Browser tab or the terminal. `ports_list` names a
  closed project's ports too.
- **The header** gets its chevron back once the project has rows, and folds as others do; it
  stays dimmed while the project is closed.
- **The keyboard:** Next and Previous Thread pass over a closed project's threads, as Next Project
  passes over its header (#606): going to one would open the project.

### Out (explicitly deferred)
- Terminals, browser tabs and worktree rows of a closed project: they live in its workspace.
- Live statuses of a closed project's threads (nothing runs them while it is closed).
- Linked worktrees' ports of a closed project (their folders are not in the key).

## Reference (§20)
Upstream Zed: the Threads Sidebar lists a project group's threads from `ThreadMetadataStore` by
its path list whether a workspace is open or not (`crates/sidebar/src/sidebar.rs`), and opens a
group's workspace with `MultiWorkspace::find_or_create_workspace` (`multi_workspace.rs:1105-1121`).
A closed group's thread opens through `open_workspace_and_activate_thread` (`sidebar.rs:4043`): the
thread's folder paths with the group's key as the provisional key, then
`load_agent_thread_in_workspace` (`sidebar.rs:3683`), which loads the Agent Panel with
`AgentPanel::load` when the new workspace has none yet and adds it only if Zed's own
initialization has not by then (`zed.rs` `setup_or_teardown_ai_panel` checks the same).

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
  - `marley_rail::cycle_row` and `cycle_project` (`marley_rail.rs:1260-1287`).
  - `AgentPanel::load` (`agent_panel.rs:1279`), public, and `Workspace::add_panel`.

## UI proof
The scenario `script/e2e/617-rows-under-a-closed-project.sh` (sway) opens two projects, seeds a
thread for the second and starts a server listening in its folder, restarts Marley so the second
is closed (as #606's scenario does), then:
- `closed.png`: the second project's dimmed header with its chevron, its thread and port rows;
- clicks the chevron (`folded.png`: the rows folded), and again;
- clicks the older of its two threads, which a restored Agent Panel would not show (`thread.png`:
  the project open and that thread in its Agent Panel);
- restarts again, double-clicks the port row (`port.png`: the project open and the Browser tab on
  the port).

## Locked-In Decisions
- D1 — Only rows that need no workspace to be listed are shown; opening one opens the project.
- D2 — A closed group's port folders come from its key's path list.
- D3 — A closed project's thread opens its own folders under the group's key, as Zed's Threads
  Sidebar does, so a thread that ran in a linked worktree opens there.
- D4 — A closed group's thread icons and agent names are read against the shown workspace's
  project: the agent servers are the user's settings and extensions, the same for every local
  project.
- D5 — Next and Previous Thread pass over a closed project's threads.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a project is closed, the rail shall list its threads and the ports listening in its folders under its header. | Shot `closed.png` |
| REQ-002 | WHEN the user opens a closed project's thread row, Marley shall open the project and then the thread. | Shot `thread.png` |
| REQ-003 | WHEN the user opens a closed project's port row, Marley shall open the project and then the port in a Browser tab. | Shot `port.png` |
| REQ-004 | WHILE a closed project has rows, its header shall fold and unfold them. | Shots `closed.png`, `folded.png` |
| REQ-005 | WHEN the user goes to the next or previous thread, the rail shall pass over a closed project's threads. | Review of `cycle_row` |

## Phase Plan
- **P1 Plan** — promote, recall, the design (threads without a workspace, the ports' folders).
- **P2 Code** — the rows, the openers, the header; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
