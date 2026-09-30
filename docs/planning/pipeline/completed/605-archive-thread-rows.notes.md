# Archive an agent thread from the rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-605-archive-thread-rows.md
- **Pipeline spec:** 605-archive-thread-rows.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad chose "Archive thread rows" after asking how to close the agent conversation
  in the rail.
- **Discovery (2026-09-30):** `render_thread_row` (`rail.rs:3649-3679`) has no end slot and no
  menu; `group_threads` (`rail.rs:4838-4899`) reads `entries_for_main_worktree_path` and
  `entries_for_path`, which filter `!s.archived` (`thread_metadata_store.rs:621-654`).
  `threads_archive_view.rs:420-423` archives with `store.archive(thread_id, None, cx)`. Zed's
  sidebar's `archive_thread` (`sidebar.rs:5338+`) also plans worktree folder archives and closes
  workspaces, which the rail does not need.
- **Workaround until then:** `marley: use zed layout`, right-click the thread in the Threads
  Sidebar, Archive Thread, `marley: use marley layout`.

### Visual check plan
- As in the spec; the scripted ACP agent server from `script/e2e/508-approvals-inbox.sh` makes the threads.

### Risks
- A thread row's end slot today holds its status mark (`thread_status_mark`); the button must
  replace it on hover only, as the terminal row's bell gives way to its close button.

### Promotion (Opus, 2026-09-30, back-to-back run)
- **Pre-flight:** no active pipeline, cargo idle, README marker present, the branch level with its
  origin after #604.
- **Recall (§18.3):** #452 (the terminal row's close button over its bell, and its right-click
  menu), the pattern the thread row takes. #501's scenario is the smallest one that makes an
  agent thread (a stand-in ACP agent in the run's settings, then the rail's `+` → New Agent
  Thread; #598 left New Agent Thread's place in that menu). Brain: nothing on this seam; the
  consultation is noted at Complete.
- **Seams re-verified:**
  - `ThreadMetadataStore::archive(thread_id, None, cx)` (`thread_metadata_store.rs:858`) sets
    `archived`, keeps it, and emits `ThreadArchived`. Zed's history view calls it with no job
    (`threads_archive_view.rs:420-423`).
  - The Threads Sidebar's hover button is `IconButton::new("archive-thread", IconName::Archive)`
    with the tooltip "Archive Thread" (`sidebar.rs:6375`).
  - `ThreadEntry::thread_id` (a `Copy` `ThreadId`) is at hand in `render_thread_row`.
  - `thread_status_mark` is the row's end today.
  - The rail reads the store through `try_global` and refreshes on its notify.

### Design
- **Approach:**
  - `render_thread_row` builds an end slot as the terminal row does. The Archive button
    (`IconName::Archive`, tooltip "Archive Thread") shows on hover, and the status mark is laid
    over the same place, hidden on hover (`group_hover(ROW_GROUP, invisible)`). The button's
    click stops propagation and runs `archive_thread(thread_id)`.
  - The card is wrapped in a `right_click_menu` with one entry, Archive Thread, which does the
    same.
  - `archive_thread` updates `ThreadMetadataStore::try_global(cx)` with `store.archive(thread_id,
    None, cx)`. The store's notify refreshes the rail, and `entries_for_path`, already skipping
    archived threads, drops the row.
- **File manifest:** `crates/marley_workbench/src/rail.rs` (Marley);
  `script/e2e/605-archive-thread-rows.sh` (Test). No Zed path.
- **Visual check plan:** as the spec's UI proof. Each criterion has its shot; nothing is out of a
  scenario's reach.
- **Risks:**
  - `render_thread_row` grows by the menu; a `thread_context_menu` helper keeps it under
    `too_many_lines`, as #602 did for the terminal row.
  - A thread's first prompt is what lists it in the rail. The scenario sends each one a prompt.

## Phase 2 — Code (2026-09-30)
- **Built:** in `rail.rs`:
  - `render_thread_row` has an end slot: an Archive `IconButton` (tooltip "Archive Thread") shown
    on hover, and the status mark laid over it and hidden on hover.
  - The card sits in a `right_click_menu` with Archive Thread.
  - `archive_thread(thread_id, cx)` calls `ThreadMetadataStore::archive(thread_id, None, cx)`
    through `try_global`.
- **Deviations:**
  - `archive_thread` is an associated function on `App`, with no rail handle: the store's notify
    already refreshes the rail through its subscription, so neither the button nor the menu entry
    needs the rail.
  - The planned `thread_context_menu` helper was not needed; the function stays under the line
    limit.
- **Review of the diff:**
  - REQ-001: the button, its tooltip and the hover swap, as the terminal row's.
  - REQ-002: the button stops propagation, so the row's `open_thread` does not run.
  - REQ-003: the menu's entry archives the row's own thread id, bound when the row drew.
  - REQ-004: the store persists `archived`, and `entries_for_path` skips it.
  - No entity is updated inside another's update: the store is updated from a click handler and
    from a menu entry's handler.
- **Gate:** `just gate-diff` 17 PASS, 0 FAIL (`GATE GREEN [diff]`).

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/605-archive-thread-rows.sh` (sway). A stand-in external agent (enough
  ACP to start sessions and answer prompts) joins the run's `agent_servers`. `new_thread` makes a
  thread from the project's `+` → New Agent Thread → Stand-in, and prompts it.
- **Run 1: a fixture fault, not the change.** The stand-in formatted the request id with `%d`,
  and Zed sends ids as strings. It died at `session/new` (its traceback is in Marley's log as
  `agent stderr`), so no thread was made; later keys went astray, and the quit then failed. The
  fix is in the fixture: sessions are counted apart.
- **Run 2: every shot read.**
  - `threads.png`: kept, second and first under `repo`, each "Stand-in · idle".
  - `hover.png` (REQ-001): the pointer on first's row shows the Archive icon at the row's end, and
    its tooltip "Archive Thread".
  - `archived.png` (REQ-002): first's row is gone, kept and second remain, and the Agent Panel
    still shows kept: the archive opened nothing.
  - `menu.png` (REQ-003): second's right-click menu holds Archive Thread.
  - `menu-archived.png` (REQ-003): after it, only kept remains.
  - `restart.png` (REQ-004): after a quit and a start with no path, the rail lists kept and not
    first or second. The panel's "Failed to Launch: Loading or resuming sessions is not supported
    by this agent." is the stand-in's `loadSession: false`, a fixture limit.
- **Focus:** its own headless sway; Hyprland had 0 Marley windows before and after.
- **Fixes:** none to the source; the Phase 2 gate stands.
- **Pre-existing, not in scope:** a right-click in the rail gives the rail the focus, so its
  highlight moves from the panel's thread to the shown terminal while the menu is open
  (`menu.png`).

## Phase 4 — Complete (2026-09-30)
- **Documented:**
  - `CHANGELOG.md` (Added).
  - `docs/marley_architecture/marley_workbench.md` (Threads: Archive).
  - `docs/marley/workbench-shell.md` (the slice line).
  - `docs/marley/guide.md`: closing a thread; the port row's click, which #604 changed and the
    table still said "opens".
  - `docs/marley/walkthrough.md` (6.1).
  - The guide page: the rail overview's steps and its table's port row, the latter #604's missed
    line.
  - No Zed path touched.
- **Knowledge:**
  - L-claude-605-zed-sends-acp-request-ids-as-strings-001;
  - AD-claude-605-the-rail-archives-a-thread-as-zeds-history-does-001.
  - No product bug was found.
  - Brain: consultation fe902ea2 closed as
    `decisions/marleys-rail-archives-agent-threads-as-zeds-history-does`.
