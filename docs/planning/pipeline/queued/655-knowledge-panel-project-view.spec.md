---
pipeline_id: 10854dfc-6550-4c64-b51e-264282c4bf7b
ticket: docs/planning/tickets/open/TICKET-655-knowledge-panel-project-view.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "The Knowledge panel's project view: a project's brain page, follow-ups and tasks"
type: feature
slice: Rusty in Marley R6 (rusty-in-marley.md R-D5, R-D3); after #646 and #647
references: [docs/marley/rusty-in-marley.md, docs/planning/pipeline/queued/646-knowledge-panel.spec.md, docs/planning/pipeline/queued/646-knowledge-panel.notes.md, docs/planning/pipeline/queued/647-brain-graph-tab.spec.md]
---

## Title
The project join (R-D5) and the Knowledge panel's project view. A workspace's project, its folders
as Zed groups them for the rail, resolves to a brain project page: the page whose `path:` property
lists one of the folders, else the one page named like a folder. While no Page tab is the active
item, #646's panel shows that page's title and summary, the follow-ups due among the decisions
linked to it, the open tasks of its Rusty task group (named by a `task_group` property on the page,
else the group named like the page), and Open Page. When no page matches, or several share the
name, the panel offers to link one: the user picks it and Marley adds the folder to its `path:`
through `brain_set_property`, keeping what was there. #647's Graph tab centres its local graph on
the project's page while no page is focused. Chad, 2026-10-02: an all in one system "so that you
can manage these projects", and the plan's R-D5: "This is what 'manage these projects' needs
first."

## Scope
### In
- **After #646 and #647, in that order.** Built on #643 (the `marley.rusty` switch; the client in
  `marley_workbench::rusty` with its bounded tool call, its connection state and its change signal
  on `notifications/resources/list_changed`; the crate `crates/marley_rusty` with `fixtures/` and
  the Python stand-in `stand_in/rusty-mcp`, named in `MARLEY_RUSTY_MCP`), #644 (the stand-in's
  `SIGUSR1` trigger and pid file), #645 (`rusty::page::open_later` and the `rusty` action
  namespace), #646 (the Knowledge panel, its no-page state, its section headers with a count, its
  coalesced reads, and its stand-in's `brain_get_links`) and #647 (the Graph tab's centre, taken as
  a slug, and its `rusty: open graph` and `rusty: open local graph`). This ticket adds to them and
  replaces none of it; the names here are theirs as queued and are re-read at promotion.
- **The project:** the workspace's project group key (`Workspace::project_group_key`): its folders,
  the main worktree paths Zed groups the rail by, so a linked worktree's workspace joins its
  repository's page, and its host. A workspace with no folder has no project and keeps #646's
  no-page line (D1).
- **The join rule** (`marley_rusty::project`, pure, no gpui, no IO): the path tier (D2), the name
  tier (D3), the tie rules, and the `path:` value a link writes (D4).
- **The task group rule** (`marley_rusty::project`): the `task_group` property, else the group
  named like the page (D5).
- **The follow-ups:** `brain_due`'s due list kept where the decision links to or from the project
  page (D6).
- **The summary:** the page's `summary:` property, else the first paragraph of its body (D7).
- **The project pages** (`marley_workbench::rusty::project`): one app-wide cache of every project
  page's slug, title, aliases, `path:`, `task_group` and summary, read when first needed, kept up on
  the change signal, written to its global only when it differs; each workspace's resolution
  computed from it (D8).
- **The project view,** in #646's no-page state under the search field: the page's title and slug,
  how it matched, Open Page, the summary, Follow-ups due and Tasks; the unmatched, tied, loading
  and failed states (D9).
- **Linking:** `rusty::LinkProjectPage` and `rusty::LinkTaskGroup`, each a picker over Rusty's
  project pages or task groups, opened by the view's buttons or the palette; a candidate's Link
  button; each write one `brain_set_property` call on the user's pick (D4, D5, D10).
- **The Graph tab's centre** (#647's `graph_tab.rs`): while the tab has no focused page, its Local
  scope centres on the project's page, named in the header as the project's; a Page tab made active
  takes over, as #647 has it (D11).
- **Live:** the cache and each project view read again on #643's change signal, coalesced; the
  resolution again when the workspace's folders change; the written page read again at once after
  a link.
- **The stand-in** (#643's, with #644 to #647's additions) gains `brain_list_pages`,
  `brain_read_page`, `brain_due`, `brain_set_property`, `list_task_groups` and `list_tasks` over its
  scratch vault and a `tasks.json` in its state folder, and the fixtures gain this ticket's pages,
  decisions and task groups (invented, none of the user's).
- **The in-app guide page** (`crates/marley_workbench/guide/index.html`): the Knowledge panel's
  article (#646's) gains the project view and linking, in the Code phase (the receipt binds it).
- `script/e2e/655-knowledge-panel-project-view.sh`.

### Out (explicitly deferred)
- **Creating a project page** when none matches (`brain_create_page { page_type: "project", title
  }`, which needs no slashed name, so Rusty's TICKET-041 does not bite): beside #654's create on a
  miss in the `rusty: open page` picker (R3a), its own change.
- **A third tier by `repo:`.** 27 project pages carry `repo:` (owner/name), 4 of them with no
  `path:`; matching it against the folder's git remote is a later tier.
- **A folder inside a listed path** (a project opened on a subfolder of a page's repository): none
  of the 32 pages with `path:` needs it, and a broad path such as `~` would then claim every
  project.
- **Projectless groups by their Marley name** (#600): a folderless group has no `path:` to write;
  its no-page line stays #646's.
- **Remote projects by path:** `path:` names folders on this machine, so a remote project matches by
  name only and offers no link.
- **Acting on what the view lists:** ticking, adding or reordering tasks and answering a follow-up
  (`brain_follow_up`) are R7's, #658's Tasks tab and #659's Decisions tab; the rows here are read
  only.
- **Opening the Tasks tab on the project's list** through #658's seam, `TasksView::show_list(id)`:
  a button on the Tasks header, added by whichever of #655 and #658 lands second; if #658 has
  landed when this ticket is promoted, the button comes into it with a shot.
- **Unlinking or editing `path:` and `task_group`** in Marley: #645's Edit shows the file, and
  #656's property edits (R2b) cover it.
- **Upcoming follow-ups** (`brain_due` with `days`), the page's `status`, `stack` and `repo` on the
  card, and a mark on the rail's project row.
- **One call for every project page's properties.** No tool lists pages with their frontmatter, so
  the cache reads each project page once (101 today) and again only when it changed. A Rusty-side
  request (`brain_list_pages` with the properties named, or a property query) would make it one
  call; filed in Rusty when Chad confirms.
- **Rusty's TICKET-035, the change cursor:** a task another `rusty-mcp` process adds touches the
  database alone and is announced to no one, so it shows at the next change signal or showing.
  Nothing here depends on it.
- **Rusty's TICKET-040** (a deleted folder's pages come back from `archive/`): until it lands, a
  slug under `archive/` is never a candidate (D8).
- **Rusty's TICKET-037** (bookmarks and favourites): nothing here reads them.
- A `project_join` switch of its own (the design note's settings sketch): rejected in D12.

## Reference (§20)
Upstream Zed, kept as it is: the workspace's project group (`project::ProjectGroupKey`, the main
worktree paths and the host, which the `MultiWorkspace` and the rail group workspaces by) as the
project's identity; the `picker` crate's `Picker<D>` in the workspace's modal layer
(`Workspace::toggle_modal`), filtered by Zed's `fuzzy`, as the file finder and Marley's remote
terminal picker are; `ui::Callout` for the unmatched state. What the view lists follows Rusty's own
app and skills (MIT, the same owner): `DecisionsPage.qml` (`brain_due` with `days: 0`, "follow up
by" the date, overdue marked), `TasksPage.qml` (`list_task_groups`, then `list_tasks` for one
group), and the store skill `intake`'s project page template, which defines `path: <local path, if
any>` and `summary: <one or two plain sentences>`. The join itself is Marley-specific: neither Zed
nor Warp joins a workspace to a notes page, so no Warp behavior applies and no Warp source or spec
was used.

### Prior art
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` (`:51` the
  picker as the generic modal list host, `:74` a modal through `toggle_modal`, `:224-227` one
  `Workspace` per project group in a `MultiWorkspace`). `docs/orca_architecture/
  02-worktrees-and-review.md` §2.7: Orca keys a note to a worktree and tells agents to "merge rather
  than overwrite" it, the rule D4 keeps for `path:`. `docs/warp_architecture/` holds Warp Drive's
  notebooks (`crates/cloud_object_models.md`), a synced store with no folder join; nothing to take.
  The plan's R-D5 and the design note's K1 and K2 (`herdr-and-hermes-2026-10-02.md:327-332`): the
  project resolves "by `path:` in its frontmatter, then by name", and the panel shows "the current
  project's page, follow-ups due (`brain_due`), the project's tasks".
- **Published material:** Obsidian's properties (frontmatter values as text or a list of text,
  edited in Obsidian's properties view), which `path:` and `task_group` are, so a person can read
  and fix either by hand; herdr-projects (the design note's Part 3), where a project's memory is a
  folder named for the project, joined by name alone; the MCP specification's
  `notifications/resources/list_changed`. Rusty, read at `295565c`: `rusty-mcp/src/main.rs`
  (`list_task_groups` `:773-776`, `list_tasks` `:786-796` with `ListTasksParams` `:43-51`,
  `brain_read_page` `:908-919`, `brain_list_pages` `:921-932` with `ListPagesParams` `:98-105`,
  `brain_decide` `:1134-1152`, `brain_due` `:1185-1190` with `DueParams` `:403-409`,
  `brain_set_property` `:1427-1440` with `SetPropertyParams` `:658-668`, `brain_get_links`
  `:1614-1621`); `rusty-core/src/brain/mod.rs` (`BrainPage` `:31-52` with the frontmatter whose
  extra keys are flattened, `frontmatter.rs:18-41`; `BrainPageSummary` `:54-65`; `list_pages`
  `:573-620`, newest first, 50 by default; `set_property` `:1086-1097`; `get_links` `:2151-2184`;
  `index_links` `:3089-3112`, every wikilink of the page, timeline included);
  `rusty-core/src/brain/decisions.rs` (`DecisionSummary` `:19-27`; `decide` writes each consulted
  page as `[[slug]]` under Consulted, `:242-250`, and a timeline entry on that page linking back,
  `:282`; `due` `:416-438`: decided or revised, `follow_up_by` on or before the horizon);
  `rusty-core/src/engine/user_tasks.rs` (`TaskHeader` `:10-18` is id, name and order only;
  `UserTask` `:20-37`); `rusty-core/src/brain/vault.rs:494-530` (`title_to_slug`, the name rule D3
  copies); `rusty-app/qml/DecisionsPage.qml:22`, `:59` and `TasksPage.qml:22-25`, `:279-290`.
- **Code we already ship:** `project` (`ProjectGroupKey` `project.rs:6590-6594`, `path_list`
  `:6623`, `display_name` `:6627-6648`, which strips a `.git` extension, `host` `:6650`,
  `Event::WorktreePathsChanged` `:376`; `WorktreeStore::main_worktree_path_list`
  `worktree_store.rs:97`); `workspace` (`Workspace::project_group_key` `workspace.rs:2506`,
  `toggle_modal`, `ModalView`); `picker` (`Picker::uniform_list` `picker.rs:460`); `fuzzy`
  (`match_strings`); `ui` (`Callout` `components/callout.rs:28`, `Headline`
  `styles/typography.rs:206`, `ListItem`, `CountBadge`, `Button`, `IconButton`, `Label`, and the
  icons `ArrowUpRight`, `Circle`, `Clock`, `Link`, `ListTodo`); `util::paths::home_dir`
  (`paths.rs:24`). `ui::ProjectEmptyState` (`project_empty_state.rs:7`) was read and is not taken:
  it offers to open or clone a project, for a window with none. Nothing in Zed or `Cargo.lock` reads
  YAML frontmatter or joins a folder to a note. Marley's own: `remote.rs` (`RemotePicker` and
  `RemoteDelegate` `:143-330`, `fuzzy::match_strings` `:253`, `toggle_modal` `:63`: the picker
  shape this ticket copies); `marley_workbench.rs:933-947` (`group_names`) and `rail.rs:6992-7037`
  (`rail_groups`: a rail project is a Zed project group, or a projectless group of `groups.rs:119`);
  #646's panel, section headers and coalesced reads; #647's centre; #645's opener; #643's client.
  No new dependency: `marley_workbench` already builds `picker`, `fuzzy`, `ui`, `project`,
  `workspace` and `util`, and `marley_rusty` `serde` and `serde_json`.
- **Ely GPUI Components (R-D10), read at HEAD 2f8b2f6:** nothing ported, since Zed's own covers
  each piece and Zed's own wins. Read: `src/chat/projects.rs` (`ProjectList` `:31-117`, a column of
  chat projects with counts; `ProjectKnowledgePanel` `:117-247`, a chat project's attached files and
  a capacity meter: another meaning of "project knowledge", not taken); `src/project/tasks.rs`
  (`TaskItem` `:30-139`: without a toggle its status mark leads, then the title, the due day at the
  end in the danger colour when late; its order is the rows' order here, drawn as `ui::ListItem`s,
  no code copied; `TaskList` `:155-251` and its `SelectableList` keys not taken, the rows being read
  only); `src/project/work.rs:246-257` (`due_words`, "Today", "Yesterday", a weekday: not taken,
  Rusty's "follow up by" and its `overdue` flag say it without a date library in the pure crate);
  `src/feedback/states.rs:143-173` (`EmptyState`: `ui::Callout` covers it);
  `src/navigation/palette/kinds.rs` (`QuickSwitcher` `:309`, `SearchPalette` `:391-487`: Zed's
  `picker` covers both). The R-D10 row for R6 names `backlinks-graphview` and
  `searchpanel-searchresultitem-searchfilters`, which #646 ports; this view reuses #646's rows.

## UI proof
`script/e2e/655-knowledge-panel-project-view.sh` (`compositor sway`: it clicks buttons and rows in
the panel). Fixtures: a scratch folder `$E2E_WORK/repos/demo` opened with `open_path`; #643's
stand-in named in `MARLEY_RUSTY_MCP` (never first on the PATH, L-531), its state in
`$E2E_WORK/rusty`, over a scratch vault built from `marley_rusty`'s fixtures with #646's pages
(`projects/demo` "Demo", `concepts/orbit`, `decisions/demo-uses-orbit`) and this ticket's
additions: Demo's `summary:` and, written at setup, `path:` naming the scratch folder's real path
(`realpath`); `decisions/demo-uses-orbit` due on 2026-01-05; `decisions/demo-keeps-its-name`
linking Demo, due 2099-01-01; `decisions/orbit-gets-a-ring` linking only Orbit, due 2026-01-06;
`projects/orbit-site` with a `path:` elsewhere; `tasks.json` with the groups Demo (two open tasks,
one done) and Chores (one open). Never the user's brain (R-D8). The run's settings set
`marley.rusty` on with `connection: embedded` (`profile_setting`); later changes are edits of the
vault from outside followed by the stand-in's `SIGUSR1` (#644's trigger). Each write's check reads
the stand-in's call log. Shots:
- `655-01-by-path`: no item open; `rusty: toggle knowledge panel`: the project view: Demo,
  `projects/demo`, "matched by path", the summary; Follow-ups due 1, "Demo uses Orbit", "follow up
  by 2026-01-05" in the warning colour; Tasks · Demo 2, the two open tasks.
- `655-02-open-page`: Open Page clicked: Demo in a Page tab; the panel on Demo's page view.
- `655-03-back`: the Page tab closed: the project view again.
- `655-04-group-property`: `task_group: Chores` written into Demo's file, the trigger sent: Tasks ·
  Chores 1, with no input.
- `655-05-graph-project`: `rusty: open graph`: the Graph tab, "Local graph · Demo (project) · depth
  1", Demo in the middle; the log holds `brain_graph` with `around: projects/demo`.
- `655-06-follow-up-opens`: the panel's follow-up row clicked: "Demo uses Orbit" in a Page tab.
- `655-07-graph-page-wins`: `rusty: open graph`: the Graph tab in front, centred on the decision.
- `655-08-graph-local`: the decision's tab and the Graph tab closed; `rusty: open local graph`: a
  Graph tab around Demo, "(project)" in its header.
- `655-09-by-name`: the Graph tab closed; Demo's `path:` set to `old laptop ~/code/demo`, the
  trigger: "matched by name"; the stand-in logged no write.
- `655-10-name-tie`: `projects/demo-2` titled Demo written, the trigger: "2 project pages are named
  demo", two rows with Link, no page chosen.
- `655-11-linked-from-tie`: Link on `projects/demo`'s row: the log holds `brain_set_property` with
  `slug: projects/demo`, `key: path` and `value: "old laptop ~/code/demo, <the folder>"`; the
  view on Demo, "matched by path".
- `655-12-none`: `projects/demo.md` moved to `projects/demo-site.md` (title Demo Site, no `path:`,
  no `task_group`), `projects/demo-2.md` deleted, the trigger: the callout "No brain page for this
  project", naming the folder and the name `demo`, with Link a Page.
- `655-13-page-picker`: Link a Page clicked, `demo` typed: the picker, Demo Site first, Orbit Site
  listed.
- `655-14-page-linked`: Enter: the log holds `brain_set_property` with `slug: projects/demo-site`,
  `key: path`, `value` the folder alone; Demo Site "matched by path"; Follow-ups due "None due.";
  Tasks "No task group for this project." with Link a Task Group.
- `655-15-group-picker`: Link a Task Group clicked, `chores` typed: Chores first.
- `655-16-group-linked`: Enter: the log holds `brain_set_property` with `key: task_group`,
  `value: "Chores"`; Tasks · Chores 1.

## Locked-In Decisions
- D1: **The project is the workspace's project group.** Its folders are
  `Workspace::project_group_key(cx).path_list()`, the main worktree paths the `MultiWorkspace` and
  the rail group workspaces by, so every workspace of a group (a linked worktree's included)
  resolves to one page; its name, for the name tier, is each folder's last part with a `.git`
  extension stripped, as Zed's `display_name` strips it; its host says whether it is remote. A
  workspace with no folder (a projectless group, #600; L-602) has no project and keeps #646's
  no-page line. Rejected: the rail's disambiguated name (`group_names` adds parent folders when two
  projects share a name, which no page title carries); the visible worktree roots alone (a linked
  worktree under `/mnt/fast` would match nothing).
- D2: **The path tier.** `path:` is read as text split on commas, or as a list of text; each part
  trimmed and stripped of quotes, and taken as a path on this machine only when it starts with `/`,
  or is `~` or starts with `~/`, `~` being the home folder the caller passes in. Anything else, such
  as another machine's path in words ("old laptop ~/code/x"), never matches. Parts and folders are
  normalised the same way, lexically (`.` dropped, `..` taken back, a trailing `/` dropped), with no
  disk read and no symlink followed, and a folder matches a part that equals it. A local project
  matches every page any of its folders is listed by. Two or more: the one whose name also matches
  by D3's rule wins, else the first by slug, and the view names the others as also listing it.
  Measured read only on 2026-10-03: 101 project pages, 32 with `path:` (24 one path, 8 comma
  lists), 43 parts of which 39 absolute, 2 under `~` and 2 in words; one folder is listed by two
  pages; no listed path lies inside another. Rejected: a folder inside a listed path (Out); the
  folder's canonical path (a disk read per folder for a case the vault does not show; Risks).
- D3: **The name tier, only when no page lists a folder.** The folder's name and the page's slug
  name, title and aliases are each put through Rusty's slug rule (`title_to_slug`: lowercase,
  letters and digits kept, space, `_` and `-` as one `-`, anything else dropped, ends trimmed) and
  compared. One page: matched by name. Several: the view lists them, each with Link, and picks none,
  since a guess would show one project's decisions and tasks under another. A remote project uses
  this tier alone and offers no link (D4). Measured: of the 53 folders under `/srv/stacks`, 37
  match by path (one of them listed by two pages), 3 more by name (one of them by two pages, a page
  and its `-2` copy), 13 by neither. Rejected: a fuzzy name (a near name is a guess); the page title
  alone (pages are found by slug as often as by title).
- D4: **A link writes `path:` once, on the user's pick, through `brain_set_property`, keeping what
  was there.** The value: absent or empty, the project's folders as one text (joined with `, `
  when there are several); a text, that text, then `, ` and each folder it does not already list; a
  list, the list with the new folders after it; any other value, refused with "path on <slug> is
  not text or a list; edit the page", and nothing written. Folders are written as Zed holds them,
  absolute. After a write Marley reads that page again at once, since the service connection hears
  no change (#643's D5). Rusty commits each write ("property: path on <slug>"). Rejected:
  overwriting (it would lose another machine's words and the comma lists); a map kept by Marley
  (the brain, Obsidian and every agent would not see it, and Rusty is the store, D11 as amended);
  a confirmation after the pick (the pick is the confirmation, as #646's Create is one click).
- D5: **The task group is a property of the page, by name.** `task_group` names one group as text
  or several as a list; each name is matched to `list_task_groups`' names through D3's slug rule.
  With no `task_group`, the group whose name matches the page's title or slug name the same way.
  With neither, the view says "No task group for this project." and offers Link a Task Group,
  which writes the group's name as text with `brain_set_property`. A name that matches no group
  says "No task group named <name>." with the same offer. By name, not id: the page is read and
  edited by a person in Obsidian, and Rusty's own pages refer to their groups by name; a group
  renamed in Rusty loses the link, and the view says so. Measured: Rusty holds 5 task groups,
  named by area, and none is named like a project page, so the fallback alone joins nothing today.
  Rejected: the group's id (unreadable in the page, and lost when a group is deleted and made
  again); the fallback alone (it would show no tasks for any project now); a property on the group
  (a group is an id, a name and an order, nothing more).
- D6: **Follow-ups due are `brain_due`'s, joined by links.** `brain_due { days: 0 }` gives today's
  and the overdue follow-ups of decided and revised decisions, Rusty's rule and its Decisions
  page's; the view keeps those whose slug is among the project page's backlinks or outbound links
  (`brain_get_links`). Both ways, because `brain_decide` writes each consulted page as a link under
  Consulted and writes a timeline entry on that page linking back. Each row: the decision's title
  and "follow up by <date>", the date in the warning colour when Rusty marks it overdue, as
  Rusty's page marks it; a click opens the decision through `rusty::page::open_later`. Measured:
  15 follow-ups due today, all 15 linked to a project page. Rejected: the decision's `consulted`
  property (the same pages, at a read per decision); `brain_graph`'s typed edges (the whole graph
  for one page's decisions).
- D7: **The summary.** The page's `summary:` property (36 of 101 project pages, 107 to 306
  characters), else the first paragraph of `compiled_truth` that is not a heading, list, table,
  quote or code fence, with each wikilink shown as its alias or target; cut at 300 characters on a
  word, with an ellipsis. Every project page has such a paragraph today.
- D8: **The data.** One app-wide cache, `ProjectPages`, a global written only when what it holds
  differs (L-572): `brain_list_pages { page_type: "project", limit: 1000 }` (Rusty's default of 50
  would miss half of the 101), then `brain_read_page` for each page not held or whose `updated_at`
  moved, eight at a time, each bounded as #643's client bounds a call; a page that fails to read is
  left out and its error logged once; a slug under `archive/` is skipped (TICKET-040). It is built
  when a project view or a project centre first asks, re-listed on each change signal, and an empty
  or failed answer is kept as an outcome, so nothing re-asks in a loop
  (PR-claude-retry-keyed-on-empty-cache-needs-every-outcome-to-write-001). A workspace's resolution
  is the pure rule over the cache and its group key, computed again when the cache changes and,
  deferred, when its folders change (L-458). Per project view: `brain_get_links` for the page,
  `brain_due`, `list_task_groups`, then `list_tasks { group_id, include_archived: false }` per
  group, kept
  `completed: false`; one read in flight per kind and one queued, answers for a page no longer
  shown dropped (#646's D7). Rejected: a read of every project page per window (each would read all
  101); `brain_search` with `type:project` (no frontmatter in a hit).
- D9: **The view, from Zed's `ui`.** Under #646's search field, in its no-page place: a header with
  the title (`Headline`, a click opens the page), the slug muted, "matched by path" or "matched by
  name", and Open Page (`Button` with `IconName::ArrowUpRight`), which calls
  `rusty::page::open_later(workspace, slug, false, ..)`; the summary in muted text; Follow-ups due
  and Tasks · <group> as #646's section headers with a `CountBadge`, rows as `ListItem`s
  (`IconName::Clock` and the decision's title; `IconName::Circle` and the task's title), "None
  due." and "No open tasks." when empty; a path tie's "<title> also lists this folder.", a click
  opening that page; a name tie as "<n> project pages are named <name>. Link one to this folder."
  over their rows (title, slug muted, a Link `Button` each); no match as a `ui::Callout`
  ("No brain page for this project", "No project page lists <folder> in its path, and none is
  named <name>.", Link a Page); while the cache is read, "Reading the brain's project pages…";
  while #643's client is not connected, its state line, as #646 shows it; a failed call, its root
  cause in its section (F-611). Buttons inside rows stop their click
  (PR-claude-a-button-in-a-card-that-takes-the-focus-stops-its-click-001). The view scrolls with the
  panel. Nothing of Ely's is ported (Prior art).
- D10: **The pickers are Zed's.** `rusty::LinkProjectPage` and `rusty::LinkTaskGroup`, registered on
  each workspace while `marley.rusty.enabled` is on and answering as #643's other `rusty:` actions
  while off, open a `Picker::uniform_list` in the workspace's modal layer, filtered with
  `fuzzy::match_strings`, as `remote.rs` does: the cache's project pages by title with the slug
  muted, or the task groups by name. Enter writes (D4, D5) and closes; a refusal shows Rusty's
  message in a toast. `LinkTaskGroup` with no resolved page says "Link this project to a brain page
  first." Rejected: Ely's `SearchPalette` (Zed's picker wins); a picker over `brain_list_pages`
  calls as typed (the cache already holds every project page).
- D11: **The Graph tab's centre falls back to the project's page.** #647's tab keeps the page of
  the Page tab last made active (its D2); while it has none, Local centres on the project's page and
  the header reads "Local graph · <title> (project) · depth <n>". `rusty: open graph` starts a new
  tab Local when a page is focused or the project has a page, else Vault; `rusty: open local graph`
  with neither keeps #647's notice, worded "Open a page first, or link this project to its page in
  the Knowledge panel." A Page tab made active takes over the centre for good, as #647 has it; a
  project centre follows the resolution, so a link moves it. This changes #647's REQ-001 for a
  workspace whose project has a page; its scenario's scratch folder matches no fixture page, so
  its shots stand.
- D12: **No switch of its own.** The view is part of the Knowledge panel, which exists only while
  `marley.rusty.enabled` is on; it reads, and writes only on a pick. Rejected: the design note's
  `project_join` key (R-D0 replaced its `knowledge` block, and a switch for a view that writes
  nothing by itself would widen three touchpoint rows for nothing).

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE no Page tab is the active item and a project page's `path:` lists one of the workspace's folders, the Knowledge panel shall show that page's title, slug and "matched by path". | Shot `655-01-by-path` |
| REQ-002 | The project view shall show the page's `summary:` property, or else the first paragraph of its body. | Shot `655-01-by-path`; review for the fallback |
| REQ-003 | The project view shall list the follow-ups due today or overdue among the decisions linked to or from the project page, each with its title and date, an overdue date marked, and no other decision. | Shot `655-01-by-path`; the fixtures' unlinked and later decisions absent |
| REQ-004 | WHERE the project page has no `task_group`, the project view shall list the open tasks of the task group named like the page. | Shot `655-01-by-path` |
| REQ-005 | WHEN the user clicks Open Page, the system shall open the project page in a Page tab, and the panel shall show that page's view. | Shot `655-02-open-page` |
| REQ-006 | WHEN the active item stops being a Page tab, the panel shall show the project view again. | Shot `655-03-back` |
| REQ-007 | WHERE the project page's `task_group` names a group, the project view shall list that group's open tasks in place of the group named like the page. | Shot `655-04-group-property` |
| REQ-008 | WHEN Rusty announces a change, the project view shall show the project page, its follow-ups and its tasks read again, without input. | Shot `655-04-group-property` |
| REQ-009 | WHEN `rusty: open graph` opens the Graph tab while no page is focused and the project has a page, the tab shall show the local graph around that page, its header marking it as the project's. | Shot `655-05-graph-project`; the call log |
| REQ-010 | WHEN the user clicks a follow-up, the system shall open its decision in a Page tab. | Shot `655-06-follow-up-opens` |
| REQ-011 | WHEN a Page tab becomes active while the Graph tab is centred on the project's page, the Graph tab shall centre on that page instead. | Shot `655-07-graph-page-wins` |
| REQ-012 | WHEN `rusty: open local graph` runs while no page is focused and the project has a page, the Graph tab shall centre on the project's page. | Shot `655-08-graph-local` |
| REQ-013 | WHERE no project page lists a folder and one page is named like a folder, the system shall take that page and show "matched by name", a `path:` part that is not a path on this machine matching nothing. | Shot `655-09-by-name` |
| REQ-014 | WHERE several project pages match by name, the project view shall list them, each with Link, and show none of them as the project's. | Shot `655-10-name-tie` |
| REQ-015 | WHEN the user clicks Link on a page, the system shall add the project's folders to that page's `path:` through `brain_set_property`, keeping the value it held, and show the page as matched by path. | Shot `655-11-linked-from-tie`; the call log |
| REQ-016 | WHERE no project page matches, the project view shall say so, naming the folders and the name it looked for, and offer Link a Page. | Shot `655-12-none` |
| REQ-017 | WHEN the user picks a page in Link a Page's picker, the system shall write the project's folders to that page's `path:` through `brain_set_property` and show the page. | Shots `655-13-page-picker`, `655-14-page-linked`; the call log |
| REQ-018 | WHERE neither a `task_group` property nor a group named like the page names a task group, the project view shall say so and offer Link a Task Group. | Shot `655-14-page-linked` |
| REQ-019 | WHEN the user picks a group in Link a Task Group's picker, the system shall write its name to the page's `task_group` through `brain_set_property` and list its open tasks. | Shots `655-15-group-picker`, `655-16-group-linked`; the call log |
| REQ-020 | WHERE two or more project pages list a folder, the system shall take the one that also matches by name, else the first by slug, and name the others as also listing it. | Review of the pure rule |
| REQ-021 | WHERE the project is remote, the system shall match by name only and offer no link. | Review |
| REQ-022 | WHERE the workspace has no folder, the panel shall show #646's no-page line. | Review |
| REQ-023 | WHILE the project pages are being read, or #643's client is not connected, or a call fails, the project view shall say which, a failure by its root cause. | Review |
| REQ-024 | The system shall write to the brain only through `brain_set_property` and only on the user's pick. | Review of the diff; the call log (no write before `655-11`) |
| REQ-025 | The join, task group, follow-up and summary rules shall live in `marley_rusty::project` with no gpui and no IO, and no slug under `archive/` shall be a candidate. | Review; `script/gates.sh --diff` (gate:2) |

## Phase Plan
- **P1 Plan**: promote after #647 completes; replace this spec's names with what #643 to #647
  shipped (the client's call, its state line and change signal, the stand-in's trigger, pid file,
  log and fixture layout, #646's no-page state and section header, #647's centre field, its open
  rule and notice, #645's `open_later`); take #659's `marley_rusty::decisions` and #658's
  `marley_rusty::tasks` where they have landed, and #658's `TasksView::show_list` button with a
  shot if #658 is in; re-measure the vault's counts read only; ask the brain
  (`brain_ask`) on D2 to D5; confirm with Chad the `task_group` key and that a pick writes to the
  brain with no second confirmation.
- **P2 Code**: the `README.md` marker first; `marley_rusty::project`; the stand-in's tools and the
  fixtures; `marley_workbench::rusty::project` (the cache, the resolution, the pickers and the two
  actions); the project view in #646's `knowledge_panel.rs`; the centre in #647's `graph_tab.rs`;
  the in-app guide page; a review of the diff (re-entrancy on the writes and opens, errors reaching
  the view, provenance); `script/gates.sh --diff` green.
- **P3 Test**: write and run the scenario under sway, read every shot, check the call log's writes.
- **P4 Complete**: CHANGELOG (Added: the project view and linking; Changed: the Graph tab starts on
  the project's page) and architecture docs (§21), the user docs the notes list, ledger capture
  (§19), the brain's decision recorded, the Rusty-side request filed if Chad confirms it, close the
  ticket, archive, commit.
