---
pipeline_id: d7423d48-c830-4c1a-8618-efee06796437
ticket: docs/planning/tickets/open/TICKET-777-archive-that-sticks.md
status: Phase 4 — Complete PASS
title: Archive that sticks, and an archived thread that comes back when opened
type: bug
slice: the Marley layout (threads in tabs, #734/#772)
references: [605, 616, 702, 734, 736, 737, 745, 772]
---

## Title
Archiving a thread from the rail closes its tab and keeps its row away, and opening an archived
thread unarchives it. Today the row returns at once while the thread's tab is open, the tab comes
back after a restart, and no Marley opener unarchives, so a thread archived this way never shows
in the Threads view again (Chad, 2026-10-12).

## Scope
### In
- A thread's center tab closes when its thread is archived, from any window or view (the rail's
  Archive and Archive Thread, Zed's archive view, the Threads page).
- The rail lists no archived thread, a tabbed one included.
- A saved tab whose thread is archived is not restored.
- Every Marley opener unarchives the thread it opens: the Threads page and Reopen Thread…
  (`threads_page::open_thread`), a saved conversation (`thread_tab::open_saved`), a rail row
  (`show_thread`, also when the tab is already open).
- Talk to Marley/Rusty takes the newest conversation that is not archived, as its doc says.
- Deleting a thread closes its open tabs before its record goes.

### Out (explicitly deferred)
- Unarchiving Chad's existing archived threads in bulk: each comes back when he opens it.
- The rail's Threads view showing archived threads (the Threads page has Show archived).
- A thread with no folder that Zed archives at birth (`thread_metadata_store.rs`, upstream's rule).

## Reference (§20)
Upstream Zed, `agent_ui`: the Agent Panel drops a retained thread when the store emits
`ThreadMetadataStoreEvent::ThreadArchived` (`agent_panel.rs`, the store subscription), and
`load_agent_thread` unarchives a thread before it opens it (`agent_panel.rs`), as the sidebar does
(`sidebar.rs`). Marley's thread tabs and openers follow the same two rules.

### Prior art
- Behavior maps: `docs/zed_architecture/` describes the thread history and its archive view; no
  Warp analog (Warp has no agent thread archive). Checked `docs/orca_architecture/`: Orca archives
  worktrees, not threads.
- Published material: none needed; ACP has no archive concept (it is Zed's local metadata).
- The code we ship: `agent_ui::thread_metadata_store::ThreadMetadataStore` owns the flag
  (`archive`, `unarchive`, `ThreadArchived`); `agent_panel.rs`'s subscription and
  `load_agent_thread`'s unarchive are the models taken. #605 (AD-claude-605) archives through
  the same call; #616's Delete already lets the panels drop the thread first; F-claude-754 fixed
  the Threads page's archived double count.

## UI proof
`script/e2e/777-archive-that-sticks.sh` (`compositor sway`, #772's set-up: the Marley entry on a
scripted ACP agent, the copy's thread list emptied):
- `777-00-thread`: a Marley thread "first" open in a tab, its row in the rail.
- `777-01-archived`: after Archive on its row, no tab and no row (REQ-001).
- `777-02-threads-view`: the rail's Threads view without "first" (REQ-001).
- `777-03-page`: the Threads page with Show archived open, "first" under ARCHIVED.
- `777-04-reopened`: "first" open again in a tab with its turn (REQ-003).
- `777-05-threads-view`: the rail's Threads view lists "first" (REQ-003).
- `777-06-restart`: archived again with its tab open, quit and restarted: no "first" tab, no row
  (REQ-002).
- `777-07-talk`: Talk to Marley opens a new conversation, not "first" (REQ-004).
- `777-08-deleted`: a second thread, deleted from its row's menu: its tab closed (REQ-005).

## Locked-In Decisions
- D1 — The tab closes on `ThreadArchived`, a subscription in `ThreadTab::new`, as the Agent Panel
  drops its retained thread; no polling of the store.
- D2 — `group_threads` filters archived threads out of the tabbed loop too, so no path that leaves
  a tab open brings a row back.
- D3 — Unarchive lives in the openers (`open_thread` before it looks for a tab, `open_saved`,
  `show_thread` before `show_here`), as upstream's `load_agent_thread` does, not in the store's
  save path: an archived thread still running in some view stays archived until it is opened.
- D4 — A restored tab of an archived thread is refused like one of a deleted thread.
- D5 — Delete closes the thread's tabs in every workspace of the window before the record goes,
  so no tab writes the record back.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user archives a thread whose tab is open, the system shall close the tab and remove the thread's row from the rail's Projects and Threads views. | shots `777-01-archived`, `777-02-threads-view` |
| REQ-002 | WHEN Marley restarts after a thread was archived, the system shall restore neither the thread's tab nor its rail row. | shot `777-06-restart` |
| REQ-003 | WHEN the user opens an archived thread from the Threads page, Reopen Thread… or a rail row, the system shall unarchive it, so the rail's Threads view lists it. | shots `777-04-reopened`, `777-05-threads-view`; review of the diff for Reopen Thread… and the rail row |
| REQ-004 | WHEN Talk to Marley or Talk to Rusty runs and the entry's newest conversation is archived, the system shall open the newest one that is not archived, or start a new one. | shot `777-07-talk` |
| REQ-005 | WHEN the user deletes a thread whose tab is open, the system shall close the tab. | shot `777-08-deleted` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `thread_tab.rs`, `rail.rs`, `threads_page.rs`, `assistant.rs`; a review of the
  diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
