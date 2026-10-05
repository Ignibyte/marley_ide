# `marley_rusty`

> Per-crate architecture note, written 2026-10-04 at #643, extended at #644 to #647. Provenance:
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
  `brain_daily_note` answer; `RenameReport` from `brain_rename`. The rail's Brain view's model:
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
  in a vault, `None` for one that would leave it); `BRAIN_RENDER`. Since #654 `NewPage { path }`:
  `from_target` (the heading cut, the target normalised, `None` for an empty part or a `..`) and
  `arguments()`, `brain_new_page`'s `path` with its `folder` and `name`, so a Rusty with its
  TICKET-041 and an older one both make the page at the path.
- `switcher` (#654): the page picker's pure part. `PageSummary` (`brain_list_pages`' `slug` and
  `title`, `shown_title` falling back to the file's name) and `parse_page_list`; `RecentPages`,
  the recently opened newest first (`visit`, at most `RECENT_CAP` 20, `from_json`, `to_json`,
  `rank`); `empty_order(pages, recent, active)` (the active page, the recent pages the list holds,
  the separator after them, the rest in Rusty's order, the selection on the second row after the
  active page); `merge(titles, slugs, pages, recent, cap)` (each page at the better of its two
  fields' scores, lit in the field that gave it, ties to the more recently opened then the list's
  order); `create_target(query, pages)`; `BRAIN_LIST_PAGES`, `LIST_LIMIT` 100,000, `MATCH_CAP` 100.
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

## Fixtures and the stand-in
- `fixtures/settings_list.json`: an answer in `rusty-mcp`'s shape, neutral values.
- `stand_in/rusty-mcp`: a Python program on the standard library that answers as `rusty-mcp` does
  for the tools Marley reads (since #654 `brain_list_pages` too, the newest file first), over a
  scratch folder (`$RUSTY_STAND_IN_STATE`): stdio with no
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
  as Rusty's watcher does. It rewrites no links and matches search words, not embeddings. The e2e
  scenarios name it with `MARLEY_RUSTY_MCP` and never reach the user's own Rusty (R-D8).

## Later slices
The Knowledge panel's project view (R6) and the graph's groups and sliders (R5b) are their own tickets; a Barnes-Hut pass would lift the 2,000-node cap.
