---
pipeline_id: b738bd02-d8d7-45e8-af92-cd95dee453ec
ticket: docs/planning/tickets/open/TICKET-657-brain-graph-groups-and-forces.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Colour groups, sliders and arrows on the Graph tab, restored after a restart"
type: feature
slice: Rusty in Marley R5b (rusty-in-marley.md R-D3, R-D10)
references: [docs/marley/rusty-in-marley.md, docs/planning/pipeline/queued/647-brain-graph-tab.spec.md]
---

## Title
#647's Graph tab gains the rest of Rusty's graph panel (`GraphView.qml:503-559`), Obsidian's
Groups, Display and Forces: colour groups (a query in #647's filter grammar and a colour each; a
shown node takes the first matching group's colour, else its page type's), Arrows on the edges
that have a direction, Text fade threshold, Node size and Link thickness, and the four force
sliders acting on #647's layout. Zed's `ui` has no slider, so Ely GPUI Components' `Slider` is
ported with Ely's MIT notice. The graph settings become one record for all of Marley in Zed's
key-value store, and the tab a `SerializableItem` with Marley's own table, so after a restart it
is back in its pane with its scope, centre, filter and panel; with Rusty off at the start it is
not. Chad, 2026-10-02 and 2026-10-03: "lets make a plan to begin the work and spec out the
tickets", "lets make sure we use the gpui components we found here", "Queue all five"; #647's
draft split this slice out (its Out, "R5b, the second slice").

## Scope
### In
- **Built on #647, after it, replacing none of it.** #647's `GraphView` (`marley_workbench::
  rusty::graph_tab`, an `impl workspace::Item`) with its scope, centre, `Filters`, `Shown`, the
  layout lent to the background executor a batch at a time with its `pending` changes, its panel
  (`Disclosure` sections Filters and Legend, the four switches, depth 1 to 4), its canvas and its
  run log line; `marley_rusty::graph` (`Query::parse`, `Query::matches`, `shown`) and
  `marley_rusty::graph_layout` (`Layout`, `Viewport`, Ely's notice). Through #647: #643's switch,
  client and `Rusty` global with its connection state, the crate `crates/marley_rusty` and its
  Python stand-in named in `MARLEY_RUSTY_MCP`; #645's opener; #646's following of the active item.
- **The record** (`marley_rusty::graph_settings`, new, pure): `GraphSettings`, Rusty's `persist()`
  list (`GraphView.qml:101-106`): Tags, Unresolved links, Decision edges, Orphans, the depth, the
  groups, Arrows, Text fade threshold, Node size, Link thickness and the four forces, with Rusty's
  defaults, ranges and steps (D11); read from JSON with every field defaulted and clamped, written
  back as JSON; `Group { query, color }` and `GroupColor` (six named hues, D6); which group colours
  each shown node and how many each colours (D5); the label fade (D7).
- **The forces** (`marley_rusty::graph_layout`): `Forces`, and `Layout` stepping with them as
  multipliers on #647's constants, Rusty's defaults giving #647's layout place for place (D9);
  `set_forces` warming the layout from where it is; the arrowhead's three points (D8).
- **The slider** (`marley_workbench::rusty::slider`, new, Ely's MIT notice): Ely's single-thumb
  `Slider` rewritten onto the fork's gpui and Zed's theme (D10).
- **The tab** (`graph_tab.rs`): Groups, Display and Forces sections in the panel (D12): a row per
  group (swatch, query field, count, remove) and New group; the Arrows checkbox and three sliders;
  four sliders; each slider's value beside it. The canvas draws group colours, node size, link
  thickness, the label fade and arrowheads. Every Graph tab observes one settings global, so a
  change shows in each (D2).
- **Kept across a restart** (D1 to D4): the record in Zed's key-value store (scope
  `marley-rusty-graph`), read at `init` and written 300 ms after the last change and at quit; the
  tab's own state (scope, centre slug and title, filter, hidden page types, the panel and its open
  sections) in `marley_rusty_graph_tabs`, keyed by workspace and item and read into memory at
  `init`; `impl SerializableItem for GraphView` (kind `MarleyRustyGraph`), registered in
  `rusty::init`. A restored tab waits for #643's connection before it reads.
- **Off** (D4): with `marley.rusty.enabled` off at the start, no Graph tab is restored.
- **The e2e harness** (D13): `script/e2e.sh` drops the `marley-rusty-graph` scope from each run's
  copy of the user's database.
- **The in-app guide page** (`crates/marley_workbench/guide/index.html`): the Graph article gains
  Groups, Display, Forces and the restore, in the Code phase (the commit receipt binds the file).
- `script/e2e/657-brain-graph-groups-and-forces.sh`.

### Out (explicitly deferred)
- **A reset to the defaults.** Obsidian's panel has one; Rusty's has none. Until a ticket adds it,
  a slider's Home and End and the defaults in the guide page are the way back.
- **Settings of the local graph's own.** Obsidian keeps a local graph's settings in its pane;
  Rusty shares one set between its two graph tabs (`Main.qml:251-252`, `:927-930`), and so does
  this ticket.
- **Settings per vault.** Obsidian keeps `graph.json` in the vault. Marley does not know Rusty's
  vault root until Rusty's TICKET-042 serves it, and the box has one vault, so the record is
  Marley's.
- **Reordering groups.** The first matching group wins, so the order matters; a group goes to the
  end by removing it and adding it again.
- **A free colour picker** (Ely's `ColorPicker`, `src/forms/color/picker.rs:121`): six theme hues
  cycled from the swatch, as Rusty's seven are.
- **Obsidian's Animate and Attachments**: Rusty draws neither, and the vault has no attachment
  nodes.
- **The viewport and the node places across a restart.** #647's layout has no randomness and its
  Fit runs after the first settle of a tab's first read, so a restored tab comes back fitted on the
  same picture; Rusty restores neither.
- **Page tabs restored** (#645's D10): a restored local graph keeps its saved centre until a Page
  tab is made active.
- **Importing the Qt app's graph settings** (`ui.graph` in Rusty's window state): the Qt app is
  frozen (RQ5) and retires at R9.
- **R6's project centre** (#655: with no focused page, the local graph centres on the project's
  page). A restored tab's saved centre counts as its focused page until a Page tab is made active,
  so #655's rule applies only to a tab saved with no centre; the saved state is a JSON object, so
  whichever of #655 and this ticket lands second adds a "the centre is the project's" flag with no
  migration.
- `changes_since` (Rusty's TICKET-035) and Barnes-Hut stay where #647 left them.

## Reference (§20)
Upstream Zed for the restore: a center tab saved with its workspace through `SerializableItem`
(`workspace/src/item.rs:409-437`, registered with `register_serializable_item`), its state in a
table of its own keyed by workspace and item and cleaned with `delete_unloaded_items`, as
`git_ui`'s `GitGraph` keeps its log source and search (`git_graph.rs:4189-4421`), and small app
state in `db::kvp::KeyValueStore::scoped` (`db/src/kvp.rs:89-130`). Kept as Zed has it: the save
on `added_to_workspace` and on the item's events, throttled to 200 ms, the flush at quit, a failed
`deserialize` logged and skipped. The panel's behaviour is Marley-specific with no Zed or Warp
analog: Warp draws no knowledge graph and Zed's only graph is the git graph's lanes. Its
reference is Rusty's `GraphView.qml` (MIT, `/srv/stacks/rusty-v3/crates/rusty-app/qml/
GraphView.qml`, the screen this rebuilds) and Obsidian's graph view, whose Groups, Display and
Forces Rusty copies. No Warp code.

### Prior art
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md:76`
  (`SerializableItem: Item`, a facet with its own registry) and `:176`
  (`to_serializable_item_handle`); `docs/zed_architecture/crates/editor.md:170` (an item's state
  in SQLite for a session restore); `08-terminal-tasks-fusion.md:34` (`terminal_view`'s
  `persistence.rs`). `docs/warp_architecture/subsystems/01-ui-framework-rendering.md:388-393` names
  a `slider` among `warpui_core`'s widgets and says nothing of its behaviour; its source is not
  read (§20). Orca's settings draw shadcn's React `Slider` (`NotificationSoundSection.tsx:5`,
  `:169`), nothing for GPUI. No map draws graph groups or force sliders.
- **Published material:** Obsidian's Graph view help (Groups: a search query and a colour each;
  Display: Arrows, Text fade threshold, Node size, Link thickness, Animate; Forces: Center force,
  Repel force, Link force, Link distance), and the file Obsidian writes them to, a vault's
  `.obsidian/graph.json`, observed on this box for its key names only: `colorGroups`,
  `showArrow`, `textFadeMultiplier`, `nodeSizeMultiplier`, `lineSizeMultiplier`,
  `centerStrength`, `repelStrength`, `linkStrength`, `linkDistance`, beside `search`, the four
  switches and a `collapse-*` flag per section: one record for the vault's graph. WAI-ARIA's
  slider role (a value, its minimum, maximum and step, an orientation), which gpui's AccessKit
  roles carry. ProUI (brain page `research/ui-and-design/proui`), a paid GPUI kit with sliders:
  its licence keeps paid source out of public repositories, and Marley's origin is public, so it
  is not taken.
- **Code we already ship, Zed and Cargo.lock:** `workspace`'s `SerializableItem`
  (`item.rs:409-437`), `register_serializable_item` (`workspace.rs:1357`), `delete_unloaded_items`
  (`persistence.rs:2777`), the save on `added_to_workspace` (`item.rs:755-765`) and on events
  (`:872-876`), `SERIALIZATION_THROTTLE_TIME` (`workspace.rs:178`, `:7892-7927`),
  `flush_serialization` (`:7650-7680`), a failed `deserialize` logged and skipped
  (`persistence/model.rs:387-396`). `git_ui::git_graph`'s `GitGraph` (`:4189-4371`, its
  `git_graphs` table `:4389-4421`, whose `item_id INTEGER UNIQUE` at `:4396` Marley does not copy,
  F-576). `db`: `static_connection!` with a `Domain`'s migrations, and `KeyValueStore::scoped`.
  gpui: `Window::use_keyed_state` (`window.rs:4164`), `FocusHandle::tab_stop` (`:584`), `on_drag`
  (`div.rs:1614`), `on_drag_move` and `DragMoveEvent` (`:1042`, `:67-80`), the aria values
  (`:1406-1450`), `Role` and `Orientation` (`gpui.rs:91`), `EmptyView` (`view.rs:413`),
  `PathBuilder::fill` (`path_builder.rs:96`); handle drags in Zed: `editor`'s
  `split_editor_view.rs:198` and `git_graph.rs:548`, `:4023`. `ui`: no slider (searched `ui`,
  `settings_ui` and `gpui`; the word appears only in `_accessibility.rs:210-216` and
  `button_like.rs:38`), and `Disclosure`, `Checkbox`, `Button`, `IconButton`, `Tooltip` and
  `Label`, as #647 uses them. `settings_ui::NumberField` (`number_field.rs:258-330`, reachable
  once #643 adds `settings_ui` to `marley_workbench`): a value with minus and plus buttons and an
  edit mode, a stepper rather than a track, so not the control (D10). `theme`: `terminal_ansi_*`
  (`colors.rs:281-311`) for the group hues; `AccentColors` (`accents.rs:24-40`, thirteen hues),
  which #647 gives to page types. Cargo.lock holds no GPUI component crate (no `gpui-component`,
  `egui` or `iced`); `serde` and `serde_json` are in both manifests already.
- **Marley's own:** `browser.rs:7002-7160` (#494's `BrowserView`: the kind, the cleanup, a row
  per item in `MarleyBrowserTabsDb`, and #576's migration that drops `UNIQUE(item_id)`) and
  `:7580` (its registration); `terminal_ids.rs:1-47` (#575: the rows read at `init` and the
  restore served from memory); `terminal_size.rs:8-35`, `groups.rs`, `launch.rs` and
  `shortcut_note.rs` (the scoped key-value store holding JSON, read at `init`, written at quit);
  AD-601's `marley-groups` scope.
- **Ely GPUI Components (R-D10), read at HEAD 2f8b2f6:**
  - `src/forms/slider.rs` (466 lines, story `slider-rangeslider-verticalslider`). Taken: `Slider`
    (`:286-360`) and the parts of `Track` (`:60-280`) one thumb uses: `fraction` and `value_at`
    with their step grid (`:17-25`), `keyed` (arrows a step, Page Up and Page Down ten, Home and
    End the ends, `:28-39`), `along` (`:42-48`), the owner-checked `Thumb` drag (`:51-54`,
    `:192-198`, `:237-253`), a press on the track jumping there and taking the focus
    (`:254-266`), the track's bounds measured by a `canvas` (`:221-232`), the rail, the fill and
    the thumb, and `Role::Slider` with its aria values (`:161-166`). Left: `RangeSlider`
    (`:363-445`) and the vertical form; Ely's theme, sizes, `Elevation` shadow, `tab_stop`
    helper and i18n (Zed's theme colours and plain words instead); f64 values (f32 throughout, no
    cast); `assert!` in `range` and `step` (`:308`, `:315`); `log::debug!` on every move
    (`:344`); and its two tests (no ticket writes a unit test, §0; their properties are in the
    doc comments and the review).
  - `src/forms/color/swatch.rs` (`ColorSwatch` `:68`, `ColorPalette` `:152`): read, not taken. A
    cycle of six hues needs one dot, and the checkerboard is for see-through colours, which none
    of the hues are.
  - `src/charts/network.rs:255`, `:364` with `paint.rs:54-56` (`tint`): each node carries a group
    index and the paint looks its colour up in a palette. D5 takes the shape: a group index per
    shown node, the colour found at paint.

## UI proof
`script/e2e/657-brain-graph-groups-and-forces.sh` (`compositor sway`: it clicks, drags and quits
and relaunches Marley). Fixtures: a scratch repository opened with `open_path`; #643's stand-in
`rusty-mcp` named in `MARLEY_RUSTY_MCP` (never first on the PATH, L-531), serving #647's scratch
graph vault (never the user's brain, R-D8) and logging each call with its pid; the run's settings
set `marley.rusty` `enabled` true and `connection` `embedded` (`profile_setting`), later changes
made by editing the run's settings file from outside. The harness copy starts with no graph
settings (D13), which setup checks. Controls in the panel are reached at places measured from the
first run's shots (L-504); a slider is set exactly by a click on its track and then Home or End.
Before the relaunch the scenario clears `OPEN` so the session restores (L-601). Shots:
- `657-01-group`: Vault; Groups opened; New group with `path:research/`: the research pages red,
  the row's count.
- `657-02-first-group-wins`: a second group, `type:research`: the research pages still red, the
  second row's count 0.
- `657-03-group-removed`: the first group removed: the research pages in the second group's green.
- `657-04-next-colour`: that group's swatch clicked: the research pages yellow.
- `657-05-arrows`: Display opened, Arrows on, Tags on, three wheel steps over the canvas's
  middle: heads at the links' targets, none on a tag's edge.
- `657-06-node-size`: Node size's track clicked, then End: "3.0", every node larger.
- `657-07-slider-keys`: Left twice: "2.8".
- `657-08-link-thickness`: Link thickness to End: "4.0", every edge thicker.
- `657-09-thumb-drag`: Link thickness's thumb dragged to the track's middle: about "2.1", thinner
  edges.
- `657-10-text-fade-low` and `657-11-text-fade-high`: Fit; Text fade threshold to Home, then End:
  no labels, then labels at the same zoom.
- `657-12-repel`: Forces opened; Repel force to End: the graph wider in the same view.
- `657-13-link-distance`: Link distance to Home: the graph tighter.
- `657-14-before-restart`: the local graph around orbit, depth 2, the filter `path:research/`,
  the legend's `note` off, the panel open.
- `657-15-restored`: `quit_marley`, `launch_marley`: the Graph tab back in its pane, "Local graph
  · Orbit · depth 2", the filter and the faint `note` as before, the research pages in the
  group's yellow, the arrows.
- `657-16-restored-settings`: the panel scrolled to Groups, Display and Forces: the group's
  query, Arrows on, 2.8, the dragged thickness, 1.00, 20.0, 30.
- `657-17-settings-kept`: the tab closed; `rusty: open graph`: a Vault tab with the group's
  colour, the arrows and the slider values, the filter empty and `note` shown.
- `657-18-off-not-restored`: `marley.rusty.enabled` set false, quit, relaunch: no Graph tab;
  Marley.log's refusal; no new pid in the stand-in's log.

## Locked-In Decisions
- D1 — Two stores, split as Rusty splits them. The graph settings, Rusty's `persist()` list
  (`GraphView.qml:101-106`), are one record for all of Marley in Zed's key-value store
  (`KeyValueStore::scoped("marley-rusty-graph")`, key `settings`), as Rusty keeps one `graph`
  entry in its window state for both its graph tabs (`Main.qml:251-252`) and Obsidian one
  `graph.json`. The tab's own state (scope, centre slug and title, filter, hidden page types,
  whether the panel and each section are open) is its row in Marley's table, restored with the
  tab and gone when it closes, as Rusty's `saveTabs` keeps a tab's kind and slug
  (`Main.qml:466-473`) and `GitGraph` its search. Rejected: `settings.json` (a slider drag would
  rewrite the user's file over and over, and an outside edit after Marley's own write no longer
  reloads in that session, L-607); everything in the row (closing the tab would lose the groups,
  which Rusty keeps); everything in the store (two workspaces' tabs would share one centre).
- D2 — The record is live and written late. One `Global` holds it and every Graph tab observes
  it, so a change in one workspace's tab shows in every other's; a change of a switch, the depth
  or Unresolved links reads again in a shown tab and marks a hidden one stale (#647's D9). It is
  read once at `init`, before any window restores, and written 300 ms after the last change and
  once more at quit (`on_app_quit`, as `terminal_size.rs` does), so a slider drag writes once. A
  record that does not parse, or a field out of its range, falls back to Rusty's default for that
  field, and the reason is logged.
- D3 — The tab's table follows #494, #575 and #576. `marley_rusty_graph_tabs(workspace_id,
  item_id, state)`, `PRIMARY KEY(workspace_id, item_id)`, no `UNIQUE(item_id)` (F-576: item ids
  repeat across launches), `state` a JSON object so a later field needs no migration, `ON DELETE
  CASCADE` from `workspaces`; `cleanup` through `delete_unloaded_items`. The rows are read into
  memory at `init`, each save updates that copy, and `deserialize` reads the copy, never the table
  (PR-claude-a-row-a-restore-reads-is-read-before-any-cleanup-001). Zed saves the tab on
  `added_to_workspace` and on `ItemEvent::UpdateTab`, throttled to 200 ms, and flushes at quit;
  the tab emits `UpdateTab` whenever its own state changes, and its `added_to_workspace` points
  its workspace handle at the new workspace (L-613).
- D4 — Off is not restored (R-D0), and a restored tab waits for Rusty. With
  `marley.rusty.enabled` off at the start, `deserialize` refuses with "Rusty is off; the Graph tab
  is not restored", which Zed logs, adding no tab, and the workspace's cleanup drops the row; the
  record stays, since it is Marley's own state and no process or connection. A restored tab goes
  through the same construction as `rusty: open graph`'s (PR-claude-restore-is-a-second-
  constructor-001) and, until #643's `Rusty` global says connected, shows the connection's state
  in its notice line ("Connecting to Rusty", or #643's reason it is down) and reads nothing; it
  reads once connected. A restored local graph whose page is gone gets Rusty's empty answer
  (`rusty-core/src/brain/mod.rs:1022`) and #647's "No links around this page yet".
- D5 — Groups as Rusty's (`GraphView.qml:169-174`, `:503-537`). A group is a query in #647's
  filter grammar (`Query::parse`) and a colour; an empty query matches nothing. A shown node takes
  the colour of the first group that matches it, else its page type's colour (#647's D4), so with
  no group the graph is #647's. Each group's row shows how many shown nodes it colours, so a group
  the first one shadows reads 0. A query applies on each edit, as #647's filter does, and changes
  only the paint: no read, no new layout. Rejected: Obsidian's and Rusty's single colour for every
  node no group matches, which would drop #647's page-type colours at the first group.
- D6 — Group colours are the theme's terminal hues in Rusty's palette order without its accent
  (`GraphView.qml:62`): red, green, yellow, magenta, cyan, blue (`terminal_ansi_red` to
  `terminal_ansi_blue`). They are stored by name, so a theme change recolours them, as #647's
  colours follow the theme. A new group takes the hue at its place in the list (the first red, the
  second green), as Rusty's next-after-the-accent; a click on its swatch takes the next hue
  (Rusty's "Next colour", `:516-523`). Rejected: the accents (#647 gives them to page types) and a
  free picker (Out).
- D7 — Display as Rusty's (`:33-36`). Node size multiplies every node's radius (Rusty's
  `radiusOf`, `:244`), and the hit test uses the same radius, so the pointer finds a node where it
  is drawn. Link thickness multiplies every edge's width (`:308`); dashes keep their length. Text
  fade threshold sets the zoom labels start at, 2.2 − 1.6 × the value, fading in over the next 0.6
  of zoom (`:348`); 0.5 gives #647's threshold. #647's cap of 200 labels a frame and its labels
  that always show (the hovered node, its neighbours, the centre) stay.
- D8 — Arrows on the edges that have a direction: every edge `brain_graph` sends, from the
  linking page to its target (an unresolved target too) and from a decision to the page its
  property names (`decisions.rs:436-459`); not the tag edges Marley builds (#647's D3), which say
  a page carries a tag and point nowhere. A head is a filled triangle in its edge's colour, lit or
  dimmed with it, its tip on the target's rim, 10 px long and 10 px wide at any zoom (Rusty's,
  `:314-326`), batched into one fill path per edge style under #647's u16 split (three vertices a
  head); a head whose target is out of view is skipped. Two pages linking each other get a head at
  each end. Off by default, as Rusty's.
- D9 — Rusty's force sliders on #647's layout. Rusty's simulation (velocities and damping,
  `:189-240`) is not #647's (Ely's Fruchterman-Reingold), so the four sliders keep Rusty's names,
  ranges and defaults and act as multipliers on #647's step: the ideal edge length k is 0.6 × Link
  distance (Rusty's spring rest length, `:214`: 150 at the default 250, #647's constant);
  repulsion k²/d × Repel force / 10; attraction d²/k × Link force; the pull to the origin ×
  Center force / 0.5. At the defaults every multiplier is 1, so a graph with no record is laid out
  as #647 lays it out, place for place. A change warms the layout to its starting heat from the
  places it has, pins kept, as Rusty's `restart` (`:188`); one made while a batch is out waits in
  `pending` for its return (#647's D5); the run's log line names the forces it ran with.
- D10 — The slider is Ely's, ported (R-D10), into `marley_workbench::rusty::slider` with Ely's MIT
  notice: one thumb, horizontal, on the fork's gpui (`use_keyed_state` for the focus handle, the
  owner and the track's bounds; `on_drag` with an `EmptyView`; `on_drag_move` checking the owner;
  `Role::Slider` with `aria_numeric_value`, its minimum, maximum and step, the orientation and a
  label), drawn in Zed's theme colours (the rail in `border`, the fill in `text_accent`, the thumb
  in `elevated_surface_background` edged in `border`, `border_focused` with the focus). Values are
  f32 and go to the aria values through `f64::from`, so nothing is cast; a change under half a
  step is no change, so no float is compared with `==`; a range or step that makes no sense draws
  a slider that does not move, where Ely asserts. Zed's own wins where it has one: `ui` has none,
  and `settings_ui::NumberField` is a stepper. Each row shows the label, the slider and the value;
  Obsidian and Rusty show no value, and the value is what the visual check reads.
- D11 — Rusty's ranges and defaults (`GraphView.qml:538-559`, `:34-41`), with steps that put each
  default on the grid:

  | Slider | Range | Default | Step | Acts on |
  |---|---|---|---|---|
  | Text fade threshold | 0 to 1 | 0.5 | 0.05 | the zoom labels start at (D7) |
  | Node size | 0.3 to 3 | 1 | 0.1 | each node's radius |
  | Link thickness | 0.2 to 4 | 1 | 0.1 | each edge's width |
  | Center force | 0 to 1 | 0.5 | 0.05 | the pull to the origin (D9) |
  | Repel force | 0 to 20 | 10 | 0.5 | the repulsion |
  | Link force | 0 to 1 | 1 | 0.05 | the attraction |
  | Link distance | 30 to 500 | 250 | 10 | the ideal edge length |

  The switches keep #647's defaults, which are Rusty's (Tags off, Unresolved links off, Decision
  edges on, Orphans on), the depth 1, Arrows off, no group.
- D12 — The panel takes Obsidian's order: Filters, Groups, Display, Forces, then #647's Legend,
  each a `Disclosure`. The three new sections start folded on a new tab, so #647's panel looks as
  it did until one is opened; the panel scrolls when its open sections pass the tab's height.
  Rusty's words: "Groups", "New group", the placeholder "tag:x path:y type:z or text", "Next
  colour", "Remove group"; "Display", "Arrows", "Text fade threshold", "Node size", "Link
  thickness"; "Forces", "Center force", "Repel force", "Link force", "Link distance".
- D13 — Every scenario starts with no graph settings. `script/e2e.sh` copies the user's database
  into the run's profile, which would bring the user's own groups (queries that name the user's
  tags and folders, R-D8) and slider values into every run, so it deletes the `marley-rusty-graph`
  scope from each copied `db.sqlite`, beside turning Rusty off in the settings copy (L-633).

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a group is added and its query typed, the Graph tab shall draw every shown node the query matches in the group's colour and show in the group's row how many it colours. | Shot `657-01-group` |
| REQ-002 | WHERE a shown node matches more than one group, the Graph tab shall draw it in the colour of the first of them. | Shot `657-02-first-group-wins` |
| REQ-003 | WHEN a group is removed, the Graph tab shall draw the nodes it coloured in the colour the remaining groups or their page type give them. | Shot `657-03-group-removed` |
| REQ-004 | WHEN a group's swatch is clicked, the system shall give the group the next hue of the palette. | Shot `657-04-next-colour` |
| REQ-005 | WHILE Arrows is on, the Graph tab shall draw a head at the target end of every link and decision edge, and none on a tag's edge. | Shot `657-05-arrows` |
| REQ-006 | WHEN Node size changes, the Graph tab shall scale every node's radius by it. | Shot `657-06-node-size`; review (the hit test's radius) |
| REQ-007 | WHEN Link thickness changes, the Graph tab shall scale every edge's width by it. | Shot `657-08-link-thickness` |
| REQ-008 | WHEN Text fade threshold changes, the Graph tab shall show labels from the zoom the threshold sets. | Shots `657-10-text-fade-low`, `657-11-text-fade-high` |
| REQ-009 | WHEN a force slider changes, the system shall lay the graph out again from its current places with that force. | Shots `657-12-repel`, `657-13-link-distance`; Marley.log's run lines naming the forces |
| REQ-010 | WHEN a slider's thumb is dragged, the system shall set the value under the pointer, on the slider's step, and show it beside the slider. | Shot `657-09-thumb-drag` |
| REQ-011 | WHILE a slider has the focus, the system shall step its value with the arrow keys, ten steps with Page Up and Page Down, and to its ends with Home and End. | Shots `657-06-node-size`, `657-07-slider-keys` |
| REQ-012 | WHEN Marley starts again after a quit with a Graph tab open and Rusty on, the system shall restore the tab in its pane with its scope, centre, filter, hidden page types and panel. | Shot `657-15-restored` |
| REQ-013 | WHEN Marley starts again after a quit, the system shall keep the groups, the Display and Forces values, the four switches and the depth. | Shots `657-15-restored`, `657-16-restored-settings` |
| REQ-014 | WHILE Rusty's connection is not yet up, a restored Graph tab shall show the connection's state and read the graph once connected. | Review; the stand-in's log (the new pid's `initialize` before its `brain_graph` around orbit at depth 2) |
| REQ-015 | WHEN a Graph tab is opened after the last one closed, the system shall start it with the kept graph settings and an empty filter. | Shot `657-17-settings-kept` |
| REQ-016 | IF `marley.rusty.enabled` is off when Marley starts, THEN the system shall restore no Graph tab and make no call to Rusty. | Shot `657-18-off-not-restored`; Marley.log; the stand-in's log |
| REQ-017 | WHERE more than one workspace shows a Graph tab, a change of the graph settings in one shall show in every other. | Review |
| REQ-018 | The system shall keep the graph settings in Zed's key-value store under the `marley-rusty-graph` scope and each tab's state in `marley_rusty_graph_tabs` keyed by workspace and item, read into memory before any window restores. | Review |
| REQ-019 | IF the kept graph settings cannot be read, or a value lies outside its range, THEN the system shall use Rusty's default for it and log why. | Review |
| REQ-020 | WHERE no graph settings are kept, the system shall lay out and draw the graph as #647 does. | Review (every multiplier 1 at the defaults, D9) |
| REQ-021 | WHEN a scenario's profile is made, the e2e harness shall remove the user's graph settings from its copy of the database. | The scenario's setup check (`expect`); review |

## Phase Plan
- **P1 Plan** — promote after #647 completes; confirm what #647 shipped against the names used here
  (`Layout`'s constants and heat, `Filters`' fields, the batch loop's `pending`, the panel's
  sections, the run log line, the tab's workspace handle) and what #643 shipped (`Rusty`'s state
  names); re-verify the seams cited in the notes; ask the brain before locking D1, D5 and D9.
- **P2 Code** — the `README.md` marker first; `marley_rusty::graph_settings` and the forces in
  `graph_layout`; `rusty::slider` with Ely's notice; the tab's sections, paint, settings global
  and `SerializableItem`; `script/e2e.sh`'s scope drop; the in-app guide page; a review of the diff
  (re-entrancy in the observers, errors reaching the tab, the restore's construction, provenance
  of the port); `script/gates.sh --diff` green.
- **P3 Test** — write and run the scenario under sway with its two relaunches, read every shot.
- **P4 Complete** — CHANGELOG (Added) and architecture docs (§21), the user docs the notes list,
  ledger capture (§19), close the ticket, archive, commit.
