---
pipeline_id: 9911f083-1847-4c96-b5bf-1bd7f0f6b0e5
ticket: docs/planning/tickets/open/TICKET-605-archive-thread-rows.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Archive an agent thread from the rail"
type: feature
slice: workbench shell (the rail's thread rows), after #439
references: [docs/planning/pipeline/completed/439-rail-zed-threads.spec.md, docs/planning/pipeline/completed/452-rail-rename-and-close.spec.md]
---

## Title
A thread row gets an Archive button on hover and a right-click menu with Archive Thread, which
archives the thread as Zed's thread history does.

## Scope
### In
- **The button.** Pointing at a thread row shows an Archive icon button in the row's end slot
  (where a terminal row shows its close button and a dot shows otherwise), with the tooltip
  "Archive Thread". A click on it archives the thread and never opens it.
- **The menu.** A right-click on a thread row opens a menu with Archive Thread.
- **Archiving** calls `ThreadMetadataStore::archive(thread_id, None, cx)`, the call Zed's threads
  history view makes (`crates/agent_ui/src/threads_archive_view.rs`); the rail's thread rows
  already leave archived threads out (`entries_for_path` and `entries_for_main_worktree_path`
  filter `archived`), so the row goes.
- **A thread that is running** is archived as Zed's history view archives it; if the thread is the
  one the Agent Panel shows, the panel keeps showing it until the user moves on, as it does when
  Zed's history archives it.

### Out (explicitly deferred)
- Deleting a thread, and unarchiving from the rail (Zed's archive view in the Agent Panel does
  that).
- Zed's Threads Sidebar's archive of a linked worktree's folders with its last thread
  (`sidebar.rs` `archive_thread`): Marley's worktrees (#510) are removed from their own row (#589).

## Reference (§20)
Upstream Zed: archiving a thread from the agent thread history (`threads_archive_view.rs`,
`archive_thread` and `ArchiveSelectedThread`, bound to Shift-Backspace there) and from the Threads
Sidebar's hover button and menu ("Archive Thread", `crates/sidebar/src/sidebar.rs`). Marley keeps
the hover button and the menu entry, and the history view's plain archive call.

### Prior art
- **Behavior maps.** #439's spec (thread rows); #452 (the terminal row's close button on hover and
  its right-click menu, the pattern this copies).
- **Published material.** None needed.
- **Code we already ship.** `ThreadMetadataStore::archive` (`crates/agent_ui/src/thread_metadata_store.rs`);
  the rail's `render_thread_row` (`crates/marley_workbench/src/rail.rs` around 3649) and
  `render_terminal_row`'s hover close button and `right_click_menu` (around 3681-3790).

## UI proof
The scenario `script/e2e/605-archive-thread-rows.sh` (sway) makes a Zed agent thread in the scratch
project with a scripted ACP agent server (as `script/e2e/508-approvals-inbox.sh` does), points at its row and shoots the
Archive button (`hover.png`), clicks it and shoots the rail without the row (`archived.png`),
makes a second thread and archives it from the right-click menu (`menu.png`, `menu-archived.png`),
then restarts Marley and shoots the rail (`restart.png`).

## Locked-In Decisions
- D1 — Archive, not delete; the same store call Zed's history view makes.
- D2 — The button takes the row's end slot on hover, as the terminal row's close button does.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the pointer is over a thread row, the row shall show an Archive button with the tooltip "Archive Thread". | Shot `hover.png` |
| REQ-002 | WHEN the user clicks the Archive button, Marley shall archive the thread and the rail shall no longer list it, without opening it. | Shot `archived.png` |
| REQ-003 | WHEN the user right-clicks a thread row, the rail shall open a menu with Archive Thread, which archives it. | Shots `menu.png`, `menu-archived.png` |
| REQ-004 | WHEN Marley restarts, an archived thread shall stay out of the rail. | Shot `restart.png` |

## Phase Plan
- **P1 Plan** — promote this pair, recall, the design in the notes.
- **P2 Code** — `render_thread_row`'s end slot and menu; the archive call; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, the guide page and the walkthrough (§21); the ledger; close,
  archive, commit.
