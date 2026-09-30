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
