# Threads in the rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-745-threads-in-the-rail.md
- **Pipeline spec:** 745-threads-in-the-rail.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** Chad, 2026-10-11, the tabs-everywhere shelf
  (`docs/planning/design-notes/tabs-everywhere-2026-10-11.md`): "we need to have a button that
  shows all agent threads or something again … add a button in the side panel that toggles between
  the two? with an option on the home page as well", then "moving the threads back over to the top
  left".
- **Classification:** feature, Marley crate only (`marley_workbench`); no Zed hunk.
- **Order:** after #743 (the rail's header and groups) and #744 (`open_thread` keeps threads out of
  the Agent Panel; this list's click relies on it). Lines in `rail.rs` are at 19dda499a2.
- **Recall (§18.3):**
  - AD-claude-678: the rail's Projects | Brain switch went because the Brain tab already held the
    vault's tree ("two homes for one tree"). The spec's D2 says why Threads differs; Complete
    records it as an AD beside 678.
  - #644's D2 and D3 (`644-brain-view-in-the-rail.spec.md`): a second rail view is an entity drawn
    in place of the filter, inbox and rows, not rows in `marley_rail`'s model (the cursor, the
    switcher, Next and Previous Project and Thread walk projects only); the end button follows the
    view; Projects carries the attention dot while the other view shows; `ToggleButtonGroup` was
    too wide. The rail keeps its `Sidebar` duties whichever view shows.
  - AD-claude-737: the Threads page over Zed's store, its search rule (title, any folder, case
    folded), its click through `open_thread` (one place per conversation, #697).
  - AD-claude-454: the rail takes Zed's thread switcher with Zed's binding; the same here for
    `ToggleThreadHistory`.
  - #739: the status bar's Threads button; its scenario found the buttons by a first run.
  - PR-claude-701: a page's handlers that update its workspace are plain closures over a weak
    handle (the Home page's new button).
- **Discovery** (lines at 19dda499a2):
  - `crates/marley_workbench/src/rail.rs`: the `Rail` struct (`filter_editor` 143, the store
    subscription set at 2532 of the working tree), `render_inbox` (2026), the row keys
    (`select_next` 3057, `select_previous` 3061, `select_first` 3065, `select_last` 3069,
    `select_parent` 3074, `select_child` 3083, `focus_filter` 3112, `cancel` 3123, `confirm`
    3148), `empty_space_menu` (4154), `render_header` (4390), `render_rows` (4663),
    `render_filter` (4695), `render_add_project` (4743), `row_card` (9176), `impl Render for
    Rail` (9420: header, filter, inbox, rows).
  - `crates/marley_workbench/src/threads_page.rs`: `Row` (83), `Sections` (93), `sections` (126),
    `render_row` (239), `matches` (283), `shown_folder` (294), `open_thread` (310).
  - `crates/marley_rail/src/marley_rail.rs`: `has_attention` (1791).
  - `crates/marley_workbench/src/home_page.rs`: `render_agents` (258), the NEW AGENT card (465).
  - `crates/marley_workbench/src/status_buttons.rs`: the Threads button (97-113).
  - `crates/marley_workbench/keymap.json`: the `MarleyRail && !Picker` block (263).
  - Zed: `sidebar.rs` `ToggleThreadHistory` (95), `render_sidebar_bottom_bar` (7521),
    `toggle_archive` (7697), its serialized view (7908-7909); `default-linux.json` `ctrl-g` in
    `ThreadsSidebar` (755); `threads_archive_view.rs` `ThreadsArchiveView::new` (163),
    `ThreadsArchiveViewEvent` (131), `render_header` (860).
  - Guide: `docs/marley/guide.md` "Buttons at the right of the status bar" (1175, the Threads row
    1184), "The Threads page" (1690); the in-app page's matching sections.

### Design
- **`threads_page.rs`.** `pub(crate) fn conversations(query: &str, cx: &App) -> Vec<Row>`: the
  store's entries, drafts and archived left out, newest first, filtered by `matches`; `Row` and its
  fields `pub(crate)`. The page's `sections` keeps its own walk (it needs archived rows and
  sections) or calls a shared record walk; the rule for a row is written once.
- **`rail.rs`.**
  - `enum RailView { Projects, Threads }` on the `Rail`, `Projects` at start; `threads_search:
    Entity<Editor>` made with the rail, placeholder "Search conversations…", whose edits notify.
  - `render_header`: two `ButtonLike`s with XSmall labels, PROJECTS and THREADS, the shown one
    `toggle_state(true)`, each click setting the view (and on Threads, focusing the search field);
    PROJECTS gets `Indicator::dot().color(Color::Accent)` while Threads shows and
    `marley_rail::has_attention(&self.snapshot.rail)`; `render_add_project` and the empty-space
    right-click only in Projects.
  - `render` (9420): in Threads, the search row (the filter row's height and border) and
    `render_thread_list` in place of `render_filter` and `render_rows`; the inbox stays.
  - `render_thread_list`: `threads_page::conversations(query, cx)` mapped to `row_card`s (the
    agent's icon through `agents::thread_icon`, the title, "agent · folder"), each click a
    listener that runs `threads_page::open_thread` for the shown workspace
    (`multi_workspace.read(cx).workspace()`), deferred as the rail's thread rows defer; the empty
    list reads "No conversations yet." or "No conversation matches.".
  - The row keys and `confirm`, `focus_filter`, `cancel`: return at once while Threads shows
    (`cancel` with the search field holding text clears it).
  - `ToggleThreadHistory` (`sidebar::ToggleThreadHistory`) handled on the rail's root: flips the
    view.
- **`keymap.json`** (`MarleyRail && !Picker`): `"ctrl-g": "agents_sidebar::ToggleThreadHistory"`.
  Check at promotion that `ctrl-g` reaches the rail from its search field (the editor's own
  `ctrl-g` binding, if any, is deeper and would win).
- **`home_page.rs`.** `render_agents`: a Threads button (`IconName::Thread`) after New Agent…,
  a plain closure over the weak workspace calling `threads_page::open`.
- **`status_buttons.rs`.** The Threads button and its workspace clone go; the module doc says Home,
  Rusty and Marley.
- **The in-app guide page**, under `crates/marley_workbench/guide/`: the rail's header, the Home
  entry, the status bar's buttons.
- **File manifest:** `rail.rs`, `threads_page.rs`, `home_page.rs`, `status_buttons.rs`,
  `keymap.json`, `guide/index.html` (all Marley crate); `script/e2e/745-threads-in-the-rail.sh`;
  at Complete, `docs/marley/guide.md`, `docs/marley/walkthrough.md`, `CHANGELOG.md`, the
  architecture note. No Zed path, so no ledger row.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001, 009 | Starts on the repo; three conversations made by keys, each asked one line | 745-01-header: PROJECTS pressed, THREADS; the status bar's right end without the Threads button |
| 002, 006 (click) | Clicks THREADS | 745-02-threads: `second`, `other`, the Marley one, newest first, each with agent and folder |
| 003 | Types `other` | 745-03-search: only the `other/` row |
| 004 | Clears; closes the `second/` tab; clicks its row | 745-04-open: its tab in the repo with "second plan" answered; `session/load` logged |
| 005 | Clicks the Marley row while its tab is open | 745-05-forward: that tab in front, one Marley tab |
| 006 (key) | Clicks in the rail's list, then `ctrl-g` | 745-06-key: the project rows, PROJECTS pressed |
| 008 | Clicks Home in the status bar, then Threads on the Home page | 745-07-home: the Threads page in a tab |
| 007, 010 | none | The review: the key guard and the dot; the store subscription's refresh |

Coordinates of the header's labels, the rows and the Home button come from a first run that stops
at the first shot.

### Risks
- **The header's width.** PROJECTS and THREADS beside the window controls must fit the rail's
  narrowest width; the first run's shot shows it. If they do not fit, the labels shorten before
  icons replace them.
- **Two homes.** AD-claude-678's objection applies in part: the page and the rail list one store.
  D2 answers it; if Chad finds the page redundant later, the Home entry can open the rail's view
  instead.
- **A row click inside the rail's listener** reaches every workspace through `open_thread`; the
  rail's thread rows already do this from a listener, and #744 defers it, so this follows theirs.
- **Rows past a few hundred** draw as plain children; `render_rows` scrolls them. A `uniform_list`
  waits until a run shows the need.

### Promotion (2026-10-11, at eeff2d27d8)
- Seams re-checked after #743, #744 and #772: `render_header` 4419, `render_filter` 4724,
  `render_rows` 4692, `row_card` 9120, `impl Render for Rail` 9364, the row keys 3086-3177,
  `thread_store_subscription` 2551. `threads_page::conversations` already exists (#772) and takes
  a query now; `Row`'s fields the rail reads are `pub(crate)`. `Rail::new` sits at clippy's
  100-line limit, so the view's state is a `ThreadsSide` made by one line.

## Phase 2 — Code
- **Built:** as the design, with `ThreadsSide` holding `shown`, the search editor and its edits'
  subscription; `render_view_switch`, `show_view`, `toggle_thread_history`,
  `render_threads_search`, `render_thread_list`, `open_conversation`; the guards in `move_cursor`,
  `fold`, `confirm`, `focus_filter` and `cancel`; the store observer notifies while Threads shows;
  `threads_page::conversations(query, cx)`; `home_page.rs`'s Threads button; `status_buttons.rs`
  without Threads; `keymap.json`'s `ctrl-g`; the in-app page's Threads in the rail steps.
- **Review:** a click on a list row defers the open, which reads every workspace; the list reads the
  store while drawing, so the store observer's `cx.notify()` is what keeps it current (REQ-010);
  the Zed layout has no rail.
- **Gate:** red twice on clippy: `Rail::new` and `sync_subscriptions` one line over 100, a
  `map().unwrap_or()`, then a needless `&mut Window`. The store's subscription moved into
  `ThreadsSide` (`store`) behind a `follow_thread_store` helper, the icon became `map_or`, and
  `open_conversation` takes `&Window`. GATE GREEN [diff], 17 passed. The scenario ran again on the
  final build: the same shots, and 745-04 shows the list re-sorted after alpha's reopen with no
  switch, the store observer at work (REQ-010).

## Phase 3 — Test
- **Scenario:** `script/e2e/745-threads-in-the-rail.sh`, `compositor sway`, #772's set-up; two Marley
  conversations, "alpha" then "beta". Two runs, both exit 0 with both checks passing. The first
  missed the status bar's Home (at x 1375 once Threads left, not 1355) and, in step 05, clicked
  alpha, which the reopen had made the newest row; the second clicks the second row (beta) and
  Home at 1375.
- **Shots (second run):**
  - `745-01-header`: PROJECTS lit and THREADS in the header; the status bar's right holds Home,
    Marley and the dock buttons, no Threads. REQ-001, REQ-009.
  - `745-02-threads`: THREADS lit; a "Search conversations…" field; beta, then alpha, each "Marley ·
    <the repo>"; no project rows; PROJECTS carries the accent dot (the threads' unread marks).
    REQ-001, REQ-002, REQ-006 (the click), REQ-007 (the dot).
  - `745-03-search`: "alp": alpha alone. REQ-003.
  - `745-04-open`: alpha's tab closed, its row clicked: an alpha tab with "alpha" and "Noted:
    alpha", `session/load` logged; alpha moved to the top as the newest. REQ-004.
  - `745-05-forward`: beta's row clicked while its tab is open: beta's tab in front, one beta tab.
    REQ-005.
  - `745-06-key`: a click in the rail, then Ctrl+G: the project rows again, PROJECTS lit. REQ-006.
  - `745-07-home`: the status bar's Home, then the Home page's Threads: the Threads page in a tab
    listing alpha and beta. REQ-008.
- **Reviewed, not shot:** REQ-007's key guard (the row keys return while Threads shows), REQ-010.
- **Pre-existing, not in scope:** the Ctrl+Alt+N notice (#563).

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Changed: Threads in the rail); the guide (the status bar's three
  buttons; a "Threads in the rail" section); the architecture note; the slice line in
  `workbench-shell.md`; the in-app page came with Phase 2.
- **Knowledge appended:** AD-claude-745-a-projects-threads-switch-where-678-removed-the-brain-switch-001.
- **Ticket:** closed; its BACKLOG row left at promotion.
- **Gate:** GATE GREEN [diff], 17 passed, on the tree committed.
