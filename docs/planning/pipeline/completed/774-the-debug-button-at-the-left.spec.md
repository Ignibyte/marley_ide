---
pipeline_id: acf96ae8-3a76-4cda-8e4b-619c11024b2c
ticket: docs/planning/tickets/closed/TICKET-774-the-debug-button-at-the-left.md
status: Phase 4 — Complete PASS
title: The Debug button at the status bar's left
type: feature
slice: the Marley layout (docs/marley/workbench-shell.md), tabs everywhere (docs/planning/design-notes/tabs-everywhere-2026-10-11.md)
references:
  - docs/planning/pipeline/completed/748-more-side-panels-as-tabs.spec.md
  - docs/planning/pipeline/completed/773-panel-tabs-after-a-restart.spec.md
---

## Title
The Debug button at the status bar's left. Chad, 2026-10-11: "yes debug moves to bottom left and
stays the exception". Every other side panel opens as a tab (#747, #748); the Debug panel stays in
the bottom dock, and in the Marley layout its button leaves the right-hand dock buttons for the
status bar's left end. This ticket is #748's D6 and D7.

## Scope
### In
- `debugger.button` defaulting to false in the Marley layout, with Zed's value kept for the switch
  back, beside `terminal.button`.
- A Marley status item after the left dock's buttons that draws the Debug icon, toggled while a
  dock shows the Debug panel, opening or closing it on a click, with Zed's own Hide Button.

### Out (explicitly deferred)
- The Debug panel as a tab: Chad kept it in the bottom dock.
- The Dock Left/Right/Bottom entries of Zed's button menu: the Marley button's menu is Hide Button
  only, as the status bar draws it for every hideable item.
- Moving Zed's bottom `PanelButtons` itself: the status bar finds items by type and the three docks'
  `PanelButtons` share one, so it would need a Zed hunk.

## Reference (§20)
Upstream Zed, `workspace`: the dock's `PanelButtons` (`dock.rs` 1398), which draws each panel's
icon with its tooltip, toggled while its dock is open on it, and on a click focuses the dock and
dispatches either the dock's toggle (open on this panel) or the panel's `toggle_action`; the status
bar's left and right items, `position_of_item` and `insert_item_after` (`status_bar.rs` 374, 391),
and its right-click Hide Button for any item whose `hide_setting` answers. The Debug panel's
`icon`, `icon_tooltip` and `hide_button_setting` read `debugger.button` (`debugger_panel.rs`
1558-1588). Behavior kept: the button's icon, tooltip, toggled look, click and Hide Button are
Zed's; only its place changes, in the Marley layout.

### Prior art
- **The code we ship:**
  - `PanelButtons` (`dock.rs`) is the behavior to match; it draws a dock's buttons as one item, so
    a single panel's button cannot move without a second item.
  - `Dock::panel_for_persistent_name` (#773's accessor) finds the Debug panel by `DebugPanel`
    without a `debugger_ui` dependency; through its `PanelHandle` the item gets the panel's own
    `toggle_action` (`debug_panel::ToggleFocus`) and `hide_button_setting`, so nothing is built by
    name (#748's D6 planned `App::build_action`; the accessor makes it unneeded).
  - `marley_workbench::apply_defaults` already patches `terminal.button` and `agent.dock` per
    layout with Zed's values kept in `LayoutState` (#441, #568); `debugger.button` joins them.
  - `status_buttons.rs` (#739, #746) holds Marley's status-bar buttons and adds them in
    `observe_new`.
- **Behavior maps:** `docs/zed_architecture/` has nothing on the status bar beyond the code.
- **No dependency** in `Cargo.lock` touches the status bar.

## UI proof
Scenario `script/e2e/774-the-debug-button-at-the-left.sh` (`compositor sway`): a scratch repo.

| Shot | Shows | Criteria |
|---|---|---|
| `774-01-start` | the Debug icon at the status bar's left end; none among the right-hand buttons | REQ-001 |
| `774-02-open` | a click on it: the bottom dock open on the Debug panel, the button toggled | REQ-002 |
| `774-03-closed` | a second click: the bottom dock closed, the button plain | REQ-003 |
| `774-04-zed-layout` | `marley: use zed layout`: no Debug icon at the left, Zed's own among the right-hand buttons | REQ-005 |
| `774-05-menu` | back in the Marley layout, a right-click on it: the Hide Button menu | REQ-004 |
| `774-06-hidden` | Hide Button chosen: no Debug icon anywhere; the copy's settings hold `"button": false` under `debugger` | REQ-004 |

## Locked-In Decisions
- **D1 — `debugger.button` defaults to false in the Marley layout** (`apply_defaults`), Zed's
  default read once into `LayoutState` and given back in Zed's layout, as `terminal.button` is.
  The bottom dock's own buttons drop Debug with it.
- **D2 — `DebugButton`, a Marley status item** in `status_buttons.rs`, inserted in `observe_new`
  after the left dock's `PanelButtons` (`position_of_item::<PanelButtons>`, the first one, which
  `Workspace::new` adds first). It keeps the workspace's three docks and observes them and the
  settings.
- **D3 — It draws only** in the Marley layout, while a dock holds the `DebugPanel`, while Zed's
  own button is hidden (the panel's `icon` answers `None`), and while the user's own settings do
  not set `debugger.button` false. A user who sets it true gets Zed's button among the dock's
  buttons, not two.
- **D4 — Its look and click are `PanelButtons`'**: the `Debug` icon, the tooltip "Debug Panel"
  with the panel's key, toggled while the dock holding the panel is open on it; a click focuses
  that dock and dispatches the dock's `toggle_action` when toggled, else the panel's
  `toggle_action`.
- **D5 — Hide Button is the panel's own**: `hide_setting` answers the panel's
  `hide_button_setting`, which writes `debugger.button: false`; the status bar draws the menu.
- **D6 — Zed's layout**: the item draws nothing, and Zed's button returns with Zed's default (#748's
  D7).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the Marley layout is on, the status bar shall show the Debug button at its left end and none among the right-hand dock buttons. | shot `start` |
| REQ-002 | WHEN the Debug button is clicked while no dock shows the Debug panel, the system shall open the bottom dock on the Debug panel and draw the button toggled. | shot `open` |
| REQ-003 | WHEN the Debug button is clicked while the bottom dock shows the Debug panel, the system shall close the bottom dock. | shot `closed` |
| REQ-004 | WHEN Hide Button is chosen from the Debug button's right-click menu, the system shall write `debugger.button: false` to the user's settings and stop drawing the button. | shots `menu`, `hidden`; the settings file |
| REQ-005 | WHILE Zed's layout is on, the system shall draw no Debug button at the left, and Zed's own Debug button shall show among the bottom dock's buttons. | shot `zed-layout` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `apply_defaults` and `LayoutState` (`marley_workbench.rs`), `DebugButton`
  (`status_buttons.rs`); a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
