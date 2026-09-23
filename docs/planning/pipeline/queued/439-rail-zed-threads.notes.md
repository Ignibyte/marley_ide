# Zed agent threads in the rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-439-rail-zed-threads.md
- **Pipeline spec:** 439-rail-zed-threads.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-22)
- **Request:** Chad chose "Terminals and Zed threads" for the rail rows (2026-09-22), and the
  original complaint was "I cant figure out even how to start a new agent".
- **Classification / tier:** feature, medium; `marley_workbench` only, no new Zed touchpoint.
- **Recall (§18.3):** `PR-claude-single-selection-is-a-derived-selector-not-scattered-booleans-001`
  (the Agent Panel arm goes into the one selector); `PR-claude-live-refresh-selection-identity-key-must-be-unique-001`
  (key thread rows by thread id); `PR-claude-async-answer-carries-question-identity-every-hop-001`
  (a confirmation row must name the thread it belongs to).
- **Discovery:** the sidebar sweep of 2026-09-22 (plan D3, D8): thread sources, the public
  AgentPanel API, the suppression check in `conversation_view.rs:2863-2915`.
- **Human confirmation:** Chad's goal authorizes autonomous execution through commit
  (2026-09-22). No `TaskCreate` in this harness; checklists live here.

## Seams re-verified (2026-09-22, before the pair was parked for #447)
Two Explore sweeps checked every seam the spec cites at `b1c5a38a26`. Line numbers hold; the
semantics differ in ways the design must take in.
- **Thread rows.** `ThreadMetadataStore::entries_for_main_worktree_path(&PathList,
  Option<&RemoteConnectionOptions>)` (`thread_metadata_store.rs:643`) matches the group's main
  paths exactly, skips archived rows, returns drafts (`session_id: None`) and has no order.
  Zed's sidebar also looks up legacy folder paths, each workspace's roots and each linked
  worktree, dedups by `thread_id`, hides empty drafts unless active, and sorts by
  `interacted_at.unwrap_or(updated_at)`, newest first (`sidebar.rs:1639-1804`). The store
  emits no change event except `ThreadArchived`: the hook is `cx.observe` on
  `ThreadMetadataStore::global` (`sidebar.rs:900-903`). `ThreadMetadata` has no status field.
- **Live status.** `acp_thread::ThreadStatus` is only `Idle | Generating`. The four states are
  `ui::AgentThreadStatus { Completed, Running, WaitingForConfirmation, Error }`, mapped as Zed's
  sidebar does (`sidebar.rs:8078-8087`): a pending tool call
  (`ConversationView::root_thread_has_pending_tool_call`, public) → waiting; `had_error()` →
  error; `Generating` → running; else completed. Zed reads it from `panel.conversation_views()`
  and joins by `session_id`, refreshing on the panel's `AgentPanelEvent`s
  (`ActiveViewChanged`, `ActiveViewFocused`, `EntryChanged`) and on `PanelAdded`.
- **Opening a thread.** `AgentPanel::load_agent_thread(Agent, ThreadId, Option<PathList>,
  Option<SharedString>, focus, AgentThreadSource, ..)` (`agent_panel.rs:4452`) unarchives,
  never opens the dock, and skips focus when the thread is already active; Zed's click path
  activates the workspace, calls it, then `focus_panel::<AgentPanel>` (`sidebar.rs:3690-3724`).
- **New Agent Thread.** A dispatched action reaches the *displayed* workspace, so a menu entry
  that activates another project and then dispatches would miss it; Zed's sidebar calls the
  target workspace's panel directly (`sidebar.rs:7021-7033`). `agent::NewThread` is not the
  Zed Agent: it uses the panel's selected agent, or opens a terminal after a terminal. The Zed
  Agent is `NewExternalAgentThread` with `{"agent":"Zed Agent"}` (`agent::ZED_AGENT_ID`),
  whose field is private: build it with serde or `cx.build_action`, or call
  `panel.new_external_agent_thread` / `activate_new_thread`. `AgentServerStore::external_agents()`
  yields ids in hash order (sort by label as the panel does, `agent_panel.rs:5985-6023`);
  `agent_display_name` and `agent_icon` (an SVG path) are `None` for custom agents, falling
  back to the id and `IconName::Sparkle`. The store emits `AgentServersUpdated` and never
  notifies: subscribe, don't observe.
- **Attention.** Zed marks a thread only on Running → Completed while it is not the active
  panel thread, clears it when it becomes active, and shows waiting as the row's icon plus a
  header count (`sidebar.rs:1778-1841`).
- **D4 needs revisiting.** `is_threads_list_view_active` gates only OS pop-ups, sounds and
  pop-up dismissal, window-wide (`conversation_view.rs:2863-2987`, `agent_panel.rs:2920`,
  `title_bar.rs:821-831`). Returning `true` while the rail is open would silence every
  thread's pop-up in the window, visible row or not.
- **Tests.** `StubAgentConnection` lives in `acp_thread` (test-support); `AgentPanel::test_new`
  plus `workspace.add_panel`; `agent_panel::init` must run before the workspace exists (via
  `agent_ui::test_support::init_test`, which installs its own settings store and database,
  so use it instead of marley's `init_test`, not with it); the recipe is
  `sidebar_tests.rs:2059-2089`. `ThreadItem` (`ui::ThreadItem`) has one action slot shown only
  while the caller passes `hovered(true)`.
