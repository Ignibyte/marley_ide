# An agent thread starts in a tab — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-734-an-agent-thread-starts-in-a-tab.md
- **Pipeline spec:** 734-an-agent-thread-starts-in-a-tab.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-10)
- **Request:** Chad, 2026-10-10, the agents-anywhere plan
  (`docs/planning/design-notes/agents-anywhere-2026-10-10.md`), with the goal "lets make tickets
  and build it".
- **Classification:** feature.
- **Recall (§18.3):**
  - PR-claude-697-a-hosted-view-focuses-where-its-own-host-does-001: the tab's focus is the thread's `message_editor`, never the view's root handle.
  - PR-claude-701-an-items-handlers-that-update-its-workspace-are-not-listeners-001 and PR-claude-702-making-a-workspace-in-the-background-keeps-the-focus-001.
  - #697 and #702 (completed): `ThreadTab`, `activate_for`, one rail row per thread; #697's notes record the risk of an idle thread evicted from the panel's retained threads.
  - The research of 2026-10-10: `ConversationView::new` is public; the panel's `connection_store()` is shareable; the thread store archives a thread whose project has no visible folder (`thread_metadata_store.rs:1316`); Zed's agent ignores `work_dirs` and resolves only visible worktrees.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again.

## Phase 1 — Plan (promoted 2026-10-10)
- **Pre-flight:** cargo 1.98.1, gate, e2e, shear, hooks all present; no active pipeline; README
  marker present; cargo idle; `/mnt/fast` 59 %.
- **Brain:** this repository's sessions have no `rusty` MCP server (no `.mcp.json`), so no
  `brain_ask` ran, as for #696 and #697.
- **Seams re-verified against the code:**
  - `ConversationView::new` (`conversation_view.rs:990`) takes `work_dirs` and `project` explicitly;
    `initial_state` falls back to `project.default_path_list` only when `work_dirs` is `None`
    (1245). `Agent::server(fs, thread_store)` (`agent_ui.rs:578`). `AgentPanel::{connection_store,
    selected_agent}` are `pub` (1631, 1635).
  - The panel's `create_agent_thread_inner` (`agent_panel.rs:4646`) also observes the view (its
    events, `serialize`, the sibling host): nothing the tab needs.
  - `thread_metadata_store.rs` `handle_conversation_event` (1266): `worktree_paths` is
    `project.worktree_paths(cx)` at every save unless the record is archived, and a record with
    none is archived (1316). So a fix from outside (`update_working_directories`, which also
    `debug_assert!`s the record is not archived) is overwritten at the next save: the store must
    read the thread's own folders, which `AcpThread::work_dirs()` holds.
  - `worktree_store.rs:773` `create_worktree`: a hidden worktree is scanned and watched like a
    visible one (`Worktree::local(…, scanning_enabled, …)`), and `can_trust` runs on it. Hence the
    home rule.
  - `conversation_view.rs:3118-3155` `is_visible` / `is_visible_in_agent_panel` /
    `agent_status_visible`, and the notification's accept at 3295-3326, which reveals the panel and
    calls `load_agent_thread`.
  - `rail.rs`: `new_agent_thread` (3007) → `agents::start_thread`; `agent_menu` (5143) omitted for a
    projectless group (5076); `group_threads` returns nothing for a projectless group (7519);
    `live_statuses` (7471), `live_threads` (7457, the rail's watched threads at 6422) and
    `inbox_entries` (6626) read only the panels' `conversation_views()`.
  - The e2e runs Marley with the user's own `HOME` (`launch_marley`), so a thread in Home has the
    real home as its `cwd`; nothing is written there.

### Design
- **`crates/agent_ui/src/agent_ui.rs`** (Zed crate): `pub struct MarleyThreadHost { pub shows:
  fn(&Workspace, EntityId, &App) -> bool, pub reveal: fn(&mut Workspace, ThreadId, &mut Window,
  &mut Context<Workspace>) -> bool }`, a `Global`, with a `// Marley:` comment.
- **`crates/agent_ui/src/conversation_view.rs`** (Zed crate):
  - `pub marley_own_folders: bool` on `ConversationView`, `false` in `new`;
  - `is_visible_in_agent_panel` also answers true when `MarleyThreadHost::shows` does for the
    workspace and this view;
  - the notification's accept first asks `MarleyThreadHost::reveal` for the root thread and stops
    when it brought a tab forward.
- **`crates/agent_ui/src/thread_metadata_store.rs`** (Zed crate): `handle_conversation_event` takes
  `worktree_paths` from the thread's `work_dirs()` (`WorktreePaths::from_folder_paths`) when the
  view's `marley_own_folders` is set and the record is not archived.
- **`crates/marley_workbench/src/thread_tab.rs`** (Marley crate):
  - `NewAgentThread { agent: Option<String>, folder: Option<PathBuf> }` (`Action`, `Deserialize`,
    `JsonSchema`, namespace `marley`), registered on every workspace; refused with a toast while
    AI is disabled or without an Agent Panel.
  - `start(workspace, agent, folder, window, cx)`: the folder must be a directory (a toast says
    why not); `needs_worktree` (no worktree holds it, the project is local, the folder neither is
    nor holds the home folder) → `find_or_create_worktree(folder, false)` awaited in a spawned
    task, then `open_thread` builds the view (the panel's connection store, `ThreadStore::global`
    for Zed's agent, `AgentThreadSource::Sidebar`, `marley_own_folders = true`) and adds the tab to
    the center, focused through the tab's `Focusable` (PR-claude-697).
  - `ThreadTab::new` keeps its panel subscription (the panel showing the same view closes the tab).
  - `pub(crate) fn conversations_of(workspace, cx)`: the panel's `conversation_views()` and every
    `ThreadTab`'s view not among them.
  - `MarleyThreadHost` set in `init`: `shows` is "the workspace's active item is a `ThreadTab` on
    this view"; `reveal` is `activate_for`.
  - A helper `default_folder(workspace, cx)`: the first root path, else the home folder.
- **`crates/marley_workbench/src/rail.rs`** (Marley crate):
  - `new_agent_thread` dispatches `NewAgentThread { agent, folder: None }` on the row's workspace;
  - `agent_menu` for projectless groups too;
  - `group_threads`: a thread whose tab is in another group's workspace leaves this group's rows;
    the group's own tabs' threads are added when the store's rows lack them (a draft, or a folder
    of another project), built from the view; a projectless group lists its tabs' threads;
  - `live_threads`, `live_statuses` and `inbox_entries` read `conversations_of`.
- **`docs/marley/zed-touchpoints.md`:** rows for `agent_ui.rs`, `conversation_view.rs` and
  `thread_metadata_store.rs`, written before the edits.
- **`docs/marley/guide.md`:** "An agent thread in a center tab" covers starting one, the folder, the
  home rule and the action.
- **File manifest:** the three Zed files above; `thread_tab.rs`, `rail.rs` (Marley crate); the
  ledger; the guide; the scenario.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001, 002, 004 | Home's `+` → New Agent Thread → Marley (scripted agent); "hello" | 734-01-home-thread: a tab in Home answering "Noted: hello", its row under Home; the log's `cwd` is Marley's home |
| 007 | The bound key (`marley::NewAgentThread`, agent Marley, folder `other/`) in the project; `read <other>/notes.md` | 734-02-folder: "Read: # notes from other" |
| 003, 005, 004 | The project's `+` → New Agent Thread → Marley; `read <repo>/README.md` | 734-03-project-thread: a tab in the project, "Read: # repo", its row under the project; the log's `cwd` is the repo |
| 006, 008 | — | The review of the diff |

The scripted agent of #697, extended: `read <path>` sends `fs/read_text_file` to Marley and
answers with the file's first line, or with the error Marley gave.

### Risks
- **Worktree trust.** `can_trust` runs on the hidden worktree; if Zed asks to trust the folder, the
  scenario shows it and the design must answer it (a hidden worktree inside a trusted project, or
  Zed's own prompt).
- **Hidden worktrees stay** until the workspace closes; a tab's close does not remove one, since an
  editor may hold a buffer from it.
- **Moving a tab-born thread to the panel** loads it from the agent's session as a new view (#697's
  evicted case); a running turn in the tab's view ends with the tab.
- **The Agent Panel's `update_thread_work_dirs`** rewrites the work dirs of its own threads when the
  project's worktrees change; a tab-born thread is not the panel's, and a #697 tab's thread still is.

## Phase 2 — Code
- **Checklist** (no TaskCreate here): ledger rows ✓, `agent_ui.rs` ✓, `conversation_view.rs` ✓,
  `thread_metadata_store.rs` ✓, `thread_tab.rs` ✓, `rail.rs` ✓, guide ✓, gate ✓.
- **Built:**
  - `docs/marley/zed-touchpoints.md`: rows for `agent_ui.rs`, `conversation_view.rs` and
    `thread_metadata_store.rs`, written before the edits.
  - `agent_ui.rs`: `MarleyThreadHost { shows, reveal }`, a `Global`.
  - `conversation_view.rs`: `marley_own_folders`; `is_visible_in_agent_panel` answers true when
    `shows` does; the notification's accept returns after `reveal` brought a tab forward.
  - `thread_metadata_store.rs`: a view with `marley_own_folders` is filed under the thread's
    `work_dirs()`.
  - `thread_tab.rs`: `NewAgentThread { agent, folder }` (a data action read through a fields
    struct, as `rusty::OpenPage`); `start` (folder check, hidden worktree, then `open_thread`);
    `open_thread` (the panel's connection store, `ThreadStore::global` for Zed's agent,
    `AgentThreadSource::Sidebar`); `default_folder`; `needs_worktree`; `conversations_of`;
    `tab_threads`; `shows_view`; `MarleyThreadHost` set in `init` (now `&mut App`); the tab keeps
    the hidden worktree it made.
  - `rail.rs`: `new_agent_thread` → `thread_tab::start`; New Agent Thread in every `+`;
    `group_threads` lists a tab's thread under the tab's group only, drafts included, and a
    projectless group's tab threads; `tab_homes`; `live_threads`, `live_statuses` and
    `inbox_entries` read `conversations_of`.
  - `docs/marley/guide.md`: "An agent thread in a center tab" (starting one, any folder, how the
    folder is reached, notifications) and the group `+` sentence.
- **Deviations:**
  - The tab holds the hidden worktree (`ThreadTab::_folder`): a project holds a hidden worktree
    only weakly (`WorktreeStore` keeps a strong handle for visible ones), so the first visual
    run's worktree was gone before the agent read (`Resource not found`). Holding it also drops it
    when the tab closes, which settles the plan's "hidden worktrees stay" risk.
  - `start` takes `&Window`, not `&mut Window` (clippy).
- **Review of the diff:**
  - REQ-001/005: both `+` menus reach `start`; REQ-002: no folder → `default_folder`; REQ-007: the
    worktree is awaited and held before the view exists.
  - REQ-004: a thread tabbed elsewhere leaves a group's rows; a draft tabbed here stays (the first
    draft filter dropped it after `seen` marked it: fixed before the gate).
  - REQ-006: the store reads `marley_own_folders` on the view it was given, inside the non-archived
    branch, so an archived record keeps its folders as before.
  - REQ-008: `shows` reads the workspace's active item, no entity is updated; `reveal` is
    `activate_for`, run inside the workspace's own update.
  - Re-entrancy: `start` defers the worktree and the view to a spawned task; `open_thread` runs in
    `update_in`. Provenance: no Warp; the view's construction follows Zed's public constructor,
    no Zed function body copied into Marley. Upstream: three additive hunks, each with a
    `// Marley:` comment and its row.
- **Gate:** `734-gate-1.log` RED (gate:1 formatting, gate:2 clippy: `too_many_lines` in
  `build_snapshot`, `clone_on_ref_ptr`, `needless_pass_by_ref_mut`). Fixed at the source:
  `tab_homes` split out, `Arc::clone`, `&Window`; formatted. `734-gate-2.log`: GATE GREEN [diff], 17 passed.

## Phase 3 — Test
- **Scenario:** `script/e2e/734-an-agent-thread-starts-in-a-tab.sh`, under `compositor sway`, on
  the debug build. The Marley entry runs a scripted ACP agent (`MARLEY_ASSISTANT_ADAPTER`) that
  answers "Noted: <prompt>", answers `read <path>` with Marley's `fs/read_text_file` reply, and
  logs every message. A key binds `marley::NewAgentThread` with the Marley agent and `other/`.
  Before typing into any thread, the run checks the scripted agent started one more session
  (PR-claude-687).
- **Runs before the gate's green** (they shaped the scenario and found one bug):
  - `shots-734a` (layout only): Home's `+` at (236, 89), the project's at (236, 136).
  - `shots-734b` (red, REQ-007): `read <other>/notes.md` answered "Read failed: Resource not
    found"; the log's worktree diagnostics listed one live worktree, the project's. A project
    keeps a hidden worktree only weakly, so the one `start` made was dropped at once. Fixed: the
    tab holds it (Phase 2's deviation), rebuilt.
  - `shots-734c` (red, REQ-002): the Home step's check failed; the scenario typed before the
    check, so `shots-734d` printed the folders: only two sessions reached the scripted agent.
  - `shots-734d`: Home's submenu lists Zed Agent, Claude Agent, Marley, one more than the
    project's, so two steps chose **Claude Agent**, the user's own Claude Code. The run typed
    "hello" into it and it answered "Hi Chad.": one real turn on Chad's login, nothing else
    touched. The scenario broke PR-claude-687 there. Fixed in the scenario: the Home step takes
    three steps, shoots the submenu, and every thread is checked to run the scripted agent before
    anything is typed.
  - `shots-734e`: every check passes.
- **The Test phase's run (`shots-734-test`), after `just build` (no-op) and `734-gate-2.log`
  green: every check passes.**
  - **734-00-layout:** the rail with Home and repo, each header with its `+`; repo's terminal.
  - **734-03-project-thread (REQ-003, REQ-004, REQ-005):** a center tab in repo, "read …/repo/
    README.md" answered "Read: # repo"; the rail lists the thread under repo, "read /mnt/fast/
    tmp/claud… · Marley · idle", marked. The checks: the session's `cwd` is the repo, and the
    agent log holds the file's content.
  - **734-02-folder (REQ-007):** the bound key's thread in a tab, "read …/other/notes.md"
    answered "Read: # notes from other", through the hidden worktree; the rail lists both
    threads under repo, where their tabs are. The check: the session's `cwd` is `other/`.
  - **734-01a-home-menu (REQ-001):** Home's `+` menu has New Agent Thread, its submenu Zed Agent,
    Claude Agent and Marley, Marley chosen.
  - **734-01-home-thread (REQ-001, REQ-002, REQ-004):** Home shown, a "hello" tab beside Home's
    page answering "Noted: hello"; the rail lists "hello · Marley · idle" under Home, marked, and
    the repo's two threads stay under repo. The check: the session's `cwd` is Marley's home
    folder.
  - **REQ-006, REQ-008:** the review of the diff (Phase 2).
  - Focus report: one Marley window before and after on Chad's Hyprland, no rule added.
- **Seen, not in scope:** after the project thread read its README, an editor tab of that README
  opened in the project's center: Zed's "follow the agent" (`Workspace::handle_agent_location_
  changed`), which opens the file an agent reads while the workspace follows it, as for a panel
  thread.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added: An agent thread starts in a tab, anywhere); the
  architecture note `docs/marley_architecture/marley_workbench.md` ("A thread that starts in a
  tab"); the slice line in `docs/marley/workbench-shell.md`; the guide came with Phase 2. The three
  `zed-touchpoints.md` rows (`agent_ui.rs`, `conversation_view.rs`, `thread_metadata_store.rs`)
  describe what shipped.
- **Knowledge appended:** F-claude-734-a-hidden-worktree-was-dropped-before-the-agent-read-001,
  F-claude-734-a-scenario-typed-into-the-users-own-claude-001,
  PR-claude-734-whoever-makes-a-hidden-worktree-holds-it-001,
  L-claude-734-zed-scans-a-hidden-worktree-whole-001,
  AD-claude-734-a-thread-marley-starts-is-filed-under-its-own-folders-001.
- **Brain:** this repository's sessions have no `rusty` MCP server, so no brain loop ran.
- **Ticket:** closed (`tickets/closed/`); its BACKLOG row left at promotion.
- **Gate:** `734-gate-2.log`, GATE GREEN [diff], 17 passed, on the tree committed.
