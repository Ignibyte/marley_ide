# Reopen Thread…, and every thread opens in a tab — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-772-reopen-thread-and-threads-in-tabs.md
- **Pipeline spec:** 772-reopen-thread-and-threads-in-tabs.spec.md

## Phase 1 — Plan
- **Request:** split from #744 at its promotion (2026-10-11). Chad: "add a New Agent from Thread or
  something that restarts", then "my thought is that you would re open it".
- **Classification:** feature, Marley crate only (`marley_workbench`); no Zed hunk.
- **Recall (§18.3):** #744's notes carry this half's recall (AD-claude-737's one-place rule, #697's
  `open_in_center`, PR-claude-735: a picker's confirm defers, code in a workspace's update reads it
  by reference, #616's unarchive on load). F-claude-679: an open from inside an update defers.
- **Discovery** (at 358f910a61): `thread_tab.rs` `open_in_center` 504, `open_thread_in_center` 537,
  `move_to_panel` 559, `activate_for` 592, `start` 658, `open_saved` 776, `init` 128 (the
  `MarleyThreadHost` global), `tab_extra_context_menu_actions`; `threads_page.rs` `sections` 127,
  `open_thread` 311 (panel branch 334-351); `rail.rs` `load_thread` 375, `show_thread` 399,
  `open_manager_thread` 2015, `open_thread` 2866, `open_listed_thread` 2877, `open_thread_with`
  2909, `archived_threads_menu` 5087, `render_thread_row` 5321 (Open in Center), `render_project_menu`
  (New Agent…); `agents.rs` `new_item_entries`, `start_thread` 192; `harness.rs` `manager_in_front`
  441; Zed: `AgentPanel::activate_retained_thread` 3273, `is_retained_thread` 4168,
  `active_thread_id` 3300, `clear_base_view` 1743, `ThreadMetadataStore` (`entry`, `entries`,
  `archived_entries`, unarchive).

### Design
- **`thread_tab.rs`.** `show_here(workspace, thread_id, window, cx) -> bool`: `activate_for`; else
  with the workspace's panel holding the thread (active or retained, not a draft): a retained one
  made active with `activate_retained_thread(id, false, …)`, then `open_in_center`'s steps for the
  active view (`clear_base_view`, a `ThreadTab` on the same view, focus). `reveal_for` (the
  `MarleyThreadHost.reveal`): `show_here` in the Marley layout, `activate_for` in Zed's. The tab's
  extra menu returns no Move Thread to Panel in the Marley layout; `CommandPaletteFilter` hides
  `MoveThreadToPanel` and `OpenThreadInCenter` while the Marley layout is on (rechecked on each
  settings change).
- **`threads_page.rs`.** `open_thread`'s panel branch, in the Marley layout, activates the member
  and runs `show_here` there. `conversations(cx) -> Vec<Row>` (unarchived, not drafts, newest
  first) for the picker. `ReopenThread` registered on every workspace → `reopen(workspace, …)`
  toggles `ReopenThreadPicker` in the modal layer; its delegate matches title and folder with
  `fuzzy`, draws the agent's icon, title, "agent · folder" and the time, and on confirm dismisses,
  then `window.defer` → `open_thread(&workspace, id, …)`.
- **`rail.rs`.** `show_thread`: `thread_tab::show_here`; else the record from the store,
  unarchived first when archived, through `thread_tab::open_saved`; with no record, the panel load
  as before. Open in Center leaves the row menu (and `open_listed_thread`'s `center`).
  `open_manager_thread` with no listed thread: `thread_tab::start(…, MANAGER_ENTRY, None, …)`. The
  `+` gains Reopen Thread… under New Agent….
- **`agents.rs`.** `new_item_entries` adds Reopen Thread… after New Agent…; `start_thread` goes.
- **`harness.rs`.** `manager_in_front`: the panel test, or the shown workspace's active item a
  thread tab on the Manager entry (`thread_tab::shows_agent`).

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001, 007 | Starts a Marley thread, asks "first", closes its tab; the repo `+` → Reopen Thread… | 772-01-reopen |
| 002 | Enter | 772-02-reopened |
| 003 | Reopen Thread… again, Enter | 772-03-again |
| 004 | Closes the tab, clicks the thread's rail row | 772-04-row |
| 006 | Right-clicks the thread tab | 772-05-tab-menu |
| 005, the rest of 006, 007 (pane) | none | The review |

### Risks
- A retained thread made active only to be moved: `activate_retained_thread` with `focus` false
  does not open the dock; checked in the review and by 772-04 (a running thread clicked).
- The scripted agent's `session/load` must replay "first"; #739's agent does, #744's does not:
  this scenario takes #739's.

## Phase 2 — Code
- **Built:** `thread_tab.rs`: `show_here` (the tab forward; else a thread the workspace's panel holds,
  a kept one made active with `activate_retained_thread(id, false, …)`, moved out with
  `clear_base_view` into a `ThreadTab` on the same view; a draft is left alone); `reveal_for` as
  `MarleyThreadHost.reveal` (`show_here` in the Marley layout, `activate_for` in Zed's);
  `shows_agent`; `filter_palette` hides `MoveThreadToPanel` and `OpenThreadInCenter` while the
  Marley layout is on, rechecked on each settings change and at each workspace's setup; the tab's
  extra menu is empty in the Marley layout; `open_thread_in_center` went with the rail's Open in
  Center. `threads_page.rs`: `ReopenThread`, `reopen`, `conversations` and `row_of` (shared with
  the page), `ReopenThreadPicker` and its delegate (fuzzy over "title folder", the title
  highlighted, "agent · folder" on a second line, the time at the end; confirm dismisses, then
  defers `open_thread`); `open_thread`'s panel branch runs `show_here` in the Marley layout.
  `rail.rs`: `show_thread` runs `show_here`, else opens the store's record in a tab through
  `open_saved` (unarchiving it first), else loads the panel as before; Open in Center and the
  `center` flag went; `reopen_thread` and the `+` entry; `open_manager_thread` starts the Manager
  in a tab. `agents.rs`: Reopen Thread… in the pane's `+`; `start_thread` went with its last
  caller. `harness.rs`: `manager_in_front` also counts a thread tab on the Manager entry in front.
  The guide, the in-app page and the walkthrough.
- **Review:** a thread the panel shows is moved, never drawn twice (the panel's base view is
  cleared before the tab takes the view); every open from a picker or a menu defers, so nothing
  reads a workspace inside its own update; Zed's layout keeps each old path (`reveal_for`,
  `open_thread`'s branch, the menu and the palette all check `marley_layout`).
- **Found in the visual check and fixed:** the picker's row drew the title as "…": the folder, on
  the same line, took the width. The detail moved to a second line, as the page's rows have it,
  and the picker's width matched New Agent's (34 rem; at 40 the list stopped short of its box).
- **Gate:** the first run was red: `agents_tests.rs` used `AgentPanel` through `agents.rs`'s import,
  which went with `start_thread`, and clippy's `option_if_let_else` on the candidate text. The
  tests import it themselves now, and the text is a `map_or_else`. GATE GREEN [diff], 17 passed.

## Phase 3 — Test
- **Scenario:** `script/e2e/772-reopen-thread-and-threads-in-tabs.sh`, `compositor sway`, #739's
  scripted agent (answers "Noted: <prompt>", replays a session's prompts on `session/load`), the
  copy's thread list emptied. Three runs, all exit 0 with both checks passing (`session/new`, then
  `session/load` after the reopen); the second and third followed the row fix and the width fix.
- **Shots (third run):**
  - `772-00-plus`: the repo row's `+`: New Terminal, New Browser Tab, New Agent…, Reopen Thread….
    REQ-007 (the pane's `+` by review: `new_item_entries` adds it after New Agent…).
  - `772-01-reopen`: "Reopen a conversation…" with one row: the Marley icon, "first", "Marley ·
    <the repo's folder>" and "Oct 11, 10:10". REQ-001.
  - `772-02-reopened`: Enter: a "first" tab in the repo with "first" and "Noted: first", the
    earlier turn replayed by `session/load`. REQ-002.
  - `772-03-again`: `marley: reopen thread` and Enter again: the same "first" tab in front, one
    such tab. REQ-003.
  - `772-04-row`: the tab closed, the rail's "first · Marley · idle" row clicked: the thread in a
    tab with its history; no Agent Panel (the right dock shows the project panel it had since the
    start). REQ-004.
  - `772-05-tab-menu`: the tab's right-click menu: Close … Pin Tab, no Move Thread to Panel.
    REQ-006.
- **Reviewed, not shot:** REQ-005 (a thread the panel holds: no scenario path puts one there in the
  Marley layout without Zed's own panel, which #747 hides); the palette filter.
- **Pre-existing, not in scope:** the Ctrl+Alt+N notice from the run's terminal (#563's note that
  the key went to Marley); the project panel's dock open on a fresh project (#747).

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Changed: Reopen Thread…, and threads in tabs); the architecture
  note (`marley_workbench.md`); the slice line in `workbench-shell.md`; the guide, the in-app page
  and the walkthrough came with Phase 2. No Zed path is touched.
- **Knowledge appended:** F-claude-772-a-pickers-title-shrank-to-an-ellipsis-001.
- **Ticket:** closed; its BACKLOG row left at promotion.
- **Gate:** GATE GREEN [diff], 17 passed, on the tree committed.
