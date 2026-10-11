# Side panels open as tabs - Collab, Fleet, Knowledge and Containers; Debug to the left — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-748-more-side-panels-as-tabs.md
- **Pipeline spec:** 748-more-side-panels-as-tabs.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** Chad, 2026-10-11 (see #747): "everything opens in the main pane now then the user can
  just split panel it out themselves. that way its consistent". Of Debug, first "debug which we
  should move to the bottom right", then, asked whether Debug keeps its bottom panel: "yes debug
  moves to bottom left and stays the exception". Shelf:
  `docs/planning/design-notes/tabs-everywhere-2026-10-11.md`.
- **Classification / tier:** feature; Marley crate only (`marley_workbench`), no Zed hunk. Size M.
  Needs #747 shipped.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **Recall (§18.3):**
  - AD-claude-646: the Knowledge panel is always added, hides itself while Rusty is off, closes its
    dock in a deferred update, and follows the active item as the outline panel does. The last part
    is the bug a tab brings (D2), the same one #747's outline hunk fixes.
  - AD-claude-673: Containers is a right-dock panel because Chad asked for "an icon in the botton
    bar and it opens on the right"; a status item of Marley's own was rejected then as a `zed.rs`
    touchpoint. #739 later made status items from `observe_new` with no `zed.rs` edit, which D6
    reuses.
  - #607 and #611: Fleet polls only while a Fleet surface shows; the read asks the windows rather
    than count `set_active` calls (`fleet.rs` 325-329's comment), so D3 adds the tab to that read.
  - F-claude-496: Ctrl+Shift+C in a tab's field opened the collab panel; after this it opens a tab.
  - PR-claude-defer-in-does-not-leave-the-entitys-own-update-001: the tab's close on a disabled
    panel runs from a settings observer, deferred with `window.defer`.
- **Discovery:**
  - The four panels' persistent names: `CollabPanel` (`collab_panel.rs` 3998),
    `MarleyFleetPanel` (`fleet.rs` 1500), `MarleyKnowledgePanel` (`knowledge_panel.rs` 1338),
    `MarleyContainersPanel` (`rail_containers.rs` 351). Their toggles: `collab_panel::ToggleFocus`
    and `Toggle`, `marley::ToggleFleet`, `rusty::ToggleKnowledgePanel`, `marley::ToggleContainers`
    (each `toggle_panel_focus`, which #747's observer catches).
  - What each did on its dock's open and close:
    - Collab: `set_active(true)` dismisses its notification toast. Works through the tab.
    - Fleet: no `set_active`. `fleet_shows` reads the right dock's visible panel or an Agent tab;
      a Fleet tab counts as neither, so `read_providers` stops on its first call and the list never
      fills. Fix in `fleet_shows`.
    - Knowledge: `set_active` flips `shown_in_dock` and refreshes the join on every `true`; it
      follows the active item, and its own tab in front would show the project view. Fix both.
    - Containers: `set_active` starts and stops `ports::watch` only on a change. Works through the
      tab.
  - Both Knowledge and Containers hide by `enabled` and close their dock from `render`; a tab needs
    its own close (D4).
  - Debug: `DebugPanel` is a bottom-dock panel; its button comes from `DebuggerSettings.button`.
    The bottom dock's `PanelButtons` is a right status item; the left dock's is left item 0 and
    draws nothing in the Marley layout. `StatusBar::insert_item_after(0, …)` places an item right
    after it, which is the left end on screen (after the sidebar toggle when the rail is closed).
  - After this ticket the Agent Panel is the right dock's only panel in the Marley layout. It zooms
    (`PanelEvent::ZoomIn` in `agent_panel.rs`), so as a tab it would also be drawn by the zoom
    layer: D8 keeps it in the dock and asks Chad.

### Design
- **`panel_tabs.rs`** (Marley crate, #747's): the moved list gains the four names; `PanelTab`
  observes the settings and closes itself (deferred) when `Panel::enabled` is false; the restore
  refuses a disabled panel.
- **`fleet.rs`** (Marley crate): `fleet_shows` also returns true when a window's shown workspace
  has a pane whose front item is a `PanelTab` on `MarleyFleetPanel` (`panel_tabs` exposes
  `shows(workspace, name, cx)`).
- **`rusty/knowledge_panel.rs`** (Marley crate): `follow_active_item` returns early when the active
  item's focus handle is the panel's own; `set_active` returns early when `shown_in_dock` already
  matches.
- **`marley_workbench.rs`** (Marley crate): `LayoutState` keeps Zed's `debugger.button`;
  `apply_defaults` sets it false in the Marley layout.
- **`status_buttons.rs`** (Marley crate): `DebugButton`, a `Render + StatusItemView` over the
  workspace's weak handle, inserted after the left dock's `PanelButtons` from `observe_new`; it
  observes the bottom dock and the settings; draws nothing in Zed's layout or when the user's raw
  settings set `debugger.button` false; `hide_setting` returns a `HideStatusItem` writing
  `debugger.button = false`. Its click is a plain closure over the weak workspace
  (PR-claude-701-an-items-handlers-that-update-its-workspace-are-not-listeners-001).
- **File manifest:** `panel_tabs.rs`, `fleet.rs`, `rusty/knowledge_panel.rs`, `marley_workbench.rs`,
  `status_buttons.rs` (all Marley crate); `docs/marley/guide.md`, `docs/marley/walkthrough.md`,
  `docs/marley_architecture/marley_workbench.md` (Complete); `script/e2e/748-more-side-panels-as-tabs.sh`
  (new). No Zed path, so no ledger row.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 007 | Starts on the repo with the set-up in the spec; a first run (`STOP_AT_BAR=1`) finds the buttons | `bar` |
| 001, 002 | Clicks Collab, Fleet, Knowledge, Containers in turn, settling after Fleet for a poll | `collab`, `fleet`, `knowledge`, `containers` |
| 003 | Opens a brain page from the palette (`rusty: open page`), drags the Knowledge tab to the pane's right edge, clicks inside it | `knowledge-split` |
| 004 | Writes `marley.rusty.enabled` false to the profile's settings | `rusty-off` |
| 008 | Clicks the Debug button at the left | `debug` |
| 009 | Runs `marley: use zed layout` | `zed-layout` |
- REQ-005 and REQ-006 are read in the diff; REQ-004's Containers half too (the same path as
  Knowledge's, by `enabled`).

### Risks
- The Knowledge tab's skip compares focus handles; a Page tab focused inside Knowledge's tab (its
  search field) is still the panel's, so the comparison uses the item's focus handle, not the
  window's focus.
- The Debug item's place: another left item inserted at 0 later would push it right. `bar`
  checks the place on this tree.
- `debugger.button` patched as a default: a user who sets it true in their settings sees Zed's
  button at the right as well as Marley's at the left. Accepted, as `terminal.button` is.
- With Rusty on and no connection, Knowledge is enabled and its tab stays, showing its
  connection line, as the dock did.

## Phase 2 — Code
- **Built:** `panel_tabs.rs`: `MOVED` takes `CollabPanel`, `MarleyFleetPanel`,
  `MarleyKnowledgePanel` and `MarleyContainersPanel`; the settings observer closes a tab whose
  panel is disabled (and every tab on a switch to Zed's layout) through one `close_tabs`;
  `shows_panel::<P>` tells whether an item is a tab of a `P`. `fleet.rs`: `fleet_shows` counts a
  Fleet tab in front of a pane. `knowledge_panel.rs`: `follow_active_item` returns early when the
  active item is the panel's own tab (its focus handle is the panel's), and `set_active` acts only
  on a change. Collab and Containers needed nothing (D5).
- **Review:** `follow_active_item` asks the tab for its focus handle inside the panel's update,
  which #747's kept handle makes safe (PR-claude-747); a panel turned off closes its tab before
  it could draw an empty one.
- **Gate:** GATE GREEN [diff], 17 passed.

## Phase 3 — Test
- **Scenario:** `script/e2e/748-more-side-panels-as-tabs.sh`, `compositor sway`: Rusty's stand-in
  over #646's scratch pages, Fleet's pseudo provider, a fake `docker` and a stand-in
  `docker-proxy` on port 748. A first run with `STOP_AT_BAR=1` measured the buttons (Containers
  1402, Knowledge 1429, Fleet 1455, Collab 1507); the full run exits 0 with no panic.
- **Shots:**
  - `748-01-collab`: a Collab Panel tab ("Work with your team…", Sign In with GitHub), no right
    dock. REQ-001.
  - `748-02-fleet`: a Fleet tab: the pseudo provider's hosts and agents (build-1 working, review-1
    waiting, docs-1 in error). REQ-001, REQ-002.
  - `748-03-knowledge`: the Demo page tab, then the Knowledge tab in front: Demo's tags
    (#project, #area/demo, #draft), backlinks (Orbit, Demo uses Orbit) and links (Orbit, Missing
    Page). REQ-001, REQ-003.
  - `748-04-containers`: a Containers tab listing :748 docker (and the box's own published ports).
    REQ-001.
  - `748-05-rusty-off`: `marley.rusty.enabled` false: the Knowledge tab is gone; Collab, Fleet,
    Demo and Containers stay. REQ-004.
- **Reviewed, not shot:** REQ-004's Containers half (the same `close_tabs` on
  `marley.rail_containers`), REQ-005 (`set_active` acts only on a change).
- **Pre-existing, not in scope:** the Containers tab lists the box's own published ports, as #673
  noted for the panel.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md`; the guide's "Side panels as tabs"; the architecture note; the
  slice line. No Zed path is touched.
- **Knowledge appended:** none: no bug reached a run.
- **Ticket:** closed; its BACKLOG row left at promotion; #774 carries the Debug button.
- **Gate:** GATE GREEN [diff], 17 passed, on the tree committed.

### Promotion (2026-10-11, at fb109b8906)
- **Split.** The Debug button (D6, D7; REQ-007 to REQ-009) goes to TICKET-774; the restore of a
  disabled panel's tab (REQ-006) to #773, which restores panel tabs at all.
- **Seams re-checked against #747's `panel_tabs.rs`:** `MOVED` takes the four names; the settings
  observer in `init` is where D4's close goes; `PanelTab` gains a typed check for Fleet
  (`shows_panel::<P>`). `fleet.rs` `fleet_shows` 330 (the dock's visible panel or an Agent tab in
  front; `FleetPanel`'s render starts the polling, 1432); `knowledge_panel.rs`
  `follow_active_item` 335 (reads the workspace's active item; clears on a non-Page item),
  `set_active` 1342 (refreshes on every `true`), `Focusable` 1292.

## Phase 2 — Code
- **Built:** `panel_tabs.rs`: `MOVED` takes `CollabPanel`, `MarleyFleetPanel`,
  `MarleyKnowledgePanel` and `MarleyContainersPanel`; the settings observer closes a tab whose
  panel is disabled (and every tab on a switch to Zed's layout) through one `close_tabs`;
  `shows_panel::<P>` tells whether an item is a tab of a `P`. `fleet.rs`: `fleet_shows` counts a
  Fleet tab in front of a pane. `knowledge_panel.rs`: `follow_active_item` returns early when the
  active item is the panel's own tab (its focus handle is the panel's), and `set_active` acts only
  on a change. Collab and Containers needed nothing (D5).
- **Review:** `follow_active_item` asks the tab for its focus handle inside the panel's update,
  which #747's kept handle makes safe (PR-claude-747); a panel turned off closes its tab before
  it could draw an empty one.
- **Gate:** GATE GREEN [diff], 17 passed.

## Phase 3 — Test
- **Scenario:** `script/e2e/748-more-side-panels-as-tabs.sh`, `compositor sway`: Rusty's stand-in
  over #646's scratch pages, Fleet's pseudo provider, a fake `docker` and a stand-in
  `docker-proxy` on port 748. A first run with `STOP_AT_BAR=1` measured the buttons (Containers
  1402, Knowledge 1429, Fleet 1455, Collab 1507); the full run exits 0 with no panic.
- **Shots:**
  - `748-01-collab`: a Collab Panel tab ("Work with your team…", Sign In with GitHub), no right
    dock. REQ-001.
  - `748-02-fleet`: a Fleet tab: the pseudo provider's hosts and agents (build-1 working, review-1
    waiting, docs-1 in error). REQ-001, REQ-002.
  - `748-03-knowledge`: the Demo page tab, then the Knowledge tab in front: Demo's tags
    (#project, #area/demo, #draft), backlinks (Orbit, Demo uses Orbit) and links (Orbit, Missing
    Page). REQ-001, REQ-003.
  - `748-04-containers`: a Containers tab listing :748 docker (and the box's own published ports).
    REQ-001.
  - `748-05-rusty-off`: `marley.rusty.enabled` false: the Knowledge tab is gone; Collab, Fleet,
    Demo and Containers stay. REQ-004.
- **Reviewed, not shot:** REQ-004's Containers half (the same `close_tabs` on
  `marley.rail_containers`), REQ-005 (`set_active` acts only on a change).
- **Pre-existing, not in scope:** the Containers tab lists the box's own published ports, as #673
  noted for the panel.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md`; the guide's "Side panels as tabs"; the architecture note; the
  slice line. No Zed path is touched.
- **Knowledge appended:** none: no bug reached a run.
- **Ticket:** closed; its BACKLOG row left at promotion; #774 carries the Debug button.
- **Gate:** GATE GREEN [diff], 17 passed, on the tree committed.
