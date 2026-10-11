---
pipeline_id: f0b56f32-0311-431b-948d-ebd6262c057a
ticket: docs/planning/tickets/closed/TICKET-748-more-side-panels-as-tabs.md
status: Phase 4 — Complete PASS
title: Side panels open as tabs - Collab, Fleet, Knowledge and Containers; Debug to the left
type: feature
slice: the Marley layout (docs/marley/workbench-shell.md), tabs everywhere (docs/planning/design-notes/tabs-everywhere-2026-10-11.md)
references:
  - docs/planning/pipeline/queued/747-side-panels-as-tabs.spec.md
  - docs/planning/pipeline/completed/607-fleet-panel-with-pseudo-agents.spec.md
  - docs/planning/pipeline/completed/646-knowledge-panel.spec.md
  - docs/planning/pipeline/completed/673-containers-panel-on-the-right.spec.md
---

## Title
The rest of the right dock's panels take #747's tab: the collab panel, Fleet, Knowledge and
Containers. Each is fixed where it relied on the dock: Fleet stops reading once no dock shows it,
and Knowledge follows the active item, which would be its own tab. The debug panel stays in the
bottom dock and its button moves to the left end of the status bar. Chad, 2026-10-11: "everything
opens in the main pane now then the user can just split panel it out themselves. that way its
consistent", and of Debug: "yes debug moves to bottom left and stays the exception".

## Scope
### In
- `CollabPanel`, `MarleyFleetPanel`, `MarleyKnowledgePanel` and `MarleyContainersPanel` join #747's
  moved panels: their buttons, keys (`collab_panel::ToggleFocus`, `marley: toggle fleet`,
  `rusty: toggle knowledge panel`, `marley: toggle containers`) and Zed's calls open or bring
  forward their tabs, and their dock never shows them in the Marley layout.
- Fleet keeps reading its providers while its tab is a pane's front item in a shown group.
- Knowledge keeps following the active item while its own tab is the active item, and acts on
  `set_active` only when it changes.
- A panel tab closes when its panel turns disabled: Knowledge while Rusty is off, Containers while
  `marley.rail_containers` is off. A disabled panel's saved tab is not restored.
- `docs/marley/guide.md` and `docs/marley/walkthrough.md`: the four panels open as tabs; the Debug
  button at the left.

### Out (explicitly deferred)
- The Debug button at the status bar's left: #774 (D6, D7), split off at promotion.
- A saved tab of a disabled panel after a restart: #773, which restores panel tabs at all.
- The Agent Panel. It stays in the right dock, the one panel left there in the Marley layout: #744
  gives its button the New Agent picker, and whatever else opens the panel (its palette actions,
  Zed's agent settings and history) still opens the dock (D8). A tab for it waits on Chad.
- The Debug panel as a tab: Chad kept it in the bottom dock.
- The terminal panel: its button stays hidden and its toggles stay routed to center terminals
  (`routing.rs`).

## Reference (§20)
Upstream Zed, `workspace`: `Dock`, `Panel` and `PanelButtons`, and `StatusBar` with
`StatusItemView`. Behavior kept: each panel's own view and actions; the bottom dock and the Debug
panel as Zed has them; a status item's right-click Hide Button (`HideStatusItem`). The tab is
#747's.

### Prior art
- **The code we ship:**
  - #747's tab, its moved list by persistent name, its dock observer and its `set_active` rule
    (the queued spec).
  - `CollabPanel` (`collab_ui/src/collab_panel.rs` 3934-4012): `set_active(true)` dismisses its
    notification toast (3969-3982); its toast's action calls `focus_panel` (4194-4203); its
    `ToggleFocus` also selects the filter's text (106-118); persistent name `CollabPanel`.
  - `FleetPanel` (`fleet.rs` 1498-1540, no `set_active`): `fleet_shows` asks whether a right dock
    shows the panel or a pane's front item is an Agent tab (325-352); `read_providers` stops the
    polling once nothing shows (448-452); `render` starts it again (1427-1434).
  - `KnowledgePanel` (`rusty/knowledge_panel.rs` 1337-1404): `set_active` sets `shown_in_dock` and
    refreshes the join on every `true` (1342-1352); it follows `workspace.active_item()` and shows
    the project view when that is no Page tab (335-366); `close_while_off` closes the dock when
    Rusty is off (594-614), called from `render` (1300-1302); `enabled` is `rusty::is_on`
    (1400-1403).
  - `ContainersPanel` (`rail_containers.rs` 350-411): `set_active` starts and stops the port scan
    and acts only on a change (400-410); `close_while_hidden` closes the dock while hidden
    (101-124), called from `render` (277-280); `enabled` is `shown` (395-397).
  - `DebugPanel` (`debugger_ui/src/debugger_panel.rs` 1513-1600): bottom dock; `icon` and
    `icon_tooltip` follow `DebuggerSettings.button` (1558-1570); `hide_button_setting` writes
    `debugger.button = false` (1584-1588); `set_active` does nothing (1590).
  - `StatusBar` (`workspace/src/status_bar.rs`): `position_of_item::<T>` finds the first item of a
    type (374-389), `insert_item_after` (391-410); left items render in order (190-206), right items
    reversed (208-230); `render_hideable_item` gives an item with a `hide_setting` its right-click
    Hide Button (292-313). The left dock's `PanelButtons` is the first left item (`workspace.rs`
    2041), empty in the Marley layout, whose left dock holds no panel.
  - `apply_defaults` already patches `terminal.button` per layout and keeps Zed's value in
    `LayoutState` (`marley_workbench.rs` 795-803, 1215-1231); `status_buttons.rs` (#739) is a
    Marley status item made from `observe_new`.
  - `routing.rs` `toggle_bottom_dock` passes `ToggleBottomDock` on to Zed unless the closed dock
    would show the terminal panel (272-294).
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` (the status
  bar cell, `StatusItemView`, the dock buttons). No Warp or Orca map covers these panels.
- **Published material:** none.

## UI proof
Scenario `script/e2e/748-more-side-panels-as-tabs.sh`, under `compositor sway`, in the Marley
layout: Rusty on through its stand-in `rusty-mcp` with a page; Fleet's pseudo provider
(`marley.fleet.providers: [{"kind": "pseudo"}]`, #607); #673's fake `docker` and stand-in
`docker-proxy` publishing port 748, with `marley.rail_containers` on. Collab is signed out.

Shots:
- `collab`, `fleet`, `knowledge`, `containers`: each button clicked: its tab in the repo's active
  pane, the right dock closed (REQ-001); `fleet` lists the pseudo agents (REQ-002); `containers`
  lists :748.
- `knowledge-split`: a brain page open in a Page tab, the Knowledge tab dragged to a split, then
  clicked: it still shows the page's tags and links (REQ-003).
- `rusty-off`: `marley.rusty.enabled` turned off: the Knowledge tab gone (REQ-004).

## Locked-In Decisions
- **D1 — The four join #747's moved list by persistent name** (`CollabPanel`, `MarleyFleetPanel`,
  `MarleyKnowledgePanel`, `MarleyContainersPanel`). With them the Marley layout's right dock opens
  only on the Agent Panel.
- **D2 — Knowledge follows the active item, skipping its own tab**: an active item whose focus
  handle is the panel's keeps the page it follows, as #747's outline hunk does; here it is Marley
  code. Its `set_active` acts only when the value changes, as Containers' does, since #747's
  re-tell may repeat a call and each `true` reads Rusty again.
- **D3 — Fleet counts its tab as a showing surface**: `fleet_shows` also asks whether a Fleet
  panel tab is the front item of a pane in a window's shown workspace. Rejected: driving the
  polling from `set_active` (Fleet has none, and the Agent tab already reads the windows).
- **D4 — A tab closes when its panel turns disabled.** `panel_tabs`' settings observer (#747's,
  which closes the tabs on a switch to Zed's layout) also closes every tab whose panel's
  `enabled` is false, as Zed hides a disabled panel's button; #773's restore refuses one. The panels' own `close_while_off` and `close_while_hidden`
  stay for the Zed layout's dock.
- **D5 — Collab and Containers need nothing of their own**: Collab's toast dismissal and
  Containers' port scan already follow `set_active`, which #747 drives from the tab.
- **D6 — Debug stays in the bottom dock; its button moves (#774, split off at promotion).** In the Marley layout `debugger.button`
  defaults to false, patched beside `terminal.button` with Zed's value kept for the switch back, so
  the bottom dock's buttons drop it. A Marley status item, inserted after the left dock's
  `PanelButtons` (`position_of_item::<PanelButtons>`, `insert_item_after`), draws the Debug icon
  with the tooltip "Debug Panel": toggled while the bottom dock is open on the Debug panel; a click
  sends `workspace::ToggleBottomDock` then, else `debug_panel::ToggleFocus` built by name
  (`App::build_action`, so `marley_workbench` gains no `debugger_ui` dependency). It hides when
  the user's own settings set `debugger.button` false, and its right-click Hide Button writes that,
  as Zed's does. Rejected: moving Zed's bottom `PanelButtons` to the left (the status bar finds
  items by type and the three `PanelButtons` share one, so it needs a Zed hunk); the Debug panel
  in the left dock (Chad kept the bottom dock).
- **D7 — The Debug item is Marley-layout only (#774)**; in Zed's layout it draws nothing and Zed's own
  button returns with the restored default.
- **D8 — The Agent Panel stays in the right dock.** As a tab it would also need its zoom handled:
  it emits `PanelEvent::ZoomIn` (`agent_panel.rs`), which draws it in the workspace's zoom layer, a
  second place beside a tab. Chad's call; until then the panel's other openers open the right
  dock.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the collab, Fleet, Knowledge or Containers button is clicked in the Marley layout, the system shall open that panel as a tab in the active pane of the shown group, with the right dock closed. | shots `collab`, `fleet`, `knowledge`, `containers` |
| REQ-002 | While a Fleet tab is a pane's front item in a shown group, the system shall keep reading Fleet's providers and list their agents. | shot `fleet` |
| REQ-003 | While its own tab is the active item, the Knowledge panel shall keep showing the Page tab it follows. | shot `knowledge-split` |
| REQ-004 | WHEN Rusty turns off, the system shall close the Knowledge tab; WHEN `marley.rail_containers` turns off, the system shall close the Containers tab. | shot `rusty-off`; review of the diff for Containers |
| REQ-005 | The Knowledge panel shall read Rusty again only when its `set_active` value changes. | review of the diff |
| REQ-010 | The change shall pass `script/gates.sh --diff`. | the gate's exit code |

## Phase Plan
- **P1 Plan** — this spec; at promotion the design in the notes, every cited seam checked again
  against #747 as it shipped.
- **P2 Code** — README marker present (§3); `panel_tabs.rs` (the four names, the disabled close,
  the restore refusal), `fleet.rs`, `rusty/knowledge_panel.rs`, `marley_workbench.rs`
  (`debugger.button` in `apply_defaults`), a Debug status item (in `status_buttons.rs`), the guide
  and walkthrough lines; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run `script/e2e/748-more-side-panels-as-tabs.sh`, read
  every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
