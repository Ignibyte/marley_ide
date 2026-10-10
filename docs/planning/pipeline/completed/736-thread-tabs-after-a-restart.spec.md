---
pipeline_id: 4753cd49-c429-4f12-b692-7f57fee5f54a
ticket: docs/planning/tickets/closed/TICKET-736-thread-tabs-after-a-restart.md
status: Phase 4 — Complete PASS
title: Thread tabs after a restart
type: feature
slice: the Marley layout (docs/marley/workbench-shell.md), agents anywhere (docs/planning/design-notes/agents-anywhere-2026-10-10.md)
references:
  - docs/planning/pipeline/queued/734-an-agent-thread-starts-in-a-tab.spec.md
---

## Title
A thread tab comes back after a restart in the group and pane it was in, with its earlier turns,
as Browser tabs come back (#494). #697's tab, and #734's, are gone after a restart today.

## Scope
### In
- `ThreadTab` becomes a `SerializableItem` (kind `MarleyThreadTab`): its own table keyed by
  workspace and item, holding the thread id, the agent id and the folders as JSON, following
  `rusty/graph_store.rs`'s pattern (no `UNIQUE(item_id)`, #576).
- Deserializing waits for the workspace's Agent Panel (up to ten seconds; Zed loads panels while it
  restores items), adds the folder back as a hidden worktree when #734 made one, then builds the
  `ConversationView` with the thread's `ThreadId` and the session id its record holds, so the agent
  loads or resumes the session as the Agent Panel does from its history.
- A thread whose record is gone, or whose agent is no longer there, leaves no tab: deserialize
  fails, and Zed drops the item.
- Tabs moved from the panel (#697) come back the same way.

### Out (explicitly deferred)
- Draft text typed and not sent: Zed's `draft_prompt_store` keeps a panel draft; a tab's draft is
  not kept.
- An agent that cannot load a session (no `loadSession`) shows the thread Zed's way: its record's
  title with no turns.

## Reference (§20)
Upstream Zed: `workspace`'s `SerializableItem` (`serialize`, `deserialize`, `cleanup`, the items
table per kind) and `agent_ui`'s thread loading (`AgentPanel::load_agent_thread` →
`create_agent_thread_with_server`, which takes the session id from `ThreadMetadataStore`).
Behavior kept: Zed's session restore of items, and Zed's own way of loading a saved thread.

### Prior art
- **The code we ship:**
  - Marley's `BrowserView` (`browser.rs:7007`, kind `MarleyBrowserTab`) and `GraphView`
    (`rusty/graph_tab.rs:2410`) with `rusty/graph_store.rs:215` (a JSON `state` column): the
    house pattern, `db::static_connection!`, `workspace::delete_unloaded_items` in `cleanup`.
  - `ConversationView::new(…, resume_session_id, thread_id, work_dirs, title, …)`
    (`conversation_view.rs:990`): a `resume_session_id` makes `initial_state` call
    `load_session` or `resume_session` (1310-1340).
  - `ThreadMetadataStore::entry(thread_id)` gives `session_id`, `agent_id` and `folder_paths()`.
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md`
  (`SerializableItem` and its registry).
- **Published material:** ACP `session/load` and `session/resume`.

## UI proof
`script/e2e/736-thread-tabs-after-a-restart.sh`, under `compositor sway`. The scripted agent
answers `session/load` by replaying the turns it logged, and `read <path>` as in #734. Two keys bound
to `marley::NewAgentThread` open a thread in the project's root and one on `other/`, outside it;
each gets a message. Marley quits through its palette and starts again on the same profile
(`quit_marley`, `launch_marley`).

Shots:
- `736-01-before`: both thread tabs in the project, the root one with "Noted: hello".
- `736-02-after`: after the relaunch, the same tabs in the project, the root one with "hello" and
  "Noted: hello".
- `736-03-typed`: "again" typed in the restored tab is answered.
- `736-04-folder`: in the restored `other/` tab, `read <other>/notes.md` answers "Read: # notes from
  other".

## Locked-In Decisions
- **D1:** the tab stores ids, not turns: the thread store and the agent hold the conversation.
- **D2:** a tab that cannot be restored is dropped, never shown empty.
- **D3:** the table is Marley's own (`marley_thread_tabs`), in the workspace database.
- **D4:** a restored tab uses the Agent Panel's connection store, as a new one does (#734); it waits
  for the panel, which Zed adds while items are still being restored, and without one after ten
  seconds it is not restored.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley starts again, the system shall show each thread tab open at quit in the same group, with the thread's earlier turns. | Shots 736-01 and 736-02 |
| REQ-002 | WHEN a message is typed in a restored tab, the thread shall send it and show the reply. | Shot 736-03 and the agent log |
| REQ-003 | IF a tab's thread can no longer be loaded, THEN the system shall not show that tab. | The review of the diff |
| REQ-004 | WHEN a thread tab on a folder its project did not hold is restored, the agent's read of a file there shall return its content. | Shot 736-04 and the agent log |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes.
- **P2 Code** — `thread_tab.rs` (`SerializableItem`, its store); a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
