---
pipeline_id: 357bfe50-ee46-4be2-8bf2-4a4854ed1745
ticket: docs/planning/tickets/open/TICKET-439-rail-zed-threads.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active for Phase 2 Design
title: Zed agent threads in the rail
type: feature
slice: workbench shell W3
references: [docs/marley/workbench-shell.md, docs/planning/pipeline/queued/438-marley-layout-and-rail.spec.md]
---

## Title
List each project's Zed agent threads in the rail under its terminals, with live status, and
open a thread in the right-hand Agent Panel on click. The project's `+` gains New Agent
Thread for the Zed Agent and every configured ACP agent, so starting an agent is one visible
click.

## Scope
### In
- Thread rows per project group, read from `ThreadMetadataStore::entries_for_main_worktree_path`
  for the group's paths: title, agent icon, and a status from the live `ConversationView`s in
  that workspace's Agent Panel (running, waiting for confirmation, finished, error).
- Click: activate the thread's workspace, then `AgentPanel::load_agent_thread` with focus; the
  panel sits on the right in the Marley layout.
- The project `+` menu gains New Agent Thread: the Zed Agent (`agent::NewThread`) and each
  agent from `AgentServerStore::external_agents` with its display name and icon
  (`agent::NewExternalAgentThread`, built by name because its field is private).
- Attention: a thread that finished or needs a confirmation while unfocused shows the dot,
  counts in `has_notifications`, and clears when the thread is opened.
- `is_threads_list_view_active` returns `true` in the Marley layout once thread rows exist,
  so Zed routes thread attention to the rail as it does for its own sidebar.
- Selection: when the Agent Panel has focus, the thread it shows is the selected row; the
  #438 selector gains that first arm.
- The rail rebuilds on `ThreadMetadataStore` changes and on the Agent Panel's events.

### Out (explicitly deferred)
- Archive, delete, history, rename and worktree archive/restore: they stay in the Agent Panel's
  own views (Zed keeps that orchestration private to its sidebar, `sidebar.rs:5338-5712`).
- Draft threads, Zed's agent-panel Terminal Threads, and cross-window thread activation.
- A thread opened as a center-pane item (possible later from `ConversationView::new`).

## Reference (§20)
- **Upstream Zed:** the Threads Sidebar (`crates/sidebar`) groups threads under their project,
  shows each thread's status and agent, and opens it in the Agent Panel on click; the rail
  keeps that behavior for thread rows and reads the same stores.
- **Warp:** agent sessions sit in the same session list as terminal sessions, grouped by
  project (observed on Chad's Warp, `docs/planning/design-notes/session-tabs-vs-sidebar.md`;
  `docs/warp_architecture/subsystems/04-agent-ai-mcp.md` for Warp's agent model at the
  behavior level).

### Prior art
- **Behavior maps:** `docs/warp_architecture/subsystems/04-agent-ai-mcp.md`; the Threads
  Sidebar as shipped (read for behavior, not copied: the rail is `MIT OR Apache-2.0`).
- **Published material:** Zed's `docs/src/ai/parallel-agents.md` (thread types, status,
  switching) and `docs/src/ai/external-agents.md`; the ACP spec for thread states.
- **Code we already ship:** `ThreadMetadataStore` (`crates/agent_ui/src/thread_metadata_store.rs:621`,
  `:643`) and its observe hook; `AgentPanel::conversation_views` (`agent_panel.rs:4151`),
  `load_agent_thread` (`:4452`), `new_thread` (`:1783`), `new_external_agent_thread`
  (`:1951`); `AgentServerStore::external_agents`, `agent_display_name`, `agent_icon`
  (`crates/project/src/agent_server_store.rs:243-609`); `acp_thread::ThreadStatus`
  (`crates/acp_thread/src/acp_thread.rs:2434`); the shared row component
  `crates/ui/src/components/ai/thread_item.rs` (Zed's own UI building block, reused as is);
  `cx.build_action` for the private-field action.

## UI proof
UI-AFFECTING.
- **Driven tests:** thread rows appear for seeded metadata under the right project; a stub
  connection's status change updates the row; a click opens the thread in the Agent Panel;
  New Agent Thread adds a row; the attention dot and `has_notifications` follow an unfocused
  finish; the selected row follows Agent Panel focus. Zed's `agent_ui::test_support`
  (`StubAgentConnection`, `open_thread_with_connection`) drives the threads.
- **Live drive:** in the Marley layout, New Agent Thread > Claude Code (ACP, Chad's
  `claude-acp`), send a prompt, screenshot the row going from running to done; click it after
  switching projects and screenshot the Agent Panel on the right.

## Locked-In Decisions
- D1 — Thread rows follow a group's terminal rows.
- D2 — Opening a thread uses the Agent Panel; no center-pane thread item in this ticket.
- D3 — Thread management (archive, history, rename) stays in the Agent Panel.
- D4 — The rail reports `is_threads_list_view_active = true` in the Marley layout from this
  ticket on, and draws thread attention itself.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The rail shall show a row for each thread whose metadata belongs to a project group's main worktree paths, under that group, after its terminal rows | driven test with seeded metadata |
| REQ-002 | WHILE a thread's conversation is live, its row shall show the conversation's status and update when the status changes | driven test with a stub connection |
| REQ-003 | WHEN a thread row is clicked, the thread's workspace shall be displayed and the thread shall open, focused, in the Agent Panel | driven test |
| REQ-004 | WHEN New Agent Thread > an agent is chosen from a project's `+`, a new thread for that agent shall start in that project's Agent Panel and appear as a row | driven test |
| REQ-005 | The New Agent Thread submenu shall list the Zed Agent and every agent `AgentServerStore::external_agents` reports, by display name | driven test with a configured custom agent |
| REQ-006 | WHEN a thread finishes or asks for confirmation while its panel is not focused, its row shall show the attention dot and `has_notifications` shall be true until the thread is opened | driven test |
| REQ-007 | WHILE the Agent Panel has focus, the selected row shall be the thread it shows | unit test on the selector + driven test |
| REQ-008 | WHILE the layout is `marley`, `is_threads_list_view_active` shall be `true` | unit test |
| REQ-009 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P2 Design** — the snapshot's thread fields, subscriptions, the status mapping, the menu
  building, the selector's new arm, the test harness reuse from `agent_ui::test_support`.
- **P3 Implement** — rows, menu, subscriptions, selection.
- **P3.5 Inspect** — critics: status mapping gaps, notification suppression side effects,
  entity re-entrancy when opening threads from a row handler.
- **P4 Validate** — driven tests, `script/gates.sh --diff`, the live drive.
- **P5 Complete** — CHANGELOG, crate note, ledger, close, archive.
