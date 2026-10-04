---
pipeline_id: ae960003-4725-4049-9cd6-7ca99c0bf319
ticket: docs/planning/tickets/closed/TICKET-646-knowledge-panel.md
status: Phase 4 — Complete PASS
title: "The Knowledge panel: a page's backlinks, links and tags, and brain search"
type: feature
slice: Rusty in Marley R3 and R6 (R-D3, R-D5); R6's project view split out
references: [docs/marley/rusty-in-marley.md, docs/marley/three-prong-plan.md]
---

## Title
Rusty's right pane and search pane, rebuilt as Marley's Knowledge panel in the right dock. While
#645's Page tab is the active item, the panel shows that page's tags with their page counts, its
backlinks with the line each sits on and the link lit, and its outgoing links, an unresolved one
offered as a new page. A search field at its top runs `brain_search` with Rusty's operators. It is
drawn from `rusty-mcp` over #643's connection and exists only while `marley.rusty.enabled` is on.
Chad, 2026-10-03: "lets make sure we use the gpui components we found here"; the rows port Ely
GPUI Components' Backlinks and SearchResultItem onto Zed's `ui` crate.

## Scope
### In
- **Builds on #643 and #645.** #643's `crates/marley_rusty` (the pure core) and its client in
  `marley_workbench::rusty` (the connection, its state, its change signal on
  `notifications/resources/list_changed`, the stand-in `rusty-mcp` and its fixture vault), and
  `marley.rusty.enabled`; #645's Page tab (the `Item` that shows one brain page, and the slug it
  shows) and its open-page action. This ticket adds to both crates; the names are theirs as they
  shipped (the notes' promotion entry): `rusty::call_tool`, `rusty::Announced`,
  `rusty::page::PageView` and `open_later`, `rusty::OpenPage`.
- **The panel:** `KnowledgePanel`, an `impl workspace::Panel` in a new
  `marley_workbench::rusty::knowledge_panel` module, in the right dock only, one per workspace,
  added where `fleet::init` adds the Fleet panel. Persistent name `MarleyKnowledgePanel`, icon
  `IconName::Book`, tooltip "Knowledge", activation priority 21 (the Fleet panel holds 20), 320 px
  wide by default. Its toggle is `rusty::ToggleKnowledgePanel` ("rusty: toggle knowledge panel"),
  with no default key; it opens the panel with the search field focused.
- **The switch:** `Panel::enabled` reads `marley.rusty.enabled`; `Panel::icon` is `None` while
  it is off, so the dock shows no button (the Agent Panel's way); the toggle, while off, opens
  nothing and shows a toast: "Rusty is off. Turn it on in the Rusty section of the Marley
  settings." A panel that draws while Rusty is off closes its dock, deferred, but only when it is
  what the dock shows, so a live flip and a dock restored at start both end closed.
- **Following the page:** the panel follows the workspace's active item, as Zed's outline panel
  follows the active editor (`workspace::Event::ActiveItemChanged`). A Page tab gives its page,
  and the panel follows that tab's page as it changes (back, forward, a link followed inside it);
  any other item, or none, gives the no-page state: the search field and "Open a brain page to see
  its tags, backlinks and links."
- **The page view,** under the page's title and slug:
  - **Tags:** the page's tags as Rusty's index holds them (frontmatter and inline), each with its
    page count from `brain_tags`; a click puts `tag:<name>` in the search field and runs it.
  - **Backlinks** with their count: each linking page's title, then the line its link sits on,
    cut to start near the link and the link lit; a click opens that page.
  - **Links** with their count, in the page's order: a resolved link by its page's title, a click
    opening it; an unresolved one as written, with "Create", a click making the page with
    `brain_new_page { path: <target>, folder, name }` (promotion: Rusty's main takes `path`, the
    older binaries `folder` and `name`, the target split at its last `/`) and opening it.
- **Brain search:** a single-line Zed `Editor` at the panel's top, "Search the brain…". After
  Enter it calls `brain_search { query, limit: 60, case_sensitive, regex }` with the query as
  typed, operators included (D5: on Enter, not as typed, the rule #644's rail search keeps). Two toggles beside it, match case and regular
  expression, drawn as Zed's search bar draws them. A non-empty query replaces the page view with
  the results: a count line, then rows of title, slug and the snippet with Rusty's `<b>` marks
  drawn as highlights. Up and Down move the selection from the field, Enter opens it (the first
  row when none is selected), Escape clears the query and brings the page view back. An empty
  query shows the operator hint: "Narrow with tag: path: file: type:, quotes keep spaces, a
  leading - excludes."
- **Live:** on the client's change signal the panel reads the shown page and the current query
  again, coalesced; an answer for a page no longer shown is dropped.
- **States:** while the client is not connected, the panel shows the client's state line (#643's
  words) where the sections go; a failed call shows its root cause in its section.
- **The Ely port** (R-D10): the pure parts of Ely's `Backlinks` (`src/documents/knowledge.rs`) and
  of `SearchResultItem` (`src/editor/search.rs`, its line trim) go into
  `crates/marley_rusty/src/knowledge.rs` with Ely's MIT notice; the rows are rewritten over the
  `ui` crate's `ListItem`, `HighlightedLabel`, `ListSubHeader`, `CountBadge`, `Chip` and
  `IconButton`, in Zed's theme colours.
- **The stand-in:** the pages, links, tags and search answers this ticket's scenario reads join
  #643's fixture vault, and its stand-in answers `brain_get_links`, `brain_graph` with `around`,
  `brain_tags`, `brain_search` (logging each query as sent) and `brain_new_page`, where #643's
  does not already.
- **The in-app guide page** (`crates/marley_workbench/guide/index.html`) gains the Knowledge
  panel, in the Code phase (the commit receipt binds it).
- `script/e2e/646-knowledge-panel.sh`.

### Out (explicitly deferred)
- **The project view (R6, R-D5), its own ticket:** with no page focused, the workspace's project
  resolved to a brain project page (its `path:` frontmatter, then its name), that page's follow-ups
  due (`brain_due` joined to the decisions that link to it) and the project's Rusty task group (the
  task tools, by a join rule that ticket locks). The no-page state is where it goes.
- **`rusty: open page`** (R3's other half, the picker over `brain_list_pages`, favourites first,
  create on a miss): with #645 if it takes it, else its own ticket.
- The outline: the Page tab's own table of contents (#645, from `brain_render`'s `outline`) and
  Zed's `outline_panel` over the source buffer cover it (D4).
- Following a vault file opened in an editor buffer (#645's source toggle); the panel follows Page
  tabs only.
- The vault-wide tag tree Rusty's Tags tab shows, and tagging the page from it
  (`brain_set_property`); property edits are #645's.
- Bookmarking a search: waits on Rusty's TICKET-037 (bookmarks in the vault).
- Rusty's change cursor (`changes_since`, Rusty's TICKET-035): not built; a write another
  `rusty-mcp` process makes to the database alone shows on the next change signal or focus.
- Keyboard navigation of the page view's rows, folding its sections, pinning the panel to a page
  (Zed's outline panel's `pinned` mode).
- The page-tied agent of Rusty's right pane: R-D6 (Zed's Agent Panel with the page @-mentioned).
- Unlinked mentions (Obsidian's second backlinks list): Rusty serves none.

## Reference (§20)
Upstream Zed, for the panel and its keys: `workspace`'s `Panel` and `Dock` (a right-dock panel
whose dock button `PanelButtons` draws only when `Panel::icon` is `Some`, redrawn on every settings
change; `Panel::enabled`), kept as they are; the Agent Panel's way of hiding its button and
refusing its toggle while AI is off (`agent_ui::agent_panel`); `outline_panel`'s following of the
active item through `workspace::Event::ActiveItemChanged`; a picker's single-line `Editor` whose
Up, Down, Enter and Escape reach the list as `menu` actions. What each section shows is Rusty's own
app (MIT, `/srv/stacks/rusty-v3/crates/rusty-app/qml`): `RightPane.qml` and `SearchPane.qml`. No
Warp behavior applies: Warp has no linked-notes panel.

### Prior art
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` §1.6 (`Dock`,
  `Panel`, `PanelHandle`: every docked view is a `Panel` impl) and §4.2 (the outline panel follows
  the active editor through a workspace subscription, has its own single-line filter editor and a
  `pinned` mode). `docs/warp_architecture/` and `docs/orca_architecture/` hold nothing on
  wikilinks, backlinks or a notes search (grepped for backlink and wikilink; Orca's right sidebar
  holds git status and ports, `05-terminal-and-workspace.md` §2.3 and §2.4).
- **Published material:** Obsidian's Backlinks (linked mentions with their context), Outgoing links
  (unresolved links listed to create) and search operators (`tag:`, `path:`, `file:`), which
  Rusty's `brain_search` takes; the MCP specification's `notifications/resources/list_changed`.
  Rusty's app, read for what it shows: `RightPane.qml` (backlinks with up to three lines of
  context; outgoing links, an unresolved one with "create"; the outline; the vault's tag tree with
  counts, a click searching `tag:`; the page's agent), `SearchPane.qml` (`brain_search` with limit
  60, match case, regex, a 250 ms debounce, the operator hint, title, slug and snippet rows,
  re-searching on a data change) and `DecisionsPage.qml` (`brain_due`: the due first, then every
  decision; for the deferred project view). Ely GPUI Components at `2f8b2f6` (`MIT OR
  Apache-2.0`), read in full: `src/documents/knowledge.rs` `Backlink` and `Backlinks` (a page that
  links here: icon, title, the words around the link with the link's range lit; a press opens it)
  is taken as the backlink model and its row; `Favorites` and `TrashBin` in the same file are not.
  `src/editor/search.rs`: `trimmed` (a line cut to start near its first match, the ranges moved
  along) is taken; `SearchResultItem`'s row is rewritten (a page with a snippet, not a numbered
  code line); `SearchPanel`'s per-file grouping and folding, the replacement preview and
  `ReferencesPanel` are not (one hit per page). `src/editor/filters.rs` `SearchFilters`
  (include and exclude globs) is not taken: the operators are typed in the query and Rusty parses
  them.
- **Code we already ship:** `workspace/src/dock.rs` (the `Panel` trait `:36-104`, `enabled` `:92`;
  `PanelButtons` filters on `icon` `:1409` and observes the settings store `:1382`;
  `first_enabled_panel_idx` `:552`); `agent_ui/src/agent_panel.rs` (`icon` reads `enabled`
  `:5113-5115`, `enabled` `:5129-5131`, `toggle_focus` checks it `:1614-1626`); `zed/src/zed.rs`
  `setup_or_teardown_ai_panel` `:818-851` (adds and removes the Agent Panel; considered, D1);
  `outline_panel/src/outline_panel.rs:1090-1114` and `:5424-5433` (following the active item);
  the `ui` crate's `ListItem`, `ListSubHeader`, `HighlightedLabel::from_ranges`
  (`label/highlighted_label.rs:41-60`, byte ranges), `CountBadge`, `Chip`, `Disclosure`,
  `IconButton` with `toggle_state`; `editor::Editor::single_line` (`editor.rs:1802`); the `search`
  crate's `SearchOption::icon` (`IconName::CaseSensitive`, `IconName::Regex`, `search.rs:109-115`)
  and `as_button` (`:129-160`, bound to the search crate's actions; its look is copied, not the
  call); the `picker` crate (considered, D5); `context_server`'s `ResourcesListChanged`
  (`types.rs:120-121`) and `on_notification` (`client.rs:509`). Marley's own: the Fleet panel
  (`marley_workbench/src/fleet.rs`, the right-dock panel this one copies: `init` `:298-313`, the
  `Panel` impl `:1498-1542`, the `menu` key context `:1440`), the rail's filter field (#457), and
  `marley_workbench::rusty` (#633). No new dependency: the mention scan and the snippet marks are a
  few lines over `str`.

## UI proof
`script/e2e/646-knowledge-panel.sh` (`compositor sway`: it clicks rows in the panel). Fixtures: a
scratch repository opened with `open_path`; #643's stand-in `rusty-mcp`, named in `MARLEY_RUSTY_MCP` (never put first on the PATH: Marley takes its PATH from the login shell, L-531, so the real `rusty-mcp` could win; reconciled 2026-10-03 to #643's rule),  over a
scratch vault from `marley_rusty`'s fixtures with this ticket's pages (`projects/demo`, tagged
`project` and `area/demo`, with an inline `#draft`, linking `[[concepts/orbit|Orbit]]` and
`[[Missing Page]]`; `concepts/orbit`, tagged `area/demo`, and `decisions/demo-uses-orbit`, each
linking Demo), logging every tool call with its arguments to a file; never the user's Rusty
(R-D8). The run's settings
copy has Rusty off (#643's harness line); the scenario sets `marley.rusty.connection` to
`embedded` and later `marley.rusty.enabled` true from outside (L-607). The run's keymap binds
Ctrl+Alt+Shift+D to #645's open-page action for `projects/demo` (L-633's way). Shots:
- `646-01-off-no-button`: Rusty off: the status bar's right dock buttons, Fleet's and no
  Knowledge.
- `646-02-off-toast`: `rusty: toggle knowledge panel` from the palette: the toast; no panel.
- `646-03-no-page`: Rusty on: the Knowledge button; the toggle: the panel, the search field
  focused, the no-page line.
- `646-04-demo-page`: Ctrl+Alt+Shift+D: Demo in a Page tab; the panel: Demo and its slug, the tags
  `#project`, `#area/demo`, `#draft` with counts, Backlinks 2 (Orbit and the decision, each line
  with its link lit), Links 2 (Orbit; Missing Page with Create).
- `646-05-followed`: a click on Orbit's backlink: Orbit in the Page tab, the panel on Orbit.
- `646-06-other-item`: `workspace: new file`: an editor active, the panel's no-page line.
- `646-07-created`: Demo again; a click on Missing Page's Create: the log holds `brain_new_page`
  with `path` `Missing Page`; that page in the Page tab, the panel on it.
- `646-08-tag-search`: Demo again; a click on `#area/demo`: the field holds `tag:area/demo`, two
  results; the log holds that query.
- `646-09-search`: the field cleared, `orbit` typed: the count line and three rows, `Orbit` bold
  in each snippet; the log holds no `orbit` while it is typed, and one after Enter.
- `646-10-toggles`: match case and regular expression clicked: both toggles on; the log holds the
  query again with `case_sensitive` and `regex` true.
- `646-11-enter-opens`: the field focused, Down, Enter: the selected result in the Page tab.
- `646-12-escape`: back on Demo, `orbit` typed, Escape: the field empty, Demo's page view back.
- `646-13-refresh`: Demo again; a new page linking Demo written into the scratch vault and the
  stand-in's change announced: Backlinks 3, with no input.
- `646-14-off-live`: `marley.rusty.enabled` false: the right dock closed, no Knowledge button.

## Locked-In Decisions
- D1 — The panel is always added and hides itself, as the Agent Panel does: `enabled` reads the
  switch, `icon` is `None` while off (the dock's buttons redraw on every settings change), the
  toggle checks `enabled`, and a panel that draws while off closes its dock in a deferred update
  of the workspace, only when the dock's `visible_panel` is this panel (PR-607; Zed's
  `close_panel` closes the whole dock). Rejected: adding and removing the panel on the switch
  (`setup_or_teardown_ai_panel`'s way), which loses the panel's width and place in the dock on
  each flip; a close from inside the panel's own update (re-entrant). The panel does not read
  `disable_ai`: R-D9 keeps it working where Zed hides the rail.
- D2 — The panel follows the active item, as the outline panel does: a Page tab gives its page,
  anything else gives the no-page state. Rejected: keeping the last page shown, which would leave
  the no-page state (the project view's place) unreachable once any page was opened.
- D3 — A page's view comes from three tools. `brain_get_links` gives the backlinks (with the line
  each sits on) and the outgoing links in order with `resolved`. `brain_graph { around: <slug>,
  depth: 1 }` gives the title of every page one link away and the page's own tags as Rusty's
  index holds them, frontmatter and inline. `brain_tags` gives each tag's page count; it is read
  once and again on each change signal. Rejected: `brain_read_page`'s frontmatter `tags` (misses
  inline `#tags`); a vault-wide title index from `brain_list_pages` (a cache of all titles for the
  few this view names).
- D4 — No outline in the panel. Rusty's pane had one; in Marley Zed's `outline_panel` covers the
  source buffer in Edit mode, and a headings list for Read mode is the follow-up R2b (the Page
  tab's outline and property edits, rusty-in-marley.md), not this panel. Reconciled 2026-10-03:
  #645 as drafted has no table of contents.
- D5 — Search is a single-line `Editor` over a list the panel draws, with the `menu` actions from
  the field (L-457). The query goes to Rusty as typed: Rusty parses the operators, and Marley
  parses only the `<b>` marks in snippets. Limit 60, as Rusty's app, but sent on Enter, not after
  Rusty's 250 ms pause: when an embedding provider is set Rusty embeds every query, and a hosted
  provider (this box names `openai`) would send each partial query off the machine. The rail's
  search (#644) keeps the same rule, so the two never differ. The two
  toggles are `IconButton`s with the search bar's icons and `toggle_state`. Rejected: a `picker`
  (it ranks by its own fuzzy match, and here Rusty ranks), and Ely's `SearchFilters` (globs, which
  the operators replace).
- D6 — The Ely port: `marley_rusty::knowledge` holds the backlink model (a page, its title, its
  line, the link's byte range in it), the scan that finds the range (the `[[…]]` whose target, up
  to `|` or `#`, names the page by slug, last segment or title, case aside), Ely's `trimmed`, and
  the snippet's marks as byte ranges; the file carries Ely's MIT notice. The rows are written over
  the `ui` crate in `marley_workbench`; nothing of Ely's theme, fonts, icons or `gpui` comes along.
  A link written through an alias is not lit; its row shows the line plain.
- D7 — Live by the change signal: one read in flight per kind (page, search, tag counts), a signal
  that arrives during one queues exactly one more; every answer carries the page or query it was
  asked for and is dropped when that is no longer shown. Each call is bounded as #643's client
  bounds it.
- D8 — Create from an unresolved link is one click with no confirmation, as in Rusty's app: a new
  page is cheap to delete, and the call is Rusty's own, so its index and git history stay right.
  The call is `brain_new_page { path, folder, name }` (promotion, 2026-10-04): `path` for Rusty's
  main (its TICKET-041), `folder` and `name` for the binaries the box runs until Chad reinstalls;
  never `{ folder: "", name: "a/b" }`, which Rusty flattens to `a-b`.
- D9 — The scope is split: the project view (R6) is a second slice. This ticket's panel, page view
  and search are one shippable piece, and the project join needs a resolution rule and a task-group
  rule of its own.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `marley.rusty.enabled` is off, the right dock shall show no Knowledge button. | Shot `646-01-off-no-button` |
| REQ-002 | WHEN `rusty::ToggleKnowledgePanel` runs while Rusty is off, the system shall show a toast saying Rusty is off and where to turn it on, and open no panel. | Shot `646-02-off-toast` |
| REQ-003 | WHEN `marley.rusty.enabled` turns on while Marley runs, the right dock shall show the Knowledge button, and its toggle shall open the panel with the search field focused. | Shot `646-03-no-page` |
| REQ-004 | WHILE no Page tab is the active item, the panel shall show the search field and the no-page line. | Shots `646-03-no-page`, `646-06-other-item` |
| REQ-005 | WHILE a Page tab is the active item, the panel shall show that page's title, slug and tags, each tag with its page count. | Shot `646-04-demo-page` |
| REQ-006 | WHILE a Page tab is the active item, the panel shall list the page's backlinks with their count, each by its page's title with the line its link sits on, the link lit. | Shot `646-04-demo-page` |
| REQ-007 | WHILE a Page tab is the active item, the panel shall list the page's outgoing links in order, a resolved one by its page's title and an unresolved one as written with Create. | Shot `646-04-demo-page` |
| REQ-008 | WHEN the user clicks a backlink or a resolved link, the system shall open that page through #645's action, and the panel shall follow it. | Shot `646-05-followed` |
| REQ-009 | WHEN the user clicks an unresolved link's Create, the system shall call `brain_new_page` with the target as its `path` (and as `folder` and `name` for older Rusty), then open the new page. | Shot `646-07-created`; the call log |
| REQ-010 | WHEN the user clicks a tag, the system shall put `tag:<name>` in the search field and search for it. | Shot `646-08-tag-search`; the call log |
| REQ-011 | WHEN the user presses Enter in the query field, the system shall call `brain_search` with the query as typed, and shall send no query while the user is still typing; and list each result's title, slug and snippet with the matches highlighted. | Shot `646-09-search`; the call log |
| REQ-012 | WHEN the user turns on match case or regular expression, the system shall search again with `case_sensitive` or `regex` true. | Shot `646-10-toggles`; the call log |
| REQ-013 | WHEN the user presses Enter in the search field, the system shall open the selected result, or the first when none is selected. | Shot `646-11-enter-opens` |
| REQ-014 | WHEN the user presses Escape in a non-empty search field, the system shall clear it and show the page view again. | Shot `646-12-escape` |
| REQ-015 | WHEN Rusty announces a change, the panel shall read the shown page again and show the result without input. | Shot `646-13-refresh` |
| REQ-016 | WHEN `marley.rusty.enabled` turns off while the panel shows, the system shall close the right dock and hide the Knowledge button without a restart. | Shot `646-14-off-live` |
| REQ-017 | WHILE #643's client is not connected, the panel shall show the client's state line in place of its sections; a failed call shall show its root cause in its section. | Review of the diff |
| REQ-018 | The ported file shall carry Ely's MIT notice, and no crate shall depend on Ely's. | Review; the gate (gate:2 builds, cargo-deny) |

## Phase Plan
- **P1 Plan** — promote after #643 and #645 complete; replace this spec's placeholder names
  (client, change signal, Page tab, open action, stand-in, its trigger and log) with theirs;
  confirm whether `"rusty"` is already in `test_action_namespaces`; ask the brain.
- **P2 Code** — the `README.md` marker first; the `zed.rs` ledger row widened first if the
  namespace is new; `marley_rusty::knowledge` with Ely's notice, the stand-in's additions, the
  panel, its action and init, the in-app guide page; a review of the diff; `script/gates.sh
  --diff` green.
- **P3 Test** — write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG (Added) and architecture docs (§21), the user docs the notes list,
  ledger capture (§19), the brain's decision recorded, close the ticket, archive, commit.
