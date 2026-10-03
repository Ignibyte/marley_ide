# Colour groups, sliders and arrows on the Graph tab, restored after a restart — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-657-brain-graph-groups-and-forces.md
- **Pipeline spec:** 657-brain-graph-groups-and-forces.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-03)
- **Request:** Chad, 2026-10-02, the idea behind the plan: "an all in one system for Marley which
  brings in obsidian like knowledge graphs" (`docs/marley/rusty-in-marley.md`, What Chad asked
  for), then "maybe rusty becomes Marley" and "Plan it now". For the Rusty batches: "lets make a
  plan to begin the work and spec out the tickets" and "lets make sure we use the gpui components
  we found here", confirmed as "Queue all five". This is slice R5b of the plan's slices table ("The
  graph's colour groups, display and force sliders (Ely's slider), arrows, the tab restored after
  a restart"), which #647's draft split out (its Out: "R5b, the second slice: graph settings,
  kept"). It comes after #647.
- **Classification / tier:** feature, M. The plan sized R5b S; the restore (a table, a store and
  an item kind) and the slider port make it M. One new pure module in `marley_rusty`
  (`graph_settings`) and additions to #647's `graph_layout`; one new view module
  (`marley_workbench::rusty::slider`) and additions to #647's `graph_tab`; one line in
  `rusty::init`; a short pass in `script/e2e.sh`. No new dependency (`serde` and `serde_json` are
  in `marley_rusty` since #643, `db` in `marley_workbench` since #494), no spawn site, no socket,
  no Zed touchpoint. One slice: the restore keeps what the panel sets, so the two ship together.
- **Recall (§18.3):**
  - AD-claude-494-browser-tabs-reattach-or-reopen-001 and #494's notes: a Marley center tab saved
    as a Zed serializable item whose layout entry is the item alone, its state in a table of its
    own (`MarleyBrowserTabsDb`); Zed saves an item on `added_to_workspace` and on its events,
    throttled, and runs `cleanup` with the ids it loaded.
  - L-claude-494-zed-item-ids-change-at-each-launch-001 and
    F-claude-576-the-browser-tabs-table-kept-a-unique-item-id-001: item ids are entity ids that
    repeat across launches, so the table is keyed by workspace and item with no `UNIQUE(item_id)`
    (D3). Zed's own `git_graphs` table still carries the constraint (`git_graph.rs:4396`); it is
    not copied.
  - PR-claude-a-row-a-restore-reads-is-read-before-any-cleanup-001,
    F-claude-575-the-terminal-panels-cleanup-deleted-the-center-terminals-rows-001 and
    AD-claude-575-a-restored-terminal-keeps-its-id-through-marleys-table-001: the rows a restore
    reads come from memory read at `init`, plus each save since, never from the table at
    `deserialize` (D3); `terminal_ids.rs` is the pattern.
  - AD-claude-601-groups-are-restored-from-two-stores-001: Marley's own app state in Zed's
    key-value store (`marley-groups`), read once at startup before Zed restores a window; the
    precedent for the record's store and its read at `init` (D1, D2).
  - PR-claude-restore-is-a-second-constructor-001: a restored tab must get every subscription and
    seed `rusty: open graph`'s tab gets (the settings observer, #643's `Rusty` observer, the
    active-item follower, the group editors); D4 sends both through one constructor.
  - PR-claude-persist-verify-trigger-not-just-codec-001: a restore is proven by a real quit and
    relaunch and by checking the save runs on each mutation path (a group edit, a swatch, each
    slider, each switch, the depth, the filter, a legend toggle, a fold); the scenario relaunches
    twice and the review walks every path.
  - AD-claude-609-the-agent-tab-is-read-only-and-its-samples-live-in-memory-001: #647 followed it
    and left the restore to this slice ("restoring the Graph tab waits for R5b").
  - L-claude-601-a-launch-with-a-path-restores-no-session-001: the scenario clears `OPEN` before
    each relaunch, or Zed opens the path instead of the session.
  - F-claude-494-the-relaunched-marley-got-no-keymap-001: `launch_marley` starts a keyboard holder
    before each launch under sway; nothing to add.
  - L-claude-507-a-window-restores-only-its-active-workspace-001: a Graph tab in a window's other
    workspace deserializes when that workspace opens; nothing may assume every tab exists after a
    launch (Risks).
  - L-claude-507-subscribe-self-calls-back-inside-the-entitys-update-001: observers run inside
    updates; a tab never updates the settings global from inside its own observer of it (Risks).
  - L-claude-613-zeds-added-to-workspace-is-the-hook-for-an-item-that-changes-workspace-001: a
    tab that keeps a workspace handle re-points it in `added_to_workspace` (D3).
  - L-claude-607-a-settings-edit-after-marleys-own-write-does-not-reload-001: one reason the
    record is not in `settings.json` (D1); and since nothing here writes `settings.json`, the
    scenario's outside edit turning Rusty off still reloads.
  - L-claude-633-scenarios-copy-the-users-settings-so-defaults-reach-real-services-001 and
    `script/e2e.sh:643` (`cp -r "$data/db" "$E2E_PROFILE/db"`): the run's profile copies the
    user's database, so the user's own record would reach every run (D13).
  - PR-claude-drag-flag-needs-buttonless-move-selfheal-001: the slider keeps no drag flag of its
    own; gpui's `on_drag` and `on_drag_move` own the drag, as Ely's does.
  - L-claude-482-background-work-in-a-marley-crate-is-a-lazy-future-001 and
    L-claude-507-sync-work-for-the-background-executor-goes-in-future-lazy-001: the settings
    write's wait is an executor timer and the write an async db call; no `async` block without an
    await.
  - L-claude-465-a-doc-opens-with-one-short-line-001 and
    L-claude-504-rustdoc-checks-what-clippy-and-dylint-do-not-001: two new modules with docs.
  - L-claude-504-a-click-that-passes-can-still-miss-its-target-001: the panel's controls are
    measured from the first run's shots, and a slider is set exactly with Home or End after a
    click rather than by a click alone.
  - #609's notes and #647's D10: no `as` casts in the Marley crates (the pedantic cast lints);
    the slider's values are f32 throughout.
  - #647's queued spec and notes (2026-10-03), whose names this ticket builds on: `GraphView`,
    `Scope::{Local, Vault}`, `Filters { query, hidden_types, tags, decision_edges, orphans }`,
    `Shown`, `Layout` lent a batch at a time with `pending` changes, the 300-step cap, the
    200-label cap, the u16 path split, the run log line, D9's stale rule.
  - Rusty's completed pipeline `graph-views` (`/srv/stacks/rusty-v3/docs/planning/pipeline/
    completed/graph-views.notes.md:34`, `:71-72`): its panel (Filters, Groups with palette
    swatches, Display, Forces) shipped with the graph in TICKET-004, "settings persisted through
    the window's state file"; Marley split them, so the defaults must reproduce #647 exactly (D9).
  - Brain (`rusty-cli brain search` and `brain read`, read only): `projects/rusty-v3` records
    TICKET-004's graph views with "Obsidian's panel"; `projects/brain-roadmap-hermes-obsidian-
    research` lists the graph view's global and local forms and its filters;
    `research/ui-and-design/proui` (a paid GPUI kit with sliders whose licence keeps its source
    out of public repositories). Promotion asks the brain (`brain ask`) before locking D1, D5
    and D9.
- **Discovery:**
  - Rusty's app (`/srv/stacks/rusty-v3/crates/rusty-app/qml/GraphView.qml`, MIT): the settings'
    properties `:31` (groups), `:33-36` (display), `:38-41` (forces); `applySettings` `:90-100`
    and `persist` `:101-106` (the list D1 keeps); `matches` `:156-168` (the grammar #647 ports);
    `colour` `:169-174` (first matching group); `restart` `:188`; `tick` `:189-240` (repulsion
    `repelForce × 220 / d²` `:193`, springs `linkForce × (d − linkDistance × 0.6) × 0.05` `:214`,
    the pull `centerForce × 0.02` `:223-224`, cooling ×0.985 `:234`); `radiusOf` `:244` (× node
    size); the edge width `:308` (link thickness / zoom); arrows `:314-326` (a triangle 2s long and
    2s wide, s = 5 px, tip at the target's radius plus 2 px); the label fade `:348`; the palette
    `:62` (accent, red, green, yellow, magenta, cyan, blue); Groups `:503-537` (swatch "Next
    colour" `:516-523`, the query field with its placeholder `:525`, "Remove group" `:526`, "+ New
    group" `:529-536`); Display `:538-548`; Forces `:549-559`; `SettingSlider` `:595-606`.
    `Main.qml:251-252` (one `graph` record in the window state), `:113`, `:128` (saved with the
    window), `:466-473` (`saveTabs`: kind, title, slug per tab), `:927-930` (both graph tabs get
    the same settings and write them back).
  - Rusty's engine: `rusty-core/src/brain/mod.rs:943-999` (`push_edge` keyed by the directed pair,
    so A to B and B to A are two edges; links from the linking page, unresolved targets as
    `new:` nodes, tag edges from the page to `tag:`), `:1000-1044` (an unknown `around` keeps
    nothing); `decisions.rs:436-459` (typed edges from the decision page).
  - Obsidian, observed on this box: `~/.rusty/brain/.obsidian/graph.json`'s keys, printed with
    their types only (`collapse-filter`, `search`, `showTags`, `showAttachments`,
    `hideUnresolved`, `showOrphans`, `collapse-color-groups`, `colorGroups`, `collapse-display`,
    `showArrow`, `textFadeMultiplier`, `nodeSizeMultiplier`, `lineSizeMultiplier`,
    `collapse-forces`, `centerStrength`, `repelStrength`, `linkStrength`, `linkDistance`, `scale`,
    `close`); no value read or kept.
  - Ely (HEAD 2f8b2f6, MIT OR Apache-2.0, `LICENSE-MIT`: "Copyright (c) 2026 Ely GPUI Component
    contributors"): `src/forms/slider.rs:17-25` (`fraction`, `value_at`), `:28-39` (`keyed`),
    `:42-48` (`along`), `:51-54` (`Thumb`), `:60-280` (`Track`), `:74-79` (three keyed states),
    `:161-166` (the role and aria values), `:192-198`, `:237-253` (the drags), `:254-266` (the
    press), `:221-232` (the measuring canvas), `:286-360` (`Slider`), `:308`, `:315` (`assert!`),
    `:344` (`log::debug!`), `:363-445` (`RangeSlider`), `:447-466` (tests);
    `src/forms/options.rs:75` (`OnNumber`); `src/primitives/focus.rs:214-225` (`tab_stop`);
    `src/theme/tokens.rs:420-426` (track 4 px, thumb 16 px); `src/forms/color/swatch.rs:68`,
    `:152`; `src/charts/network.rs:255`, `:364`, `paint.rs:54-56`.
  - Zed: `crates/workspace/src/item.rs:409-437` (`SerializableItem`), `:755-765`, `:872-876`;
    `workspace.rs:178`, `:1357`, `:7650-7680`, `:7892-7927`, `:7929-7936`;
    `persistence.rs:2777` (`delete_unloaded_items`); `persistence/model.rs:355-396` (each saved
    item deserialized; a failure `log_err`'d and skipped). `crates/git_ui/src/git_graph.rs:1051`
    (registration), `:4189-4371`, `:4373-4421`. `crates/db/src/kvp.rs:20-41` (the store's two
    tables), `:89-130` (`scoped`, `read`, `write`, `delete`). `crates/gpui/src/window.rs:584`,
    `:4164`; `elements/div.rs:67-80`, `:360`, `:1042`, `:1406-1450`, `:1614`; `gpui.rs:89-91`;
    `view.rs:413`; `path_builder.rs:88-123`. `crates/settings_ui/src/components/
    number_field.rs:258-330`. `crates/theme/src/styles/colors.rs:281-311`, `accents.rs:24-40`,
    `:65-67`. `crates/icons/src/icons.rs:74` (`Close`), `:213` (`Plus`).
  - Marley: `crates/marley_workbench/src/browser.rs:7002-7072` (`impl SerializableItem for
    BrowserView`), `:7074-7160` (`persistence`), `:7580`; `terminal_ids.rs:1-47`;
    `terminal_size.rs:8-35`; `shortcut_note.rs:11`, `:51-57`; `Cargo.toml:23` (`db`), `:62-63`
    (`serde`, `serde_json`).
  - The e2e runner: `script/e2e.sh:25-45` (the steps, `quit_marley` and `launch_marley`),
    `:297-319` (`launch_marley`, a keyboard holder first under sway), `:321-338` (`quit_marley`
    through the palette), `:619-646` (the profile: the settings copy, then the database copy at
    `:643`); `script/e2e/494-browser-restore.sh:109-120` and
    `601-projectless-groups-survive-a-restart.sh:93-97` (relaunches, `open_path ""`).
- **Decisions:** D1 to D13 in the spec. In short: the graph settings one record in Zed's
  key-value store and the tab's own state its row; the record live across tabs, read at `init`,
  written 300 ms after the last change and at quit; the table keyed by workspace and item and read
  into memory; Rusty off means no restored tab, and a restored tab waits for the connection;
  groups first, then page types; six terminal hues; Display and Forces as Rusty's, the forces as
  multipliers that leave #647's picture at the defaults; arrows on directed edges only; Ely's
  slider ported in f32; Rusty's ranges with steps on the defaults; Obsidian's section order; the
  harness drops the user's record.

### Design
- **`marley_rusty::graph_settings`** (new, pure, no Ely code):
  - `GraphSettings { tags, unresolved, decision_edges, orphans, depth: u8, groups: Vec<Group>,
    display: Display, forces: Forces }` with `Serialize`, `Deserialize`, `#[serde(default)]` and
    `Default` (D11). `Display { arrows, text_fade, node_size, link_thickness }`.
  - `SliderRange { min, max, step, default }` and the seven consts of D11; `SliderRange::clamp`
    (a non-finite value takes the default, any other is held to the range and snapped to the
    step).
  - `GraphSettings::from_stored(&str) -> (Self, Vec<String>)`: the settings and a line for each
    field that fell back (D2), for the log; `to_stored(&self) -> String`. A depth outside 1 to 4
    is held to it; an unknown hue name takes the hue at the group's place.
  - `Group { query: String, color: GroupColor }`; `GroupColor::{Red, Green, Yellow, Magenta, Cyan,
    Blue}`, serialized lowercase, with `ALL` in that order, `for_place(index)` and `next()` (D6).
  - `group_colors(nodes: &[Node], groups: &[Group]) -> GroupColoring { per_node:
    Vec<Option<usize>>, counts: Vec<usize> }`: each query parsed once with #647's `Query::parse`,
    an empty one skipped; a node's first matching group; how many nodes each group colours (D5).
  - `Display::label_alpha(zoom) -> f32`: `((zoom - (2.2 - 1.6 * text_fade)) / 0.6)` held to 0 to
    1, written with `mul_add` where `suboptimal_flops` asks (D7).
- **`marley_rusty::graph_layout`** (#647's, additions under its Ely notice):
  - `Forces { center, repel, link, distance }` (`Copy`, `Serialize`, `Deserialize`, `Default`
    Rusty's), `ideal_length()` = 0.6 × distance, and the three multipliers (D9).
  - `Layout` takes `Forces` at its seeding; `set_forces(forces)` stores them, puts the heat back
    to its starting value and the step count to zero, places and pins kept; `step` uses k from the
    forces and the multipliers.
  - `arrowhead(from, to, target_radius) -> Option<[Point; 3]>` in view pixels: the tip on the
    target's rim plus 2 px, the base 10 px back, 5 px each side; `None` for two points closer than
    the radius (D8).
- **`marley_workbench::rusty::slider`** (new, Ely's MIT notice at the top):
  - `Slider` (`#[derive(IntoElement)]`, `RenderOnce`): `new(id, value: f32)`, `range(min, max)`,
    `step(step)`, `label(text)` (the aria label), `on_change(Fn(f32, &mut Window, &mut App))`.
  - Private `fraction`, `value_at`, `keyed` and `along` (Ely's, in f32, with no clamp-free path);
    a change of less than half a step is dropped.
  - Render: three `use_keyed_state`s under the id (the focus handle with `tab_stop(true)`, the
    drag's owner, the track's measured bounds); the rail and fill as absolute `div`s; the thumb
    with `Role::Slider`, the aria value, minimum, maximum, step, orientation and label,
    `track_focus`, `on_drag(Thumb { owner }, |..| cx.new(|_| EmptyView))` and `on_key_down`
    (`keyed`); the track with `on_drag_move::<Thumb>` (owner checked), `on_mouse_down` (jump and
    focus) and a `canvas` that records its bounds. Colours of D10.
- **`marley_workbench::rusty::graph_tab`** (#647's, additions):
  - `GraphSettingsStore` (`Global`): the `GraphSettings`, the pending write's `Task<()>`.
    `update(cx, |settings| ..)` applies a change, notifies the global's observers and replaces
    the write task: an executor timer of 300 ms, then
    `KeyValueStore::global(cx).scoped("marley-rusty-graph").write("settings", json)`, errors
    logged. `flush(cx)` writes now; `on_app_quit` calls it.
  - `GraphView` gains `group_editors: Vec<(Entity<Editor>, Subscription)>`, `coloring:
    GroupColoring`, `open_sections` (Filters, Groups, Display, Forces, Legend), the panel's
    `ScrollHandle`; its `depth` and the four switches move into the record. One constructor,
    `GraphView::build(saved: Option<SavedGraphTab>, workspace, window, cx)`, behind both
    `rusty: open graph` and `deserialize` (D4).
  - The observer of `GraphSettingsStore`: compares the old and new record and does only what
    changed: the group editors rebuilt when the count changes, an unfocused editor's text set when
    its query differs (an equal text is skipped, so no edit event writes back); `coloring`
    recomputed for a group change; `shown` again for Tags, Decision edges or Orphans; a read for
    Unresolved links or the depth (or `stale` when hidden, #647's D9); `set_forces` (through
    `pending` while a batch is out) for a force; a repaint for Display.
  - The observer of #643's `Rusty` global, added here if #647's tab lacks one: while the state is
    not connected the notice line shows it and no read starts; on connected, the pending read runs
    (D4).
  - Paint (#647's canvas): a node's colour is its group's hue (`terminal_ansi_<name>`) when
    `coloring` gives one, else #647's; the radius × node size, in the paint and in `nearest`'s
    radii; every stroke × link thickness; labels with `label_alpha`; with Arrows on, one
    `PathBuilder::fill()` per edge style for the heads of the edges that have a direction, counted
    into #647's split at three vertices a head, a head skipped when its target is culled.
  - The panel (D12): `Disclosure` sections in Obsidian's order, the new three folded on a new
    tab; the scroll container. Groups: a row per group (a 12 px round swatch with an id, `on_click`
    to `next()`, `Tooltip::text("Next colour")`; the single-line `Editor` with Rusty's
    placeholder, applied on each `BufferEdited`; the count as a muted `Label`; an `IconButton`
    with `IconName::Close` and the tooltip "Remove group"), then `Button` "New group" with
    `IconName::Plus` (the new group takes `GroupColor::for_place(count)` and the focus goes to its
    field). Display: a `Checkbox` "Arrows" and three slider rows. Forces: four slider rows. A row:
    the label (Rusty's word, muted, a fixed width), the `Slider` taking the rest, the value as a
    muted `Label` (two decimals for a step of 0.05, one for 0.1 and 0.5, none for 10).
  - `impl SerializableItem for GraphView`: `serialized_item_kind` "MarleyRustyGraph"; `cleanup`
    through `workspace::delete_unloaded_items(alive, workspace_id, "marley_rusty_graph_tabs",
    ..)`, dropping the same rows from the memory copy; `deserialize` refusing while Rusty is off
    (D4), then the memory copy's `SavedGraphTab` (none: "no Graph tab was saved for the item"),
    then `window.spawn` and `GraphView::build`; `serialize` (`workspace.database_id()?`, the state
    as JSON, the memory copy updated, the row saved in `background_spawn`); `should_serialize` on
    `ItemEvent::UpdateTab`. The tab emits `UpdateTab` on each change of its own state (scope,
    centre, filter, a legend toggle, the panel, a fold). `added_to_workspace` re-points the
    workspace handle (L-613).
  - `SavedGraphTab { scope, centre: Option<SavedCentre { slug, title }>, filter, hidden_types,
    panel_open, open_sections }`, `Serialize` and `Deserialize` with defaults, in the module's
    `persistence` beside `MarleyRustyGraphTabsDb`: one migration creating
    `marley_rusty_graph_tabs(workspace_id INTEGER, item_id INTEGER, state TEXT NOT NULL, PRIMARY
    KEY(workspace_id, item_id), FOREIGN KEY(workspace_id) REFERENCES workspaces(workspace_id) ON
    DELETE CASCADE) STRICT`; `db::static_connection!(MarleyRustyGraphTabsDb, [WorkspaceDb])`;
    `save_tab` (`INSERT OR REPLACE`) and `all_tabs` through `query!`.
  - `init(cx)`: `all_tabs` into a `SavedGraphTabs` global; the record read and parsed into
    `GraphSettingsStore`, each fallback logged; `workspace::register_serializable_item::<
    GraphView>(cx)`; the quit flush.
  - Log lines (from this module, L-489): #647's run line gains "forces center 0.50 repel 20.0
    link 1.00 distance 30"; the restore's refusal; each fallback of the record.
- **`marley_workbench::rusty`** (`rusty.rs`): `pub(crate) mod slider;`; `graph_tab::init(cx)`
  from `rusty::init`, whatever the switch says, since Zed must know the item kind to refuse it
  while Rusty is off.
- **`script/e2e.sh`**: after the database copy (`:643`), a `python3` pass with `sqlite3` over
  each `$E2E_PROFILE/db/*/db.sqlite` that holds `scoped_kv_store`, deleting the
  `marley-rusty-graph` rows, with a comment giving D13's reason.
- **Performance.** Group colouring is one pass over the shown nodes per group and query change
  or read (2,000 nodes and a handful of groups), not per frame. Heads add three vertices an edge:
  about 4,200 heads with Tags on the real vault's size, a few fill paths after the split. Text fade
  at 1 starts labels at zoom 0.6, and #647's cap of 200 labels a frame holds the cost. A force
  slider's drag restarts the run on each move, and #647's one batch out at a time folds the moves
  into the next batch; the record is written once the drag has been still 300 ms. The tab's row is
  written by Zed at most every 200 ms. `init` reads one record and a row per saved Graph tab.
- **File manifest.**
  - Marley: `crates/marley_rusty/src/graph_settings.rs` (new), `crates/marley_rusty/src/
    graph_layout.rs` (forces and heads), `crates/marley_rusty/src/marley_rusty.rs` (the module);
    `crates/marley_workbench/src/rusty/slider.rs` (new, Ely's notice),
    `crates/marley_workbench/src/rusty/graph_tab.rs`, `crates/marley_workbench/src/rusty.rs`;
    `crates/marley_workbench/guide/index.html` (the Graph article); `script/e2e.sh`;
    `script/e2e/657-brain-graph-groups-and-forces.sh` (new, Test phase).
  - Zed: none. `workspace`, `db`, `gpui`, `ui`, `theme`, `editor` and `settings_ui` are used as
    they are; no manifest or lockfile changes.
- **The ledger rows it extends:** none. Every path is Marley-owned (`crates/marley_*`,
  `script/e2e.sh`, `script/e2e/*`, `marley_owned_path` in `.claude/hooks/lib-hook-helpers.sh`), so
  `docs/marley/zed-touchpoints.md` is not touched.

### Visual check plan
The scenario `script/e2e/657-brain-graph-groups-and-forces.sh`, `compositor sway`, three launches.
Setup: a scratch repository (`open_path`); #643's stand-in named in `MARLEY_RUSTY_MCP` over
#647's graph vault, its call log in `$E2E_WORK`; `profile_setting marley.rusty '{"enabled": true,
"connection": "embedded"}'`; an `expect` that the copied database holds no `marley-rusty-graph`
row (REQ-021). Each step that moves the layout waits for #647's "settled" line in Marley.log
before its shot. The panel's section headers, rows and slider tracks are measured from the first
run's shots (L-504); the layout has no randomness, so they hold.

| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | `rusty: open graph` (Vault); open Groups; New group; type `path:research/` | `657-01-group`: the research pages red, the row's count |
| REQ-002 | New group; type `type:research` | `657-02-first-group-wins`: still red; the second row 0 |
| REQ-003 | Remove the first group | `657-03-group-removed`: the research pages green, the row's count |
| REQ-004 | Click the group's swatch | `657-04-next-colour`: yellow |
| REQ-005 | Open Display; Arrows on; Tags on (Filters); three wheel steps over the middle | `657-05-arrows`: heads at the links' targets, the tag edges bare |
| REQ-006, REQ-011 | Click Node size's track; End | `657-06-node-size`: "3.0", larger nodes |
| REQ-011 | Left twice | `657-07-slider-keys`: "2.8" |
| REQ-007 | Click Link thickness's track; End | `657-08-link-thickness`: "4.0", thicker edges |
| REQ-010 | `pointer_down` on that thumb, `pointer_to` the track's middle, `pointer_up` | `657-09-thumb-drag`: about "2.1" (the value under the pointer), thinner edges than `657-08` |
| REQ-008 | Fit; Text fade threshold's track, Home; then End | `657-10-text-fade-low`: no labels; `657-11-text-fade-high`: labels at the same zoom |
| REQ-009 | Open Forces; Repel force, End | `657-12-repel`: wider than `657-11`; the run line with "repel 20.0" |
| REQ-009 | Link distance, Home | `657-13-link-distance`: tighter; the run line with "distance 30" |
| REQ-012 (before) | Filter `path:projects/orbit`, Fit, click the centre, `rusty: open local graph`; depth 2; filter `path:research/`; the legend's `note` off | `657-14-before-restart` |
| REQ-012, REQ-013, REQ-014 | `quit_marley`; `open_path ""`; `launch_marley`; wait for the new pid's `brain_graph` in the stand-in's log | `657-15-restored`: the tab, "Local graph · Orbit · depth 2", the filter, `note` faint, yellow research pages, arrows; the log's `initialize` before `brain_graph` with `around` and `depth: 2` |
| REQ-013 | Scroll the panel through Groups, Display and Forces | `657-16-restored-settings`: `type:research` in yellow, Arrows on, 2.8, the dragged thickness, 1.00, 20.0, 30 |
| REQ-015 | Ctrl+W on the Graph tab; `rusty: open graph` | `657-17-settings-kept`: Vault, the yellow research pages, the arrows and values, the filter empty, `note` shown |
| REQ-016 | `set_setting marley.rusty.enabled false`; settle; `quit_marley`; `launch_marley` | `657-18-off-not-restored`: no Graph tab; Marley.log's "Rusty is off; the Graph tab is not restored"; no new pid in the stand-in's log |
| REQ-017 to REQ-020 | Not shot | Review: the observers (REQ-017), the store's scope and the table read at `init` (REQ-018), `from_stored`'s fallbacks (REQ-019), the multipliers at the defaults (REQ-020) |

Not reached by a scenario: a second workspace's Graph tab (REQ-017; the run has one workspace, and
the observer is the same code path the run drives); a record that does not parse (REQ-019; D13's
drop leaves the run on the defaults, and planting a bad record would cost a fourth launch for one
fallback line); the connecting notice itself, which lasts until the stand-in's `initialize`
returns, under a second, so the log's order carries REQ-014.

### Risks
- **#647 is queued, not built,** and #643 to #646 under it. The names come from their queued
  specs of 2026-10-03; promotion re-reads what shipped (P1) and adjusts the manifest without
  changing the behaviour. If #647's `Layout` has no starting heat to return to, `set_forces`
  warms it as #647's reads do.
- **The forces are multipliers on a different model.** Rusty's Repel force of 20 in the Qt app
  and Marley's do not give the same picture; they push the same way. The guide page says so.
- **Live sharing and re-entrancy.** A tab updates the global from its own listeners, never from
  inside its observer of the global; setting a group editor's text emits an edit, so an equal
  text is skipped and an edit that changes nothing writes nothing (L-507).
- **A slider drag restarts the layout on every move.** The run folds them (#647's D5); if a drag
  still stutters on the debug build, the force change applies on the thumb's release instead,
  recorded as a deviation.
- **Group hues can match a page type's accent.** The group's row names its count and the legend
  its types; the swatch cycles to another hue. Rejected alternative in D5.
- **The restore and the connection.** A restored tab can exist before #643's client is up; it
  must not call through a client that is still starting (D4). A Graph tab in a window's other
  workspace comes back only when that workspace opens (L-507).
- **#655 (R6) touches the same centre.** Its project centre applies while the tab has no focused
  page; a restored tab's saved centre counts as one (spec, Out). Promotion checks which landed
  first and whether the saved state needs #655's flag.
- **Item ids repeat across launches** (L-494, F-576): the table has no `UNIQUE(item_id)`; the
  review checks the migration against #576's.
- **The harness's relaunches:** `OPEN` cleared before each (L-601); the keyboard holder is the
  runner's (F-494). The third launch turns Rusty off by an outside edit; Marley writes nothing to
  `settings.json` in this run, so the edit reloads (L-607).
- **Node size 3 and text fade 1** crowd a fitted Vault: nodes overlap and many labels compete;
  #647's nearest-node hit test and its label cap hold. The shots read the values and the size,
  not each label.
- **The user's database in the run** (D13): if the pass finds no `scoped_kv_store` (a fresh
  profile), it does nothing; the setup's `expect` reads the copy either way.
- **The receipt.** The in-app guide page, `script/e2e.sh` and the scenario are fingerprinted; a
  change after the Code phase's green needs `--diff` again before the commit.
- **Public origin.** No fixture changes; the user's record never reaches a run or a shot (D13);
  Obsidian's key names above are Obsidian's, not the user's values.

### User-facing docs to update at Complete (P4)
- `docs/marley/guide.md`: the Graph tab's part (#647's) gains Groups (the grammar, first match,
  the hues), Display, Forces (the ranges and defaults, and that they are not the Qt app's
  numbers), what a restart keeps, and that Rusty off restores no Graph tab.
- `docs/marley/walkthrough.md`: a stop after #647's, with the scenario's checks.
- `crates/marley_workbench/guide/index.html`: changed in the Code phase (the receipt binds it).
- Architecture (§21): `docs/marley_architecture/marley_rusty.md` (`graph_settings`, `Forces`)
  and `marley_workbench.md` (the slider and its notice, the Graph tab's store and table);
  `docs/marley/rusty-in-marley.md`'s slices table (R5b done); `docs/marley/three-prong-plan.md`'s
  Rusty line if it tracks R slices; `CHANGELOG.md` under Added. `zed-touchpoints.md` is not
  touched.

### Checklist (no TaskCreate in this harness)
- [x] Read CONSTITUTION §3, §7, §14, §18, §19, §20, the three templates, and #633's spec for the
      shape.
- [x] Read `docs/marley/rusty-in-marley.md` whole (R-D0 to R-D10, the slices table, Rusty's
      triage) and #647's queued spec, notes and ticket, whose Out names this slice; #643's spec and
      design for the client and its `Rusty` global.
- [x] Read `GraphView.qml` whole and its wiring in `Main.qml` (the shared settings record and
      `saveTabs`); `brain_graph`'s edges in `rusty-core`.
- [x] Read Ely's `src/forms/slider.rs` whole, its `tab_stop`, `OnNumber` and size tokens, the
      colour swatch and palette, and `network.rs`'s group colouring.
- [x] Checked Zed for a slider (`ui`, `settings_ui`, `gpui`: none; `NumberField` a stepper) and
      Cargo.lock for a GPUI component crate (none); read `SerializableItem`, its registration,
      throttle, flush and failure path, `git_graph`'s and `browser.rs`'s items and tables, and the
      key-value store.
- [x] Recall: the ledgers (AD-494, L-494, F-576, PR a-row-a-restore-reads, F-575, AD-575,
      AD-601, PR restore-is-a-second-constructor, PR persist-verify-trigger, AD-609, L-601, F-494,
      L-507 twice, L-613, L-607, L-633, PR drag-flag, L-482, L-507, L-465, L-504 twice), the
      completed pipelines 494, 575, 576, 601 and 609, Rusty's `graph-views`, and a read-only brain
      search; Obsidian's `graph.json` keys read for their names only.
- [x] Discovery with file:line across Rusty, Ely, Zed, Marley and the e2e runner.
- [x] Prior-art sweep, three legs plus Ely and Rusty, written into the spec.
- [x] Decisions D1 to D13 with reasons; the performance case.
- [x] Spec: scope (with the reset, local settings, per-vault settings, reordering, a picker, the
      viewport and R6 in Out), Reference (§20), Prior art, UI proof, twenty-one EARS rows, phase
      plan.
- [x] Design: two new modules and the additions, the file manifest by crate, no touchpoint row,
      the visual check plan, risks, the user docs for Complete.
- [x] Docs only: the ticket doc and this pair; no BACKLOG.md, no `active/`, no source, no cargo.

### Reconciliation, 2026-10-03
- A `rusty:` action run while Rusty is off or not connected shows a toast saying so and where to
  turn it on, and opens nothing (rusty-in-marley.md R-D0, settled across #643 to #659).
- Every scenario names its stand-in in `MARLEY_RUSTY_MCP`, never first on the PATH (#643).
