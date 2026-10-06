# `marley_rusty`

> Per-crate architecture note, written 2026-10-04 at #643, extended at #644 to #657. Provenance:
> **`[Marley-original]`** (`serde` and `serde_json` only), with one port from Ely GPUI Components
> (MIT) in `vault`, `knowledge` and `graph_layout`, its notice on each file. The design record is
> [rusty-in-marley.md](../marley/rusty-in-marley.md) (R-D1, R-D8), D8 and D11 in
> [three-prong-plan.md](../marley/three-prong-plan.md).

The pure core of Rusty in Marley: typed views of what `rusty-mcp`, Rusty's MCP server, answers,
with no gpui, no clock and no IO. The adapter, `marley_workbench::rusty`, connects to the server
through Zed's MCP client and draws. Rusty stays the store and the only writer of its data; Marley
reads and writes through Rusty's tools. Plan D8's two adapters for Rusty (`marley_rusty` for
the agent socket, `marley_brain` for the brain's tools) are this one crate, since both talk to one
program (R-D1); Rusty's agent sessions are not rebuilt (R-D6).

## Modules
- `settings` (#643): `ServerSettings`, `settings_list`'s answer (a JSON array of `{key, value}`,
  only the keys Rusty stores, a credential-like value masked by Rusty), with `get` and
  `embedding_provider`; `EmbeddingProvider { Auto, Ollama, OpenAi, Off }`, read from a stored
  value as Rusty's `semantic.rs` reads it (trimmed and lower-cased; `off`, `none` and `false` off;
  anything else, and no value, auto), written back by `as_setting`, with each provider's label and
  Rusty's words for it; the tool names `SETTINGS_LIST` and `SETTING_SET`, and
  `setting_set_arguments`.
- `vault` (#644): `VaultNode`, `brain_tree`'s nested answer (`name`, `path`, `kind`, `pages` at any
  depth, `children` folders first), with `holds` and `find`; `NodeKind { Folder, Page, File }`,
  any other kind read as `File`; `SearchHit` from `brain_search`; the slugs `brain_new_page` and
  `brain_daily_note` answer; `RenameReport` from `brain_rename` (since #656 with `pages_rewritten`,
  0 from a Rusty that does not send it). The rail's Brain view's model:
  `rows` (the visible rows through the open folders, each with its depth and parent; the root and
  files that are not pages left out), `step` over `Key { Next, Previous, First, Last, Child,
  Parent }` giving a `Move { To, Open, Close }`, `folder_of`, `name_of`, `folders_above`,
  `child_path`, `rename_target` (one part, `/` made `-`, `None` when empty or unchanged),
  `move_target` (`"<folder>/"` or `"/"`, `None` onto itself, under itself or into its own folder)
  and `reopen` (open folders following a moved folder). `rows`, `step`, `holds` and the drop guard
  are ported from Ely's `src/lists/tree/model.rs` at `2f8b2f6`, on vault paths in place of Ely's
  keys; Ely's rendering, lazy children, checkboxes and before/after drop places are not taken,
  since Zed's `ui` draws the tree. The tool names `BRAIN_*`, `VAULT_PATH_KEY` (`brain_vault_path`)
  and `SEARCH_LIMIT`.
- `page` (#645): `RenderedPage` (`brain_render`'s `slug`, `title`, `properties` as `Property { key,
  value }`, `raw`, `file` since Rusty's TICKET-042, `links` as `LinkOut { target, slug }`), read
  with `from_answer`, `None` for Rusty's `null`; `body_of` (Rusty's `split_raw` rule),
  `split_fragment` and `normalise_target` (Rusty's own); `page_markdown`, the pass that finds
  wikilinks and embeds with Rusty's parse options (pulldown-cmark 0.13, `ENABLE_WIKILINKS`) and
  splices `[TEXT](<rusty:page/SLUG#HEADING>)` or `[TEXT](<rusty:new/TARGET>)` over each, back to
  front, the text's punctuation escaped and `%`, `<`, `>` and `\` percent-encoded in the address;
  `PageLink::parse` (`Page`, `Missing`, `Heading`, `Local`, `External`), which decodes them;
  `Visit` and `PageHistory` (a tab's Back and Forward, 100 behind); `page_file_in` (a slug's file
  in a vault, `None` for one that would leave it); `BRAIN_RENDER`. Since #656 `RenderedPage.outline`,
  Rusty's headings as `Heading { level, text, line }` (`line` from 0 in `body_of`'s text, which
  `page_markdown` keeps line for line), with `line_offset` and `outline_label` (a heading's text as
  drawn, from pulldown-cmark's text and code events with wikilinks on); `PropertyKind` (`Text`,
  `Number`, `Date`, `Checkbox`, `List`, `ReadOnly`) with `of`, `ADDABLE`, `name`, `empty_value`
  and `parse` (the words for a refused number or date); `is_date`; `PageHistory::rename`, which
  moves a renamed page or folder through the history; `BRAIN_REMOVE_PROPERTY`. Since #654 `NewPage { path }`:
  `from_target` (the heading cut, the target normalised, `None` for an empty part or a `..`) and
  `arguments()`, `brain_new_page`'s `path` with its `folder` and `name`, so a Rusty with its
  TICKET-041 and an older one both make the page at the path.
- `project` (#655): the project join. `ListedPage` (`brain_list_pages`' summary with `aliases`
  and the asked `properties`, absent from a Rusty before that parameter) and `PageRead`
  (`brain_read_page`'s `slug`, `title`, `compiled_truth`, `frontmatter`); `ProjectPage` from
  either (`path` as `PathValue::{Absent, Text, List, Other}`, `task_groups`, `summary`).
  `local_paths(value, home)` takes the parts that are paths on this machine (absolute, `~`),
  `lexical` normalises without the disk, `name_key` is Rusty's `title_to_slug` rule.
  `resolve(project, pages, home)`: the pages listing a folder (sorted by slug; the one also named
  like a folder wins, the rest named `also`), else the one page named like a folder, else
  `Candidates` or `Unmatched`; a remote project matches by name only; `archive/` never matches.
  `path_value_with` adds the folders to a `path` (text with `, `, a list, or new text), `None` when
  all are listed, refused for any other value. `group_join` (the `task_group` names, else the group
  named like the page), `due_for` (the due decisions among the page's backlinks and outbound
  links), `summary` (the property, else the body's first paragraph, its wikilinks as words, cut
  near 300 characters).
- `decisions` (#655): `DecisionSummary` and `due_from_answer` (`brain_due`'s `due`). Since #659
  the whole answer: `Due { due, all }` and `parse_due`; the summary's `status` and `decided`, with
  `follow_up` (Rusty's empty `follow_up_by` read as none), `decided_line` and `follow_up_line`
  ("· overdue" when Rusty flags it); `DecisionStatus` (decided, kept, revised, superseded, or a
  word Rusty adds later, an empty status read as decided); and the tab's lines, `Entry` (the Due
  header only when something is due, the count, a row by `Section` and index), `entries` and
  `count_line`. Nothing here works out a date. Since #660 the summary's `followed_up` and
  `superseded_by` (`followed_up_line`, `successor`), `BRAIN_FOLLOW_UP`, `FollowUpStatus` (kept,
  revised, superseded; `ALL`, `word`, `label`), `Missing` with its hint, and `FollowUpDraft`
  (`missing`, and `arguments`: the outcome trimmed, `follow_up_by` only for revised and given, the
  successor only for superseded; the day goes as typed, Rusty judges it).
  `tasks` (#655): `TaskGroup`, `UserTask`, `groups_from_answer`, `tasks_from_answer`. Since #658
  the ten write names, `UserTask`'s `archived` and `header_id`, `id_from_answer`, and `TaskWrite`
  (one variant a tool, `tool` and `arguments` in Rusty's parameter names: a task `id`, a list
  `group_id`), with `typed_name` (trimmed, `None` when empty or unchanged), `moved_one` (one
  place up or down, `None` at an end), `kept_list` and `kept_task` (the choice and the selection
  across a read).
- `switcher` (#654): the page picker's pure part. `PageSummary` (`brain_list_pages`' `slug` and
  `title`, `shown_title` falling back to the file's name) and `parse_page_list`; `RecentPages`,
  the recently opened newest first (`visit`, at most `RECENT_CAP` 20, `from_json`, `to_json`,
  `rank`); `empty_order(pages, recent, active, favourites)` (the active page, since #662 the
  favourite pages, then the recent pages the list holds, each group once with a separator after it
  in `Order::separators_after`, the rest in Rusty's order, the selection on the second row after
  the active page); `merge(titles, slugs, pages, recent, cap)` (each page at the better of its two
  fields' scores, lit in the field that gave it, ties to the more recently opened then the list's
  order); `create_target(query, pages)`; `BRAIN_LIST_PAGES`, `LIST_LIMIT` 100,000, `MATCH_CAP` 100.
- `bookmarks` (#662): Rusty's bookmarks (its TICKET-037). `BOOKMARK_LIST`, `BOOKMARK_ADD`,
  `BOOKMARK_REMOVE`, `BOOKMARK_SET`; `BookmarkKind` (file, folder, search, heading, and any other
  kind as written); `Bookmark` as Rusty serves it (`kind`, `title`, `path`, `query`, `heading`),
  with `page(slug)`, `is_page`, `key` (Rusty's identity: the kind and the path, the query, or the
  page and heading), `shown_title` (Rusty's fallback for a blank title) and `arguments` (empty
  fields left out); `bookmarks_from_answer`; `retitled(list, key, title)`, the list a rename
  sends; `BookmarkWrite` (`Add`, `Remove`, `Set`) with `tool` and `arguments`.
- `capture` (#663): `BRAIN_CAPTURE`, `SOURCE_CAPTURE`, `BRAIN_IMPORT_PLAN`, `BRAIN_IMPORT` and their
  deadlines (`CAPTURE_URL_DEADLINE` 45 s, `IMPORT_PLAN_DEADLINE` 60 s, `IMPORT_DEADLINE` 600 s);
  `CaptureTarget` (`daily`, `inbox`; the form's title; `arguments`); `CaptureReceipt` and
  `receipt_from_answer`; `source_arguments`; `SourcePage` from `source_page_from_answer` (the
  slug, and Rusty's `error` when `frontmatter.status` is `failed`); `import_arguments`;
  `ImportPlan` (`brings_anything`, `summary`, `details`, its bookmarks `bookmarks::Bookmark`) and
  `ImportReport` (`summary`), worded as Rusty's app words them, with `plan_from_answer` and
  `report_from_answer`.
- `memories` (#664): `LIST_MEMORIES`, `STORE_MEMORY`, `UPDATE_MEMORY`, `DELETE_MEMORY`;
  `DEFAULT_CATEGORY` (`context`, Rusty's app's default); `Memory` as Rusty serves it (`id`,
  `category`, `importance`, `content`, `source`, `updated_at` in seconds) with `importance()`;
  `memories_from_answer`; `Importance` (`ALL`, `word`, `label`, `parse` reading `medium` as normal);
  `categories` (sorted, once each); `count_line`; `category_or_default`; `MemoryWrite` (`Store`,
  `Update` leaving out an importance not picked, `Delete`) with `tool` and `arguments`.
- `knowledge` (#646): what the Knowledge panel shows of a page. `LinkEntry` and `PageLinks`
  (`brain_get_links`'s `outbound` and `backlinks`, each with `context`, the line the link sits on,
  and `resolved`), `TagCount` (`brain_tags`), `GraphNode` and `Graph` (`brain_graph`'s nodes; the
  edges are not read). `page_knowledge` joins the three into `PageKnowledge` (title, tags with
  their counts, `Backlink`s, `Outgoing::{Page, Missing}`), titles from the graph's page nodes and
  the slug's last part where none is named. `mention` finds the bytes of the first `[[…]]` in a
  line that names the page (by slug, last part or title, case aside, the target read up to `|` or
  `#`); `trimmed` cuts a long line to start near it, the ranges moved; `snippet` takes Rusty's
  `<b>` marks out of a search snippet and gives their byte ranges. `Backlink`, `mention` and
  `trimmed` are ported from Ely's `src/documents/knowledge.rs` and `src/editor/search.rs` at
  `2f8b2f6`. The tool names `BRAIN_GET_LINKS`, `BRAIN_GRAPH`, `BRAIN_TAGS`, and `SEARCH_LIMIT`
  (60, as Rusty's search pane). `vault::SearchHit` carries `snippet` since #646.

- `graph` (#647): `brain_graph`'s answer as `Graph { nodes, edges }`, `Node { id, kind, title,
  page_type, tags }` with `NodeKind { Page, Tag, Unresolved, Other }` and `Edge { from, to, kind }`
  with `EdgeKind { Link, Consulted, Supersedes, FollowsUp, Other(String) }`, an unknown kind read as
  `Other` rather than refused; `page_types_from_answer` (`brain_page_types`) and `type_order`
  (Rusty's types first, then the graph's others by name, so a type keeps its colour). `Query` is
  Rusty's graph filter (`GraphView.qml`'s `matches`): every space-separated term must match, `tag:`
  with nested tags, `path:` a slug prefix, `type:`, else text in the title or slug. `shown` applies a
  `Filters` (the query, the legend's hidden types, and the Tags, Decision edges and Orphans
  switches) around an optional centre, always kept: tags become nodes for the pages kept, built
  here so a local graph stays local; past `CAP` (2,000) the most linked stay, ties by id. `Shown`
  carries the nodes, the edges by index, each node's degree, the count before the cap and the
  centre's index.
- `graph_layout` (#647): `Layout`, Ely's Fruchterman-Reingold `Force` in world units (`LINK` 80,
  repulsion k²/d over every pair, attraction d²/k along each edge, a pull of 4 per unit to the
  origin, each move capped by the heat, cooling ×0.96 from 4 links to 0.05, at most `MAX_STEPS`
  300); `seeded` keeps a node's place by index, starts a new one beside a placed neighbour or on
  the golden-angle spiral, pins the centre where it was kept (the origin when new), and starts
  warm when most places are kept; `run(pairs)` steps until a pair budget is spent; `pin`, `release`, `warm`. Two nodes on one
  spot part along an angle their indices give, so a run is the same every time. `Viewport` (Ely's,
  kept about the view's middle): `to_view`, `to_world`, `zoomed` about a point (0.15 to 6),
  `panned`, `fitting` (about the centre when given, no nearer than 1.6). `nearest` is Ely's hit
  test. The pair loop pushes by the offset times k²/d² with plain products: `mul_add` and `hypot`
  are library calls without the FMA target feature. The crate builds at `opt-level = 3` in the dev profile (#647).
  Since #657 a `Layout` holds its `Forces`: `seeded` takes them, every length and heat is in units
  of `forces.length(LINK)` (`LINK` at the default distance), the pair push is scaled by Repel force
  over 10, the pull along an edge by Link force, and the pull to the origin by Center force over
  0.5, so the defaults are #647's arithmetic. `set_forces` puts the run back to its starting heat
  from the places it has; `forces()` reads them. `arrowhead(from, to, radius)` gives a head's
  three points in view pixels (10 px long, 10 wide, its tip 2 px off the target's rim), `None` when
  the two ends are too close.
- `graph_settings` (#657): the Graph tab's record. `SliderRange` (`min`, `max`, `step`,
  `default`; `clamp` onto the grid, `holds`, `decimals`) and Rusty's seven ranges; `DEPTHS`;
  `Forces` (`length`, `repel_scale`, `center_scale`, `describe` for the run line); `Display`
  (arrows, text fade, node size, link thickness; `label_alpha(zoom)`, which starts at #647's 0.9
  at 0.5 and moves 1.6 a unit, Rusty's slope); `GroupColor` (six terminal hues by name, `next`,
  `for_place`); `Group`; `Switch` (Tags, Unresolved, Decision edges, Orphans, a set rather than
  four bools); `GraphSettings` with `is_on`, `toggle`, `from_stored` (field by field, each
  fallback to Rusty's default given as a reason) and `to_stored` (flat JSON).
  `group_colors(graph, shown, groups)` gives each shown node the first group whose query matches
  its graph node, and each group's count.

## Fixtures and the stand-in
- `fixtures/settings_list.json`: an answer in `rusty-mcp`'s shape, neutral values.
- `stand_in/rusty-mcp`: a Python program on the standard library that answers as `rusty-mcp` does
  for the tools Marley reads (since #654 `brain_list_pages` too, the newest file first; since #655
  `brain_read_page`, `brain_due`, `brain_set_property`, `list_task_groups` and `list_tasks` (since
  #658 the ten task writes too, with Rusty's answers and refusals and a `list_changed` after each
  that succeeds; the file is not watched) from a
  `tasks.json` in the state folder, and `brain_list_pages` typed by frontmatter with `aliases` and
  `properties`), over a scratch folder (`$RUSTY_STAND_IN_STATE`): stdio with no
  arguments, or `--http ADDR` (port 0 picks one, written to `http-addr`). It logs every request with
  its pid in `calls`, seeds `settings.json` from the fixture, and on stdio sends
  `notifications/resources/list_changed` when that file changes. Since #644 it also serves the vault
  tools (`brain_tree`, `brain_search`, `brain_daily_note`, `brain_new_page`, `brain_new_folder`,
  `brain_rename`, `brain_delete_page`, `brain_delete_folder`) over a scratch vault at
  `$RUSTY_STAND_IN_STATE/vault`, made with `archive/` and stored as `brain_vault_path`,
  `brain_render` (no HTML; `raw`, `properties` from simple YAML, `file`, `links` resolved by path or
  by a unique name), with Rusty's answers and refusals as of Rusty's TICKET-040 (the root's
  `archive/` left out of the tree and the search, writes into it refused), and since #646
  `brain_get_links`, `brain_graph` (`around` and `depth`), `brain_tags`, `brain_search`'s `<b>`
  snippets with `tag:` terms and its options, and `brain_new_page { path }`, and since #647
  `brain_graph` with Rusty's page types (the frontmatter's `type`, else the folder's, else `note`),
  a decision's `consulted`, `supersedes` and `follows_up` edges first, `tags`, `unresolved`, and the
  neighbourhood walk, and `brain_page_types`; it announces a change to any file in that vault too,
  as Rusty's watcher does. Since #656: `brain_render`'s `outline` by Rusty's rule (ATX headings
  outside fences, `line` from 0 in the body), `brain_set_property` keeping a replaced key in its
  place, `brain_remove_property`, and `brain_rename` rewriting `[[from]]`, `[[from|` and `[[from#`
  in the other pages (a folder's pages by prefix), counting them, and moving a title equal to the
  old name, as Rusty's TICKET-046 does. Since #659 `brain_due` sorts `all` as Rusty does (decided
  descending, then slug), a `today` file in the state folder fixes the day it counts from, and a
  `fail` file (a tool's name, then a message) makes that tool answer a JSON-RPC error. Since #660
  it serves `brain_follow_up` as Rusty does (its five refusals in Rusty's words; the follow-up
  section before `## Timeline`; `status`, `followed_up`, `follow_up_by` set or removed,
  `superseded_by`; Rusty's timeline line left out), and `brain_due` serves `followed_up` and
  `superseded_by` as strings. Since #662 it serves `bookmark_list`, `bookmark_add`,
  `bookmark_remove` and `bookmark_set` over `vault/.rusty/bookmarks.json` with Rusty's rules (a
  second of the same key dropped, a blank title filled, an unknown one refused); `brain_rename`
  carries a page's or a folder's bookmarks and the deletes drop them; and a change to the file is
  announced as a change to the vault is. Since #663 it serves `brain_capture` (the daily note or
  `inbox/inbox` made when missing, the line under `## Timeline`, the receipt), `source_capture` (a
  20 s fetch with `urllib`, an HTML page's title and paragraphs, a failure recorded on the page as
  `status: failed` and `error`, a URL that is not http or https refused in Rusty's words),
  `brain_import_plan` and `brain_import` (pages and attachments with dot entries skipped,
  collisions, tags, unresolved links, `.obsidian/bookmarks.json`, bare names rewritten to paths,
  the bookmarks added, a report page under `inbox/`). The last three answer on a thread of their
  own, as Rusty answers requests side by side, so Marley's pings go on during a slow fetch; its
  daily note takes today from the `today` file too. Since #664 it serves `list_memories`,
  `store_memory`, `update_memory` and `delete_memory` over `memories.json` in the state folder with
  Rusty's order, defaults, importance words and refusals, announces each memory write, and watches
  the file. It matches search words, not embeddings. The e2e
  scenarios name it with `MARLEY_RUSTY_MCP` and never reach the user's own Rusty (R-D8).

## Later slices
The Knowledge panel's project view (R6) and the graph's groups and sliders (R5b) are their own tickets; a Barnes-Hut pass would lift the 2,000-node cap.
