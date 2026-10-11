# Panel tabs come back after a restart — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-773-panel-tabs-after-a-restart.md
- **Pipeline spec:** 773-panel-tabs-after-a-restart.spec.md

## Phase 1 — Plan
- **Request:** the queue's top row, split from #747 at its promotion (#747's D9). Chad,
  2026-10-11: "everything opens in the main pane now"; a tab that vanishes at every restart is not
  a tab like the others.
- **Classification / tier:** feature, one Marley file (`panel_tabs.rs`), no Zed hunk.
- **Recall (§18.3):**
  - L-claude-736-a-restored-item-waits-for-its-workspaces-panels-001: Zed adds panels and restores
    items in the same load, in no fixed order; wait in the item's own task, bounded.
  - PR-claude-a-row-a-restore-reads-is-read-before-any-cleanup-001: rows read at init into a
    global, never from the table at `deserialize`.
  - PR-claude-restore-is-a-second-constructor-001: the restore builds the same tab a click does,
    so one constructor.
  - #736's scenario relaunches with the same path and its tabs come back: a path's workspace
    restores its items (L-601 and L-670 are about the window's own state).
  - The brain (consultation db182ee1ee3d4b67826b37b6dece6dca): nothing on this seam.
- **Discovery:** `crates/marley_workbench/src/panel_tabs.rs` (the tab, `show_tab`, `init`);
  `thread_tab.rs`'s `persistence` module and `SavedThreadTabs` as the pattern;
  `workspace::persistence::model::SerializedPane::deserialize_to` (items restored concurrently by
  `join_all`, then added with focus); `item.rs` 761 (serialization queued when an item joins a
  pane); Marley's Fleet, Knowledge and Containers panels are added synchronously in
  `observe_new`, Zed's four in `initialize_panels` after their `load`s.
- **Decisions:** spec D1-D6.

### Design
- **Approach.** In `panel_tabs.rs`:
  - `PanelTab::new(panel: Arc<dyn PanelHandle>, workspace: WeakEntity<Workspace>, window, cx)`
    returns the struct (name from `icon_tooltip`, else the persistent name; icon; the panel's
    focus handle; `front: false`). `show_tab` uses it.
  - `mod persistence` with `MarleyPanelTabsDb` (`static_connection!` over `WorkspaceDb`):
    `marley_panel_tabs(workspace_id INTEGER, item_id INTEGER, panel TEXT NOT NULL, PRIMARY
    KEY(workspace_id, item_id), FOREIGN KEY(workspace_id) REFERENCES workspaces ON DELETE
    CASCADE) STRICT`; `save_tab(item_id, workspace_id, panel)`, `all_tabs()`.
  - `SavedPanelTabs(HashMap<(WorkspaceId, ItemId), SharedString>)` global, loaded in `init`;
    `save_tab` writes the global and the table.
  - `impl SerializableItem for PanelTab`: `cleanup` trims the global and calls
    `workspace::delete_unloaded_items(…, "marley_panel_tabs", …)`; `serialize` saves the panel's
    persistent name; `should_serialize` false; `deserialize` refuses at once (Zed's layout, no row,
    not `is_moved`), then `window.spawn`s the wait: each 100 ms, read the workspace for a panel of
    that name across `all_docks()`; then in `cx.update`, refuse when disabled or when a tab of that
    panel is already open, else `cx.new(|cx| PanelTab::new(…))`.
  - `init` registers `workspace::register_serializable_item::<PanelTab>(cx)` and loads the rows.
  - The `persistent_name` is `&'static str`; the row holds a `String`, matched against `MOVED`.
- **File manifest:** `crates/marley_workbench/src/panel_tabs.rs` (Marley crate); the crate
  already depends on `db` (the thread tab's table). No Zed file, no ledger row.

### Visual check plan
| Criterion | The scenario | Shot |
|---|---|---|
| setup | Files, Git buttons; lib.rs through the file finder; the Outline button; `marley: toggle containers`; the Files button, then `pane: split right` | `773-01-before` |
| REQ-001 | `quit_marley`, `launch_marley` (same path) | `773-02-restored` |
| REQ-002, REQ-003 | the Outline button | `773-03-outline` |
| REQ-003 | the Files button | `773-04-again` |
| REQ-004 | quit; `profile_setting marley.rail_containers false`; launch; then quit and read the profile's database for a Containers row | `773-05-disabled`, `expect` |
| REQ-005 | `profile_setting marley.rail_containers true`, `profile_setting marley.layout '"zed"'`; launch | `773-06-zed-layout` |
| REQ-006 | not reachable: only Marley writes the rows, always with a moved name and a panel that exists; the review of the diff checks both refusals | — |

### Risks
- A panel that never arrives holds its pane's restore up to 10 s, since Zed joins every item of
  a pane before adding any. Every moved panel is added in the same load (Marley's three
  synchronously), so only a panel removed from the build would wait; the thread tab took the same
  bound.
- The restored tab is added with focus by Zed (`add_item(…, true, true, …)`). The dock observer
  acts only on a dock that opens, so a focused panel in a tab opens nothing; Test reads this.
- After a restart the dock may restore with a moved panel active but closed; `activate_panel`
  then tells the panel it is inactive while its tab is in front. `retell` answers a dock change
  for the panel last active; Test reads the restored Outline tab to see that it still lists.

## Phase 2 — Code
- **Built** (`crates/marley_workbench/src/panel_tabs.rs`): `PanelTab::new`, which `show_tab` now
  calls; `impl SerializableItem for PanelTab` (kind `MarleyPanelTab`): `serialize` saves the
  panel's persistent name, `should_serialize` false, `cleanup` trims the global and calls
  `delete_unloaded_items`, `deserialize` refuses in Zed's layout, without a row and outside
  `MOVED`, then waits in a `window.spawn` for the panel (100 × 100 ms) and refuses a disabled panel
  or one the group already shows in a tab; `SavedPanelTabs` loaded in `init`; `mod persistence`
  with `MarleyPanelTabsDb` and its `marley_panel_tabs` table; the registration in `init`.
- **Deviation: one Zed hunk.** The plan said none, but `Dock` hands out a panel only by type or
  by entity id, and a name gets only an index (`panel_index_for_persistent_name`), with no public
  way from an index to the panel. `Dock::panel_for_persistent_name` (`dock.rs`, 6 lines, beside
  `panel_for_id`) is the smallest seam; the alternative, typed lookups, would add
  `project_panel`, `outline_panel` and `collab_ui` to `marley_workbench` for one search. Its row
  is in `docs/marley/zed-touchpoints.md`.
- **Review of the diff:**
  - Re-entrancy: `deserialize`'s own call reads only globals and settings; the workspace is read
    in the spawned task only, after the restoring update returns (L-736). The duplicate check
    reads the workspace's panes inside `cx.update`, when no pane is being updated.
  - REQ-006 by reading: a name outside `MOVED` fails before the spawn; a panel that never arrives
    fails after the last wait with "the workspace has no … for its tab".
  - A restored tab whose panel belongs to another workspace cannot occur: the panel is looked up
    in the restoring workspace's docks, and `added_to_workspace` still closes a foreign one.
  - Provenance: the shape follows Marley's own thread tab; no Zed function body is copied.
- **Clippy:** one `use_self` on `items_of_type::<PanelTab>` inside the impl, fixed.
- **Gate:** `just gate-diff` GREEN, 17 passed (log in the scratchpad, `gate-773.log`).

## Phase 3 — Test
- **Scenario:** `script/e2e/773-panel-tabs-after-a-restart.sh` (`compositor sway`), three runs.
- **First run, two reds:**
  - No Containers tab after the restart, the log saying "MarleyContainersPanel is turned off".
    The runner's profile copy turns `marley.rail_containers` off (#669), so the refusal was right
    and the scenario was wrong: its `setup` now turns the setting on. (The palette's
    `marley: toggle containers` opened a tab for the disabled panel anyway: Zed's
    `toggle_panel_focus` does not ask `enabled`. Pre-existing, from #748; not in scope.)
  - The restored Outline tab showed "No outlines available", and so did the Outline tab after the
    Files button (`again`). Two causes. (1) The outline hunk (#747) let only the panel's own tab
    stand for the editor used last; a restored session's active item is the Files tab in the
    other pane, and any panel tab or terminal made the outline clear while it stayed in view.
    `marley_active_editor` now takes the panel's entity id and, while no dock shows the panel,
    lets any non-editor active item stand for the editor used last; a dock that shows it keeps
    Zed's rule. (2) The Files button moves the right dock's active panel from the outline panel
    to the project panel, and `activate_panel` tells the outline panel it is inactive, which
    clears it; `dock_changed` re-told only the panel just asked for. It now re-tells the dock's
    previous panel too (`retell(workspace, last.0, …)`).
- **Third run, every shot read:**
  - `773-01-before`: left pane repo — bash, Git Panel, lib.rs, Outline Panel, Containers (in
    front, listing two ports); Files alone in the right pane. The saved rows: GitPanel,
    MarleyContainersPanel, Outline Panel, Project Panel.
  - `773-02-restored` (REQ-001): after the quit and start, the same five tabs on the left with
    Containers in front and listing, and Files on the right showing the tree.
  - `773-03-outline` (REQ-002, REQ-003): the Outline button brought the restored Outline tab
    forward, no second one; it lists Ledger, entries, impl Ledger, total, open_ledger.
  - `773-04-again` (REQ-002, REQ-003): the Files button: the restored Files tab focused on the
    right (lib.rs row ringed), one Files tab; the Outline tab on the left still lists lib.rs.
  - `773-05-disabled` (REQ-004, REQ-002): a start with Containers off: Git, lib.rs and Outline on
    the left, Files on the right, no Containers tab; the Outline tab in front lists lib.rs from the
    start. The log: "MarleyContainersPanel is turned off". After the next quit the rows are
    GitPanel, Outline Panel, Project Panel: `check the Containers tab's row was dropped: pass`.
  - `773-06-zed-layout` (REQ-005): a start in Zed's layout: one pane with repo — bash and lib.rs,
    no panel tab; three log lines "panel tabs are not restored in Zed's layout".
- **REQ-006** by the review (Phase 2): no scenario can write a row with a foreign name or a panel
  the build lacks.
- **Gate after the fixes:** `just gate-diff` GREEN, 17 passed, over the fixed code and the
  scenario.

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Added: panel tabs after a restart; Fixed: the Outline tab keeps its
  outline); `docs/marley/guide.md` (Side panels as tabs: the outline line, After a restart);
  `docs/marley_architecture/marley_workbench.md` (a #773 section, #747's outline line);
  `docs/marley/workbench-shell.md` (the slice line); `docs/marley/zed-touchpoints.md` (a new
  `dock.rs` row, the `outline_panel.rs` row widened). The walkthrough has no panel-tab stop to
  change.
- **Ledgers:** F-claude-773-the-outline-tab-cleared-when-another-item-took-the-focus-001,
  F-claude-773-the-dock-observer-re-told-only-the-panel-just-asked-for-001,
  PR-claude-773-a-dock-call-tells-two-panels-re-tell-both-001,
  L-claude-773-the-runners-copy-turns-containers-off-001,
  L-claude-773-a-restore-is-checked-by-relaunching-with-the-same-path-001,
  AD-claude-773-a-panel-tab-is-saved-as-its-panels-name-001.
- **Brain:** `brain decide` on consultation db182ee1…: decisions/marley-saves-a-panel-tab-as-its-panels-name.
- Ticket closed, pipeline archived, committed.
