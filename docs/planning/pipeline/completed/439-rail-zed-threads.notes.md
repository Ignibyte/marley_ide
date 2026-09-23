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

## Phase 1 — Plan (promoted 2026-09-22, after #447)
- **Checklist** (no `TaskCreate` in this harness): [x] pick · [x] pre-flight · [x] recall ·
  [x] promote and re-verify · [x] prior-art sweep · [x] spec · [x] design · [x] present (autonomous).
- **Pre-flight:** cargo 1.98.1, the gate, llvm-cov 0.9.0, nextest 0.9.143, shear 1.13.4, hooks
  wired, no active pipeline, README marker present, cargo idle.
- **Recall.**
  - `PR-claude-single-selection-is-a-derived-selector-not-scattered-booleans-001`: the thread
    arm goes into the one `selection()` function, with the clamp to a listed, expanded row.
  - `PR-claude-live-refresh-selection-identity-key-must-be-unique-001`: rows keyed by
    `ThreadId`, never by title or position (D8).
  - `PR-claude-state-another-entity-reads-is-kept-outside-render-001` and
    `F-claude-438-a-a-sidebar-flag-read-a-value-only-render-wrote-001`: `has_notifications`
    reads the stored snapshot, which now carries thread attention too.
  - `PR-claude-a-marley-crate-writes-from-the-contract-not-the-gpl-body-001`: the thread
    collection and the status mapping are written from the stores' and the panel's public
    contracts in the rail's own shape (pure functions in `marley_rail`), not after
    `sidebar.rs`.
  - Brain: consultation `563eb50299964dc58e333be51f661abc`, nothing on this seam (a duplicate
    ask, `935da436…`, was closed with `no-decision`).
- **Re-verified at promotion.** No Zed crate this ticket uses changed since `b1c5a38a26`
  (`git diff` over `agent_ui`, `sidebar`, `acp_thread`, `project`, `ui`, `workspace`,
  `title_bar`, `agent`, `agent_servers` and `agent_settings` is empty), so the cited lines
  hold. On the Marley side, #447 changed the rail's shape:
  - `Rail::new(&Entity<MultiWorkspace>, Option<KeptSidebar>, &Window, cx)`;
  - `register_sidebar(&Entity<MultiWorkspace>, ..)`;
  - the row renderers are associated functions taking `&Context<Self>`;
  - element ids come from `EntityId` and `u64`;
  - `unused_results` is allowed in the crate.

  The design below builds on that shape.
- **D4 resolved.** Kept `false`. The reasons are in the spec's D4.

### Design
- **Pure model (`marley_rail`).**
  - `ThreadSnapshot { key: String, title: String, agent: String, status: ThreadStatus,
    attention: bool }`, and `ProjectSnapshot.threads: Vec<ThreadSnapshot>`, sorted by the
    collector.
  - `ThreadStatus { Done, Running, Waiting, Error }` (default `Done`).
    `thread_status(waiting_for_confirmation, errored, generating)` holds the precedence.
  - `thread_attention(previous: Option<ThreadStatus>, current, shown, noted) -> bool`: false
    when shown; otherwise true when already noted, or when a run just ended (previous
    `Running`, current `Done` or `Error`).
  - `Focus.thread: Option<String>`: set only while the displayed workspace's Agent Panel
    holds focus. `Selection::Thread(String)`. `selection()` tries the thread arm first
    (expanded and listed), then the #438 arms.
  - `Row::Thread(ThreadRow { project, key, title, agent, status, attention, selected })`,
    after a project's terminals.
  - `ProjectRow.attention`: folded, and a bell, or a thread's attention or wait.
    `has_attention`: any bell, attention or wait, folded or not.
- **Collector (`marley_workbench::rail`).** `build_snapshot` gains thread collection per
  listed group:
  - the three lookups, deduplicated by thread id;
  - drafts kept only when they are a member panel's `active_thread_id`;
  - live status from each member workspace's `AgentPanel::conversation_views()`, joined to
    rows by the conversation's root thread session id, as the metadata stores it (a draft's
    session is `None`, so a draft is never live);
  - the sort, newest first.

  `Snapshot.threads: HashMap<String, ThreadEntry { workspace, thread_id, agent: Agent,
  work_dirs: PathList, title, icon }>` holds what a click and the render need. The rail keeps
  `thread_statuses: HashMap<String, ThreadStatus>` (last seen) and a `noted` set, fed through
  `thread_attention`. "Shown" is the displayed workspace's panel being visible
  (`AgentPanel::is_visible`) on that thread (`active_thread_id`).
- **Subscriptions.** `refresh` and `sync_subscriptions` take the window: the existing ones move
  to `subscribe_in` and `observe_in`. New ones:
  - `observe_in` on `ThreadMetadataStore::global` (the store notifies and emits nothing
    else);
  - per workspace Agent Panel, `subscribe_in` for `AgentPanelEvent`, plus `on_focus_in` and
    `on_focus_out` on its focus handle;
  - per project, `subscribe_in` on its `AgentServerStore` for `AgentServersUpdated`;
  - per live root thread, a subscription to its `AcpThread` events. It must refresh on a
    status change without redoing the whole snapshot for every streamed chunk: only the
    events that can change status, found in Code.

  All are rebuilt from what exists, as W2's are.
- **Render.** Thread rows use `ui::ThreadItem` (title, agent icon, `status`, `notified`,
  `selected`, `on_click`), indented under the project. Element and debug selectors use the
  thread key.
- **Actions.**
  - Opening a thread: activate the workspace in the `MultiWorkspace`, then
    `panel.load_agent_thread(agent, thread_id, Some(work_dirs), title, true,
    AgentThreadSource::Sidebar, ..)`, then `workspace.focus_panel::<AgentPanel>`.
  - New Agent Thread: build `NewExternalAgentThread` from JSON (`{"agent": id}`; its field is
    private), then `panel.new_external_agent_thread`, then `focus_panel`.
  - The submenu's entries: `Agent::NativeAgent.id()` first, then the sorted external agents.
    Names come from `agent_display_name`, else `AgentRegistryStore`, else the id; icons from
    `agent_icon`, else the registry, else `Sparkle`.
- **Manifest.** `marley_workbench` gains `agent_ui`, `acp_thread` and `serde_json` as normal
  dependencies. `agent_ui` is GPL, and AD-claude-438-marley-crates-may-link-zeds-gpl-crates-001
  covers linking it; the crate stays MIT OR Apache-2.0.
- **File manifest.** All Marley-owned; no Zed touchpoint.
  - `crates/marley_rail/src/marley_rail.rs` and its tests (the pure model above);
  - `crates/marley_workbench/src/rail.rs` (collector, subscriptions, render, actions);
  - `crates/marley_workbench/src/rail_tests.rs` (driven tests);
  - `crates/marley_workbench/src/marley_workbench_tests.rs` (an agent-ready harness);
  - `crates/marley_workbench/Cargo.toml`;
  - docs: `docs/marley_architecture/marley_rail.md`, `marley_workbench.md`,
    `docs/marley/workbench-shell.md` (W3's status) and `CHANGELOG.md`.

### Test plan
| REQ | Test |
|---|---|
| 001 | `marley_rail` unit: thread rows follow terminal rows, only under an expanded group, in the given order. Driven: seeded metadata (a main-path row, a legacy folder-path row, an archived row, a row for another group) lists the right rows in the right order |
| 002 | unit: `thread_status` precedence over all eight input combinations. Driven: a stub connection's thread shows running while generating and done after, with the row redrawn by the status change alone |
| 003 | driven: clicking a thread row in a project the window does not show displays that workspace and opens the thread, focused, in its Agent Panel |
| 004 | driven: New Agent Thread > the stub agent in a second project's `+` starts a thread in that workspace's panel, focuses it, and its row appears |
| 005 | driven: two custom agents configured (one with a display name) plus the Zed Agent, listed Zed Agent first and then by name without regard to case |
| 006 | unit: `thread_attention` over the transitions (running→done, running→error, done→done, shown, noted). Driven: a thread finishing while another project is shown lights its dot and `has_notifications`; showing it clears both |
| 007 | unit: a waiting thread lights a folded header and `has_attention`. Driven: a stub tool call that needs authorization shows the waiting status and `has_notifications` |
| 008 | unit: the selector's thread arm (listed and expanded wins; folded or unlisted falls back). Driven: focusing the Agent Panel selects its thread's row; focusing the center terminal selects the terminal's row |
| 009 | driven: `is_threads_list_view_active` is false with thread rows listed |
| 010 | `script/gates.sh --diff` |

The live drive needs input (spec, UI proof). No path is unreachable by a driven test.

### Risks
- **Streaming churn.** A thread emits an event per streamed chunk. Rebuilding on each is wasted
  work, though harmless for redraws (an unchanged pure snapshot does not notify). The Code
  phase subscribes only to the events that change status, and measures.
- **Re-entrancy.** Row handlers run inside the rail's listener. The `MultiWorkspace`,
  workspace and panel updates they make are other entities, so this is safe, and the events
  they emit arrive after the handler returns. W2's clicks already work this way.
- **Harness.** `agent_ui::test_support::init_test` installs its own settings store and
  database, so the agent-ready harness replaces `init_test` for these tests rather than
  running after it. A unique metadata database per test (`TestMetadataDbName`) keeps parallel
  tests apart.
- **The joins.** A live conversation is joined to its row by the root thread's session id,
  never by title. `ConversationView::parent_id` names the conversation's root thread and is
  used for `is_retained_thread`, as the panel does.

## Phase 2 — Code
- **Checklist** (no `TaskCreate` in this harness): [x] `marley_rail` pure model · [x] collector ·
  [x] subscriptions · [x] rows · [x] menu and actions · [x] manifest · [x] tests · [x] review.
- **Built.**
  - `marley_rail`: `ThreadSnapshot`, `ThreadStatus`, `thread_status` (waiting over error over
    running), `thread_attention`, `Focus.thread`, `Selection::Thread`, `Row::Thread` after a
    project's terminals, and the folded-header and `has_attention` rules extended to a
    thread's dot or wait. `Focus` and `Selection` lose `Copy`, since a thread key is a
    `String`. 15 unit tests, 7 of them new.
  - `marley_workbench::rail`:
    - thread collection per group (`group_threads`: the three lookups, drafts only while
      active, newest first);
    - live status by thread id (`live_statuses`);
    - agent icons and the submenu's choices (`agent_icon`, `agent_choices`), with registry
      fallbacks;
    - `ThreadEntry` in the snapshot, and the attention bookkeeping (`note_ended_runs`);
    - thread rows through `ui::ThreadItem`, a thread row's click (`open_thread`), and New
      Agent Thread (`new_agent_thread`, the action built from JSON);
    - window-aware subscriptions (`subscribe_in` / `observe_in`), plus panel events and focus,
      live threads' status events (`changes_the_row`), agent servers and the metadata store.

    `is_threads_list_view_active` stays `false`, now with its reason.
  - The manifest gains `acp_thread`, `agent_ui` and `serde_json`, and the dev-dependencies
    `agent-client-protocol`, `chrono` and `menu`. The harness gains `init_agent_test`.
  - 11 driven tests in `rail::tests::threads`: listing and order, live status, click to open,
    New Agent Thread from the menu, the submenu's order, attention until shown, waiting,
    focus choosing the row, registry names and icons, the status mapping, and the
    threads-list flag.
- **Deviations from the design.**
  - `ThreadSnapshot` carries no agent: a thread's agent never changes, so the icon lives in
    the gpui-side `ThreadEntry` only.
  - Live status joins by `ConversationView::parent_id`, the conversation's `ThreadId`, rather
    than by session id: the same join with one type fewer, and drafts need no special case.
  - `Rail::new` takes `&mut Window` again (#447 had narrowed it to `&Window`), because
    `observe_in` needs it.
  - Two things from clippy and coverage:
    - clippy's `too_many_lines`: a `resubscribe` helper now carries the
      keep-or-make-then-drop pattern all six subscription maps share;
    - line coverage (L-claude-438-the-coverage-floor-counts-lines-per-function-001): the
      status mapping moved into `ui_status` with a test of all four arms; `live_statuses`
      filters conversations without a thread instead of `continue`; `agent_menu` defaults an
      empty choice list instead of returning early.
  - Test mechanics:
    - A popover menu takes focus two frames after it opens (`on_next_frame`). Test windows
      run those only on a platform frame request, which a driven test cannot send. So the
      submenu is opened with the pointer, clicking the row under New Terminal, not the
      keyboard.
    - Focus lands in the Agent Panel only once its dock is open (`focus_panel`); focusing the
      handle of an undrawn panel does nothing.
- **Review of the diff.**
  - Correctness per REQ: each is exercised by a test above.
  - Re-entrancy: row handlers run inside the rail's listener and update only the
    `MultiWorkspace`, the workspace and the panel. The events those raise (store notify,
    panel events, focus) arrive after the handler returns.
  - Provenance: the collector is an iterator chain over the store's documented lookups (main
    paths, folder paths, member roots), not the sidebar's four-loop body. It drops the
    legacy linked-worktree lookup (spec Out) and keeps its own draft rule. Status precedence
    and attention are the rail's own pure functions.
  - Upstream discipline: no Zed file changed apart from `Cargo.lock`, whose row covers the
    Marley crates' dependencies.
  - Nothing found that needed a fix beyond the coverage reshapes above.
- **Checks.** Clippy is clean at deny level on `marley_rail` and `marley_workbench` with all
  targets. `cargo nextest run -p marley_rail -p marley_workbench`: 53 passed. `cargo llvm-cov`
  over the two crates: 100.00% of lines (546 in `marley_rail.rs`, 139 in `marley_workbench.rs`,
  923 in `rail.rs`). `cargo fmt --all --check` is clean.

## Phase 3 — Test
- **Checklist** (no `TaskCreate`): [x] REQ-001 to REQ-009, each with its row in the test plan ·
  [x] the gate · [x] the live drive (recorded below: not run, and why).
- **Tests** (written in Code, run here).
  - `marley_rail`: 7 new units cover status precedence (all eight inputs), the attention
    transitions, thread rows after terminals, the folded header's dot or wait, the rail's
    attention, the selector's thread arm, and exactly one selected row with threads listed.
  - `marley_workbench::rail::tests::threads`, 11 tests:
    - `threads_list_under_their_project_newest_first` (REQ-001: a legacy row found by folder
      path; archived and inactive-draft rows left out; another project's row under it; thread
      rows after the terminal; every row drawn);
    - `a_live_threads_row_follows_its_status` (REQ-002);
    - `a_thread_row_opens_its_thread_in_its_projects_agent_panel` (REQ-003, and REQ-008's
      thread arm after a click);
    - `new_agent_thread_starts_one_in_that_projects_panel` (REQ-004);
    - `the_agent_submenu_lists_the_zed_agent_then_agents_by_name` (REQ-005);
    - `a_run_that_ends_unseen_lights_the_dot_until_the_thread_is_shown` (REQ-006);
    - `a_thread_waiting_for_a_confirmation_needs_the_user` (REQ-007);
    - `focus_picks_the_thread_row_or_the_terminal_row` (REQ-008);
    - `agents_take_their_name_and_icon_from_the_registry` (the registry fallbacks, and the SVG
      icon in both a row and the menu);
    - `each_rail_status_draws_as_zeds_thread_status`;
    - `the_rail_does_not_claim_the_threads_list` (REQ-009).
- **The gate** (REQ-010).
  - The first run was red at gate:9. cargo-shear found `menu` unused: the tests had moved
    from keyboard to pointer to open the submenu. Removed.
  - The second run passed all 18 gates but failed the receipt step. Dropping the dependency
    meant the run's first cargo command rewrote `Cargo.lock` mid-run, so the tree at the end
    was not the one the run started on. That is #447's REQ-011 doing its job.
  - The third run: `script/gates.sh --diff`, scope the seven Marley crates with
    `marley_rail` and `marley_workbench` touched; **19 passed, GATE GREEN [diff]** after
    60 s. gate:3: 326 tests passed. gate:4: 53 tests; TOTAL 1,608 lines, 0 missed, 100.00%.
    The receipt matches `gate_state_hash`.
- **Live drive: not run.** It would need no input: a throwaway `--user-data-dir`, seeded
  `sidebar_threads` rows, and a screenshot of the rail's thread rows. `rusty headless status`
  showed Chad's Microsoft Teams fullscreen on workspace 3, next to Nautilus and Sublime. The
  headless output takes workspace 3 when it comes up
  (L-claude-438-the-headless-output-borrows-one-of-chads-workspaces-001), so the drive would
  move his windows off his monitor while he uses the machine, and it risks capturing his
  chat. What the drive would add is the look of `ThreadItem` inside the rail: its indent,
  background and truncation. What the rows do is proven by the driven tests above, which draw
  and click them in a gpui window. The pixel check is owed to the next moment Chad is away
  from the desk, or to a compositor of our own.
- **Pre-existing — not in scope:** none.

## Phase 4 — Complete
- **Checklist** (no `TaskCreate`): [x] document · [x] capture knowledge · [x] close the ticket ·
  [x] archive · [x] commit.
- **Documented.**
  - `CHANGELOG.md` Added: Zed agent threads in the rail.
  - `docs/marley_architecture/marley_rail.md`: the thread model, `thread_status`,
    `thread_attention`, and the test count.
  - `docs/marley_architecture/marley_workbench.md`: a Threads section, the test harness, and
    the known limits.
  - `docs/marley/workbench-shell.md`: W3 shipped as #439.
  - The only path touched outside the owned set is `Cargo.lock`, whose row ("the Marley
    crates and their dependencies") still describes it.
- **Knowledge appended.**
  - `L-claude-439-a-popover-menu-takes-focus-only-on-a-platform-frame-001`
  - `L-claude-439-focus-lands-in-a-dock-panel-only-once-its-dock-is-open-001`
  - `L-claude-439-a-manifest-edit-makes-the-next-gate-rewrite-cargo-lock-001`
  - `L-claude-439-the-agent-panel-test-recipe-001`
  - `AD-claude-439-the-rail-does-not-claim-zeds-threads-list-001`
  - `AD-claude-439-focus-decides-between-a-terminal-row-and-a-thread-row-001`

  No F block: Code and Test found no bug in shipped code.
- **Brain.** Consultation `563eb50299964dc58e333be51f661abc` closed with
  `decisions/the-marley-rail-lists-zed-agent-threads-without-claiming-zeds-thread-list`,
  follow-up by 2026-10-15.
- **Ticket.** #439 moved to `tickets/closed/`. Its backlog row left at promotion.
