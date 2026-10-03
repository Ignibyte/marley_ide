# The Knowledge panel: a page's backlinks, links and tags, and brain search — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-646-knowledge-panel.md
- **Pipeline spec:** 646-knowledge-panel.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-03)
- **Request:** Chad, 2026-10-02, on Rusty in Marley: "maybe rusty becomes Marley. We would take
  our rusty custom QML app and build it inside of marley." The plan
  (`docs/marley/rusty-in-marley.md`) puts Rusty's RightPane (backlinks, outgoing, tags) and its
  SearchPane in a Knowledge panel in the right dock (R-D3), opening on the project's page when no
  page is focused (R-D5). For this batch, 2026-10-03: "lets make a plan to begin the work and spec
  out the tickets", "lets make sure we use the gpui components we found here" (R-D10), and "Queue
  all five". Order: #643 the switch and connection, #644 the rail's Brain view, #645 the Page tab,
  #646 this panel, #647 the Graph tab.
- **Classification / tier:** feature, medium. A new right-dock panel in `marley_workbench`, a new
  pure module in `marley_rusty`, additions to #643's stand-in and fixture vault. One Zed touch is
  possible: `"rusty"` in `zed.rs`'s `test_action_namespaces`, if no earlier ticket of the batch
  added it. No new crate, no new dependency, no new spawn site.
- **Scope split (reported to the batch):** the assignment named R3 and R6. Drafted whole, the
  ticket held a new panel with three page sections, a search with toggles and keys, live refresh,
  the switch's two states, and then the project view's own reads (every project page's frontmatter,
  `brain_due`, the task tools), a resolution rule and a task-group rule: eighteen criteria before
  the project view's five or six. The panel, page view and search are one shippable piece; the
  project view (R6, R-D5) is drafted in Out with its own scope, and its discovery is kept below
  for its ticket. `rusty: open page`, R3's other half, is also Out.
- **Recall (§18.3):**
  - PR-claude-607-ask-the-dock-whether-a-panel-shows-001: whether the panel shows is
    `dock.visible_panel()` downcast, not `set_active` counts. Used for "close the dock only when it
    shows this panel" (D1).
  - L-claude-457-a-single-line-editor-hands-zeds-list-keys-to-its-container-001: in a single-line
    `Editor`, Up, Down and Escape arrive as `menu::SelectPrevious`, `SelectNext` and `Cancel` on
    the container; a container that handles `Cancel` propagates when it has nothing to do;
    `HighlightedLabel` takes byte offsets on char boundaries (the snippet and mention ranges).
  - L-claude-439-focus-lands-in-a-dock-panel-only-once-its-dock-is-open-001: focusing the search
    field goes through `workspace.focus_panel`, which opens the dock first.
  - AD-claude-456-a-layout-round-trip-gives-each-dock-back-its-panel-001 and
    F-claude-456-...-001: the Marley layout switch notes each dock's shown panel by persistent name
    and gives it back, so the Knowledge panel survives a layout round trip as the Fleet panel does.
  - PR-claude-a-button-in-a-card-that-takes-the-focus-stops-its-click-001: the Create button in a
    link row stops propagation.
  - L-claude-493-zed-gives-a-lost-focus-to-the-panes-front-item-001: opening a page from the panel
    may move the focus to the new Page tab; #645's action decides, and the scenario re-focuses the
    field before typing.
  - AD-claude-611-workflow-stores-are-polled-together-each-call-bounded-001 and
    F-claude-611-an-unreachable-stores-header-showed-its-outer-error-001: calls bounded; an error
    shown by its root cause, the chain logged once (D7, REQ-017).
  - F-claude-608-a-third-of-the-panel-hid-the-snapshots-question-001: a default size chosen before
    the content is drawn; the Test phase checks 320 px against the shots.
  - L-claude-633-scenarios-copy-the-users-settings-so-defaults-reach-real-services-001,
    L-claude-607-a-settings-edit-after-marleys-own-write-does-not-reload-001 and
    L-claude-633-the-mcp-servers-page-opens-by-a-keymap-in-the-runs-profile-001: the scenario
    sets `marley.rusty` from outside only, sets `connection` to `embedded` so nothing reaches the
    running service, and binds the open-page action with its argument in the run's keymap.
  - L-claude-635-a-harness-helper-replaces-a-scenarios-own-of-the-same-name-001: the scenario's
    helpers are named apart from the harness's.
  - Completed pipelines: 607 and 608 (the Fleet panel: `MarleyFleetPanel`, right dock only,
    priority 20, unique priorities per dock, `dock.rs:784-796`), 611 (the MCP client over
    `context_server`), 457 (the rail's filter field and its keys), 633 (the stand-in `rusty-mcp`
    written by a scenario).
  - Brain (`rusty-cli brain search`, read only):
    `decisions/marley-draws-rustys-knowledge-workspace-rusty-keeps-its-data-marley-d11-amended`
    (D11 as amended: Rusty stays the only writer, so Create goes through `brain_new_page`) and
    `decisions/rusty-files-marleys-rq1-rq5-as-ticket-035-to-039-and-m10-the-qt-app-frozen`
    (TICKET-035 to 037 open, none built). Promotion asks the brain (`brain_ask`); Complete records
    the decision.
- **Discovery:**
  - Marley: `crates/marley_workbench/src/fleet.rs` (`init` `:298-313` registers `ToggleFleet` and
    adds the panel on each new workspace with a window; `render` `:1427-1488` with key context
    `FleetPanel menu` and the `menu` actions; `Panel` impl `:1498-1542`; `ACTIVATION_PRIORITY`
    `:44`). `crates/marley_workbench/src/marley_workbench.rs` (`actions!(marley, …)` `:117`,
    `ToggleFleet` `:333`, `MarleySettings` `:342`, `rusty::init` `:770`, `fleet::init` `:799`).
    `crates/marley_workbench/src/rusty.rs` (#633's offer, one file today; #643 adds the client).
  - Zed, the dock: `crates/workspace/src/dock.rs` `Panel` `:36-104` (`enabled` defaults true at
    `:92`); `first_enabled_panel_idx` `:552-561` (the only reader of `enabled` in `workspace`);
    `PanelButtons::new` observes the dock and the settings store `:1380-1384`; its render skips an
    entry whose `icon` is `None` `:1409`. The dock's own render does not read `enabled`
    (`:1271-1273`), so a shown panel stays shown until something closes it.
    `crates/workspace/src/workspace.rs` `close_panel` `:4903-4911` closes every dock that holds
    the panel, shown or not (hence D1's check); `toggle_panel_focus` `:4776`; `active_item`
    `:4392`; `Event::ActiveItemChanged` `:1552`.
  - Zed, the Agent Panel: `crates/agent_ui/src/agent_panel.rs` `icon` `:5113-5115` returns `None`
    unless `enabled`; `enabled` `:5129-5131`; `toggle_focus`, `focus` and `toggle` check it
    `:1614-1656`. `crates/zed/src/zed.rs` `setup_or_teardown_ai_panel` `:818-851` adds or
    removes the panel when `disable_ai` changes (not copied, D1).
  - Zed, following: `crates/outline_panel/src/outline_panel.rs:1090-1114` (a `subscribe_in` on the
    workspace for `ActiveItemChanged`, clearing when the active item is not an editor) and
    `:5424-5433`.
  - Zed, the parts: `crates/ui/src/components/label/highlighted_label.rs:17-60` (`new` takes char
    boundaries and debug-panics otherwise; `from_ranges` takes byte ranges);
    `crates/ui/src/components/list/` (`ListItem` with `start_slot`, `end_slot`,
    `end_slot_on_hover`, `on_click`, `toggle_state`; `ListSubHeader`); `count_badge.rs`;
    `chip.rs`; `disclosure.rs`; `crates/editor/src/editor.rs:1802` (`Editor::single_line`);
    `crates/search/src/search.rs:109-115` (the option icons) and `:129-160` (`as_button`, an
    `IconButton` with `IconButtonShape::Square` and `toggle_state`, tied to the search crate's
    actions); `crates/icons/src/icons.rs` (`Book`, `Link`, `Hash`, `Plus`, `CaseSensitive`,
    `Regex`). `crates/context_server/src/types.rs:120-121` (`ResourcesListChanged`) and
    `client.rs:509` (`on_notification`), which #643's client uses.
  - Zed's action namespaces: `crates/zed/src/zed.rs:5868` `test_action_namespaces`, with
    `"marley"` at `:5954` under a `// Marley:` comment; a `rusty` namespace needs its line there,
    between `"repl"` and `"search"`, and the existing `zed.rs` row of `zed-touchpoints.md` (`:68`)
    names that list.
  - Rusty's tools (`/srv/stacks/rusty-v3/crates/rusty-mcp/src/main.rs`): `brain_search`
    `:831-864` (operators in its description; `limit` defaults to 10; captured sources marked
    `untrusted`), `brain_get_links` `:1614-1621`, `brain_graph` `:1402-1418`, `brain_tags`
    `:1420-1426`, `brain_new_page` `:1264-1277`, `brain_due` `:1185-1190`, `brain_list_pages`
    `:921-932` (default limit 50), `brain_read_page` `:908-919`, `list_task_groups` `:773-776`,
    `list_tasks` `:786-796`; parameters `BrainSearchParams` `:78-89`, `NewPageParams` `:595-601`.
  - Rusty's core (`crates/rusty-core/src/brain/`): `LinkEntry` `mod.rs:150-164` (`from_slug`,
    `to_slug`, `link_type`, `context` = the line the link sits on, `resolved`); `get_links`
    `:2151-2184` (outbound in source order with `resolved`; an unresolved `to_slug` is the target
    as written, `links.rs:159-163`; backlinks ordered by `from_slug`); `TagCount` `:208-216` and
    `tags` `:1046-1080` (nested tags counted under their parents); `graph` `:885-1043` (each page
    node carries its tags from the `brain_tags` table, frontmatter and inline; `around` keeps the
    neighbourhood; it reads every decision page for the typed edges); `BrainSearchResult`
    `:67-79` (`slug`, `page_type`, `title`, `snippet`, `rank`); `ParsedQuery` `:95-133`; snippets
    marked with `<b>` and `</b>` (`:1159`, `:1177`, `:3183-3200`); `sources.rs:530-557`
    (`untrusted` on captured sources). `BrainFrontmatter` `frontmatter.rs:18-41` (`tags` is the
    frontmatter list only).
  - Rusty's app (`crates/rusty-app/qml/`): `RightPane.qml` backlinks `:164-203` (title from a
    titles map, context up to three lines), outgoing `:205-240` (link or unlink icon, "create"
    on an unresolved one), outline `:242-285`, tags `:287-354` (the vault's tags as a tree with
    counts; a click searches, `+` or T tags the page), the agent `:356-414`. `SearchPane.qml`
    `:45-56` (`brain_search` with limit 60, `case_sensitive`, `regex`), `:60` (250 ms), `:83-97`
    (the field, Aa, `.*`, bookmark), `:98-105` (the hint), `:112-141` (title, slug, snippet),
    `:73` (re-search on a data change). `Main.qml:506` (`createPage` is `brain_new_page { folder:
    "", name }`), `:929` and `:1339` (a tag search is `"tag:" + tag`). `DecisionsPage.qml` (the
    due, then every decision; a click opens).
  - Ely (`…/scratchpad/repos/ely`, `2f8b2f6`, `LICENSE-MIT`: "Copyright (c) 2026 Ely GPUI
    Component contributors"): `src/documents/knowledge.rs:20-119` (`Backlink`, `Backlinks`;
    asserts the mention range lies on char boundaries); `src/editor/search.rs:30-62` (`trimmed`,
    with its tests `:441-490`), `:76-197` (`SearchResultItem`), `:199-373` (`SearchPanel`);
    `src/editor/filters.rs:12-127` (`passes`, `SearchFilters`). Its story pages
    `backlinks-graphview` and `searchpanel-searchresultitem-searchfilters`
    (`examples/gallery/stories.json`).
  - For the deferred project view (kept for its ticket): Rusty's project pages carry a free-text
    `path:` property: one absolute path, a comma-separated list of paths, or a path on another
    machine in words; matching a workspace's worktree roots needs a split on commas, a trim and `~`
    expanded, and the other-machine form never matches. No tool returns a page's frontmatter in a
    list, so resolution reads each `type: project` page (`brain_list_pages` with a raised limit,
    then `brain_read_page`) and keeps the map until a change signal. Rusty's task groups today are
    named by area, not by project, so a rule by name alone would match few projects; an explicit
    property on the project page naming its group, falling back to a group named as the page's
    title, is the candidate rule. `brain_due` carries no link to a project: the due decisions for a
    project are those among the project page's backlinks.
- **Decisions:** D1 to D9 in the spec. In short: the panel is always added and hides itself as the
  Agent Panel does, closing its dock only when it shows; it follows the active item; a page view
  from `brain_get_links`, `brain_graph` around the page and `brain_tags`; no outline; search in a
  single-line `Editor` with Rusty parsing the operators; the Ely port's pure parts in
  `marley_rusty` and the rows over Zed's `ui`; live by the change signal, coalesced; Create in one
  click; the project view split out.

### Design
- **Approach.**
  - `crates/marley_rusty/src/knowledge.rs` (new; Ely's MIT notice at the top, naming the files it
    ports). Typed views of the answers this ticket reads, where #643 has not made them: `Link`
    (`from_slug`, `to_slug`, `context`, `resolved`), `PageLinks { outbound, backlinks }`,
    `TagCount { tag, count }`, the `brain_graph` node subset (`id`, `kind`, `title`, `tags`),
    `SearchHit { slug, page_type, title, snippet }`, all `serde::Deserialize`. `PageKnowledge`
    built from them: the page's title, its tags with counts (sorted as `brain_tags` sorts),
    `Vec<Backlink>` and `Vec<Outgoing>`. `Backlink { slug, title, context, mention:
    Option<Range<usize>> }` (Ely's model, the icon dropped and the range optional);
    `fn mention(context, slug, title) -> Option<Range<usize>>`: the first `[[…]]` whose target up
    to `|` or `#` equals the slug, its last segment or the title, case aside, the range covering
    the brackets. `fn trimmed(text, ranges, lead) -> (String, Vec<Range<usize>>)`: Ely's, with the
    lead a parameter. `fn snippet(marked) -> (String, Vec<Range<usize>>)`: Rusty's `<b>` and
    `</b>` removed and their spans returned as byte ranges; anything else is text. No gpui.
  - `crates/marley_rusty/src/marley_rusty.rs`: `pub mod knowledge;`.
  - #643's stand-in and fixture vault (Marley crate `marley_rusty`): the pages the UI proof names;
    answers for `brain_get_links`, `brain_graph` with `around`, `brain_tags`, `brain_search` (a
    substring match is enough; the stand-in need not parse operators beyond `tag:` for 08) and
    `brain_new_page` (writes the page into the scratch vault and announces a change); a log of
    each `tools/call` with its arguments; where #643's stand-in already does any of it, nothing.
  - `crates/marley_workbench/src/rusty/knowledge_panel.rs` (new), declared from `rusty.rs`:
    - `actions!(rusty, [ToggleKnowledgePanel])` with its doc ("Shows or hides the Knowledge panel:
      the open brain page's tags, backlinks and links, and brain search").
    - `init`, called from `rusty::init` (or beside `fleet::init`): `observe_new` on `Workspace`
      registers the toggle (checks `MarleySettings::rusty.enabled`; off, `show_toast` with
      `NotificationId::unique::<KnowledgePanel>()`; on, `toggle_panel_focus`) and adds the panel.
    - `KnowledgePanel { workspace: WeakEntity<Workspace>, focus_handle, search: Entity<Editor>,
      options (case, regex), page: Option<Shown>, results: Option<Results>, selected:
      Option<usize>, debounce: Task<()>, reads, _subscriptions }`. Subscriptions: the workspace's
      `ActiveItemChanged` (find the Page tab with #645's downcast, read its slug, observe it for a
      page change), the search editor's `confirm` on Enter (no timer; see D5, reconciled 2026-10-03, on the background
      executor), the settings store (`cx.notify()` so the switch is seen at the next render), and
      #643's change signal.
    - `render`: with Rusty off, an empty `div` and a `cx.defer` that updates the workspace,
      reads the right dock's `visible_panel()`, and closes the dock only if it is this panel. With
      the client not connected, the header, the field and the client's state line. Otherwise the
      header (`KNOWLEDGE`, as `FLEET`), the field row (the `Editor`, two `IconButton`s with
      `IconName::CaseSensitive` and `IconName::Regex`, `IconButtonShape::Square`,
      `toggle_state`, a tooltip, re-focusing the field after a click), then the results, the page
      view or the no-page line. Key context `KnowledgePanel menu` with `menu::SelectNext`,
      `SelectPrevious`, `Confirm` and `Cancel` (propagate when the field is empty).
    - Rows: tags as clickable `Chip`s (`#name` and the count) in a wrapping row; backlinks as
      `ListItem`s, the title over `HighlightedLabel::from_ranges` of the trimmed line in muted
      colour; links as `ListItem`s with `IconName::Link` or `IconName::Plus`, an unresolved one's
      `end_slot` a `Button` "Create" that stops propagation; results as `ListItem`s with the
      title, the slug muted and the snippet's `HighlightedLabel`. Section headers are
      `ListSubHeader`s with a `CountBadge`.
    - `Panel`: `persistent_name` and `panel_key` `MarleyKnowledgePanel`, right only,
      `default_size` 320 px, `icon` `Some(IconName::Book)` only while `enabled`, tooltip
      "Knowledge", `toggle_action`, `activation_priority` 21, `enabled` reading the switch,
      `activation_focus_handle` the search field's.
    - Reads: page reads issue `brain_get_links` and `brain_graph { around, depth: 1 }` together
      and `brain_tags` when no counts are held; a search issues `brain_search`; each read keeps
      its generation and drops a stale answer; a change signal during a read sets a pending flag
      that runs one more read when it ends.
    - Opening: a click or Enter dispatches #645's open-page action with the slug; Create calls
      `brain_new_page`, then opens the slug it returns; an error from either goes to a toast with
      its root cause.
  - `crates/zed/src/zed.rs` (Zed crate `zed`): `"rusty"` in `test_action_namespaces` with a
    `// Marley:` comment, only if no earlier ticket of the batch added it.
  - `crates/marley_workbench/guide/index.html`: a Knowledge panel entry beside the Fleet panel's
    (`:252`, `:1525`), in the Code phase.
- **File manifest.**
  - Marley: `crates/marley_rusty/src/knowledge.rs` (new), `crates/marley_rusty/src/marley_rusty.rs`,
    #643's stand-in and fixture files in `crates/marley_rusty`;
    `crates/marley_workbench/src/rusty.rs`, `crates/marley_workbench/src/rusty/knowledge_panel.rs`
    (new); `crates/marley_workbench/guide/index.html`;
    `script/e2e/646-knowledge-panel.sh` (new, Test phase).
  - Zed: `crates/zed/src/zed.rs` (crate `zed`), conditional, one line.
- **The ledger rows it extends** (`docs/marley/zed-touchpoints.md`, written before the code, §14):
  only if the namespace line is added, the `crates/zed/src/zed.rs` row (`:68`): "`"marley"` in
  `test_action_namespaces`" becomes "`"marley"` and `"rusty"` (#646, the Knowledge panel's toggle)
  in `test_action_namespaces`". No other Zed file changes.

### Visual check plan
The scenario `script/e2e/646-knowledge-panel.sh`, `compositor sway`. Setup: a scratch repository
opened with `open_path`; #643's stand-in `rusty-mcp`, named in `MARLEY_RUSTY_MCP` (never put first on the PATH: Marley takes its PATH from the login shell, L-531, so the real `rusty-mcp` could win; reconciled 2026-10-03 to #643's rule), over a scratch vault built
from `marley_rusty`'s fixtures plus this ticket's three pages, its call log in `$E2E_WORK`; the
run's keymap binds Ctrl+Alt+Shift+D to #645's open-page action with `projects/demo`; the copy's
`marley.rusty.enabled` stays false (checked with `expect`) and `marley.rusty.connection` is set to
`embedded`. Checks of the call log are shell functions over the log file. Row coordinates are
measured from `646-04` in the Test phase.

| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | Trust the project; settle | `646-01-off-no-button`: Fleet's button, no Knowledge |
| REQ-002 | The palette: `rusty: toggle knowledge panel` | `646-02-off-toast`: the toast's words, no panel |
| REQ-003 | `set_setting marley.rusty.enabled true`; settle; the toggle | `646-03-no-page`: the button; the panel, the field's cursor |
| REQ-004 | As 03; later `workspace: new file` | `646-03-no-page`, `646-06-other-item`: the no-page line |
| REQ-005 | Ctrl+Alt+Shift+D | `646-04-demo-page`: Demo, `projects/demo`, three tags with counts |
| REQ-006 | As 05 | `646-04-demo-page`: Backlinks 2, each line with its link lit |
| REQ-007 | As 05 | `646-04-demo-page`: Links 2, Orbit, Missing Page with Create |
| REQ-008 | Click Orbit's backlink row | `646-05-followed`: Orbit in the tab and in the panel |
| REQ-009 | Ctrl+Alt+Shift+D; click Create on Missing Page | `646-07-created`: Missing Page shown; the log's `brain_new_page` line |
| REQ-010 | Ctrl+Alt+Shift+D; click `#area/demo` | `646-08-tag-search`: `tag:area/demo`, two rows; the log's query |
| REQ-011 | Click the field; select all; type `orbit`; settle 1 | `646-09-search`: three rows, `Orbit` bold; one `orbit` query in the log |
| REQ-012 | Click the two toggles | `646-10-toggles`: both on; the log's flags |
| REQ-013 | Click the field; Down; Enter | `646-11-enter-opens`: the second row's page in the tab |
| REQ-014 | Ctrl+Alt+Shift+D; the field; type `orbit`; Escape | `646-12-escape`: the field empty, Demo's sections |
| REQ-015 | Write a new page linking Demo; the stand-in's trigger; settle | `646-13-refresh`: Backlinks 3 |
| REQ-016 | `set_setting marley.rusty.enabled false`; settle | `646-14-off-live`: the right dock closed, no button |

Not reached by a scenario: the real Rusty (R-D8); a client that is not connected and a call that
fails (REQ-017, review: the stand-in could be made to fail, but #643 owns the connection's states
and their words); a right dock restored at start on the Knowledge panel while Rusty is off (one
launch per scenario; review of the deferred close). REQ-018 by review and the gate.

### Risks
- **Two tickets ahead.** Every name here from #643 (the client, its state, its change signal, the
  stand-in, its trigger and log, the setting's path) and from #645 (the Page tab type, its slug,
  its page-change event, the open action and whether it opens in the same tab) is a placeholder;
  promotion replaces them. If #645 does not emit an event when its tab's page changes, the panel
  observes the tab entity and compares slugs.
- **`brain_graph` per page.** It builds the whole graph and reads every decision page (a few
  hundred here) for the typed edges, on each page shown and each change signal. Fine at this size;
  if it shows, the answer is a Rusty-side request (tags and titles in `brain_get_links`), not a
  cache in Marley.
- **Mentions not lit.** A link written through an alias, or to a heading of another name, has no
  range; the row shows the line plain (D6).
- **Snippet marks.** A page whose text holds a literal `<b>` is drawn as a highlight, as Rusty's app
  does.
- **The close while drawing.** Closing from the panel's own render would update the workspace
  inside the panel's update; it is deferred. A dock restored at start on this panel with Rusty off
  draws one empty frame before it closes.
- **The toggles and focus.** A click on an `IconButton` can take the focus from the field; the
  handler gives it back, as Zed's buffer search does, or Up, Down and Enter stop reaching the list.
- **Change storms.** An agent writing many pages sends many `list_changed`; the coalescing keeps it
  to one read in flight and one queued per kind.
- **Writes.** Create writes to the user's brain in real use; in the scenario only to the stand-in's
  scratch vault. The scenario's `connection: embedded` keeps a copied `service` setting from
  reaching the running `rusty-mcp.service`.
- **The namespace.** If #643 or #645 adds `rusty` actions first, it adds the namespace line, and
  this ticket touches no Zed file.

### User-facing docs to update at Complete (P4)
- `docs/marley/guide.md`: a "The Knowledge panel" section beside "The Fleet panel" (`:1536`), the
  palette list (`:1698`), and the Rusty section #643 adds.
- `docs/marley/walkthrough.md`: a stop for the panel after Part 12, the Fleet panel (`:1504`), and
  the palette table (`:1599`).
- `crates/marley_workbench/guide/index.html`: in the Code phase (above).
- Architecture (§21): `docs/marley_architecture/marley_workbench.md` gains a section beside "The
  Fleet panel" (`:2464`) and "Rusty's tools for Zed's agents" (`:1821`); `marley_rusty`'s
  architecture page (#643's) gains `knowledge`; `docs/marley/rusty-in-marley.md`'s slices table
  notes R6 split from this ticket; `CHANGELOG.md` under Added.

### Checklist (no TaskCreate in this harness)
- [x] Read CONSTITUTION §3, §7, §14, §18, §19, §20, the templates and the brief.
- [x] Read `docs/marley/rusty-in-marley.md` whole (R-D0 to R-D10, the slices, Rusty's triage).
- [x] Read Rusty's `RightPane.qml`, `SearchPane.qml` and `DecisionsPage.qml`, and the tools and
      types behind them in `rusty-mcp` and `rusty-core`.
- [x] Read Ely's `knowledge.rs`, `search.rs` and `filters.rs` whole; named what is taken and left.
- [x] Recall: the knowledge ledgers (PR-607, L-457, L-439, AD-456, F-456, L-493, AD-611, F-611,
      F-608, L-633 twice, L-607, L-635), the completed pipelines 457, 607, 608, 611 and 633, and a
      read-only brain search.
- [x] Discovery with file:line: the dock and `Panel`, the Agent Panel's switch, the outline panel's
      following, the `ui` parts, the search crate's toggles, the Fleet panel, the action namespace
      list, Rusty's tools and types, Ely's components; the project view's facts for its ticket.
- [x] Prior-art sweep, three legs: Zed's, Warp's and Orca's behavior maps; Obsidian, MCP, Rusty's
      app, Ely; Zed's crates and Marley's own.
- [x] Scope split: the project view and `rusty: open page` to Out, each with its scope line.
- [x] Spec: scope, Reference (§20), Prior art, UI proof, D1 to D9, eighteen EARS rows, phase plan.
- [x] Design: approach, file manifest by crate, the one conditional touchpoint row, the visual
      check plan, risks, the user docs for Complete.
- [x] Docs only: the ticket doc and this pair; no BACKLOG.md, no `active/`, no source, no cargo.
