---
pipeline_id: 357bfe50-ee46-4be2-8bf2-4a4854ed1745
ticket: docs/planning/tickets/open/TICKET-439-rail-zed-threads.md
status: Phase 4 — Complete PASS
title: Zed agent threads in the rail
type: feature
slice: workbench shell W3
references: [docs/marley/workbench-shell.md, docs/planning/pipeline/completed/438-marley-layout-and-rail.spec.md]
---

## Title
List each project's Zed agent threads in the rail under its terminals, with live status, and
open a thread in the right-hand Agent Panel on click. The project's `+` gains New Agent
Thread for the Zed Agent and every configured ACP agent, so starting an agent is one visible
click.

## Scope
### In
- Thread rows per listed project group, after its terminal rows. The rows come from
  `ThreadMetadataStore`:
  - the group's main worktree paths (`entries_for_main_worktree_path`);
  - the same paths as folder paths, for rows written before main paths existed
    (`entries_for_path`);
  - each member workspace's root paths.

  Rows are deduplicated by thread id; archived threads never appear. A draft appears only while
  it is its panel's active thread, so a new thread shows at once. Rows run newest first, by
  last interaction, else last update.
- Each row shows the thread's title, its agent's icon, and a status read from the live
  conversations in its workspace's Agent Panel (`AgentPanel::conversation_views`): waiting for
  a confirmation, error, running, or done. A thread with no live conversation shows as done.
- Click: display the thread's workspace, `AgentPanel::load_agent_thread` with focus, then
  `focus_panel::<AgentPanel>`; the panel sits on the right in the Marley layout.
- The project `+` menu gains a New Agent Thread submenu. It lists the Zed Agent first, then
  every agent `AgentServerStore::external_agents` reports, sorted by display name, as the
  Agent Panel's own menu shows them. The name comes from the store, else the agent registry,
  else the id; the icon likewise, else `Sparkle`. Choosing one displays the project and calls
  that workspace's panel directly (`new_external_agent_thread`), because a dispatched action
  would reach whichever workspace the window shows.
- Attention: a run that ends (running to done, or to error) while its thread is not shown
  lights the row's dot. "Shown" means the displayed workspace's visible Agent Panel is on that
  thread. The dot counts in `has_notifications` and clears once the thread is shown. A thread
  waiting for a confirmation shows the waiting status and counts in `has_notifications`, as the
  dot does, and a folded project header shows its dot for a hidden thread's attention or wait.
- Selection: while the displayed workspace's Agent Panel has focus, the thread it shows is the
  selected row. Otherwise the #438 order holds: the active center terminal, then the project
  header.
- The rail rebuilds on `ThreadMetadataStore` changes, on each Agent Panel's events and focus
  changes, on status changes of the live threads, and on `AgentServersUpdated`.
- `is_threads_list_view_active` stays `false` (D4).

### Out (explicitly deferred)
- Archive, delete, history, rename and worktree archive/restore: they stay in the Agent Panel's
  own views (Zed keeps that orchestration private to its sidebar, `sidebar.rs:5338-5712`).
- Drafts that are not their panel's active thread; Zed's agent-panel Terminal Threads;
  cross-window thread activation; legacy threads stored under a linked worktree's own path
  before main worktree paths existed.
- Reporting `is_threads_list_view_active = true` (D4).
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
- **Re-swept at promotion (2026-09-22).** None of these crates changed since `b1c5a38a26`, so
  every line cited here and in the notes holds. Newly adopted:
  - `project::AgentRegistryStore` (the agent menu's name and icon fallbacks) and
    `AgentPanel::active_thread_id`, `is_retained_thread` and `AgentPanel::is_visible`;
  - `AgentThreadSource::Sidebar`, the source Zed's own sidebar reports;
  - `ConversationView::parent_id` and `root_thread_has_pending_tool_call`;
  - gpui's `on_focus_in` and `on_focus_out` on the panel's stable focus handle (D5);
  - `agent_ui::test_support` (`init_test`, `open_thread_with_connection`, `send_message`,
    `active_thread_id`) with `AgentPanel::test_new` and the `TestMetadataDbName` global for
    an isolated metadata database per test.

## UI proof
UI-AFFECTING.
- **Driven tests:** thread rows appear for seeded metadata under the right project; a stub
  connection's status change updates the row; a click opens the thread in the Agent Panel;
  New Agent Thread adds a row; the attention dot and `has_notifications` follow an unfocused
  finish; the selected row follows Agent Panel focus. Zed's `agent_ui::test_support`
  (`StubAgentConnection`, `open_thread_with_connection`) drives the threads.
- **Live drive:** in the Marley layout, New Agent Thread > Claude Code (ACP, Chad's
  `claude-acp`), send a prompt, screenshot the row going from running to done; click it after
  switching projects and screenshot the Agent Panel on the right. The drive needs clicks and
  keys. With no compositor of its own on the box, it runs only while Chad is away from the desk
  (L-claude-438-the-headless-output-borrows-one-of-chads-workspaces-001); otherwise the Test
  phase records why it did not run.

## Locked-In Decisions
- D1 — Thread rows follow a group's terminal rows.
- D2 — Opening a thread uses the Agent Panel; no center-pane thread item in this ticket.
- D3 — Thread management (archive, history, rename) stays in the Agent Panel.
- D4 (revised at promotion, 2026-09-22) — The rail keeps `is_threads_list_view_active =
  false`. The flag defaults to `true` on the trait, and `true` makes Zed treat everything in
  the window as seen while the sidebar is open (`conversation_view.rs:2863-2987`,
  `agent_panel.rs:2913-2932`, `title_bar.rs:812-835`):
  - no OS pop-up or sound for any thread;
  - none for the Agent Panel's terminal threads, which the rail does not list;
  - the title bar's project button swaps to the recent-projects popover.

  A pop-up beside the rail's dot is noise; a finish with no signal anywhere is a loss.
  Revisit when the rail lists every kind of thread Zed would silence.
- D5 — Focus decides between a terminal row and a thread row: the thread wins only while the
  displayed workspace's Agent Panel holds focus, so the selected row is what the user is typing
  into.
- D6 — A run that ends in an error lights the dot as a finished one does (Zed marks only a
  finish); a failed run needs a look more than a clean one.
- D7 — Status precedence (waiting over error over running) and the attention transition are
  pure functions in `marley_rail`; `marley_workbench` only collects the inputs.
- D8 — A thread row's identity is `ThreadId::to_key_string()`
  (PR-claude-live-refresh-selection-identity-key-must-be-unique-001).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The rail shall show a row for each non-archived thread whose metadata matches a listed group's main worktree paths, its folder paths or a member workspace's root paths, under that group after its terminal rows, newest first, with a draft listed only while it is its panel's active thread | unit test on the rows + driven test with seeded metadata (a legacy row, an archived row, two groups) |
| REQ-002 | WHILE a thread's conversation is live, its row shall show waiting, error, running or done by that precedence, and shall update when the status changes | unit test on the precedence + driven test with a stub connection |
| REQ-003 | WHEN a thread row is clicked, the thread's workspace shall be displayed and the thread shall open, focused, in its Agent Panel | driven test |
| REQ-004 | WHEN an agent is chosen under New Agent Thread in a project's `+` menu, a new thread for that agent shall start, focused, in that project's Agent Panel and appear as a row | driven test |
| REQ-005 | The New Agent Thread submenu shall list the Zed Agent first, then every agent `AgentServerStore::external_agents` reports, by display name, sorted without regard to case | driven test with configured custom agents |
| REQ-006 | WHEN a thread's run ends (running to done or to error) while the thread is not shown, its row shall show the attention dot and `has_notifications` shall be true until the thread is shown | unit test on the transition + driven test |
| REQ-007 | WHILE a thread waits for a confirmation, its row shall show the waiting status, `has_notifications` shall be true, and a folded project header shall show the dot | unit test + driven test |
| REQ-008 | WHILE the displayed workspace's Agent Panel has focus, the selected row shall be the thread it shows; WHEN focus moves to a center terminal, that terminal's row shall be selected | unit test on the selector + driven test |
| REQ-009 | The rail shall keep `is_threads_list_view_active` false | driven test |
| REQ-010 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — the design: the snapshot's thread fields, subscriptions, the status mapping, the
  menu building, the selector's new arm, the test harness reuse from `agent_ui::test_support`.
- **P2 Code** — the pure model first (`marley_rail`: thread rows, status precedence, the
  attention transition, the selector's thread arm), then rows, menu, subscriptions and
  selection in `marley_workbench`. The review of the diff checks for status-mapping gaps,
  subscription churn while a thread streams, and entity re-entrancy when a row handler opens
  a thread.
- **P3 Test** — driven tests, `script/gates.sh --diff`, the live drive.
- **P4 Complete** — CHANGELOG, crate note, ledger, close, archive, commit.
