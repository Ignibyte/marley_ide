# Archive that sticks — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-777-archive-that-sticks.md
- **Pipeline spec:** 777-archive-that-sticks.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-12: Archive on a rail thread row "doesn't work" while the thread's
  tab is open, and the rail's Threads view shows CLI sessions but not the Agent Panel agents'
  threads (Marley and the rest). Run autonomously; the plan was confirmed with the batch
  (#778-#781 queued).
- **Classification / tier:** bug, Marley crate only (`marley_workbench`), no Zed hunk.
- **Checklist:** pick ✓, pre-flight ✓ (no active pipeline, README marker present, cargo idle),
  recall ✓, mint ✓ (tickets #777-#781, backlog rows #778-#781), prior art ✓, spec ✓, design ✓.
- **Recall (§18.3):**
  - AD-claude-605: the rail archives through `ThreadMetadataStore::archive(thread_id, None, cx)`;
    its acceptance said the row stays gone after a restart. #734's tabbed-thread loop in
    `group_threads` (later) broke that for tabbed threads.
  - F-claude-754: `conversations` and the Threads page now leave archived threads out or fold
    them; this ticket keeps that and makes opening unarchive.
  - The brain (`rusty-cli brain ask`, consultation d783a5bb…): nothing on this seam.
- **Discovery** (two Explore passes, then read):
  - `rail.rs` `group_threads` (~8746-8758): the loop over `tabbed` takes `store.entry(id)`, which
    does not check `archived`; the row returns on the store's own refresh.
  - `thread_tab.rs` `ThreadTab::new`: subscriptions to the conversation and the panel only;
    `deserialize` checks only that the entry exists; `open_saved` never unarchives.
  - `threads_page.rs` `open_thread`: tab, then panel, then `open_saved`; no unarchive.
  - `rail.rs` `show_thread`: unarchives only after `show_here` failed.
  - `assistant.rs` `latest_conversation`: doc says "not archived", no filter.
  - `rail.rs` `delete_thread`: panels drop the thread, the record goes, the tab stays and its next
    update saves the record again.
  - Chad's store (read-only): every archived thread has folders, so none was archived at birth
    by Zed's no-folder rule; they were archived by a click.
- **Decisions:** D1-D5 in the spec.

### Design
- `thread_tab.rs` (Marley crate):
  - `ThreadTab::new`: a third subscription, to `ThreadMetadataStore::global(cx)`; on
    `ThreadArchived(id)` equal to the view's `parent_id()`, `cx.emit(ItemEvent::CloseItem)`.
    `_subscriptions` becomes `[Subscription; 3]`. Where the store is absent (`try_global` none),
    no subscription is possible; the store is initialised in `agent_ui::init`, before any tab, so
    take `try_global` and keep a `Vec<Subscription>` if needed.
  - `deserialize`: `known` also requires `!entry.archived`; the error says the thread is archived.
  - `open_saved`: unarchive the record's thread when archived, before building the view.
  - A `close_for(workspace, thread_id, window, cx)` helper beside `activate_for`: closes every
    tab of `workspace` that shows the thread.
- `rail.rs` (Marley crate): `group_threads` takes `store.entry(id).filter(|row| !row.archived)`;
  `show_thread` unarchives before `show_here`; `delete_thread` closes the thread's tabs in every
  workspace of the window in the same update that lets the panels drop it.
- `threads_page.rs` (Marley crate): `open_thread` unarchives first when the record is archived.
- `assistant.rs` (Marley crate): `latest_conversation` filters `!record.archived`.
- No Zed crate changes, no ledger rows.

### Visual check plan
| REQ | Set-up and action | Shot |
|---|---|---|
| 001 | Marley thread "first" in a tab; hover its row, click Archive; switch the rail to THREADS | `777-01-archived`, `777-02-threads-view` |
| 003 | Threads page (`marley: open threads`), Show archived, open "first"; rail THREADS | `777-04-reopened`, `777-05-threads-view` |
| 002 | Archive "first" again with its tab open; quit; launch | `777-06-restart` |
| 004 | `marley: talk to marley` after the restart | `777-07-talk` |
| 005 | Send "second" in that conversation; right-click its row, Delete Thread, confirm | `777-08-deleted` |

Reopen Thread… unarchiving is the same `open_thread` call the Threads page makes, checked by the
diff's review; a rail row's unarchive (the tab already open) is too.

### Risks
- Closing the tab on archive drops a running thread's view, as upstream drops a retained one; an
  archive of a working thread stops following it. Accepted: that is what archive means in Zed.
- A restored tab of an archived thread disappears after this ships: Chad's two archived tabs go at
  his next restart. They are listed under the Threads page's ARCHIVED and come back when opened.

## Phase 2 — Code
- **Built** (all `marley_workbench`):
  - `thread_tab.rs`: `ThreadTab::new` subscribes to the store when there is one and closes the tab
    on its thread's `ThreadArchived` (`_subscriptions` is a `Vec`); `deserialize` refuses an
    archived thread as it refuses a deleted one; `open_saved` unarchives first; new
    `close_for(&Entity<Workspace>, thread_id, cx)` and `unarchive(thread_id, cx)`.
  - `rail.rs`: `group_threads`' tabbed loop skips archived rows; `show_thread` unarchives before
    `show_here` (its old unarchive, after `show_here` failed, moved there); `delete_thread`
    closes the thread's tabs in every workspace of the window in the update that lets the panels
    drop it.
  - `threads_page.rs`: `open_thread` unarchives before it looks for a tab, a panel or the store.
  - `assistant.rs`: `latest_conversation` skips archived records.
- **Deviations:** `close_for` takes the workspace entity, not `&Workspace`: reading the workspace
  and updating its tabs need the same `cx`.
- **Review:**
  - REQ-001: archive → `ThreadArchived` → `CloseItem` → the pane closes the tab with
    `SaveIntent::Close`; a thread tab is never dirty, so no prompt. The rail's own refresh then
    finds no row (store rows skip archived; the tabbed loop now does too).
  - REQ-002: a restored tab of an archived thread is refused before any view is built.
  - REQ-003: every opener reaches `unarchive`: the Threads page and Reopen Thread… through
    `open_thread`, a rail row through `show_thread`, Talk to through `open_thread`, a saved
    conversation through `open_saved`.
  - REQ-005: the store's view observation saves only on conversation events, not on release, so
    a tab closing after the record is deleted writes nothing back.
  - Re-entrancy: the tab's subscription runs after the store's update; `close_for` runs inside
    the multi-workspace's update and updates only tab entities; `unarchive` updates the store
    while only a workspace (or nothing) is being updated.
  - Provenance: Marley code calling `agent_ui`'s public API; no Zed hunk, no ledger row.
- **Gate:** `script/gates.sh --diff` GATE GREEN [diff], 17 of 17, receipt written
  (log in the scratchpad, `gate-777.log`). One doc line another session's agent wrote failed
  `typos` before the run (an anchor slug, `#the-harnesss-sessions`); it is plain text now.

## Phase 3 — Test
- **Scenario:** `script/e2e/777-archive-that-sticks.sh` under `compositor sway`, #772's set-up
  (the Marley entry on a scripted ACP agent; the copy's thread list emptied), plus
  `marley.list_cli_sessions` off.
- **The runs found:**
  - The first run's guessed Show archived point landed on one of the user's own Claude Code
    sessions, which the copied profile lists on the Threads page; the click resumed it in a
    terminal of the run's Marley (its HOME was the scenario's, so Claude Code stopped at its
    first-run screen and nothing of the user's changed). The scenario now turns
    `marley.list_cli_sessions` off, so the lists hold only the run's threads.
  - The Threads page's own tab adds a rail row above a thread opened after it, so the second
    archive needs the lower row (`LOWER_ROW_Y`); the first runs' shots placed every point.
  - Shot 08 alone could not tell a delete from another menu item; the run now shoots the
    question and checks the store's count before and after.
- **The final run**, every check passed (`e2e exit 0`; Hyprland untouched, sway stopped):
  - **777-00-thread:** "first" in a repo tab with "Noted: first"; its row under repo, Marley ·
    idle.
  - **777-01-archived** (REQ-001): after Archive on the row, only `repo — bash` in the tab bar and
    under repo: the tab closed, the row gone.
  - **777-02-threads-view** (REQ-001): THREADS reads "No conversations yet."
  - **777-03a-page-folded / 777-03-page:** the Threads page with "Show archived (1)", then
    ARCHIVED with "first · Marley · <repo>".
  - **777-04-reopened** (REQ-003): "first" in a tab again with its turn (the agent logged a
    `session/load`), its row back under repo.
  - **777-05-threads-view** (REQ-003): THREADS lists "first · Marley · …/repo".
  - **777-05a-archived-again** (REQ-001 again): archived with its tab open, the tab and row gone;
    the Threads page shows it under ARCHIVED at once.
  - **777-06-restart** (REQ-002): after quit and start, no "first" tab and no row.
  - **777-07-talk** (REQ-004): Talk to Marley opened "New Agent Thread", empty, not "first"; the
    agent logged a second `session/new`.
  - **777-08a-confirm / 777-08-deleted** (REQ-005): "Delete the thread “second”?" with Delete
    Thread first; after it, the tab and the row are gone, and the store keeps one Marley thread
    where it kept two.
- **Not reached:** Reopen Thread… and a rail row opening an archived thread; both go through the
  same unarchive (`open_thread`, `show_thread`), read in the diff's review.
- **Gate after the scenario:** the scenario is part of the receipt's fingerprint, so
  `script/gates.sh --diff` ran again on the final tree.

## Phase 4 — Complete
- Docs: `CHANGELOG.md` (Fixed, #777); `docs/marley/guide.md` (the rail's archive paragraph);
  `docs/marley_architecture/marley_workbench.md` (Archive, Delete). No Zed path, no ledger row.
- Ledger appends: `F-claude-777-an-archived-threads-row-came-back-while-its-tab-was-open-001`,
  `PR-claude-777-every-source-of-thread-rows-applies-the-lists-filters-001`,
  `L-claude-777-a-scenario-that-clicks-a-thread-list-turns-cli-sessions-off-001`.
- Brain: consultation d783a5bb… closed with `brain no-decision` (a fix that follows Zed's own
  archive rules).
- Ticket closed; pipeline archived; committed on `marley/workbench-shell` (the five doc agents'
  guide and walkthrough work commits separately).
