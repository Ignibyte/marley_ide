# Notes: The Knowledge panel's project view: a project's brain page, follow-ups and tasks

- **Local ticket doc:** docs/planning/tickets/open/TICKET-655-knowledge-panel-project-view.md
- **Pipeline spec:** 655-knowledge-panel-project-view.spec.md

## Phase 1: Plan (queued by /spec, 2026-10-03)
- **Request:** Chad, 2026-10-02, the idea behind Rusty in Marley: "an all in one system for Marley
  which brings in obsidian like knowledge graphs and also voice talking so that you can manage
  these projects + have a sort of developer centric knowledge system for yourself", then "maybe
  rusty becomes Marley" and "Plan it now". The plan's R-D5: "Each rail project resolves to a brain
  project page (by the page's `path:` frontmatter, then by name) and a Rusty task group. The
  Knowledge panel opens on the project's page when no page is focused: the page, follow-ups due
  for it, its tasks. This is what 'manage these projects' needs first." For the batches,
  2026-10-03: "lets make a plan to begin the work and spec out the tickets" and "lets make sure we
  use the gpui components we found here". #646 split R6 out (its D9 and Out) and kept the
  discovery for this ticket; #647 left its no-page centre to R6 (its Out).
- **Classification / tier:** feature, M (the plan's size for R6). One new pure module in
  `marley_rusty`, one new adapter module in `marley_workbench::rusty` (the cache, the resolution,
  two pickers, two actions), additions to #646's panel and #647's tab, and to the stand-in and its
  fixtures. No Zed touchpoint: the `rusty` action namespace is in Zed's namespace test since #645
  (or #646), and every Zed crate used (`project`, `workspace`, `picker`, `fuzzy`, `ui`, `util`) is
  used as it is. No new crate, no new dependency, no new spawn site, no socket.
- **Scope:** one shippable slice, not split. The project view needs the join to show anything, the
  join needs a way out when it fails (the link), and real data has every case the rule names (a
  path match, a name match, a tie at each tier, no match), so none of them can wait. Tasks would
  show for no project without Link a Task Group (no group is named like a page), so it stays in.
  The Graph tab's centre is one fallback in one function. Creating a project page, the `repo:`
  tier and acting on the listed rows are Out, each with its one-line scope in the spec.
- **Recall (§18.3):**
  - PR-claude-retry-keyed-on-empty-cache-needs-every-outcome-to-write-001: a cache whose "nothing
    held" condition starts a read loops on an error that writes nothing. The project pages cache
    keeps an empty list and a failure as outcomes (D8).
  - L-claude-572-an-observed-global-written-at-every-wakeup-redraws-the-rail-001: `default_global`
    and `global_mut` notify observers whether or not a value changed. `ProjectPages` is written only
    when its pages differ, and the read bookkeeping lives in a second global nothing observes.
  - L-claude-458-a-projects-folder-events-come-before-its-group-is-rekeyed-001: a folder change
    arrives as `WorktreeAdded` or `WorktreeRemoved`, then `WorktreePathsChanged`, and the group is
    rekeyed on the second; the resolution is recomputed with `cx.defer_in` after it.
  - L-claude-602-zeds-project-group-list-never-holds-a-folderless-workspace-001: a projectless
    workspace is not in Zed's group list; it has no project here either (D1).
  - BF-claude-root-keyed-state-aliases-duplicate-roots: state keyed by a non-unique attribute
    aliases its duplicates. One folder is listed by two pages, so the path tier needs its tie rule
    (D2) rather than a map from folder to page.
  - AD-claude-450-new-agent-is-a-picker-behind-the-marley-keymap-001 and
    L-claude-450-driving-a-picker-and-a-keymap-in-a-marley-test-001: a picker in the workspace's
    modal layer, which takes focus in a deferred callback; the scenario types after a settle.
  - L-claude-635-a-scenarios-fixed-menu-steps-can-pass-on-the-wrong-entry-001: a step that walks a
    list by position checks the entry it reached. Each pick's check is the slug and value the
    stand-in logged, not the row's place.
  - AD-claude-611-workflow-stores-are-polled-together-each-call-bounded-001 and
    F-claude-611-an-unreachable-stores-header-showed-its-outer-error-001: every call bounded; an
    error shown by its root cause, the chain logged once (D8, D9).
  - PR-claude-a-button-in-a-card-that-takes-the-focus-stops-its-click-001: the Link buttons in the
    name tie's rows stop their click, so the row under them does not take it.
  - L-claude-493-zed-gives-a-lost-focus-to-the-panes-front-item-001: closing the Page tab gives the
    focus to the pane's front item; the panel follows the active item, not the focus, and the
    scenario clicks the panel before it reads it.
  - L-claude-515-dispatch-through-the-window-from-inside-an-action-001: the view's buttons dispatch
    `rusty::LinkProjectPage` and `rusty::LinkTaskGroup` through `window.dispatch_action`.
  - L-claude-504-a-click-that-passes-can-still-miss-its-target-001: the buttons' and rows' places
    are measured from the first run's shots, and each shot is taken right after its click.
  - L-claude-531-marley-takes-its-path-from-the-login-shell-so-stand-ins-are-named-001,
    L-claude-633-scenarios-copy-the-users-settings-so-defaults-reach-real-services-001 and
    L-claude-607-a-settings-edit-after-marleys-own-write-does-not-reload-001: the stand-in named in
    `MARLEY_RUSTY_MCP`, `connection: embedded` so nothing reaches the running service, every
    change made from outside.
  - L-claude-635-a-harness-helper-replaces-a-scenarios-own-of-the-same-name-001: the scenario's
    helpers (`vault_write`, `stand_in_signal`, `logged_write`) are named apart from the runner's.
  - AD-claude-609-the-agent-tab-is-read-only-and-its-samples-live-in-memory-001: a view reads only
    while it shows; the project view's reads run while the panel draws it.
  - Completed pipelines: 450 (the picker in the modal layer), 458 (folder changes and the rekey),
    600 and 602 (projectless groups; the group list), 606 (closed projects keep their group key),
    611 (the MCP client over `context_server`), 633 (a stand-in `rusty-mcp` written by a scenario).
  - The follow-up drafts beside this one (2026-10-03): #654 (`rusty: open page`, create on a miss),
    #656 (the Page tab's property edits), #658 (the Tasks tab; its Out leaves "the project's task
    group link" to this ticket and offers `TasksView::show_list(id)`) and #659 (the Decisions tab;
    its `marley_rusty::decisions` is the `brain_due` view this ticket reads).
  - The queued batch, read whole: #643 (the client, its bounded call, `MARLEY_RUSTY_MCP`, the Python
    stand-in over a scratch state folder), #644 (the stand-in's `SIGUSR1`, pid file and call log;
    `marley_rusty::vault`), #645 (`rusty::OpenPage`, `rusty::page::open_later` in `window.defer`,
    the `rusty` namespace), #646 (the panel, its no-page state, its D7 coalescing, its section
    headers, its stand-in's `brain_get_links`, and the discovery kept for this ticket) and #647 (the
    centre as a slug, its D1 open rule and D2 following, its notice).
  - #646's kept discovery, re-checked: the forms of `path:` hold; the count is 101 project pages
    today, not about 102; `brain_list_pages` returns 50 by default, so a raised limit is needed; and
    a project's due decisions are found among its backlinks and its outbound links both, since
    `brain_decide` also writes a timeline entry on each consulted page.
  - Brain (`rusty-cli brain search`, read only, no consultation recorded): two project pages name
    their task group in their body, by its name (one also by its number in words), which is how D5
    names a group; Rusty's store skill `intake` carries the project page template with `path: <local
    path, if any>`, `repo`, `status` and `summary: <one or two plain sentences>`. Promotion asks the
    brain (`brain_ask`); Complete records the decision.
  - The vault and the store, measured read only on 2026-10-03 (`sqlite3 -readonly` and reads of the
    project page files; counts only, nothing copied): 101 project pages, all under `projects/`; 32
    with `path:` (24 one path, 6 two, 1 three, 1 four), 43 parts: 39 absolute, 2 under `~`, 2 naming
    another machine in words; 41 of the parts exist on this box; one folder listed by two pages; no
    listed path inside another; `summary:` on 36 pages (107 to 306 characters); `repo:` on 27 (23
    with `path:`); every page has a plain first paragraph; no slug under `archive/`. Over the 53
    folders of `/srv/stacks`: 37 match by path (1 tie), 3 more by name (1 tie), 13 neither. 239
    decisions; 135 links from decisions to project pages, 157 from project pages to decisions; 15
    follow-ups due today, all 15 linked to a project page. 5 task groups (named by area), 17 open
    tasks; no group named like a project page.
- **Discovery:**
  - Rusty (`/srv/stacks/rusty-v3` at `295565c`, MIT), the tools: `rusty-mcp/src/main.rs`
    `list_task_groups` `:773-776` (no parameters), `list_tasks` `:786-796` (`group_id`,
    `include_archived`, `ListTasksParams` `:43-51`), `brain_read_page` `:908-919` (`null` for a
    missing page), `brain_list_pages` `:921-932` (`page_type`, `limit`, `ListPagesParams`
    `:98-105`), `brain_decide` `:1134-1152`, `brain_due` `:1185-1190` (`days`, default 0,
    `DueParams` `:403-409`), `brain_set_property` `:1427-1440` ("text, number, true/false, a
    YYYY-MM-DD date as text, or a list of strings; other keys keep their order and the body is
    untouched", `SetPropertyParams` `:658-668`, `value` a JSON value), `brain_get_links`
    `:1614-1621`; `spawn_indexer` `:2008-2057` (the vault synced 5 s after a change, `:2049`) and
    `spawn_change_notifier` `:2062-2082`.
  - Rusty's core: `rusty-core/src/brain/mod.rs` `BrainPage` `:31-52` (`slug`, `page_type`,
    `title`, `compiled_truth`, `timeline`, `frontmatter`, `content_hash`, `created_at`,
    `updated_at`); `BrainPageSummary` `:54-65` (`slug`, `page_type`, `title`, `updated_at`);
    `read_page` `:474-498` (the file read each time, timestamps from the index); `list_pages`
    `:573-620` (newest first by `updated_at`, `limit` 50 by default); `set_property` `:1086-1097`
    and `write_edited` (an unchanged text writes nothing); `get_links` `:2151-2184` (outbound in
    source order, backlinks by `from_slug`); `index_links` `:3089-3112`, called on the whole page
    (`:2804`), timeline included. `frontmatter.rs:18-41` (`BrainFrontmatter`: `title`, `type`,
    `aliases`, `tags`, `created`, `updated`, and every other key flattened from `extra`, so `path`,
    `summary` and `task_group` arrive as top-level keys of `frontmatter`). `decisions.rs`
    `DecisionSummary` `:19-27` (`slug`, `title`, `question`, `status`, `decided`, `follow_up_by`,
    `overdue`), `Due` `:72-75` (`due`, `all`), `decide` `:205-298` (Consulted links `:242-250`, the
    timeline entry on each consulted page `:282`), `due` `:416-438`. `engine/user_tasks.rs`
    `TaskHeader` `:10-18`, `UserTask` `:20-37` (`completed`, `archived`), `list_headers` `:53-73`
    (by `sort_order`). `vault.rs:494-530` (`title_to_slug`). `lib.rs:17-74` (the data watcher: a
    change announced after 600 ms of quiet; a sentinel `~/.rusty/.changed` for database-only
    changes that `rusty-cli refresh` touches).
  - Rusty's app: `qml/DecisionsPage.qml:22` (`brain_due` with `days: 0`), `:59` ("follow up by",
    overdue in its gold); `qml/TasksPage.qml:22-25` (`list_task_groups`, then `list_tasks` for one
    group), `:279-290` (a done task checked and struck).
  - Zed, the project's identity: `crates/workspace/src/workspace.rs:2506`
    (`Workspace::project_group_key`); `crates/project/src/project.rs:376`
    (`Event::WorktreePathsChanged`), `:2491` (emitted), `:6579` (`Project::project_group_key`),
    `:6590-6594` (`ProjectGroupKey { paths, host }`), `:6604-6611` (`from_project`: the main
    worktree path list), `:6623` (`path_list`), `:6627-6648` (`display_name`, the `.git` strip),
    `:6650` (`host`); `crates/project/src/worktree_store.rs:97` (`main_worktree_path_list`);
    `crates/workspace/src/multi_workspace.rs:272-276` (`ProjectGroup`).
  - Zed, the parts: `crates/picker/src/picker.rs:460` (`Picker::uniform_list`);
    `crates/ui/src/components/callout.rs:28-110` (`Callout`: `severity`, `icon`, `title`,
    `description`, `actions_slot`); `crates/ui/src/styles/typography.rs:165`, `:206`
    (`HeadlineSize`, `Headline`); `crates/ui/src/components/project_empty_state.rs:7` (read, not
    taken); `crates/icons/src/icons.rs` (`ArrowUpRight` `:41`, `Circle` `:71`, `Clock` `:73`,
    `Link` `:184`, `ListTodo` `:187`); `crates/util/src/paths.rs:24` (`home_dir`);
    `crates/worktree/src/worktree.rs:4628`, `:4671` (roots canonicalised for the watcher, hence
    the scenario's `realpath`).
  - Marley: `crates/marley_workbench/src/remote.rs:63` (`toggle_modal`), `:143-190` (`RemotePicker`,
    `ModalView`, `Focusable`, `Render`), `:192-330` (`RemoteDelegate`: `update_matches` with
    `fuzzy::match_strings` `:253`, an empty query keeping the given order, `confirm`, `dismissed`,
    `render_match` with `HighlightedLabel`); `marley_workbench.rs:933-947` (`group_names`);
    `rail.rs:6992-7037` (`rail_groups`: Zed's groups, then Marley's projectless ones), `:7041`
    (`build_snapshot`); `groups.rs:119-136` (`groups_of`); `Cargo.toml:20`, `:28`, `:52`, `:54`,
    `:74`, `:81` (`chrono`, `fuzzy`, `picker`, `project`, `ui`, `workspace` already built).
    `rusty.rs` (#633's offer today; #643 to #647 grow it into the `rusty` module with `page`,
    `brain`, `knowledge_panel` and `graph_tab` under `rusty/`).
  - Ely (`…/scratchpad/repos/ely`, `2f8b2f6`, `LICENSE-MIT`): `src/chat/projects.rs:23-247`,
    `src/project/tasks.rs:30-251`, `src/project/work.rs:241-257`, `src/feedback/states.rs:143-173`,
    `src/navigation/palette/kinds.rs:309`, `:391-487`; what each is and why none is ported is in the
    spec's Prior art.
  - The e2e runner: `script/e2e.sh:15-45` (the scenario's contract), `:64` (`open_path`), `:384-405`
    (`holds`, `expect`), `:500-530` (`pointer_to`, `click`), `:562-581` (`profile_setting`), `:624`
    (`E2E_WORK`).
- **Decisions:** D1 to D12 in the spec. In short: the project is the workspace's project group;
  `path:` read as text or a list, only parts that are paths on this machine, compared lexically, a
  tie broken by name then slug; the name tier through Rusty's slug rule, a tie listed and never
  guessed, remote projects by name only; a link appends to `path:` on a pick through
  `brain_set_property`; the task group is the page's `task_group` by name, else the group named
  like the page, with its own link; follow-ups are `brain_due`'s joined by links both ways; the
  summary is the property, else the first paragraph; one app-wide cache with reads by
  `updated_at`, `archive/` skipped; the view from Zed's `ui`, nothing of Ely's ported; Zed's
  picker; the Graph tab falls back to the project's page; no switch of its own.

### Design
- **`crates/marley_rusty/src/project.rs`** (Marley crate `marley_rusty`, new, pure, no gpui and no
  IO; the home folder is a parameter, §14's directory rule):
  - Typed views, `serde::Deserialize`, unknown fields ignored: `PageSummary { slug, page_type,
    title, updated_at }` and the `brain_read_page` subset `PageRead { slug, title, compiled_truth,
    updated_at, frontmatter: Map<String, Value> }` here; #646's `marley_rusty::knowledge` link view
    for `brain_get_links`. The `brain_due` view and the task views have one owner each (§14):
    #659's `marley_rusty::decisions` and #658's `marley_rusty::tasks`. Whichever of #655, #658 and
    #659 lands first writes the module under that name with the fields the others read
    (`DecisionSummary`'s, and `TaskHeader`'s and `UserTask`'s), and the later ones take it.
  - `ProjectPage::from_read(PageRead) -> ProjectPage { slug, title, aliases: Vec<String>, path:
    PathValue, task_groups: Vec<String>, summary: String, updated_at }`, where `PathValue` is
    `Absent | Text(String) | List(Vec<String>) | Other` from the frontmatter's `path`.
  - `fn local_paths(value: &PathValue, home: &Path) -> Vec<PathBuf>` (D2's parse) and `fn
    lexical(path: &Path) -> PathBuf` (`Component` by `Component`: `CurDir` dropped, `ParentDir`
    pops, the rest kept; no disk).
  - `fn name_key(text: &str) -> Option<String>`: Rusty's `title_to_slug` rule, `None` when nothing
    is left.
  - `Project { folders: Vec<PathBuf>, remote: bool }` with `names()` (each folder's last part,
    `.git` stripped).
  - `fn resolve(project: &Project, pages: &[ProjectPage], home: &Path) -> Resolution`, where
    `Resolution` is `Page { slug, by: Matched::{Path, Name}, also: Vec<String> }`, `Candidates(Vec<
    String>)` (name tie, slugs in order) or `Unmatched { folders, names }`. Pages under `archive/`
    are dropped before either tier.
  - `fn path_value_with(value: &PathValue, folders: &[PathBuf]) -> Result<Value, PathRefusal>`
    (D4's four cases; `Value::String` or `Value::Array`), and nothing new when every folder is
    already listed (`Ok` with the value unchanged, so the caller writes nothing).
  - `fn task_groups(page: &ProjectPage, groups: &[TaskGroup]) -> GroupJoin`, where `GroupJoin` is
    `Named { found: Vec<TaskGroup>, missing: Vec<String> }`, `Like(TaskGroup)` or `None`.
  - `fn due_for(due: &[Decision], links: &PageLinks) -> Vec<Decision>` (due order kept) and `fn
    summary(page: &ProjectPage) -> String` (D7).
  - The module's tests carry the measured forms (one path, a comma list, `~`, another machine in
    words, a list), both ties, the four append cases, the group rule, the due join and the summary
    fallback; kept in the tree and built by gate:2, run by nothing until the testing phase (§7).
- **`crates/marley_rusty/src/marley_rusty.rs`:** `pub mod project;` and the tool names this ticket
  reads, beside #643's.
- **The stand-in and fixtures** (`crates/marley_rusty/stand_in/rusty-mcp`, `fixtures/`; added to
  where #643 to #647 left nothing): `brain_list_pages` (pages whose frontmatter `type`, else the
  folder's type, is `page_type`, newest first by file mtime, `updated_at` the mtime in seconds,
  `limit` 50 by default); `brain_read_page` (the fields above, `compiled_truth` the body above
  `## Timeline`, the frontmatter's keys flattened as Rusty's are; `null` when missing);
  `brain_due` (decisions whose `status` is decided or revised, or absent, and whose `follow_up_by`
  is on or before today plus `days`, `overdue` when before today, as `{ due, all }`);
  `brain_set_property` (the key's line replaced or added in the frontmatter, a list written as YAML
  items, the file written, the call logged with its arguments, `list_changed` sent; "Page not
  found: <slug>" for a missing page); `list_task_groups` and `list_tasks` from `tasks.json` in its
  state folder. A small reader of the fixtures' frontmatter forms (scalars, quoted text and lists),
  not a YAML parser: the standard library only, as #643's D8 has it. The fixtures: Demo's
  `summary:`; the three decisions and `projects/orbit-site` of the UI proof; `tasks.json` with the
  groups Demo and Chores and their tasks. All invented.
- **`crates/marley_workbench/src/rusty.rs`** (#643's module root): `pub(crate) mod project;`;
  `actions!(rusty, [LinkProjectPage, LinkTaskGroup])` with doc comments the palette shows ("Links
  this project to a brain project page: picks the page and adds the project's folders to its
  path." and "Links the project's brain page to a Rusty task group: picks the group and writes its
  name to the page's task_group."); both registered in `rusty::init`'s workspace observer behind
  the switch, as #645 registers `OpenPage`.
- **`crates/marley_workbench/src/rusty/project.rs`** (new):
  - `ProjectPages` (a `Global`, observed): `Option<Result<Arc<[ProjectPage]>, SharedString>>`,
    written only when it differs. `ProjectReads` (a `Global` nothing observes): the task in flight,
    a pending flag, the delayed second pass's task, and the `updated_at` each page was read at.
  - `ensure(cx)`: starts the first read when nothing is held and none is in flight. `refresh(cx)`,
    on #643's change signal: re-list now and once more 6 s later, coalesced (Rusty's indexer syncs a
    disk edit 5 s after its announcement, so the first list can predate the new `updated_at`);
    reads the pages whose `updated_at` moved, drops those gone. `reread(slug, cx)`, after a write.
    Every call through #643's client, bounded; the JSON parsed off the main thread
    (`background_spawn` with `futures::future::lazy`, L-482).
  - `fn project_of(workspace: &Workspace, cx: &App) -> Option<Project>` (the group key's paths and
    host; `None` with no folder) and `fn join(workspace, cx) -> Join`, `Join` being `NoProject`,
    `Reading`, `Failed(SharedString)` or `Resolved(Resolution)`, computed with `resolve` over the
    cache and `util::paths::home_dir()`; `fn project_page(workspace, cx) -> Option<String>` for
    #647's tab.
  - `link_page(workspace, slug, window, cx)`: `path_value_with` over the cached page, then
    `brain_set_property { slug, key: "path", value }` unless nothing is new, then `reread`; a
    refusal from the rule or from Rusty goes to a toast with its root cause
    (`NotificationId::unique::<ProjectPages>()`). `link_task_group(workspace, page, group, ..)`
    likewise with `key: "task_group"` and the group's name.
  - `PagePicker` and `GroupPicker` (`ModalView`, `Focusable`, `Render`, `EventEmitter<
    DismissEvent>`) with `PageDelegate` and `GroupDelegate` in `remote.rs`'s shape: the cache's
    project pages (title, slug muted) or `list_task_groups` read when the picker opens (name);
    `fuzzy::match_strings`, an empty query keeping Rusty's order; `confirm` calls `link_page` or
    `link_task_group` and dismisses; no-match and empty texts as the spec's D10 words.
  - The two actions' handlers: the switch checked (off, #643's toast); `LinkTaskGroup` with no
    resolved page shows "Link this project to a brain page first."; else `toggle_modal`.
- **`crates/marley_workbench/src/rusty/knowledge_panel.rs`** (#646's): the no-page state draws the
  project view when `project::join` gives anything but `NoProject`; the panel observes
  `ProjectPages`, subscribes to its workspace's project for `WorktreePathsChanged` (recomputed in
  `cx.defer_in`, L-458), calls `project::ensure` when the project view first draws, and on the
  change signal calls `project::refresh` beside its own reads. Its reads for a resolved page
  (`brain_get_links`, `brain_due { days: 0 }`, `list_task_groups`, `list_tasks` per group) follow
  #646's D7: one in flight per kind, one queued, each answer carrying the slug it was asked for.
  `render_project_view`, `render_unmatched` (the `Callout`), `render_candidates`,
  `render_follow_ups` and `render_tasks` draw D9's parts.
- **`crates/marley_workbench/src/rusty/graph_tab.rs`** (#647's): the centre is `self.centre` (the
  last Page tab's page) or else `project::project_page(workspace, cx)`; the header marks a project
  centre "(project)"; `open`'s start rule and `rusty: open local graph`'s notice per D11; the tab
  observes `ProjectPages` and reads again when a project centre's slug changes.
- **`crates/marley_workbench/guide/index.html`:** the Knowledge panel's article (#646's) gains the
  project view: how a project finds its page and its task group, linking, and the Graph tab's
  project centre. In the Code phase (the receipt binds it).
- **File manifest.**
  - Marley: `crates/marley_rusty/src/project.rs` (new), `crates/marley_rusty/src/decisions.rs` and
    `crates/marley_rusty/src/tasks.rs` (new only if #659 and #658 have not landed; else used as they
    are), `crates/marley_rusty/src/marley_rusty.rs`,
    `crates/marley_rusty/stand_in/rusty-mcp`, `crates/marley_rusty/fixtures/` (the pages,
    decisions and `tasks.json` above); `crates/marley_workbench/src/rusty.rs`,
    `crates/marley_workbench/src/rusty/project.rs` (new),
    `crates/marley_workbench/src/rusty/knowledge_panel.rs` (#646's),
    `crates/marley_workbench/src/rusty/graph_tab.rs` (#647's),
    `crates/marley_workbench/guide/index.html`; `script/e2e/655-knowledge-panel-project-view.sh`
    (new, Test phase).
  - Zed: none. `project`, `workspace`, `picker`, `fuzzy`, `ui` and `util` are used as they are.
- **The ledger rows it extends** (`docs/marley/zed-touchpoints.md`): none. The `crates/zed/src/
  zed.rs` row's `"rusty"` in `test_action_namespaces` is #645's (or #646's); if promotion finds it
  missing, the row is widened before the line, as #646's notes describe.

### Visual check plan
The scenario `script/e2e/655-knowledge-panel-project-view.sh`, `compositor sway`, one Marley run.
Setup: `mkdir -p "$E2E_WORK/repos/demo"`, `open_path` it, and keep `folder=$(realpath
"$E2E_WORK/repos/demo")`; link #643's stand-in as `$E2E_WORK/bin/rusty-mcp`, export
`MARLEY_RUSTY_MCP` to it and `RUSTY_STAND_IN_STATE=$E2E_WORK/rusty`; build the scratch vault from
`marley_rusty`'s fixtures and write `path: $folder` into `projects/demo.md`; `profile_setting
marley.rusty '{"enabled": true, "connection": "embedded"}'`; `expect` the harness's copy held Rusty
off before that (#643's line). Helpers: `vault_write <slug> <text>` (a page written whole),
`stand_in_signal` (`kill -USR1` the pid in the stand-in's pid file, then settle 2) and
`logged_write <key> <slug> <value>` (`holds` on the call log's `brain_set_property` line). Clicks
are measured from the first run's shots (L-504); the panel is opened by the palette.

| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | Trust the folder; settle; the palette: `rusty: toggle knowledge panel`; settle 3 | `655-01-by-path`: Demo, `projects/demo`, "matched by path" |
| REQ-002 | As 01 | `655-01-by-path`: Demo's `summary:` text |
| REQ-003 | As 01 | `655-01-by-path`: one follow-up, "Demo uses Orbit", "follow up by 2026-01-05" in the warning colour; neither "Demo keeps its name" (2099) nor "Orbit gets a ring" (Orbit's) |
| REQ-004 | As 01 | `655-01-by-path`: Tasks · Demo 2, the two open tasks, not the done one |
| REQ-005 | Click Open Page | `655-02-open-page`: Demo's Page tab; the panel on Demo's tags, backlinks and links |
| REQ-006 | Ctrl+W | `655-03-back`: no item; the project view |
| REQ-007, REQ-008 | `vault_write` Demo with `task_group: Chores`; `stand_in_signal` | `655-04-group-property`: Tasks · Chores 1 |
| REQ-009 | The palette: `rusty: open graph`; settle 3 | `655-05-graph-project`: "Local graph · Demo (project) · depth 1"; `brain_graph` with `around: projects/demo` in the log |
| REQ-010 | Click the panel's follow-up row | `655-06-follow-up-opens`: "Demo uses Orbit" in a Page tab |
| REQ-011 | The palette: `rusty: open graph` | `655-07-graph-page-wins`: the Graph tab in front, centred on the decision |
| REQ-012 | Ctrl+W twice; the palette: `rusty: open local graph` | `655-08-graph-local`: a Graph tab around Demo, "(project)" |
| REQ-013 | Ctrl+W; `vault_write` Demo with `path: old laptop ~/code/demo`; `stand_in_signal` | `655-09-by-name`: "matched by name"; no `brain_set_property` in the log yet |
| REQ-014 | `vault_write projects/demo-2` titled Demo; `stand_in_signal` | `655-10-name-tie`: "2 project pages are named demo", two rows with Link |
| REQ-015 | Click Link on `projects/demo`'s row | `655-11-linked-from-tie`: "matched by path"; `logged_write path projects/demo "old laptop ~/code/demo, $folder"` |
| REQ-016 | Move Demo to `projects/demo-site.md` (title Demo Site, no `path:`, no `task_group`); delete `projects/demo-2.md`; `stand_in_signal` | `655-12-none`: the callout naming `$folder` and `demo`, Link a Page |
| REQ-017 | Click Link a Page; settle 1; type `demo` | `655-13-page-picker`: Demo Site first, Orbit Site listed |
| REQ-017, REQ-018 | Enter; settle 2 | `655-14-page-linked`: Demo Site "matched by path", "None due.", "No task group for this project." with Link a Task Group; `logged_write path projects/demo-site "$folder"` |
| REQ-019 | Click Link a Task Group; settle 1; type `chores` | `655-15-group-picker`: Chores first |
| REQ-019 | Enter; settle 2 | `655-16-group-linked`: Tasks · Chores 1; `logged_write task_group projects/demo-site Chores` |
| REQ-024 | Every step | The log's only `brain_set_property` lines are the three the picks made |

Not reached by a scenario, and why: the user's own brain (R-D8; its counts above were read only and
never shown); a path tie (REQ-020: a second scratch folder would need a second project in the
window, more fixture than the pure rule it shows, which the review reads with the module's tests);
a remote project (REQ-021: no SSH host in a scenario); a projectless workspace (REQ-022: one
`open_path` per run); the reading, not-connected and failed states (REQ-023: #643 owns the
connection's states, and a stand-in made to fail proves the words, not the view); Rusty's indexer
lag behind a disk edit (the stand-in reads the disk at each call, so the delayed second list of D8
is checked by review); REQ-025 by review and the gate.

### Risks
- **Five tickets ahead, and three beside.** Every name taken from #643 to #647 is as queued on
  2026-10-03, and #646 and #647 are themselves drafted against #643 to #645. #658 and #659 may land
  before or after this ticket: the typed views go in their modules whichever lands first, and the
  Tasks tab's button goes in with whichever of #655 and #658 lands second. Promotion re-reads what
  shipped and changes the manifest, not the behavior. If #647's tab seeds its centre at creation
  from something other than the active item, D11's start rule follows what it does.
- **The first build reads every project page.** 101 `brain_read_page` calls, eight at a time,
  each a file read and a parse in Rusty; estimated well under a second over stdio, measured in the
  Code phase and logged once. After that only changed pages are read. If the vault grows past a few
  hundred project pages, the Rusty-side request in Out is the answer, not a Marley cache on disk.
- **`updated_at` as the change test.** Rusty's indexer moves a page's `updated_at` when it syncs a
  disk edit, 5 s after announcing it; D8's second list 6 s after a signal catches it. Promotion
  confirms that `sync_all` does move `updated_at` for an edited page; if it does not, the delayed
  pass reads every project page instead.
- **Symlinks.** Paths are compared lexically, so a folder opened through a symlink does not match a
  `path:` written through its target, or the other way round. Nothing in the measured vault needs
  it; the view's "No project page lists <folder>" names the folder as Zed holds it, so the cause is
  visible, and a link writes that form.
- **A write the user did not expect.** A link writes the user's brain, and Rusty commits it. Only a
  pick writes; the value keeps what was there; the commit names the key and the page, so a git
  revert in the vault undoes it.
- **Ties that change.** A page renamed, or a second page given the same `path:`, moves a project
  from one page to another at the next signal. The view always says how it matched and names the
  other pages a folder is listed by.
- **Group names.** `task_group` by name breaks when a group is renamed in Rusty; the view says "No
  task group named <name>." and offers the link again.
- **Projects without a page.** 13 of the 53 folders under `/srv/stacks` match no page today, so the
  callout is common at first. That is the honest state, and one pick fixes each.
- **#647's REQ-001 changes** for a project with a page (D11): the Graph tab opens Local on it, not
  Vault. #647's scenario folder matches no fixture page, so its shots stand; the CHANGELOG's
  Changed entry says it.
- **Focus and the picker.** A modal takes focus in a deferred callback (L-450), so the scenario
  types only after a settle; closing a tab gives the focus to the pane's front item (L-493), which
  the project view does not depend on.
- **The receipt.** The stand-in, the fixtures, the guide page and the scenario are fingerprinted by
  the commit receipt (`crates/marley_*`, `script/e2e*`); a change to any after the Code phase's
  green needs `--diff` again before the commit.
- **Public origin.** The fixtures are invented; the measured numbers above are counts only; no
  title, slug, path or task of the user's vault enters the repository.

### User-facing docs to update at Complete (P4)
- `docs/marley/guide.md`: the Knowledge panel section #646 adds gains "The project view" (how a
  project finds its page and its task group, the `path:` and `task_group` properties, linking, what
  each state says) and the two actions in the palette list; the Graph tab section #647 adds gains
  the project centre.
- `docs/marley/walkthrough.md`: a stop after #646's and #647's for the project view, with the checks
  the scenario makes.
- `crates/marley_workbench/guide/index.html`: changed in the Code phase (above).
- Architecture (§21): `docs/marley_architecture/marley_rusty.md` (#643's page) gains `project`;
  `docs/marley_architecture/marley_workbench.md` gains the project view, the `ProjectPages` cache
  and the pickers beside #646's Knowledge panel section; `docs/marley/rusty-in-marley.md`'s slices
  table marks R6 done and names the `repo:` tier, page creation and the Rusty-side request;
  `CHANGELOG.md` under Added (the project view, linking) and Changed (the Graph tab's start).

### Checklist (no TaskCreate in this harness)
- [x] Read the brief (all three parts), CONSTITUTION §3, §7, §14, §18, §19, §20, and the ticket and
      pipeline templates.
- [x] Read `docs/marley/rusty-in-marley.md` whole (R-D0 to R-D10, the slices, Rusty's triage) and
      the design note's knowledge layer (K1, K2) and settings sketch.
- [x] Read the queued #643 to #647 specs and notes, and #646's discovery kept for this ticket.
- [x] Read Rusty's tools and types behind the view (`brain_list_pages`, `brain_read_page`,
      `brain_due`, `brain_decide`, `brain_set_property`, `brain_get_links`, `list_task_groups`,
      `list_tasks`), its slug rule, its watcher and indexer, and its Decisions and Tasks pages.
- [x] Measured the vault and the store read only (counts only): the forms of `path:`, the ties, the
      summaries, the decision links both ways, the due follow-ups, the task groups, and the join's
      coverage over `/srv/stacks`.
- [x] Read Zed's `ProjectGroupKey`, `Workspace::project_group_key`, `WorktreePathsChanged`, the
      picker, `ui::Callout`, `Headline` and `ProjectEmptyState`; Marley's `remote.rs` picker, the
      rail's grouping and the projectless groups.
- [x] Read Ely's `chat/projects.rs`, `project/tasks.rs`, `project/work.rs`, `feedback/states.rs`
      and `navigation/palette/kinds.rs`; named what each is and why none is ported.
- [x] Recall: the knowledge ledgers (PR on caches that must record every outcome, L-572, L-458,
      L-602, BF on keys that alias duplicates, AD-450 and L-450, L-635 twice, AD-611, F-611, PR on
      buttons in cards, L-493, L-515, L-504, L-531, L-633, L-607, AD-609), the completed pipelines
      450, 458, 600, 602, 606, 611 and 633, and a read-only brain search.
- [x] Prior-art sweep, three legs plus Rusty and Ely, written into the spec.
- [x] Decisions D1 to D12 with what each rejected; the join rule and the task-group rule locked.
- [x] Spec: scope (In and Out, with the deferred pieces' one-line scopes), Reference (§20), Prior
      art, UI proof with sixteen shots, twenty-five EARS rows, phase plan.
- [x] Design: the modules, the stand-in's additions, the file manifest by crate, no touchpoint row,
      the visual check plan, risks, the user docs for Complete.
- [x] Docs only: the ticket doc and this pair; no BACKLOG.md, no `active/`, no source, no cargo.

### Reconciliation, 2026-10-03
- A `rusty:` action run while Rusty is off or not connected shows a toast saying so and where to
  turn it on, and opens nothing (rusty-in-marley.md R-D0, settled across #643 to #659).
- Every scenario names its stand-in in `MARLEY_RUSTY_MCP`, never first on the PATH (#643).
- Rusty's TICKET-047: `brain_list_pages` will take property names and return their values,
  with aliases, in one query. Until it lands the cache reads each project page as drafted; once
  it lands, one call with `page_type: project` and properties `path` and `task_group` replaces
  the per-page reads.
