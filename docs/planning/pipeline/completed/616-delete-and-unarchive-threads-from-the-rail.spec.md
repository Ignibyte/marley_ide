---
pipeline_id: 9aaeff01-d358-41c1-b283-406e52fda300
ticket: docs/planning/tickets/open/TICKET-616-delete-and-unarchive-threads-from-the-rail.md
status: Phase 4 — Complete PASS
title: "Delete and unarchive threads from the rail"
type: feature
slice: workbench shell, the rail's threads; after #605
references: [docs/planning/pipeline/completed/605-archive-thread-rows.spec.md]
---

## Title
A thread row's menu gains Delete Thread, confirmed, and a project's menu gains Archived Threads,
which lists that project's archived threads and brings one back.

## Scope
### In
- **Delete:** Delete Thread… in the thread row's right-click menu asks to confirm, then does what
  Zed's archive view does (`threads_archive_view.rs` `delete_thread`): closes the thread if a panel
  shows it (a live conversation re-saves its metadata), deletes its metadata, cleans up its
  archived worktrees, and deletes the agent's session where the agent supports it.
- **Archived threads:** Archived Threads in a project header's menu is a submenu of that
  project's archived threads (from `archived_entries`, matched by its main folders and host as
  the rail's listed threads are), newest first, with when each was last updated. Choosing one
  opens it, which unarchives it (`load_agent_thread` does); once back in the rail it has Delete
  Thread… on its row. (The queued spec's picker became a menu, with Zed's menu keys and no new
  view; a delete inside it would need a second level, whose choice Zed's menu does not close.)

### Out (explicitly deferred)
- Deleting an archived thread without bringing it back: Zed's archive view does that.
- Restoring archived worktrees from the rail: a thread archived with a linked worktree it removed
  opens without it; Zed's archive view restores it.
- Bulk delete, and threads of projectless groups.

## Reference (§20)
Upstream Zed: the Agent Panel's archive view (`crates/agent_ui/src/threads_archive_view.rs`):
Delete on archived rows (709-739) and its `delete_thread` (807-850), and unarchive by opening
(443-462); the Threads Sidebar's `open_thread_from_archive` (`sidebar.rs:4144-4338`).

### Prior art
- **Behavior maps:** #605 (AD-claude-605) and the archive view above.
- **Published material:** ACP's session deletion (`delete_session` where supported).
- **Code we already ship:**
  - `ThreadMetadataStore` (`agent_ui/src/thread_metadata_store.rs`): `archived_entries` (611),
    `unarchive` (873), `delete` (1140), `get_archived_worktrees_for_thread` (1084).
  - `thread_worktree_archive::cleanup_thread_archived_worktrees` (858), public.
  - `AgentPanel::connection_store` (`agent_panel.rs:1662`) and `load_agent_thread`, which
    unarchives (4454-4469); the rail's `open_thread` (`rail.rs:2338-2375`).
  - The rail's thread row menu (`rail.rs:4311-4327`) and `project_context_menu` (3941-3988).

## UI proof
The scenario `script/e2e/616-delete-and-unarchive-threads-from-the-rail.sh` (sway) seeds three
threads for the scratch project with the fixture agent #605's scenario uses, archives one, then:
- right-clicks another and chooses Delete Thread… (`confirm.png`), confirms (`deleted.png`: the row
  gone, and `store.txt` without it);
- opens the project's Archived Threads… (`archived.png`: the archived thread listed), chooses it
  (`unarchived.png`: its row back and the thread open).

## Locked-In Decisions
- D1 — Delete mirrors Zed's archive view's `delete_thread`, and closes the thread first.
- D2 — Unarchive is opening the thread, as Zed does.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user chooses Delete Thread… and confirms, Marley shall delete the thread, and the rail shall no longer list it. | Shots `confirm.png`, `deleted.png`; `store.txt` |
| REQ-002 | WHEN the user opens a project's Archived Threads…, the rail shall list that project's archived threads. | Shot `archived.png` |
| REQ-003 | WHEN the user chooses an archived thread, Marley shall open it and the rail shall list it again. | Shot `unarchived.png` |
| REQ-004 | IF the thread to delete is open in a panel, THEN Marley shall close it first, so the delete holds. | Review; `store.txt` |

## Phase Plan
- **P1 Plan** — promote, recall, the design (the picker, the delete's steps).
- **P2 Code** — delete, the picker, unarchive; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
