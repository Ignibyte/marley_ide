---
pipeline_id: ae551350-ede4-486a-943b-dda8e33a158c
ticket: docs/planning/tickets/open/TICKET-647-brain-graph-tab.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "The Graph tab: Rusty's vault as a graph, local or whole"
type: feature
slice: Rusty in Marley R5 (rusty-in-marley.md R-D3, R-D10)
references: [docs/marley/rusty-in-marley.md, docs/planning/design-notes/herdr-and-hermes-2026-10-02.md]
---

## Title
A Graph center tab draws `rusty-mcp`'s `brain_graph`: the whole vault, or the local graph of the
focused page to a depth of one to four. Pages are dots sized by their links and coloured by page
type; plain links are lines, and a decision's typed edges (`consulted`, `supersedes`,
`follows_up`) are dashed, each kind in its own colour. A panel filters by page type, tag and path
and turns tags, unresolved links, decision edges and orphans on or off. A click on a page opens it
through #645's opener; a background drag pans, the wheel zooms, a node can be dragged.
The layout is Ely GPUI Components' Fruchterman-Reingold `Force`, ported into `marley_rusty`, run on
the background executor one batch at a time, so the window never waits on it. Chad, 2026-10-02 and
2026-10-03: "lets make a plan to begin the work and spec out the tickets", "lets make sure we use
the gpui components we found here", "Queue all five".

## Scope
### In
- **Built on #643 to #646, replacing none of it.** #643's `marley.rusty` switch, its MCP client in
  `marley_workbench::rusty` (tool calls; a read again on `notifications/resources/list_changed`,
  which only the embedded connection receives), the crate `crates/marley_rusty` with `fixtures/`
  and the Python stand-in `stand_in/rusty-mcp` (it logs each request and sends `list_changed`
  when its state file changes); #644's rail Brain view (`rusty/brain.rs`) and its fixed row, where
  an entry appears with its tab; #645's Page tab and its opener, `rusty::OpenPage { slug, preview
  }` and `rusty::page::open_later`; #646's following of the active item through
  `workspace::Event::ActiveItemChanged`, and its stand-in's `brain_graph` with `around`.
- **The data.** `brain_graph` with `tags: false`, `unresolved` from the panel, and for a local
  graph `around` (the centre's slug) and `depth`; `brain_page_types` once per connection, for the
  colour order (D4). The answer is parsed into `marley_rusty::graph` (nodes with `kind`, `title`,
  `page_type`, `folder`, `tags`; edges with `EdgeKind::{Link, Consulted, Supersedes, FollowsUp,
  Other}`) on the background executor.
- **What is shown** (`marley_rusty::graph`, pure): Rusty's filter grammar
  (`GraphView.qml:155-168`): terms separated by spaces, all of which must match, `tag:x` (the tag
  or one nested under it), `path:p` (a slug prefix), `type:t`, else text in the title or slug;
  page types turned off in the legend; tags as nodes, built from each shown page's `tags` (D3);
  decision edges on or off; orphans on or off (a page with no shown edge); the cap (D6). The
  local graph's centre is always kept.
- **The layout** (`marley_rusty::graph_layout`, pure, Ely's MIT notice): Ely's `Force`
  (`src/charts/network.rs:17-93`) in world coordinates, and Ely's `Viewport`
  (`src/canvas/view.rs:59-110`) for pan, zoom about a point and fit; positions kept by node id
  across reads and filters; the local graph's centre held at the origin; a pinned node; batches,
  cooling, the step cap and the settled test (D5).
- **The tab** (`marley_workbench::rusty::graph_tab`, Ely's MIT notice on the drawing it takes):
  `GraphView`, an `impl workspace::Item` with `Focusable` and `Render`, titled "Graph" or "Local
  graph"; one canvas painting edges, nodes and labels (D7); a header at the top left ("Graph · 807
  nodes", "Local graph · <title> · depth 2 · 23 nodes", "· settling" while the layout runs, the
  cap's words when it applies); a panel at the top right that folds to a button: scope (Local,
  Vault), depth 1 to 4 in Local, the filter field, the switches Tags, Unresolved links, Decision
  edges and Orphans, the legend (page types with their counts, each a toggle; the edge kinds
  shown), and the buttons Restart layout, Fit and Hide panel; the empty states and the notice line.
- **Opening.** `rusty: open graph` opens or focuses the one Graph tab (a new one starts Local when
  a page is focused, else Vault); `rusty: open local graph` does the same and switches it to Local
  around the focused page, or says "Open a page first" with none. The rail Brain view's fixed row
  gains its Graph entry, after Today as R-D9 orders it (#644's D5: an entry appears with its tab),
  `IconName::GitGraph` with the tooltip "Graph" (the only graph icon in `icons`, so no new asset),
  dispatching `rusty::OpenGraph`.
- **Live refresh** (D9): a read again on `list_changed` while the tab is its pane's active item,
  else at its next showing; positions kept, new nodes placed beside a neighbour.
- **Off and errors** (D13): `marley.rusty.enabled` off empties an open tab, which says "Rusty is
  off" and calls nothing; a failed call shows the tool's error with a Read again button.
- **Dev profile** (D11): `marley_rusty = { opt-level = 3 }` in the root `Cargo.toml`'s
  `[profile.dev.package]`, beside `marley_browser`'s.
- **Fixtures and the stand-in** (`crates/marley_rusty/fixtures/`, `stand_in/rusty-mcp`): the
  scratch vault gains the graph this ticket's scenario reads (pages of five types, links, the
  three decision edge kinds, tags with a nested one, an unresolved link, two orphans); the
  stand-in gains a seeded generator for a 3,000-page vault, chosen through its state folder; its
  `brain_graph` (#646's, with `around`) answers the whole vault, `depth` and `unresolved` as Rusty
  does (`rusty-core/src/brain/mod.rs:885-1044`); and it answers `brain_page_types`.
- **The in-app guide page** (`crates/marley_workbench/guide/index.html`): a Graph article, in the
  Code phase (the commit receipt binds the file).
- `script/e2e/647-brain-graph-tab.sh`.

### Out (explicitly deferred)
- **R5b, the second slice: graph settings, kept.** Rusty's colour groups (a query and a colour
  each, `GraphView.qml:503-537`), the Display sliders (text fade, node size, link thickness) and
  arrows, the Forces sliders (`:549-559`), and the tab and its settings restored after a restart
  (`SerializableItem`, as `git_ui`'s `GitGraph` keeps its state). Zed's `ui` has no slider, so R5b
  ports Ely's `src/forms/slider.rs`. This slice ships Rusty's default forces and display.
- **Centring on the current project's page when no page is focused.** R-D5's project join is
  slice R6, which #646 also leaves to its own ticket: it needs each project page's `path:`
  frontmatter (32 of the vault's 102 project pages carry one, some listing several folders), and
  no tool serves that in one call. The tab takes its centre as a slug, so R6 points the tab's
  no-page case at the join in one place. Until then a local graph with no page focused says
  "Open a page first" and Vault stays one click away.
- **A Barnes-Hut pass** (repulsion in O(n log n), d3-force's many-body at θ 0.9) to lift the
  2,000-node cap (D6). The vault is 807 pages; Rusty's own graph-views notes left the same pass
  "a later tidy-up if vaults grow".
- **`changes_since`** (Rusty's TICKET-035, not built): reads follow `list_changed` only, so a
  write made by another `rusty-mcp` process shows at the next announced change or the next
  showing; with the service connection, which gets no notification (#643's D5), every showing of
  the tab reads (D9).
- **Typed links other than a decision's.** `brain_links.link_type` (`reference`, or a type set
  through `add_link`) does not reach `brain_graph`'s edges, which say `link`; all 2,758 links in
  the vault are `reference` today, so nothing is lost. If Rusty sends more kinds, `EdgeKind::Other`
  draws them as links until a ticket gives them colours.
- A tag node opening brain search, as Rusty's does (`searchTag`, `GraphView.qml:403`): search is
  R3's. Here a click on a tag sets the filter to it (D12).
- Creating a page from an unresolved node; keyboard navigation between nodes; Ctrl+click to open
  beside the graph; a Local graph button on the Page tab and Rusty's small link legend on the page
  (`NoteTab.qml:445-480`); a second Graph tab.
- Rusty's 1,000-decision limit on typed edges (`rusty-core/src/brain/decisions.rs:441`): a
  Rusty-side matter for a vault past 1,000 decisions (238 today).

## Reference (§20)
Upstream Zed for the tab and its input: a center tab is a `workspace::Item` (`git_ui`'s
`GitGraph`, `git_graph.rs:4108-4180`, is the closest: a graph drawn on a `canvas` with
`PathBuilder`, coloured from `theme.accents()`), opened or focused as Marley's `decisions::open`
does; pinch to zoom and a zoom held about the pointer as Zed's `image_viewer` does
(`image_viewer.rs:264-357`). The graph itself is Marley-specific, with no Zed or Warp analog:
Warp draws no knowledge graph, and Zed's only graph is the git graph's commit lanes. Its reference
is Rusty's `GraphView.qml` (MIT, the screen this rebuilds,
`/srv/stacks/rusty-v3/crates/rusty-app/qml/GraphView.qml`) and Obsidian's graph view, whose panel
Rusty copies. No Warp code.

### Prior art
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` (the `Item`
  trait, `SerializableItem` as a separate facet, which is why restoring waits for R5b) and
  `01-gpui-ui-framework.md` (`canvas`, `paint_path`, `paint_quad`). `docs/orca_architecture/
  02-worktrees-and-review.md:312` names Zed's git graph; nothing in `docs/warp_architecture/` or
  the Orca maps draws a knowledge graph.
- **Published material:** Fruchterman and Reingold, "Graph Drawing by Force-directed Placement"
  (1991): repulsion k²/d, attraction d²/k, displacement capped by a cooling temperature, which is
  what Ely's `Force` implements; d3-force's simulation, whose default run cools over 300 ticks
  and whose many-body force is Barnes and Hut's (1986) quadtree at θ 0.9; Obsidian's Graph view
  help (filters Search files, Tags, Attachments, Existing files only, Orphans; Groups; Display;
  Forces; the local graph's depth); the MCP specification's `notifications/resources/list_changed`.
- **Code we already ship, Zed and Cargo.lock:** no crate in the tree draws a force-directed graph.
  The `PathBuilder` users are `editor`, `git_ui::git_graph`, `ui`'s divider and circular progress,
  and Marley's `agent_tab.rs`. `git_graph.rs` lays commits out in lanes (a different layout), and
  takes from it: `canvas` with `window.paint_layer` (`:3219-3224`), one `PathBuilder::stroke` per
  line (`:3292`), filled circles (`draw_commit_circle`, `:1245-1272`), `accents().color_for_index`
  for colours, and the `Item` shape. `gpui::PathBuilder` (`path_builder.rs`): `stroke`,
  `dash_array` (`:108-123`, dashes cut by lyon's sampler), and tessellation into u16 index buffers
  (`:308`, `:321`), which caps one path near 65,535 vertices (D7). `gpui::Window::paint_quad` for
  round nodes, `TextSystem::shape_line` and `ShapedLine::paint` for labels, `on_scroll_wheel`,
  `on_pinch`, `on_mouse_down`, `on_mouse_move`, `on_mouse_up`. `image_viewer` (`zoom_level`,
  `pan_offset`, `set_zoom` about a point, pinch; its plain wheel pans and Ctrl+wheel zooms, not
  taken, D8). `theme::AccentColors` (13 colours in Zed's default themes, `accents.rs:24-40`) and
  the status colours. `ui`: `Checkbox`, `ToggleButtonGroup`, `Disclosure`, `IconButton`,
  `Tooltip`, `Label`; no slider (hence R5b). `editor::Editor::single_line` for the filter, as
  `decisions.rs` uses it. `mermaid_render` builds `merman`, whose `dugong` is a dagre-style layered
  layout for diagrams, not forces, and wrong for an 800-node graph full of cycles; `petgraph` is in
  `Cargo.lock` only for `guppy` and `prost-build`, a graph structure with no layout. `icons` has
  `GitGraph` and no other graph icon.
- **Marley's own:** `decisions.rs:51-60` (`open`: focus the tab of that type or add one);
  `agent_tab.rs:540-553` (a `canvas` and a `PathBuilder` line, with index to `f32` through
  `u16::try_from` and `f32::from`, no `as`); `browser.rs:7556` (`mul_add`, the nursery
  `suboptimal_flops` lint); `harness.rs:451` (`ContextServer::stdio`, the client #643 copies);
  `rusty.rs` (#633's offer, which #643 grows into the module this tab joins).
- **Ely GPUI Components (R-D10), read at HEAD 2f8b2f6:**
  - `src/charts/network.rs` (`NetworkGraph`, `Force`). Taken: `Force`'s step (pair repulsion
    k²/d, edge attraction d²/k, a pull to the centre, each move capped by the heat, cooling ×0.96,
    `:52-92`), `pin` and `settled` (`:41-50`), its test's two properties (linked nodes sit nearer
    than a loner; a pinned node stays, `:389-407`), the hover rule that lights a node and its
    neighbours and dims the rest (`:202-217`, `:352-365`), round nodes edged in the background
    colour (`ring`, `paint.rs:119-136`), a radius growing with the square root of the degree, and
    the nearest-node hit test (`:259-270`). Left: the `RenderOnce` element with
    `use_keyed_state` (Marley's tab is an entity), three steps a frame on the main thread with
    `request_animation_frame` (`:164-175`; Marley's run on the background executor), the unit
    square and its clamp (Marley's world is unbounded and the view zooms), a `div` per label
    (`:218-244`; Marley paints labels in the canvas), one path per edge (Marley batches them),
    `ChartTooltip`, Ely's theme and palette (`tint`), `log::info!` per drag, and `assert!` and
    `expect` (Marley's lint table denies them).
  - `src/canvas/view.rs` (`Viewport`, `Frame`). Taken: `to_view`, `to_canvas`, `zoomed` about a
    view point, `panned`, `fitting` and their three tests' properties. Left: the constructors'
    `assert!` (Marley clamps), Ely's zoom range (Rusty's 0.15 to 6 is kept), and the grid marks.
  - `src/charts/parts.rs` (`ChartLegend`, `:16-60`): an entry is a dot and a name, a hidden one
    faint, a press toggles it. Rewritten onto `ui`'s `h_flex` and `Label` for the page-type
    legend. Left: Ely's theme, `FocusRing` and `tabular`.
  - Read and not taken: `src/canvas/nodes.rs` (`NodeGraph`, a wired node editor on an endless
    plane, another kind of graph), `src/canvas/zoom.rs` (a zoom percentage control, not needed),
    and the story's ChordDiagram and ParallelCoordinates. Ely's knowledge page says "GraphView is
    charts::NetworkGraph" (`examples/gallery/pages/documents/knowledge.rs:324`), so the
    `backlinks-graphview` story is the same component.
- **Rusty, the screen rebuilt** (`/srv/stacks/rusty-v3`, MIT): `GraphView.qml` (forces on a 33 ms
  timer, `:186-241`; positions kept across a reload, `:122-153`; typed edges dashed in the accent,
  `:303-309`; labels fading in with the zoom, `:347-360`; click, drag, pan and wheel, `:365-422`;
  the panel, `:447-562`), `Main.qml:400-416` (one global and one local graph tab, the local one
  following the last page), `brain_graph` (`rusty-mcp/src/main.rs:643-656`, `:1402-1415`) and
  `BrainManager::graph` (`rusty-core/src/brain/mod.rs:885-1044`).

## UI proof
`script/e2e/647-brain-graph-tab.sh` (`compositor sway`: it clicks, drags and turns the wheel).
Fixtures: a scratch repository opened with `open_path`; #643's stand-in `rusty-mcp`, named in `MARLEY_RUSTY_MCP` (never put first on the PATH: Marley takes its PATH from the login shell, L-531, so the real `rusty-mcp` could win; reconciled 2026-10-03 to #643's rule),, serving the scratch graph vault (never the user's brain, R-D8), logging each call; the run's
settings set `marley.rusty.enabled` true and `connection` `embedded` (`profile_setting`), later
changes made by editing the run's settings file from outside (L-607). Nodes are reached in two
ways: in Vault, by filtering to one and pressing Fit, which puts the lone node in the middle of the
canvas; in Local, the centre is in the middle after Fit, and a neighbour's place is measured from
the first run's shots, the layout having no randomness (L-504). Shots:
- `647-01-vault`: `rusty: open graph` with no page open: every fixture page, coloured by type, the
  legend's types and counts, the header's node count.
- `647-02-click-opens-page`: filtered to `path:projects/orbit`, Fit, a click on the centre:
  orbit's Page tab, the Graph tab beside it.
- `647-03-local`: `rusty: open local graph`: orbit and its neighbours at depth 1, orbit ringed
  in the middle, "Local graph · Orbit · depth 1".
- `647-04-depth-2`: depth 2 in the panel: the neighbours' neighbours added.
- `647-05-hover`: the pointer on orbit: it and its neighbours lit, the rest dim, its title shown.
- `647-06-decision-edges`: a decision page opened from the graph, then the local graph around it:
  a `consulted`, a `supersedes` and a `follows_up` edge dashed, each in its colour, beside plain
  links, the legend naming the three.
- `647-07-follows-page`: a neighbour clicked in the local graph (its Page tab opens), then `rusty:
  open graph` to bring the Graph tab back: centred on the neighbour.
- `647-08-type-hidden`: Vault; the legend's `decision` entry pressed: no decision nodes, the entry
  faint.
- `647-09-filter-tag`: the filter `tag:storage`: the pages tagged `storage` or `storage/...`.
- `647-10-filter-path`: the filter `path:research/`: the pages under `research/`.
- `647-11-tags`: Tags on: tag nodes joined to their pages.
- `647-12-unresolved`: Unresolved links on: the fixture's missing target, hollow.
- `647-13-no-decision-edges`: Decision edges off: no dashed edge left.
- `647-14-no-orphans`: Orphans off: the two orphans gone.
- `647-15-pan`: a drag from an empty corner: the graph moved by the drag.
- `647-16-zoom`: five wheel steps over a node: larger, about the pointer, labels in.
- `647-17-drag-node`: the centre node pressed, moved and shot while held: it under the pointer,
  its neighbours drawn after it.
- `647-18-before-change` and `647-19-after-change`: the stand-in adds a page linking to orbit and
  sends `list_changed`: the new node beside orbit, the other nodes where they were.
- `647-20-capped`: the stand-in switched to the generated 3,000-page vault, Vault: "2,000 of 3,000
  nodes, the most linked" and "settling"; the run checks Marley.log's layout lines.
- `647-21-rail-graph`: the Page tab in front, the rail's Brain view, the Graph entry after Today
  clicked: the
  Graph tab in front.
- `647-22-off`: `marley.rusty.enabled` set false: the tab says "Rusty is off"; the stand-in's log
  shows no call after it.

## Locked-In Decisions
- D1 — One Graph tab per workspace, with a scope: Local or Vault. Rusty keeps a global and a local
  tab (`Main.qml:400-416`); one tab with a switch matches the rail's one Graph row, and depth and
  filters carry across scopes. `rusty: open graph` focuses it or adds it to the active pane
  (`decisions::open`'s way); a new one starts Local when a page is focused, else Vault. `rusty:
  open local graph` switches it to Local, or answers "Open a page first" (Rusty's notice) and
  changes nothing.
- D2 — The local graph follows the focused page, as Rusty's follows the last page
  (`Main.qml:926`). It listens to `workspace::Event::ActiveItemChanged`, as #646's panel does, but
  keeps the page of the Page tab last made active rather than the active item's, since the Graph
  tab is itself the active item while it shows. A change while the tab is hidden is read when it
  shows. The centre is held at the origin, so it sits in the middle after Fit, and it survives
  every filter and the cap.
- D3 — Tags as nodes are built in Marley from each shown page's `tags`, never through
  `brain_graph`'s `tags: true`. The server walks a local graph's neighbourhood over tag edges too,
  so a local graph with tags would take in every page sharing a tag (193 tags on 1,474 page-tag
  pairs); built here, a tag joins only the pages shown, and the switch needs no read. Ids and
  titles follow Rusty's (`tag:<lowercase>`, `#<as first written>`, `mod.rs:981-998`).
- D4 — Colours from Zed's theme. A page node takes the theme's accent at its page type's place in
  `brain_page_types`' order, then the other types in the vault by name, so a built-in type keeps its
  colour as the vault grows; a tag node one colour outside that set; an unresolved node hollow;
  the focused page ringed. Edges by kind: a link in the border colour, lit in the accent text
  colour on hover; `consulted`, `supersedes` and `follows_up` dashed (Rusty's mark) in the status
  colours info, warning and success, outside the accents so no edge reads as a page type. Rejected:
  Rusty's single accent for every typed edge, since the kinds are what the brain loop records.
- D5 — Off the main thread, with no lock and no channel. The tab owns the `Layout` and lends it to
  the background executor for one batch at a time
  (`cx.background_spawn(futures::future::lazy(..))`, L-482, L-507), getting it back with the
  positions to draw. A batch is as many steps as fit about 2 M pair checks. Each step cools ×0.96
  (Ely's), and the run stops at the heat floor or after 300 steps (d3-force's default run),
  whichever is first; a drag, a filter or a read warms it a little and starts it again. A pin or
  release made while a batch is out waits for its return. The main thread only draws and finds
  the node under the pointer (one pass over the shown nodes per move).
- D6 — The cap is 2,000 shown nodes, applied after the filters and chosen by link count, the
  local graph's centre always kept; the header says "2,000 of N nodes, the most linked; filter or
  open a local graph". The vault is 807 pages (about 1,000 nodes with tags); a step at 2,000 nodes
  is about 2 M pair checks, a few milliseconds in an optimized build, so a full run stays near a
  second. Above the cap the layout would grow with the square of the count; Barnes-Hut is the
  deferred way past it.
- D7 — Drawing: one canvas; nodes as round quads (`paint_quad`, Ely's `ring`); edges batched into
  one path per style (links, lit links, each typed kind), each path split before it can pass the
  u16 vertex limit of gpui's tessellation (a dashed edge counts its dashes); edges and nodes
  outside the view skipped. Labels are shaped and painted in the canvas, not a `div` each: past
  Rusty's zoom threshold, for nodes in view, at most 200 a frame by link count, and always for the
  hovered node, its neighbours and the centre.
- D8 — Input as Rusty's (`GraphView.qml:365-422`): the wheel zooms about the pointer between 0.15
  and 6; a drag on the background pans; a drag on a node holds it under the pointer and lets it
  go on release; a press that moves less than 3 px is a click. A pinch zooms, as Zed's image
  viewer. Rejected: the image viewer's plain wheel panning (Ctrl+wheel to zoom), since a graph
  has nothing to scroll and Rusty and Obsidian zoom on the wheel.
- D9 — Reads come on opening, on a change of scope, centre, depth or Unresolved links, and on
  `list_changed` while the tab is its pane's active item; a hidden tab marks itself stale and reads
  when it shows (AD-609's rule for the Agent tab). With the service connection, which receives no
  notification (#643's D5), every showing counts as stale. One read is in flight at a time and
  later changes fold into one more. A read keeps every staying node's place, starts a new node
  beside its first placed neighbour (else on Rusty's golden-angle spiral,
  `GraphView.qml:128-136`), and warms the layout a little, so the picture does not jump. Fit
  runs after the first settle of a tab's first read and on the Fit button, never on a refresh.
- D10 — Ported, not depended on (R-D10). Ely's `Force` and `Viewport` go into
  `marley_rusty::graph_layout` with Ely's MIT notice, in world coordinates with the ideal edge
  length a constant (Rusty's link distance) instead of Ely's unit square, rewritten to the Marley
  lint table: no `as` casts (`agent_tab.rs`'s conversions), `mul_add` where clippy's
  `suboptimal_flops` asks, no `assert!` or `expect`, a deterministic nudge for two nodes on one
  spot (Rusty uses `Math.random`). The drawing taken from `network.rs` sits in `graph_tab.rs` under
  the same notice. Zed's own parts win: `ui`'s `Checkbox`, `ToggleButtonGroup`, `Disclosure` and
  `IconButton`; Ely's legend is rewritten onto `ui`.
- D11 — `marley_rusty` builds at `opt-level = 3` in the dev profile, as `marley_browser` does
  since #489 (a debug build took 130 ms a frame there). The layout is tight float arithmetic, the
  scenarios run the debug build, and an unoptimized step is estimated at ten to thirty times
  slower. One line, in a ledger row that exists.
- D12 — A click on a page node calls #645's `rusty::page::open_later(workspace, slug, false, ..)`,
  the opener for Marley's own callers, which runs in `window.defer`, so the workspace's walk over
  its items never meets the Graph tab in the middle of its own update (AD-609's reason). The page
  opens as a kept tab with the focus, where #645 opens pages (Rusty's `openPage(slug, false)`), and
  the Graph tab stays open. A click on a tag sets the filter to `tag:<name>`; a click on an
  unresolved node does nothing.
- D13 — Off means nothing (R-D0). With `marley.rusty.enabled` off the actions answer as #643's
  other `rusty:` actions do, and an open Graph tab drops its graph, positions and tasks and says
  "Rusty is off"; on again, it reads. A failed `brain_graph` call shows the tool's error in the
  notice line with a Read again button, and keeps the last graph drawn.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `rusty: open graph` runs while Rusty is on and no page is focused, the system shall open a Graph tab showing the vault's pages as nodes and their links as edges, with the node count in its header. | Shot `647-01-vault` |
| REQ-002 | The Graph tab shall colour each page node by its page type and list the types shown, with their counts, in a legend. | Shot `647-01-vault` |
| REQ-003 | WHEN a page node is clicked without moving, the system shall open that page in a Page tab through #645's opener and keep the Graph tab open. | Shot `647-02-click-opens-page` |
| REQ-004 | WHEN `rusty: open local graph` runs while a page is focused, the Graph tab shall show that page's neighbourhood at depth 1, with the page ringed in the middle and named in the header. | Shot `647-03-local` |
| REQ-005 | WHEN the depth changes in the local graph, the system shall read the neighbourhood again to that depth. | Shot `647-04-depth-2` |
| REQ-006 | WHEN the pointer rests on a node, the Graph tab shall light the node and its neighbours, dim the rest and show the node's title. | Shot `647-05-hover` |
| REQ-007 | The Graph tab shall draw `consulted`, `supersedes` and `follows_up` edges dashed, each in its own colour, apart from plain links, and name each kind shown in the legend. | Shot `647-06-decision-edges` |
| REQ-008 | WHILE the scope is Local, WHEN another Page tab becomes active, the Graph tab shall centre on that page the next time it shows. | Shot `647-07-follows-page` |
| REQ-009 | WHEN a page type's legend entry is pressed, the Graph tab shall hide that type's nodes and show the entry faint. | Shot `647-08-type-hidden` |
| REQ-010 | WHEN the filter holds `tag:<tag>`, the Graph tab shall show only pages carrying that tag or one nested under it. | Shot `647-09-filter-tag` |
| REQ-011 | WHEN the filter holds `path:<prefix>`, the Graph tab shall show only pages whose slug starts with it. | Shot `647-10-filter-path` |
| REQ-012 | WHEN Tags is turned on, the Graph tab shall add each shown page's tags as nodes joined to the pages carrying them. | Shot `647-11-tags` |
| REQ-013 | WHEN Unresolved links is turned on, the Graph tab shall add unresolved link targets as hollow nodes. | Shot `647-12-unresolved` |
| REQ-014 | WHEN Decision edges is turned off, the Graph tab shall hide every typed decision edge. | Shot `647-13-no-decision-edges` |
| REQ-015 | WHEN Orphans is turned off, the Graph tab shall hide pages with no shown edge. | Shot `647-14-no-orphans` |
| REQ-016 | WHEN the background is dragged, the Graph tab shall pan with the pointer. | Shot `647-15-pan` |
| REQ-017 | WHEN the wheel turns over the Graph tab, the system shall zoom about the pointer, showing labels once past the zoom threshold. | Shot `647-16-zoom` |
| REQ-018 | WHEN a node is dragged, the system shall hold it under the pointer while the layout moves its neighbours, and let it go on release. | Shot `647-17-drag-node` |
| REQ-019 | WHEN Rusty sends `list_changed` while the Graph tab shows, the system shall read the graph again, keeping staying nodes in place and adding new ones. | Shots `647-18-before-change`, `647-19-after-change` |
| REQ-020 | WHERE more than 2,000 nodes pass the filters, the Graph tab shall lay out the 2,000 most linked and say in its header how many of how many it shows. | Shot `647-20-capped` |
| REQ-021 | The system shall run the layout's steps on the background executor in batches and stop them once settled or after 300 steps. | Review; Marley.log's layout lines from the capped run |
| REQ-022 | WHEN the Graph entry of the rail Brain view's fixed row is clicked, the system shall open or focus the Graph tab. | Shot `647-21-rail-graph` |
| REQ-023 | WHEN `marley.rusty.enabled` turns off while the Graph tab is open, the tab shall drop its graph, say Rusty is off and make no further call. | Shot `647-22-off`; the stand-in's call log |
| REQ-024 | IF a `brain_graph` call fails, THEN the Graph tab shall show the tool's error with a Read again button. | Review |
| REQ-025 | WHEN Fit is pressed, the Graph tab shall zoom and pan so every shown node is in view, the local graph's centre in the middle. | Shot `647-02-click-opens-page` (the fit before the click); review |

## Phase Plan
- **P1 Plan** — promote; confirm what #643 to #646 shipped against the names used here (the
  client's call and its `list_changed` read, the Python stand-in's state folder, `rusty/brain.rs`'s
  fixed row, `rusty::page::open_later`, `ActiveItemChanged` as #646 follows it, whether R6's join
  landed); re-verify the seams cited in the notes; ask the brain.
- **P2 Code** — the `README.md` marker first; the root `Cargo.toml` row widened before the line
  (§14); `marley_rusty::graph` and `graph_layout` with Ely's notice, then the tab, the actions,
  the rail row, the stand-in's fixtures and answers, the in-app guide page; a review of the diff
  (re-entrancy on the click, errors reaching the tab, provenance of the ported code); `script/
  gates.sh --diff` green.
- **P3 Test** — write and run the scenario under sway, read every shot.
- **P4 Complete** — CHANGELOG (Added) and architecture docs (§21), the user docs the notes list,
  ledger capture (§19), close the ticket, archive, commit.
