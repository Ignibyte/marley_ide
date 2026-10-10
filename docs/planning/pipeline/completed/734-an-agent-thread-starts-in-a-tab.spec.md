---
pipeline_id: 7b7a6874-8065-4e91-9e4c-c8b627ec04c0
ticket: docs/planning/tickets/closed/TICKET-734-an-agent-thread-starts-in-a-tab.md
status: Phase 4 — Complete PASS
title: An agent thread starts in a tab
type: feature
slice: the Marley layout (docs/marley/workbench-shell.md), agents anywhere (docs/planning/design-notes/agents-anywhere-2026-10-10.md)
references:
  - docs/planning/pipeline/completed/697-an-agent-thread-in-a-center-tab.spec.md
  - docs/planning/pipeline/completed/702-one-rail-row-for-a-thread-in-a-center-tab.spec.md
---

## Title
An Agent Panel style thread starts in a center tab of whatever group is shown, a group with no
folder included, and works in a folder Marley names for it. Today a thread exists only in a
project's Agent Panel (#697 moves a running one out, but cannot start one), so Marley's and
Rusty's agents, and any agent wanted outside a project, have nowhere to run.

## Scope
### In
- `marley::NewAgentThread { agent, folder }`, an action a key can bind with both: a new thread of
  `agent` (the Agent Panel's selected agent when absent) in a center tab of the shown group,
  working in `folder` (the shown project's root, else the home folder, when absent). It builds the
  `ConversationView` with `work_dirs` = `folder`, the hosting workspace's project and the
  connection store of that workspace's Agent Panel, and shows it in a `ThreadTab`.
- The folder joins the hosting project as a hidden worktree
  (`Project::find_or_create_worktree(folder, false)`) before the thread starts, unless a worktree
  already holds it, so the agent's file reads and writes resolve. The home folder, or a folder
  that holds it, never does: Zed scans and watches a hidden worktree fully, and a home folder is
  too large; there the agent's file calls reach only open projects, as in Zed.
- The thread's record in Zed's thread store names the folder it works in and is not archived
  (Zed archives a thread whose project has no visible folder).
- The rail lists the thread's row under the group whose center holds its tab, for a group with no
  folder too, once (#702's one-row rule kept), with its status, and the Needs you inbox lists its
  waits, as for the Agent Panel's threads.
- A thread in a tab in front is visible to Zed: it raises no "finished" notification while you
  look at it, and a notification's click brings its tab forward instead of opening a second copy
  in the panel.
- The rail's `+` → New Agent Thread starts the thread in a tab: in a project, in the project's
  root; in a group with no folder (Home, Rusty, a named group), in the home folder. Groups gain the
  submenu they lacked (#600).
- Three small hunks in `agent_ui` (ledger rows written first): `MarleyThreadHost`, a global a host
  outside the panel sets (`shows`, `reveal`); its two uses in `conversation_view.rs`; and
  `ConversationView::marley_own_folders`, which files the thread's record under its own folders in
  `thread_metadata_store.rs`.
- `docs/marley/guide.md`: the thread-in-a-tab section.

### Out (explicitly deferred)
- Choosing the folder and the agent from one picker: #735.
- Restoring a thread tab after a restart: #736.
- Zed's own agent on a folder its project does not show: Zed's agent reads only visible worktrees
  and ignores `work_dirs` (`agent/src/agent.rs` `new_session`), so in a group with no folder it
  works without files. Recorded in the guide, not changed.
- The panel's own `+` keeps opening threads in the panel.

## Reference (§20)
Upstream Zed, `agent_ui`: the Agent Panel's new-thread path (`AgentPanel::create_agent_thread_inner`,
which builds the `ConversationView` this tab builds, with the same agent server, connection store
and thread store), and Zed's own test host for a thread in a center pane (`ThreadViewItem` in
`conversation_view.rs`). Behavior kept: the same thread view, the same agents and their settings,
the same thread store. Zed's `project` crate: a file opened from outside a project joins it as a
hidden worktree, the same mechanism used here for the thread's folder.

### Prior art
- **The code we ship:**
  - `agent_ui::ConversationView::new` is `pub` and takes `work_dirs`, `project`,
    `connection_store`, `thread_store` and `source` explicitly (`conversation_view.rs:990`).
    `Agent::server` (`agent_ui.rs:578`) turns an agent id into its server (`NativeAgent` or
    `CustomAgentServer`). `AgentPanel::connection_store()` (`agent_panel.rs:1631`) gives the
    panel's store, so a tab thread shares the panel's agent process.
  - `acp.rs` `session_directories_from_work_dirs` (`agent_servers/src/acp.rs:1537`): the first
    work dir is the ACP session's `cwd`.
  - `AcpThread::read_text_file` / `write_text_file` resolve through
    `project_path_for_absolute_path`, which searches every worktree, hidden ones included
    (`worktree_store.rs:463`); a path outside them is `resource_not_found`.
  - `ThreadMetadataStore` records a thread from its view on `RootThreadUpdated`, wherever the view
    is (`thread_metadata_store.rs:1266`), with `worktree_paths` from the project's visible folders
    and `archived` when there are none (1316); `update_working_directories` (790) sets the folders.
  - Marley: `thread_tab.rs` (#697, the tab and its focus rule PR-claude-697), `rail.rs`
    `member_tabs` / `active_thread_tab` (#702), `agents::thread_agents` and `start_thread`
    (`agents.rs:158,185`), `terminal_view::default_working_directory` for the home-folder fallback.
- **Behavior maps:** `docs/orca_architecture/01-agents-and-sessions.md` §2.1: Orca starts an agent
  in the worktree root and downgrades to a terminal when the `cwd` is outside it; the hidden
  worktree is what lets Marley keep the thread view there. `docs/zed_architecture/subsystems/
  06-project-fs-search.md` (worktrees).
- **Published material:** ACP `session/new` (`cwd`) and the client file methods `fs/read_text_file`,
  `fs/write_text_file`.

## UI proof
`script/e2e/734-an-agent-thread-starts-in-a-tab.sh`, under `compositor sway` (the rail's `+` takes a
click). The Marley entry runs #697's scripted agent (`MARLEY_ASSISTANT_ADAPTER`), extended so a
prompt `read <path>` asks Marley for the file with `fs/read_text_file` and answers with its first
line; it logs `session/new` with its `cwd`.

Shots:
- `734-01-home-thread`: Home shown, `+` → New Agent Thread → Marley; a center tab in Home with the
  thread, and its row under Home in the rail.
- `734-02-folder`: a key bound to `marley::NewAgentThread` with `other/` (outside the project): a
  thread tab there; after `read <other>/notes.md`, "Read: # notes from other".
- `734-03-project-thread`: in the scratch project, `+` → New Agent Thread → Marley; a center tab in
  the project, its row under the project; `read <repo>/README.md` answers "Read: # repo".

## Locked-In Decisions
- **D1:** Marley's own entry points start a thread in a center tab; Zed's Agent Panel `+` keeps
  starting it in the panel, and #697 moves a thread between the two. Chad, 2026-10-10: "most
  everything should open in a main tab".
- **D2:** the folder joins the hosting project as a hidden worktree, not a visible one: the project
  panel and the rail's header stay as they were. Never the home folder or one that holds it, which
  Zed would scan and watch whole.
- **D3:** the tab thread uses the hosting workspace's Agent Panel connection store, so the panel and
  its tabs share one agent process per agent. A workspace with no Agent Panel (AI disabled) offers
  no thread.
- **D4:** a group with no folder works in the home folder, as its agent CLIs already do (#600).
- **D5:** the thread's record is filed under its own folders by a flag on its view that Zed's store
  reads (`marley_own_folders`). Fixing the record from outside does not hold: the store takes the
  project's visible folders again at every save, and archives a thread with none.
- **D6:** one action, `marley::NewAgentThread { agent, folder }`, is the way in: the rail's `+`
  dispatches it, a key can bind it, and #735's picker will.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent is picked under New Agent Thread in the `+` menu of a group with no folder, the system shall open a thread of that agent in a center tab of that group. | Shot 734-01 |
| REQ-002 | WHEN that thread starts, the system shall start its session in the home folder. | The scenario's check on the agent log's `session/new` `cwd` |
| REQ-003 | WHEN the thread's agent in a project reads a file of that project, the system shall return the file's content. | Shot 734-03 and the agent log |
| REQ-004 | WHILE the tab is open, the rail shall list one row for the thread, under the group that holds the tab. | Shots 734-01 and 734-03 |
| REQ-005 | WHEN an agent is picked under New Agent Thread in a project's `+` menu, the system shall open the thread in a center tab of that project, with its session in the project's root. | Shot 734-03 and the agent log |
| REQ-006 | WHEN the thread is saved, its record shall name the folder it works in and shall not be archived. | The review of the diff |
| REQ-007 | WHEN `marley::NewAgentThread` runs with a folder no open worktree holds, the system shall start the thread there, and the agent's read of a file in that folder shall return its content. | Shot 734-02 and the agent log |
| REQ-008 | WHILE a thread's tab is in front, Zed shall treat the thread as visible, and a notification's click shall bring the tab forward. | The review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes.
- **P2 Code** — `thread_tab.rs` (`start`, the hidden worktree, the record fix), `rail.rs` (the `+`
  submenu for every group, the row under the tab's group), the guide; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
