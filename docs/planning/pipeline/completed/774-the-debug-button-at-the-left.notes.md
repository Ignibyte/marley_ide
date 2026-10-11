# The Debug button at the status bar's left — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-774-the-debug-button-at-the-left.md
- **Pipeline spec:** 774-the-debug-button-at-the-left.spec.md

## Phase 1 — Plan
- **Request:** the queue's top row, split from #748 at its promotion. Chad, 2026-10-11: "yes debug
  moves to bottom left and stays the exception".
- **Classification / tier:** feature, two Marley files, no Zed hunk.
- **Recall (§18.3):**
  - #748's D6 and D7: the design, which planned `App::build_action` for the panel's toggle; #773's
    `Dock::panel_for_persistent_name` now hands over the panel's own `toggle_action` instead.
  - L-claude-498-a-new-toolbar-button-moves-older-scenarios-clicks-001: a new left item shifts the
    left items after it (the search button, the LSP button, diagnostics); the right-hand buttons
    that older scenarios click (Files, Git, Outline) sit right of the bottom dock's group, so
    Debug leaving that group moves none of them.
  - `apply_defaults` keeps Zed's values in `LayoutState` and needs them never `None`.
  - The brain (asked at Complete, consultation 8c65f591323d419e83424a3a1530237c): nothing on this
    seam beyond #748's decision.
- **Discovery:** `crates/marley_workbench/src/marley_workbench.rs` (`LayoutState`, `init`,
  `apply_defaults`); `crates/marley_workbench/src/status_buttons.rs`; `workspace::dock`
  (`PanelButtons`, `Dock::toggle_action`, `panel_for_persistent_name`), `workspace::status_bar`
  (`position_of_item`, `insert_item_after`, `render_hideable_item`), `debugger_panel.rs` 1513-1600.
- **Decisions:** spec D1-D6.

### Design
- **Approach.**
  - `marley_workbench.rs`: `LayoutState.zed_debug_button: Option<bool>` read from the raw
    defaults (`defaults.debugger…button`); `apply_defaults` sets `defaults.debugger…button` to
    `Some(false)` in the Marley layout and to Zed's value in Zed's.
  - `status_buttons.rs`: in the existing `observe_new`, `DebugButton::new(workspace.all_docks(),
    cx)`; then `status_bar.update`: `position_of_item::<PanelButtons>()`, `insert_item_after` it, or
    `add_left_item` when none. `DebugButton { docks: [Entity<Dock>; 3], _subscriptions }` observes
    each dock and the `SettingsStore`. `debug_panel(&self, cx) -> Option<(Entity<Dock>,
    Arc<dyn PanelHandle>)>`. `Render`: nothing unless D3 holds; else an `IconButton` (`Debug`,
    Small, `toggle_state`), tooltip `Tooltip::for_action("Debug Panel", &*action, cx)` when not
    toggled, a click that focuses the dock and dispatches the action. `StatusItemView`:
    `set_active_pane_item` empty; `hide_setting` the panel's `hide_button_setting`.
  - The user's own value: `SettingsStore::raw_user_settings()` →
    `content.debugger.as_ref().and_then(|debugger| debugger.button)`.
- **File manifest:** `crates/marley_workbench/src/marley_workbench.rs`,
  `crates/marley_workbench/src/status_buttons.rs` (Marley crate). No Zed file.

### Visual check plan
| Criterion | The scenario | Shot |
|---|---|---|
| REQ-001 | start on a scratch repo | `774-01-start` |
| REQ-002 | click the Debug button (its place found on the first run's shot) | `774-02-open` |
| REQ-003 | click it again | `774-03-closed` |
| REQ-005 | `marley: use zed layout` from the palette | `774-04-zed-layout` |
| REQ-004 | `marley: use marley layout`; right-click the button; shot; click Hide Button; shot; `expect` the copy's settings to hold `"button": false` | `774-05-menu`, `774-06-hidden` |

### Risks
- The left items after Debug move right by one button; a scenario that clicks the search or
  diagnostics button by position would miss (L-498). The tree has none that click them by x.
- The status bar's `render_hideable_item` reads `hide_setting` during its render; the item reads
  only docks and settings there, never the workspace, which may be updating.
