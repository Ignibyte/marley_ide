# The Graph tab: Rusty's vault as a graph, local or whole — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-647-brain-graph-tab.md
- **Pipeline spec:** 647-brain-graph-tab.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-03)
- **Request:** Chad, 2026-10-02, the idea behind the whole plan: "an all in one system for Marley
  which brings in obsidian like knowledge graphs" (`docs/marley/rusty-in-marley.md`, What Chad
  asked for), then "maybe rusty becomes Marley" and "Plan it now". For this batch: "lets make a
  plan to begin the work and spec out the tickets" and "lets make sure we use the gpui components
  we found here", confirmed as "Queue all five" (#643 to #647, this one last). The plan's R-D3 row:
  "A **Graph** tab: `brain_graph` around the current page or project with depth, filters and
  colour by link type"; its R-D10 row names Ely's `networkgraph-forcegraph-chorddiagram-
  parallelcoordinates` story for R5. The design note's K3: "Rusty's `GraphView.qml` is the
  reference" (`herdr-and-hermes-2026-10-02.md:333-336`).
- **Classification / tier:** feature, M (the plan's size for R5). Two new pure modules in
  `marley_rusty`, one new tab module in `marley_workbench::rusty`, two actions, additions to #643's
  stand-in and fixtures, one line in the root `Cargo.toml` (a ledger row that exists). No new
  dependency: `serde`, `serde_json`, `futures` and the gpui-side crates are already in the two
  manifests (`marley_rusty`'s as #643 makes it). No new spawn site and no socket: calls go through
  #643's client.
- **Recall (§18.3):**
  - L-claude-482-background-work-in-a-marley-crate-is-a-lazy-future-001 and
    L-claude-507-sync-work-for-the-background-executor-goes-in-future-lazy-001: a layout batch on
    the background executor is `cx.background_spawn(futures::future::lazy(move |_| ..))`; an
    `async move` block with no await fails the dylint stage (gate:21), which clippy does not run.
  - AD-claude-609-the-agent-tab-is-read-only-and-its-samples-live-in-memory-001: a read-only
    center tab, found again by its key, not a `SerializableItem`, reading only while it is its
    pane's active item, every open through `Window::defer` since the opener walks the tabs. D9
    and D12 follow it; restoring the Graph tab waits for R5b, as the Agent tab's waited.
  - L-claude-515-dispatch-through-the-window-from-inside-an-action-001: a forwarded action goes
    through `window.dispatch_action`; `cx.dispatch_action` from inside a palette dispatch logs
    "window not found". The rail's Graph entry dispatches `rusty::OpenGraph` the window's way; a
    node click calls #645's `open_later`, which defers itself.
  - L-claude-465-a-doc-opens-with-one-short-line-001 (`too_long_first_doc_paragraph`) and
    L-claude-504-rustdoc-checks-what-clippy-and-dylint-do-not-001 (private intra-doc links fail
    gate:14): both bite new modules with long docs, as these three are.
  - #609's notes: the sparkline converts indices with `u16::try_from` and `f32::from` ("per-mille
    integers, no float cast"), because the pedantic cast lints are errors in the Marley crates;
    Ely's `count as f32` lines need the same rewrite (D10).
  - #489's notes: `marley_browser` and two image crates went to `opt-level = 3` in the dev profile
    after a debug decode took 130 ms a frame; the precedent for D11.
  - L-claude-489-zlog-filters-by-the-crate-a-line-comes-from-001: Marley's log is
    `<data dir>/logs/Marley.log`, filtered by the crate the line is logged from; the layout's
    lines are logged from `marley_workbench::rusty::graph_tab`, where the batches are driven.
  - L-claude-504-a-click-that-passes-can-still-miss-its-target-001: measure click targets from
    the shots, and shoot straight after a click while the pointer hovers. Here the filter-and-Fit
    move puts the target in the middle of the canvas, so most clicks need no measuring.
  - L-claude-607-a-settings-edit-after-marleys-own-write-does-not-reload-001: the scenario turns
    `marley.rusty.enabled` off by editing the run's settings from outside, never through Marley.
  - L-claude-633-scenarios-copy-the-users-settings-so-defaults-reach-real-services-001 and R-D8:
    the scenario runs the stand-in only; the harness's copy keeps the user's Rusty off.
  - #565's `decisions.rs::open` (focus the open tab of that type or add one): D1's shape.
  - #602's scenario drags with `pointer_down`, `pointer_to` and `pointer_up` under sway, the
    moves REQ-016 and REQ-018 need.
  - Rusty's own completed pipeline `graph-views` (`/srv/stacks/rusty-v3/docs/planning/pipeline/
    completed/graph-views.notes.md:94`): "the simulation is quadratic in the node count; a timer
    stops when the layout settles and restarts on interaction; a few hundred pages is the expected
    size", accepted with "a Barnes-Hut pass is a later tidy-up if vaults grow". The vault has since
    passed 800 pages, which is why this ticket moves the work off the main thread and caps it.
  - Brain (`rusty-cli brain search`, read only): `projects/rusty-v3` records TICKET-004's graph
    views (2026-09-03: "brain_graph tool, GraphView canvas with force layout and Obsidian's panel,
    global and local graph tabs"); `projects/brain-roadmap-hermes-obsidian-research` lists
    "local graph + depth + filter + color-by-relationship-type ... our typed edges make this
    uniquely good", which is D4's case for a colour per decision edge kind. Promotion asks the
    brain (`brain ask`) before locking D4 to D6.
  - The batch's queued specs (2026-10-03): #643's D5 (only the embedded connection gets
    `list_changed`; the service one reads on connect and after its own writes) and D8 (the Python
    stand-in over a scratch state folder); #644's D5 (a fixed-row entry appears with its tab; it
    ships Today alone); #645's opener (`rusty::OpenPage { slug, preview }`, `open_later` in
    `window.defer`); #646's D2 (the panel follows `ActiveItemChanged`) and its Out (R6's project
    view is its own ticket), which settles this ticket's no-page case.
  - The vault, measured read only on 2026-10-03 (`sqlite3 -readonly`, counts only): 807 pages;
    2,758 link rows, 2,719 resolved, every one `reference`; 193 distinct tags on 1,474 page-tag
    pairs; 238 decision pages; 335 pages with no link either way; the most linked page has 120
    backlinks. Types by count: decision, research, conversation, project, concept, note, daily,
    idea, company.
- **Discovery:**
  - Rusty's tool: `rusty-mcp/src/main.rs:643-656` (`GraphParams`: `tags`, `unresolved` default
    false, `around: Option<String>`, `depth: Option<usize>`), `:1402-1415` (`brain_graph`,
    "The vault as a graph: page nodes (title, type, folder, tags) and edges from resolved links;
    tags and unresolved targets as nodes on request; `around` with `depth` keeps one page's
    neighbourhood"), `:1053-1056` (`brain_page_types`: the known types with folders and counts).
  - Rusty's engine: `rusty-core/src/brain/mod.rs:219-232` (`GraphOptions`), `:236-249`
    (`GraphNode`: `id` is the slug, `tag:<name>` or `new:<target>`; `kind` `page`, `tag` or
    `unresolved`; `title`, `page_type`, `folder`, `tags`), `:257-265` (`GraphEdge`: `from`, `to`,
    `kind` `link`, `consulted`, `supersedes` or `follows_up`, default `link`), `:269-274`
    (`Graph`), `:885-1044` (`graph`: every page and resolved link from SQLite; decision edges
    first so they win over the plain link between the same two pages, `:955`; unresolved targets,
    `:966`; tags as nodes with lowercased ids, `:981-998`; the neighbourhood breadth first over
    undirected edges, tag edges included, `:1000-1044`; depth defaults to 1 and is at least 1).
    `decisions.rs:439-459` (`decision_edges`: reads every decision page's file for `consulted`,
    `supersedes`, `superseded_by`, from `list_pages(Some(DECISION_TYPE), Some(1000))`).
    `vault.rs:11-23` (`TYPE_DIRS`, the order `brain_page_types` returns: person, company,
    project, concept, meeting, idea, daily, inbox, decision, conversation, source; `note` and
    frontmatter types such as `research` are not in it). `mod.rs:3088-3112` (`index_links` writes
    every wikilink as `reference`; `add_link` at `:2118-2135` is the only other writer).
  - Rusty's app: `GraphView.qml` (622 lines): the panel's state `:23-41` (filter, Tags,
    Unresolved, Decision edges, Orphans, groups, arrows, text fade, node size, link thickness,
    four forces); `load` `:68-72` (`tags` and `unresolved` passed through, `around` and `depth`
    for a local graph); `setGraph` `:122-153` (positions kept by id, new nodes on a golden-angle
    spiral, a fit after the first settle); `matches` `:155-168` (the filter grammar); `applyFilter`
    `:175-184` (orphans: pages with no visible edge); `tick` `:186-240` (pair repulsion, springs,
    a centre pull, damping, alpha cooling ×0.985, stop on low energy) on a 33 ms timer `:241`;
    `radiusOf` `:244` (3 + √degree × 1.6); `nodeAt` `:246-258`; `fit` `:270-279` (zoom held to 0.2
    to 1.6); the paint `:283-363` (typed edges dashed and thicker in the accent `:303-309`;
    hovered node and neighbours lit, the rest at 0.15 `:334-335`; labels fading in with the zoom
    `:347-360`); the mouse `:365-422` (press on a node pins it; a release after under 3 moves is
    a click: a page opens, a tag searches `:401-404`; the wheel zooms about the cursor, 0.15 to 6,
    `:411-421`); the header `:437-445` with Rusty's empty-state words; the panel `:447-562`
    (Filters with depth 1 to 4 `:485-502`, Groups `:503-537`, Display `:538-548`, Forces
    `:549-559`). `Main.qml:400-416` (`openGraph`: one global tab, one local tab following
    `lastPageSlug`), `:719-720` (palette entries, Ctrl+G), `:921-931` (the local graph's `around`).
    `NoteTab.qml:188` (`brain_graph` around the page at depth 3, for the page's small legend),
    `:418` (Open local graph in the page's menu).
  - Ely (HEAD 2f8b2f6, MIT OR Apache-2.0, `LICENSE-MIT`: "Copyright (c) 2026 Ely GPUI Component
    contributors"): `src/charts/network.rs:17-93` (`Force`: places in a unit square, heat 0.08,
    k = 0.6/√n, cooling ×0.96, settled under 0.001, so about 107 steps), `:147-382` (`NetworkGraph`
    render), `:385-407` (its test); `src/charts/paint.rs:54-56` (`tint`), `:111-116` (`finish`),
    `:119-136` (`ring`); `src/canvas/view.rs:1-174` (`Frame`, `Viewport` and tests);
    `src/charts/parts.rs:16-60` (`ChartLegend`); `src/canvas/nodes.rs:25` (`NodeGraph`, not
    taken); `src/forms/slider.rs` (for R5b).
  - Zed: `crates/gpui/src/path_builder.rs:88-123` (`stroke`, `fill`, `dash_array`), `:244-255`
    (`build`), `:280-318` (dashes cut with lyon's sampler, then stroke tessellation into
    `VertexBuffers<_, u16>`), `:321-345` (`build_path`); `crates/gpui/src/elements/canvas.rs:10`
    (`canvas`); `crates/gpui/src/window.rs:2630` (`request_animation_frame`, not needed: each
    batch notifies), `:4359` (`paint_layer`); `crates/gpui/src/text_system.rs:638`
    (`shape_line`) and `text_system/line.rs:108` (`ShapedLine::paint`);
    `crates/gpui/src/elements/div.rs:390-405` (`on_scroll_wheel`, `on_pinch`).
    `crates/git_ui/src/git_graph.rs:1245-1272`, `:3166-3300`, `:4108-4180` and the
    `SerializableItem` after it. `crates/image_viewer/src/image_viewer.rs:39-58`, `:264-357`.
    `crates/theme/src/styles/accents.rs:24-40`, `:65-67`. `crates/ui/src/components/toggle.rs:15`
    (`checkbox`), `button/toggle_button.rs:171` (`ToggleButtonGroup`), `disclosure.rs:9`.
    `crates/icons/src/icons.rs:169` (`GitGraph`, the only graph icon).
  - Marley: `crates/marley_workbench/src/decisions.rs:51-60`; `agent_tab.rs:540-553`;
    `browser.rs:7556`; `harness.rs:451`; `rusty.rs` (90 lines today, #633's offer);
    `rail.rs:3928` (`render_header`, which #644 turns into the Projects and Brain switch);
    `crates/marley_workbench/Cargo.toml:105-160` (the lint table: pedantic and nursery as
    warnings, `-D warnings` in the gate, `unwrap_used` and `expect_used` denied).
  - The e2e runner: `script/e2e.sh:500-560` (`pointer_to`, `pointer_down`, `pointer_up`, `click`,
    `scroll`, all sway only), `:562-581` (`profile_setting`), `:627-642` (the settings copy),
    `:688-689` (Marley.log copied beside the shots).
  - The root manifest: `Cargo.toml:1082-1110` (`[profile.dev.package]`, `marley_browser` at
    `:1108`); `docs/marley/zed-touchpoints.md:47` (its row).
- **Decisions:** D1 to D13 in the spec. In short: one Graph tab with a Local or Vault scope; the
  local graph follows the focused page, its centre held in the middle; tags as nodes built in
  Marley so a local graph stays local; colours from the theme, page types by accent and decision
  edges dashed in status colours; the layout lent to the background executor a batch at a time,
  cooling, at most 300 steps; a 2,000-node cap by link count; batched paths under lyon's u16
  limit and labels painted in the canvas; Rusty's input with a pinch; reads only while shown, one
  in flight, places kept; Ely ported, not depended on; `marley_rusty` optimized in the dev
  profile; a node click through #645's deferred opener; off means empty.

### Design
- **`marley_rusty::graph`** (new, pure, no Ely code):
  - `Graph { nodes: Vec<Node>, edges: Vec<Edge> }` deserialized from `brain_graph`'s JSON;
    `Node { id, kind: NodeKind::{Page, Tag, Unresolved, Other}, title, page_type, folder, tags }`;
    `Edge { from, to, kind: EdgeKind::{Link, Consulted, Supersedes, FollowsUp, Other(String)} }`.
    Unknown kinds deserialize to `Other` rather than failing, since Rusty may add kinds.
  - `Query::parse(&str)` and `Query::matches(&Node)`: Rusty's grammar (`GraphView.qml:155-168`),
    terms lowercased; `tag:` matches a tag or one nested under it, `path:` a slug prefix, `type:` a
    page type, other text a substring of the title or slug; an empty query matches all.
  - `Filters { query, hidden_types, tags, decision_edges, orphans }` and `shown(&Graph, &Filters,
    centre: Option<&str>, cap: usize) -> Shown`, in this order: page and unresolved nodes the
    query and the legend let through; tag nodes for the tags of the pages kept (D3); edges between
    kept nodes, decision kinds dropped when off; pages with no kept edge dropped when Orphans is
    off; above `cap`, the most linked kept (the centre always), ties by id for a stable result.
    `Shown { nodes, edges: Vec<(u32, u32, EdgeKind)>, degree, total }`, `total` being the count
    before the cap, for the header.
  - `type_order(page_types: &[String], graph: &Graph) -> Vec<String>`: `brain_page_types`' order,
    then the graph's other types by name; the tab maps a type's place to
    `accents().color_for_index`.
- **`marley_rusty::graph_layout`** (new, pure, Ely's MIT notice at the top):
  - `Layout`: positions and pins per shown node, the heat, the step count. `Layout::seeded(&Shown,
    kept: &HashMap<String, (f32, f32)>, centre)` places a kept node where it was, a new node beside
    its first placed neighbour, else on the golden-angle spiral; the centre is pinned at the
    origin. `step` is Ely's `Force::step` in world coordinates: k is a constant ideal edge length
    (Rusty's 150 at zoom 1), repulsion k²/d over every pair of shown nodes, attraction d²/k along
    each edge, a pull to the origin, each move capped by the heat; the heat cools ×0.96. Two nodes
    on one spot get a nudge by their indices, so a run is deterministic. `run(budget)` steps until
    the pair budget (about 2 M checks) is spent, it settles, or 300 steps have run; `settled()`;
    `warm(amount)`; `pin(index, place)`, `release(index)`; `positions()`.
  - `Viewport` (Ely's): `to_view`, `to_canvas`, `zoomed(factor, about)` held to 0.15 to 6,
    `panned`, `fitting(bounds, size, margin, centre)` (a given centre stays in the middle, else the
    box's), all clamping where Ely asserts.
  - `nearest(positions, radii, point, slack) -> Option<usize>`: Ely's hit test.
  - The module's own tests carry Ely's properties (linked nodes nearer than a loner, a pinned
    node stays, a point keeps its place under a zoom about it, a fit shows the box), kept in the
    tree as §7 allows and built by gate:2, run by nothing until the testing phase.
- **`marley_workbench::rusty::graph_tab`** (new, Ely's MIT notice for the drawing it takes):
  - `GraphView { focus_handle, workspace, scope: Scope::{Local, Vault}, depth: u8, centre:
    Option<PageRef>, filter_field: Entity<Editor>, filters, graph: Option<Graph>, type_order,
    shown: Shown, layout: Option<Layout>, positions, kept: HashMap<String, (f32, f32)>, pending:
    Vec<LayoutChange>, viewport, hover, gesture: Option<Gesture::{Pan, Node}>, panel_open, notice,
    stale, reading: Task<()>, laying_out: Task<()>, _subscriptions }`.
  - `open(workspace, scope_hint, window, cx)` finds `items_of_type::<GraphView>` or adds one to
    the active pane (`decisions::open`'s shape). `impl Item`: `tab_content_text` "Graph" or "Local
    graph", a tooltip with the centre's title and depth, `IconName::GitGraph` (as the rail's
    entry; no new SVG), `show_toolbar` false; `to_item_events`. No `SerializableItem` (R5b).
  - Reads (D9): through #643's client, `brain_graph` with the scope's arguments and
    `brain_page_types` once per connection; the JSON parsed into `Graph` inside the background
    task; then `shown`, `Layout::seeded` with `kept`, and the batch loop. Subscriptions: #643's
    `list_changed` read (read if shown, else `stale`; in service mode every showing is stale),
    `workspace::Event::ActiveItemChanged` (a Page tab made active sets the centre to its page, as
    #646's panel follows it; the Graph tab's own showing clears `stale`), the settings store
    (D13).
  - The batch loop (D5): `cx.spawn` takes `layout` out of `self`, awaits
    `cx.background_spawn(futures::future::lazy(move |_| { layout.run(BUDGET); let positions =
    layout.positions().to_vec(); (layout, positions) }))`, puts both back, applies `pending`,
    notifies, and goes on until `settled()`; an info line per run (nodes, edges, steps, ms) and per
    batch at debug.
  - `render`: a `relative` root with the canvas filling it, the header (top left), the panel (top
    right), the notice line and the empty states (Rusty's words: "No pages yet", "No links around
    this page yet", and "Open a page to see its local graph"). The canvas's paint closure: cull to
    the bounds through `Viewport`; edges batched per style into `PathBuilder`s, each flushed before
    an estimated 60,000 vertices (four a segment, four a dash), dashed ones with a dash array in
    view pixels; nodes with `paint_quad` (round, edged in the background colour; the centre with an
    accent ring; unresolved hollow); labels with `shape_line` and `paint` (D7). Colours from
    `cx.theme()`: `accents()` by type order, `status().info`, `.warning`, `.success` for the three
    typed kinds, `colors().border` for links, `colors().text_accent` for lit links,
    `colors().text_muted` and `.text` for labels, `colors().editor_background` behind.
  - Input (D8): `on_mouse_down` (a node under the pointer starts a node gesture and pins it, else
    a pan), `on_mouse_move` (pan the viewport, or pin the node at the pointer and warm the layout,
    or update the hover), `on_mouse_up` (a gesture that moved under 3 px is a click: D12; a node is
    released), `on_scroll_wheel` (zoom about the pointer, lines and pixels both), `on_pinch`.
  - The panel: `Disclosure` sections Filters and Legend; `ToggleButtonGroup` for scope and for
    depth 1 to 4 (Local only); the filter `Editor::single_line`, applied on each edit; four
    `Checkbox`es; the legend's type entries (a dot, the type, its count; faint when hidden; a
    click toggles) and its edge kinds (a short solid or dashed line in the kind's colour);
    `IconButton`s Restart layout (releases pins, reseeds from the spiral), Fit, Hide panel.
  - Off (D13): the settings observer drops `graph`, `layout` and both tasks when
    `marley.rusty.enabled` goes off, and reads when it comes back on.
- **`marley_workbench::rusty`** (#643's module root): `pub(crate) mod graph_tab;`, `actions!(rusty,
  [OpenGraph, OpenLocalGraph])` with docs (the palette shows them), registered on each workspace
  in `rusty::init` behind #643's switch check (as #645 registers `OpenPage`; #645 also adds
  `"rusty"` to Zed's namespace test). A page node's click calls `rusty::page::open_later(workspace,
  slug, false, window, cx)` (#645).
- **The rail's fixed row** (`crates/marley_workbench/src/rusty/brain.rs`, #644's `BrainView`): the
  Graph entry after Today, `IconName::GitGraph`, tooltip "Graph", dispatching `rusty::OpenGraph`,
  as #644's D5 adds an entry with its tab.
- **The stand-in and fixtures** (#643's Python `crates/marley_rusty/stand_in/rusty-mcp` and
  `crates/marley_rusty/fixtures/`, with #646's additions; added to only where missing): the graph
  vault's pages (five
  types: project, decision, research, concept, note; `projects/orbit` linked from most; three
  decisions whose properties give one `consulted`, one `supersedes` and one `follows_up` edge;
  the tags `storage` and `storage/cold`; a link to a missing page; two orphans); a seeded
  generator for the 3,000-page vault (about 9,000 links, a few hubs), in Python beside the
  stand-in; `brain_graph` (#646's answers `around`) answering the whole vault, `depth` (the
  breadth-first walk of `mod.rs:1000-1044`, tag edges included) and `unresolved`;
  `brain_page_types` in `TYPE_DIRS` order. #643's stand-in already logs each request and sends
  `list_changed` when its state file changes; the scenario adds a page and switches to the
  generated vault by writing that state.
- **Performance, today and on a larger vault.**
  - The vault shows about 807 nodes and 2,700 edges plus the typed ones (about 1,000 and 4,200
    with Tags). A step's pair checks are n(n-1)/2: 325 k at 807, 500 k at 1,000, 2.0 M at 2,000.
    At an estimated 1 to 2 ns a check in an optimized build, a step costs 0.3 to 0.7 ms for the
    vault and 2 to 4 ms at the cap, and a run of at most 300 steps 0.1 to 0.2 s and 0.6 to 1.2 s.
    The Code phase logs the real figures and the Test phase reads them.
  - None of it runs on the main thread. A batch holds about 2 M checks (some six steps for the
    vault, one at the cap); the tab redraws after each, so the picture settles as it is computed,
    and the run ends at the heat floor or 300 steps and costs nothing until something warms it.
  - The main thread's share: one pass over the shown nodes per pointer move; per frame at most
    2,000 quads, a few batched paths, 200 labels, with off-screen nodes and edges skipped.
  - The read: the vault's answer is about 400 KB of JSON, parsed off the main thread; a local
    answer is a fraction. Rusty builds the whole graph for each call, a local one included, and
    reads every decision page's file for the typed edges (238 today); one read in flight keeps a
    burst of `list_changed` (an agent's brain loop writing) to two reads.
  - Larger: at 5,000 pages the vault scope lays out the 2,000 most linked and says so, and the
    filters and the local graph reach the rest; the read and parse grow with the vault (about
    2.5 MB at 5,000), the layout and drawing stay at the cap's figures. Past that the next steps
    are Barnes-Hut (Out) and, Rusty-side, its 1,000-decision limit on typed edges.
- **File manifest.**
  - Marley: `crates/marley_rusty/src/graph.rs` (new), `crates/marley_rusty/src/graph_layout.rs`
    (new, Ely's notice), `crates/marley_rusty/src/marley_rusty.rs` (the two modules);
    `crates/marley_rusty/stand_in/rusty-mcp` and `crates/marley_rusty/fixtures/` (the additions
    above);
    `crates/marley_workbench/src/rusty/graph_tab.rs` (new, Ely's notice),
    `crates/marley_workbench/src/rusty.rs` (the module and the actions);
    `crates/marley_workbench/src/rusty/brain.rs` (the fixed row's Graph entry);
    `crates/marley_workbench/guide/index.html` (a Graph article);
    `script/e2e/647-brain-graph-tab.sh` (new, Test phase).
  - Zed: the root `Cargo.toml` (`marley_rusty = { opt-level = 3 }` in `[profile.dev.package]`,
    after `marley_browser`). No Zed crate's source changes: `workspace`, `gpui`, `ui`, `theme`,
    `editor` and `git_ui` are used as they are.
- **The ledger row it extends** (`docs/marley/zed-touchpoints.md`, written before the line, §14):
  the `Cargo.toml` row (`:47`), whose `[profile.dev.package]` clause becomes "`image`,
  `zune-core`, `zune-jpeg`, `marley_browser` (#489: ...) and `marley_rusty` (#647: the graph
  layout's float loop, which every scenario's debug build runs) at `opt-level = 3`". #643 will
  have added `marley_rusty` to the same row's members.

### Visual check plan
The scenario `script/e2e/647-brain-graph-tab.sh`, `compositor sway`, one Marley run. Setup: a
scratch repository (`open_path`); the stand-in named in `MARLEY_RUSTY_MCP` (#643's rule) with the graph vault, its call
log and trigger in `$E2E_WORK`; `profile_setting marley.rusty '{"enabled": true, "connection":
"embedded"}'` (the keys #643 settles); a `set_setting` helper for edits while Marley runs (642's).
In Vault a node is reached by filtering to it and pressing Fit (the lone node lands in the middle
of the canvas); in Local the centre sits in the middle after Fit, since the filter never hides the
centre, and a neighbour's place, the panel's controls and the rail's rows are measured from the
first run's shots (L-504), the layout having no randomness. Each step waits for the layout's
"settled" line in Marley.log before its shot, except where the shot is meant to catch the run.

| REQ | Scenario | Shot |
|---|---|---|
| REQ-001, REQ-002 | `rusty: open graph` from the palette, nothing else open | `647-01-vault`: the fixture's pages in five colours, the legend's five types with counts, "Graph · N nodes" |
| REQ-003, REQ-025 | Filter `path:projects/orbit`; Fit; click the canvas's centre | `647-02-click-opens-page`: orbit's Page tab in front, the Graph tab in the strip |
| REQ-004 | `rusty: open local graph`; clear the filter | `647-03-local`: orbit ringed in the middle and its neighbours, "Local graph · Orbit · depth 1" |
| REQ-005 | Depth 2 in the panel | `647-04-depth-2`: more nodes; the stand-in's log shows `depth: 2` |
| REQ-006 | Pointer on the centre | `647-05-hover`: orbit and its neighbours lit, the rest dim, "Orbit" by the pointer |
| REQ-007 | Vault; filter to the decision page; Fit; click; `rusty: open local graph` | `647-06-decision-edges`: three dashed edges in three colours and plain links; the legend's three kinds |
| REQ-008 | In that local graph, click a linked concept page at its measured place; then `rusty: open graph` to bring the tab back | `647-07-follows-page`: "Local graph · <the concept's title>", it in the middle |
| REQ-009 | Vault; clear the filter; press `decision` in the legend | `647-08-type-hidden`: no decision nodes; the entry faint |
| REQ-010 | Filter `tag:storage` | `647-09-filter-tag`: the pages tagged `storage` and `storage/cold` only |
| REQ-011 | Filter `path:research/` | `647-10-filter-path`: the research pages only |
| REQ-012 | Clear; Tags on | `647-11-tags`: `#storage` and the other tags joined to their pages |
| REQ-013 | Tags off; Unresolved links on | `647-12-unresolved`: one hollow node for the missing target |
| REQ-014 | Decision edges off | `647-13-no-decision-edges`: no dashed edge |
| REQ-015 | Orphans off | `647-14-no-orphans`: the two orphans gone; the count two fewer |
| REQ-016 | `pointer_down` on an empty corner, `pointer_to` 200 px right and 100 down, `pointer_up` | `647-15-pan`: the whole graph moved by that much |
| REQ-017 | `scroll` five steps over a node away from the centre | `647-16-zoom`: nodes larger, that node still under the pointer, labels showing |
| REQ-018 | Local graph around orbit; `pointer_down` on the centre, `pointer_to` 150 px off, shot while held, `pointer_up` | `647-17-drag-node`: orbit under the pointer, its neighbours drawn towards it |
| REQ-019 | Shot; the stand-in adds `notes/new-idea` linking to orbit and fires its trigger; wait for the read | `647-18-before-change`, `647-19-after-change`: one new node beside orbit, the others in place |
| REQ-020, REQ-021 | The stand-in switches to the generated vault and fires; Vault scope; shot as soon as the read lands | `647-20-capped`: "2,000 of 3,000 nodes, the most linked", "settling" if the run has not ended; Marley.log's run line (2,000 nodes, steps at most 300, ms) checked by the run |
| REQ-022 | Bring the Page tab to the front; switch the rail to Brain (#644's key); click its Graph row | `647-21-rail-graph`: the Graph tab in front |
| REQ-023 | `set_setting marley.rusty.enabled false`; settle | `647-22-off`: "Rusty is off" in the tab; no `brain_graph` in the stand-in's log after the edit |
| REQ-024 | Not shot: making the stand-in fail on request is more fixture than it proves | Review: the error string reaches `notice` and the button reads again |

Not reached by a scenario: the user's own brain (R-D8; its 807 pages are measured above, read
only, and never shown in a shot); a pinch (no touchpad under headless sway; the handler shares the
wheel's zoom code, which REQ-017 shows); the layout's main-thread cost, which no shot measures and
the review and the log lines carry instead.

### Risks
- **Four tickets ahead of it are queued, not built.** The names used here come from their queued
  specs of 2026-10-03 (#643's client and Python stand-in, #644's `rusty/brain.rs` and fixed row,
  #645's `rusty::OpenPage` and `rusty::page::open_later`, #646's `ActiveItemChanged` and its
  stand-in's `brain_graph`). #644 and #645 disagree on layout (`rusty/brain.rs` by `#[path]`
  against `rusty/page.rs`); this tab follows #645's `rusty/` folder. Promotion re-reads what
  shipped and adjusts the manifest without changing the behaviour.
- **The project's page.** The request named it, and it is deferred to R6 (Out), which #646 also
  leaves to its own ticket. If R6 lands first, promotion takes the no-page case into this ticket
  as one call.
- **Lint traps in the port.** Ely's code uses `as` casts, `assert!`, `expect` and plain `a * b +
  c`: the pedantic cast lints, `expect_used` and nursery `suboptimal_flops` reject them in the
  Marley crates; `float_cmp` rejects `==` on floats in library code; the dylint stage rejects an
  `async` block with no await (L-482). Write the port to the lint table from the start.
- **Dashed paths.** Several dashed edges in one path share lyon's sampler, which measures the
  whole path; dashes could run across the gap between two edges. P3 zooms on `647-06` to check;
  if they do, each typed edge gets its own path (238 decisions' worth, few enough).
- **The u16 vertex limit.** A path past 65,535 vertices fails to build and draws nothing (Ely's
  `finish` logs it). The split estimate must count dashes, which grow with the zoom; a culled
  edge clipped to the view keeps the count bounded.
- **Determinism.** The scenario's clicks rely on the same layout every run: no randomness, a fixed
  pair budget rather than a time budget, ties broken by id. A fixture change moves measured
  positions; the filter-and-Fit move avoids most of them.
- **Catching "settling".** With `marley_rusty` optimized, the capped run may end within about a
  second; the shot is taken as soon as the read lands, and if the run has already ended, REQ-021
  rests on the log line and the review, which the table allows.
- **A hub's local graph.** At depth 2 or more around the busiest page (120 backlinks) the local
  graph can hold most of the vault; the cap and filters apply as in Vault.
- **Decision edges off in a local graph.** The server's walk crosses decision edges, so a node
  reached only through one stays shown, unlinked, when they are hidden (Rusty's graph does the
  same). Accepted; the header's count says what is shown.
- **Themes.** A theme with fewer accents repeats colours across types; the legend still names
  each. Status colours on a light theme are checked once in P3 if the run's theme is light.
- **The rail hides when Zed's AI features are off** (R-D9): the palette actions still open the
  tab.
- **The receipt.** The in-app guide page, the stand-in, the fixtures and the scenario are
  fingerprinted by the commit receipt; a change to any after the Code phase's green needs `--diff`
  again before the commit.
- **Public origin.** The fixtures are invented pages; nothing of the user's vault (titles, slugs,
  text) enters the repository. The counts above are aggregate numbers.

### User-facing docs to update at Complete (P4)
- `docs/marley/guide.md`: the Rusty part (`:911` today, grown by #643 to #646) gains the Graph tab:
  opening it, Local and Vault, depth, the filter grammar, the switches, the legend, the mouse, the
  cap and what it says.
- `docs/marley/walkthrough.md`: a stop for the Graph tab after #645's and #646's, with the checks
  the scenario makes.
- `crates/marley_workbench/guide/index.html`: changed in the Code phase (the receipt binds it).
- Architecture (§21): `docs/marley_architecture/marley_rusty.md` (#643's; `graph` and
  `graph_layout`, Ely's notice, the cap and the budget) and `marley_workbench.md` (the Graph tab);
  `docs/marley/rusty-in-marley.md`'s slices table (R5 done, R5b and Barnes-Hut named);
  `docs/marley/three-prong-plan.md`'s Rusty line if it tracks R slices; the `Cargo.toml` row of
  `zed-touchpoints.md` checked against what shipped; `CHANGELOG.md` under Added.

### Checklist (no TaskCreate in this harness)
- [x] Read CONSTITUTION §3, §7, §14, §18, §19, §20, the templates, and #633's and #642's specs for
      the shape.
- [x] Read `docs/marley/rusty-in-marley.md` whole (R-D0 to R-D10, the slices, Rusty's triage) and
      the design note's K3.
- [x] Read `brain_graph`'s parameters and result (`rusty-mcp` `:643-656`, `:1402-1415`) and
      `BrainManager::graph` with `decision_edges` in `rusty-core`.
- [x] Read `GraphView.qml` whole and its wiring in `Main.qml` and `NoteTab.qml`.
- [x] Read Ely's `network.rs`, its paint helpers, `canvas/view.rs`, `charts/parts.rs`; looked at
      `canvas/nodes.rs` and `forms/slider.rs`.
- [x] Checked Zed for a node graph: `git_ui::git_graph` (lanes, not forces), the other
      `PathBuilder` users, `image_viewer`'s pan and zoom, `mermaid_render`'s `dugong`, `petgraph`
      in `Cargo.lock`; nothing draws a force-directed graph.
- [x] Recall: the knowledge ledgers (L-482, L-507, L-515, L-465, L-504 twice, L-489, L-607, L-633,
      AD-609), the completed pipelines 489, 565, 602, 609 and 633, Rusty's `graph-views` notes, a
      read-only brain search, and the vault's counts read only.
- [x] Discovery with file:line across Rusty, Ely, Zed, Marley and the e2e runner.
- [x] Prior-art sweep, three legs plus Ely and Rusty, written into the spec.
- [x] Decisions D1 to D13 with reasons; the performance case with numbers and the larger vault.
- [x] Spec: scope (with R5b, the project centre and Barnes-Hut in Out), Reference (§20), Prior
      art, UI proof, twenty-five EARS rows, phase plan.
- [x] Design: the three modules, the stand-in's additions, the file manifest by crate, the one
      touchpoint row, the visual check plan, risks, the user docs for Complete.
- [x] Docs only: the ticket doc and this pair; no BACKLOG.md, no `active/`, no source, no cargo.
