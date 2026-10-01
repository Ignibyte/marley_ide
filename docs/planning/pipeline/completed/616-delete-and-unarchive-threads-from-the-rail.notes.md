# Delete and unarchive threads from the rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-616-delete-and-unarchive-threads-from-the-rail.md
- **Pipeline spec:** 616-delete-and-unarchive-threads-from-the-rail.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones".
- **Batch:** wave 2 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - AD-claude-605: the rail archives as Zed's history does; delete and unarchive were left to Zed's archive view.
  - A live `ConversationView` re-saves its metadata on its events, so a thread open in a panel must be closed before its delete, or it comes back.
  - Opening a thread unarchives it (`load_agent_thread`), which the rail's `open_thread` already calls.
  - L-claude-605-zed-sends-acp-request-ids-as-strings-001, for the fixture agent.
- **Discovery:** one Explore sweep for the wave (2026-09-30) over the rail, the ports scan, the
  thread store, terminal views and tooltips; the spec's Prior art cites what applies here, and the
  Plan phase re-verifies each seam at promotion.

### Promotion (Opus, 2026-09-30)
- The pair moved to `active/`; the BACKLOG row removed; the ticket in-progress.
- **Recall, again:** as queued. The brain (consultation 1af98db8a141412dbbeeefdc8951bbf9):
  nothing on this seam.
- **Seams re-verified:**
  - `ThreadsArchiveView::delete_thread` (`threads_archive_view.rs:807-850`): `store.delete`, then
    `cleanup_thread_archived_worktrees(thread_id, cx)` (public, `thread_worktree_archive.rs:858`),
    then the agent's connection (`AgentConnectionStore::request_connection(agent,
    agent.server(fs, ThreadStore::global(cx)))`, `wait_for_connection`) and
    `session_list(cx).filter(supports_delete).delete_session(session_id, cx)`.
  - `AgentPanel::remove_thread(id, window, cx)` (`agent_panel.rs:3336-3380`) drops a retained
    thread, deletes its metadata, and turns the panel to a draft when the thread was its active
    one; `AgentPanel::connection_store()` (1662).
  - `ThreadMetadata { thread_id, session_id, agent_id, title, …, archived }`,
    `main_worktree_paths()`, `matches_remote_connection(host)`; `ThreadMetadataStore::
    archived_entries()`.
  - The rail's `open_thread` (`rail.rs:2341-2376`, `load_agent_thread(…, AgentThreadSource::
    Sidebar, …)`), the thread row's menu with Archive Thread, `HeaderMenu`, and `group_threads`'
    host (`rail.rs:6099`).
  - `MultiWorkspace::workspaces()` (`multi_workspace.rs:1300`); `window.prompt` as `close_guard.rs`
    asks.

### Design
- **Approach.**
  - *Delete* (`Rail::delete_thread(thread_id, title, window, cx)`): `window.prompt(Warning,
    "Delete the thread “…”?", "Its messages are deleted, not archived.", ["Delete Thread",
    "Cancel"])`; on Delete Thread, every Agent Panel of the window's workspaces runs
    `remove_thread`, so no live conversation saves it again; then the store's `delete`, then, off
    the window's update, `cleanup_thread_archived_worktrees` and the agent's `delete_session`
    where its session list supports it, as Zed's archive view does; failures logged, as there.
  - *Archived threads*: the rail's snapshot keeps, per open group, its archived threads
    (`ArchivedThread { thread_id, agent, work_dirs, title, updated_at }`, newest first);
    `HeaderMenu` gets them and builds Archived Threads ▸ (each ▸ Open Thread, Delete Thread…).
    Open runs a `open_thread_with(workspace, thread_id, agent, work_dirs, title)` that the
    existing `open_thread` now shares.
  - *The thread row's menu* gets Delete Thread… after Archive Thread.
- **File manifest** (Marley crate only): `crates/marley_workbench/src/rail.rs`.
- **Visual check plan** (`script/e2e/616-delete-and-unarchive-threads-from-the-rail.sh`, sway, with
  #605's stand-in ACP agent): three threads, "first", "second" and "kept"; "first" archived from
  its row.

  | REQ | Set up and do | Shot |
  |---|---|---|
  | REQ-001, REQ-004 | Open "second", then right-click its row, Delete Thread…, confirm | `confirm.png`, `deleted.png`; `store.txt` (the threads database without it) |
  | REQ-002 | The project header's menu, Archived Threads | `archived.png` |
  | REQ-003 | Open Thread on "first" | `unarchived.png` |

- **Risks and decisions:**
  - A thread archived with a linked worktree it removed opens without the worktree restored; the
    menu does not mark such threads (Zed's archive view restores them).

## Phase 2 — Code (2026-09-30)
- **Built,** as designed, in `rail.rs`:
  - `ArchivedThread` and `archived_threads(key)` (`archived_entries` matched by main folders and
    host, newest first, twenty at most, with the date each was last updated); `HeaderMenu`
    carries them and `project_context_menu` now takes the whole `HeaderMenu`;
    `archived_threads_menu` (Archived Threads ▸ each ▸ Open Thread, Delete Thread…, or a disabled
    No Archived Threads).
  - `open_thread_with(workspace, ThreadToOpen, …)`, which `open_thread` now shares;
    `delete_thread(thread_id, title, …)`: the prompt, `remove_thread` on every Agent Panel of the
    window, the store's `delete`, `cleanup_thread_archived_worktrees`, and the agent's
    `delete_session` where its session list supports it.
  - The thread row's menu: Delete Thread… after Archive Thread.
- `crates/marley_workbench/Cargo.toml`: `agent` (for `ThreadStore`, which the agent's server takes)
  moved into the normal dependencies; it was a dev-dependency only.
- **Deviations:** the picker became nested menus (the spec says so); `ThreadToOpen` bundles a
  thread's fields (clippy's `too_many_arguments`).
- **Review of the diff:**
  - REQ-001: the prompt, then the delete; the row goes at the store's change.
  - REQ-002: the project's archived threads in its menu.
  - REQ-003: Open Thread loads the thread, which unarchives it.
  - REQ-004: every panel's `remove_thread` runs before the store's `delete`.
  - Provenance: the delete calls Zed's public functions (`ThreadMetadataStore::delete`,
    `cleanup_thread_archived_worktrees`, `AgentConnectionStore::request_connection`,
    `AgentSessionList::delete_session`) in the rail's own shape; no Zed function body is copied.
  - Re-entrancy: the panels are updated in the window's update after the prompt answers, not
    inside the rail's.
- **Gate:** `just gate-diff` 17 PASS, 0 FAIL (`GATE GREEN [diff]`).

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/616-delete-and-unarchive-threads-from-the-rail.sh` under sway, with
  #605's stand-in ACP agent: three threads, "first", "second" and "kept"; "first" archived from its
  row. Positions measured from the first runs.
- **Shots, each read** (the last run):
  - `threads.png`, `archived-first.png`: the three rows, then two after "first"'s Archive.
  - `kept-menu.png`: "kept"'s menu, Archive Thread and Delete Thread….
  - `confirm.png` (REQ-001): the prompt "Delete the thread "kept"? Its messages are deleted, not
    archived." with Delete Thread and Cancel, "kept" open in the Agent Panel behind it.
  - `deleted.png` (REQ-001, REQ-004): after Return, the rail lists "second" alone, and the panel
    holds no thread.
  - `project-menu.png`, `archived.png` (REQ-002): the project's menu with Archived Threads ▸, and its
    submenu with "first · Sep 30, 19:54" alone ("kept" was deleted, not archived).
  - `unarchived.png` (REQ-003): after choosing it, "first" is a row again and selected, every menu
    closed; the panel says the stand-in cannot load sessions, which is the fixture's limit (#605's
    notes).
  - `restart.png` (REQ-004): after a restart, "second" and "first" listed, "kept" not.
- **Reds found and fixed:**
  - Run 2: after the delete, a "New Agent Thread" row appeared: `AgentPanel::remove_thread` puts a
    new draft in the place of an active thread it drops. The rail now calls
    `remove_thread_without_activating_draft`.
  - Run 5: choosing Open Thread from the archived thread's own submenu left the project's menu
    open: Zed's menu dismisses a submenu's parent only when that submenu was itself clicked, so a
    choice two levels down closes one. Archived Threads is now one level, where choosing a thread
    opens it; deleting an archived thread is left to its row once back, or to Zed's archive view
    (the spec's scope says so).
  - `just gate-diff` after the fixes: 17 PASS, 0 FAIL.
- **Focus:** sway stopped with the run's Marley each time; Hyprland had no Marley windows
  before or after.

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md`; `workbench-shell.md`; `marley_workbench.md` (a Delete and archived
  threads bullet in the Threads section); the guide, the guide page (the Threads in the rail
  article) and the walkthrough (two checks in 6.1).
- **Knowledge:** F-claude-616-a-deleted-threads-panel-started-a-new-draft-001,
  F-claude-616-a-choice-two-submenus-down-left-the-menu-open-001,
  L-claude-616-zeds-context-menu-closes-cleanly-one-submenu-deep-001.
- **Brain:** consultation 1af98db8a141412dbbeeefdc8951bbf9 closed with `brain decide`.

