# Side panels as one inspector — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-837-side-panels-as-one-inspector.md
- **Pipeline spec:** 837-side-panels-as-one-inspector.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #836; Chad's "lets continue on" (2026-10-21).
- **Classification / tier:** feature, L. Marley crate (`marley_workbench`) plus the Marley-owned
  settings files (`settings_content/src/marley.rs`, `settings_ui/src/marley_page.rs`).
- **Recall (§18.3):** PR-claude-747 (keep the hosted panel's focus handle; never read the panel to
  answer the pane), PR-claude-773 (re-tell both panels a dock call touched), F-claude-747's drop
  target (a panel blocks the pane's drop target), AD-claude-773 (panels found by persistent name).
  The brain (consultation 6abe7936): nothing on this seam.
- **Discovery:** `panel_tabs.rs`: `MOVED` (:39), `init` (:58), `dock_changed` (:115), `retell`
  (:165), `show_tab` (:185), `close_tabs` (:223), `PanelTab` (:255), `workspace_panel` (:540);
  `Workspace::split_pane` (workspace.rs:6369), `resize_pane` (:6115); `toggle_rail_strip`
  (marley_workbench.rs:1336); the board `SidePanels.dc.html` (Inspector, Files | Changes 1 |
  Outline, then the panel).
- **Decisions:** D1 to D4 in the spec.

### Design
- **Setting:** `MarleySettingsContent::side_inspector: Option<bool>`; `MarleySettings`
  gains `side_inspector: bool`; a settings-page row "Use the Inspector"; action
  `ToggleInspector` writes the opposite through `update_settings_file`.
- **`inspector.rs` (new, Marley):** `pub struct Inspector { workspace, panels: Vec<Shown>, current:
  usize, front: bool }`, `Shown { panel: Arc<dyn PanelHandle>, focus: FocusHandle, label }` for
  the three names present (`Project Panel` "Files", `GitPanel` "Changes", `Outline Panel`
  "Outline"). `Item`: tab "Inspector", icon, no toolbar; `deactivated` tells the current panel
  inactive; `Focusable` answers the current panel's kept handle. `Render`: a header of three
  `Button`s with `toggle_state` (a click: `show(index)`: the old panel `set_active(false)`, the new
  one `true`, deferred), then the current panel's view with #747's drop layer.
  `INSPECTED: [&str; 3]`, `pub(crate) fn inspects(name)`, `pub(crate) fn show(workspace, panel,
  window, cx)`: an Inspector in the workspace switches and comes forward; else make one, split the
  active pane to the right, add it there, and `resize_pane(Horizontal, 320 - half)` on the new
  pane. `pub(crate) fn current_panel(&self) -> Option<EntityId>` for `retell`.
- **`panel_tabs.rs`:** `dock_changed`: when asked, `inspector::inspects(name) &&
  side_inspector` → `inspector::show`, else `show_tab`. `retell` also finds an Inspector in front
  showing that panel. The settings observer: turning the setting on closes the three panels'
  tabs; off, or Zed's layout, closes every Inspector (`close_inspectors`). `close_tabs`' window walk
  is shared through a helper taking the item filter.
- **File manifest:** `crates/marley_workbench/src/inspector.rs` (new), `panel_tabs.rs`,
  `marley_workbench.rs` (action, setting, `mod`), `crates/settings_content/src/marley.rs`,
  `crates/settings_ui/src/marley_page.rs` (Marley-owned files in Zed crates; their ledger rows
  exist), `assets/settings/default.json` if Marley's keys are listed there; the scenario.

### Visual check plan
| REQ | Setup and action | Shot |
|---|---|---|
| REQ-001/002 | repo with a changed `src/main.rs`, open; setting on; status bar's Files | `837-01-files` |
| REQ-002 | the switcher's Changes | `837-02-changes` |
| REQ-002 | the switcher's Outline | `837-03-outline` |
| REQ-003 | setting off; Files | `837-04-tabs` |
| REQ-001/002 | setting on, Marley Day, Files | `837-05-day` |

### Risks
- `resize_pane`'s amount: the split starts at half the center; if the shrink misses, the column is
  set from the shot.
- The outline panel follows the editor used last only while no dock shows it (#773's hunk): the
  Inspector is a non-editor item, as a panel tab is, so it should hold.

## Phase 2 — Code
- **Built:** the setting (`side_inspector`, `SidePanels`), `ToggleInspector`, the Marley page's
  "Use the Inspector" row (Layout's array 8 → 9), `default.json`; `inspector.rs` as designed;
  `panel_tabs.rs`: `dock_changed` routes the three panels through `inspector::takes`, `retell` asks
  `inspector::front_panel`, the settings observer closes tabs or Inspectors, and the drop layer,
  `hand_drop`, `pane_of` and `close_items` are shared helpers now. Ledger rows for the three
  Marley-owned files in Zed crates extended.
- **Deviation:** `from_settings` went to 101 lines: `AgentCommandHistory::from_content` takes the
  inline `if` out, as its neighbours do. Scenario uses `project panel: toggle focus` from the
  palette, not the status bar's buttons, whose places moved since #747.
- **Review:** `narrow`'s next-frame closure first called `pane_of`, which reads the workspace the
  closure was updating (a double lease); it finds the pane from `workspace.panes()` instead. `show`
  held `items_of_type`'s borrow across `update` (a compile error, split into a `let`).
- **Install trap:** #836's install built the working tree while these edits were going in and
  failed on the half-added settings row; #837's work was stashed, #836 installed clean, and the
  stash came back (noted in memory).
- **Gate:** `just gate-diff` green (17 gates and the receipt).


## Phase 3 — Test
- Scenario `script/e2e/837-side-panels-as-one-inspector.sh` (sway), on the gated build. The probe
  found the switcher at (1356, 82) and (1418, 82). The first recorded run's file finder opened late
  and took "project panel: toggle focus" as a new file's name; its settles grew to 3 s, and the
  gate ran again after the edit.
- `837-01-files`: lib.rs in the editor, the Inspector in a pane at the right about 320 px wide, its
  tab "Inspector", the switcher's Files lit, the tree with lib.rs marked (REQ-001, REQ-002).
- `837-02-changes`: Changes lit; the Git panel: Changes (1), Tracked, `lib.rs src +2 −0`, the
  commit box (REQ-002).
- `837-03-outline`: Outline lit; Ledger, entries, impl Ledger, total, open_ledger: the outline
  follows the editor beside it (REQ-002).
- `837-04-tabs`: the setting off: no Inspector; `project panel: toggle focus` opened the
  Project Panel tab in the editor's pane (REQ-003).
- `837-05-day`: Marley Day, the setting on again: the Files tab closed and the Inspector back on
  Files (REQ-001, REQ-002).
- Not shot: the drop layer over the Inspector (#747's code, now shared, unchanged in behavior) and
  the toggle action (one settings write, as `toggle_rail_strip`).

## Phase 4 — Complete
- **Docs:** CHANGELOG (Added), the guide's Side panels section (The Inspector), the architecture
  note's "The Inspector", the look plan's W3.1 done; the three ledger rows in Phase 2.
- **Knowledge:** AD-claude-837-the-inspector-is-a-center-item-not-a-dock-001,
  L-claude-837-the-install-builds-the-working-tree-001.
- **Brain:** decision on consultation 6abe7936.
- Ticket closed, pipeline archived.
