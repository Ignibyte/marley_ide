# Side panels open as tabs - the mechanism, Files, Git and Outline — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-747-side-panels-as-tabs.md
- **Pipeline spec:** 747-side-panels-as-tabs.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** Chad, 2026-10-11: "I wanted everything to open in the main panel now with a tab. THe
  includes everything in the bottom right. With the ability to split pane it and drag it all", and,
  asked about the dock: "everything opens in the main pane now then the user can just split panel it
  out themselves. that way its consistent". Shelf:
  `docs/planning/design-notes/tabs-everywhere-2026-10-11.md`.
- **Classification / tier:** feature; Marley crate (`marley_workbench`), one small Zed hunk
  (`outline_panel.rs`). Size L: the item, its table, the dock observer and the outline hunk.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **Recall (§18.3):**
  - AD-claude-456 and F-claude-456: Zed's dock move forgets the panel a dock showed; Marley keeps a
    dock memory from outside `workspace` with Zed's public API. The same API carries this design.
  - #697 (`ThreadTab`): a panel's view hosted in a center tab, drawn in one place; the closest
    precedent. #736: a tab's restore waits for its panel (`PANEL_WAITS`).
  - AD-claude-674: every center item that is neither a terminal nor a Browser tab gets a rail row,
    so each panel tab gets one with no extra work.
  - AD-claude-673, AD-claude-646: Containers and Knowledge are right-dock panels that close their
    dock while hidden or off (#748's part).
  - L-claude-613: `Item::added_to_workspace` sees a move into another workspace.
  - PR-claude-a-callback-zed-calls-mid-update-defers-its-entity-work-001,
    PR-claude-defer-in-does-not-leave-the-entitys-own-update-001,
    PR-claude-defer-a-pane-change-out-of-an-items-own-event-001: the observer runs after the dock's
    update, but a tab added from a panel's event must still wait for the window's next turn.
  - PR-claude-701-an-items-handlers-that-update-its-workspace-are-not-listeners-001: the tab's
    handlers that touch the workspace are plain closures over a weak handle.
  - F-claude-496: Ctrl+Shift+C in a tab field opened the collab panel; with #748 it opens a tab.
- **Discovery:** see the spec's Prior art for every line. The decisive reads:
  - The Marley layout puts every side panel in the right dock: Zed's defaults dock the project,
    outline, collab and git panels right (`assets/settings/default.json` 885, 1009, 1073, 1081),
    and `apply_defaults` moves the Agent Panel there and hides the terminal button
    (`marley_workbench.rs` 1215-1231). The left dock is empty. Marley's Fleet, Knowledge and
    Containers panels are right-only.
  - Ten `Panel` impls exist: Agent (0), Project (1), Git (3), Collab (5), Outline (6), Debug (7),
    Terminal, Fleet (20), Knowledge (21), Containers (22).
  - `PanelButtons` never shows a closed dock's button as toggled and sends the panel's own
    `toggle_action` (`dock.rs` 1424-1436), so with the dock always closed every click runs Zed's
    `toggle_panel_focus`, which opens, activates and focuses: the observer sees that.
  - Two findings against the ticket's premise. The outline panel clears whenever the active item is
    not an editor (`outline_panel.rs` 1097-1112), so its own tab, once clicked, would show nothing:
    D6's hunk. The default split keys clone, and a non-cloning item ignores them
    (`workspace.rs` 6202-6205, 6355): D4 sends them as moves.

### Design
- **Approach.** `panel_tabs.rs` holds four parts:
  1. `PanelTab`, an `Item` + `SerializableItem` + `Render` + `Focusable` over an
     `Arc<dyn PanelHandle>`: `render` draws `panel.to_any()` at full size; `tab_content_text` and
     `tab_icon` from the panel's `icon_tooltip` and `icon`; `Focusable` answers the panel's
     `panel_focus_handle`; `can_split` stays false; `show_toolbar` false; `deactivated`,
     `on_removed` and its pane's `ActivateItem` drive `set_active`; `added_to_workspace` closes a
     tab that landed in another workspace. Its row: workspace id, item id, persistent name, in a
     `MarleyPanelTabsDb` domain as `rusty/graph_store.rs` keeps the Graph tab's.
  2. `open(workspace, name, window, cx)`: the tab of that panel in any of the workspace's panes
     comes forward focused, else a new tab joins the active pane, focused. Deferred to the window's
     next turn.
  3. The dock observer, installed from `observe_new` on every workspace: for each of the three
     docks it keeps the last (active panel id, open) it saw. On a change in the Marley layout:
     if the dock is open and its active panel is moved, close it, re-activate and re-open the
     panel it showed before when that was another, and, when the moved panel's activation focus
     handle is focused, `open` its tab; then re-tell `set_active` to the previous and current active
     panels whose tabs disagree with the dock's last call (D5).
  4. The split keys: `register_action_renderer` with `capture_action` for `pane::SplitLeft`,
     `SplitRight`, `SplitUp`, `SplitDown`; while the active item is a `PanelTab` and the mode is
     `ClonePane`, stop propagation and dispatch the `SplitAndMove` twin.
  - The layout switch: `panel_tabs` observes `MarleySettings`; on a switch to Zed's layout it
    closes every `PanelTab` in every window.
- **Outline hunk** (`crates/outline_panel/src/outline_panel.rs`, Zed crate): at 1097 and in
  `set_active` (5644), an active item whose `item_focus_handle` is the panel's own focus handle is
  skipped: the panel keeps the editor it follows, or takes
  `workspace.recent_active_item_by_type::<Editor>(cx)`. `// Marley:` comments.
- **File manifest:**
  - `crates/marley_workbench/src/panel_tabs.rs` (new, Marley crate).
  - `crates/marley_workbench/src/marley_workbench.rs` (Marley crate): `mod panel_tabs`, its `init`.
  - `crates/outline_panel/src/outline_panel.rs` (Zed crate): the hunk.
  - `docs/marley/zed-touchpoints.md`: a new row for `crates/outline_panel/src/outline_panel.rs`
    (what: the active item that hosts the panel is skipped at the two reads; why: in the Marley
    layout the panel is drawn by a center tab, which becomes the active item while used; merge:
    keep the skip at every read of the active item).
  - `docs/marley/guide.md`, `docs/marley/walkthrough.md`, `docs/marley_architecture/marley_workbench.md`
    (Complete).
  - `script/e2e/747-side-panels-as-tabs.sh` (new).

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 008 | Setup: repo with `src/main.rs` (two functions) and an uncommitted change; `marley.layout` "zed". First launch: Ctrl+Shift+E opens the right dock on the project panel; quit. `marley.layout` set to "marley"; launch again | `restored`: no right dock |
| 001, 006 | Opens `src/main.rs`; clicks the Files, Git and Outline buttons (their places found by a first run, `STOP_AT_BAR=1`) | `files`, `git`, `outline` |
| 002 | Clicks the editor's tab, presses Ctrl+Shift+E | `key` |
| 003 | Brings the editor forward, clicks Git | `again`: one Git tab, in front |
| 004 | The editor's right-click → Reveal in Project Panel | `reveal` |
| 005 | Drags the Outline tab to the pane's right edge; on the Git tab presses Ctrl+\ | `drag`, `split-key` |
| 006 | Clicks the second function in the split Outline tab | `outline-used`: the editor scrolled, the outline still lists both functions |
| 009 | Double-clicks `src/main.rs` in the Files tab | `open-file` |
| 007 | Opens the right dock on the Agent Panel from the palette (by whichever action #744 left opening it), then clicks Files | `agent-stays` |
| 010 | Quits and launches again | `after-restart` |
| 011 | Runs `marley: use zed layout`, then clicks the Files button | `zed-layout` |
- REQ-012 is read in the diff; `outline-used` shows the outline updating while its tab is in front.
- No scenario reaches the commit modal's path (D2's "only closes"); the review covers it.

### Risks
- **One view drawn in two places.** gpui draws an entity's view as one element per frame: a panel
  drawn by its dock and by a tab at once would share one focus handle and one view cache, and
  clicks, focus and scroll would go to one copy unpredictably. The places a panel can be drawn are
  its dock (open, the panel active), the workspace's zoom layer (a zoomed panel) and a tab. The
  design rules each out: the observer closes a dock on a moved panel within the same effect flush,
  before any frame; none of the three panels zooms; one tab per panel per workspace (D4), a split
  moves, a tab carried into another workspace closes; the Zed layout has no tabs (D10).
- `set_active` from two drivers (D5): a wrong guess of the dock's last call leaves an outline that
  does not update. The observer reads the dock's state before and after, which is what the dock's
  calls depend on.
- The observer's focus test: an open whose focus lands on the panel's activation handle in a later
  turn would close without a tab. All paths read focus synchronously (`focus_or_unfocus_panel`,
  the `Activate` handler).
- `ActiveEntryChanged(None)` clears the project panel's marks when its tab becomes the active item
  (`project_panel.rs` 744-756): marks made, then a click away and back, are lost. Accepted.
- Rename and Duplicate from the palette with no Files tab open call `open_panel` without focus
  (`project_panel.rs` 557-580): the dock closes and no tab opens, so the rename field is not seen.
  Rare; noted for the guide.
- The scenario's first launch in Zed's layout writes the profile's dock state; the second launch
  reads it on the same build, so no cross-build format question arises.

## Phase 2 — Code
- **Built:** `panel_tabs.rs` (new, Marley crate): `PanelTab` (an `Item` over the dock panel's
  `Arc<dyn PanelHandle>`, named and iconed from the panel's button, its focus handle kept, drawing
  `panel.to_any()`; `set_active(true)` deferred from its first render in front, `false` from
  `deactivated`; `added_to_workspace` closes a tab dropped in another group); `show_tab` (the
  tab forward anywhere in the workspace, else a new one in the active pane, focused); one
  observer per dock (`dock_changed`): in the Marley layout an open dock on a moved panel goes back
  to the panel it showed before, or closes, and the tab opens when the panel holds the focus;
  `retell` gives the panel back `set_active(true)` when its tab is in front; the split keys'
  capture (`move_split`: a `ClonePane` split on a panel tab becomes `MovePane`); and the switch to
  Zed's layout closes every panel tab. `outline_panel.rs` (Zed crate, its ledger row first):
  `marley_active_editor` at the panel's two reads of the active editor. The in-app page's
  Ctrl+Shift+E row. Not serializable: #773.
- **Found in the visual check and fixed (two real bugs):**
  - The first run panicked ("cannot read OutlinePanel while it is already being updated"): the
    outline panel, inside its own update, asked the active item for its focus handle, and the tab
    answered by reading the panel. The tab now keeps the panel's focus handle from when it was
    made.
  - A tab dragged over a panel tab never split: the project panel's empty space blocks the mouse
    (`block_mouse_except_scroll`), which hides the pane's own drop target (its group's hitbox).
    The pane still tracks the drag's split direction in the capture phase, so the tab draws a drop
    layer of its own over the panel, transparent, coloured only while a tab is dragged over it,
    sized by the pane's `drag_split_direction`, and hands the drop to `Pane::handle_tab_drop`.
    A non-blocking hitbox leaves the panel's own clicks and drags working.
- **Review:** the observer's dock update runs after the update that opened the dock (no
  re-entrancy); the tab and the dock both call `set_active`, and the dock's last word is corrected
  by `retell` while the tab is in front; the Zed layout returns early everywhere.
- **Gate:** red once on seven clippy findings in the new module (`clone_on_ref_ptr` ×3, an
  `unwrap_or` with a call, a doc word needing backticks, two needless `&mut`s), fixed at the
  source. GATE GREEN [diff], 17 passed.

## Phase 3 — Test
- **Scenario:** `script/e2e/747-side-panels-as-tabs.sh`, `compositor sway`: a scratch git repo with
  `src/lib.rs` changed after its first commit. Five runs: the panic above; Ctrl+P not opening a
  file (the file finder is reached through the palette now); a drag dropped in the pane's middle,
  then dropped with one jump (the drag now moves in eight steps, as #613's does); a throwaway probe
  dragging an ordinary `lib.rs` tab the same way split at once, which pointed at the panel tab; the
  fix above; the fifth run exits 0 with no panic.
- **Shots (fifth run):**
  - `747-00-start`: a fresh project with only its terminal; Zed's `starts_open` project panel never
    shows in the dock and makes no tab. REQ-008's rule for a dock opened without the focus.
  - `747-01-files`: the status bar's Files button: a "Project Panel" tab in the active pane with
    the tree, its rail row. REQ-001.
  - `747-02-git`: the Git button: a "Git Panel" tab with lib.rs changed and the commit box. REQ-001.
  - `747-03-again`: Ctrl+Shift+E: the Project Panel tab forward; one Project Panel and one Git
    Panel tab. REQ-002, REQ-003.
  - `747-04-outline`: lib.rs opened, the Outline button: an "Outline Panel" tab, itself in front,
    listing `pub struct Ledger`, `entries`, `impl Ledger`, `pub fn total`, `pub fn open_ledger`.
    REQ-001, REQ-006, REQ-012 (the panel was told active and follows lib.rs).
  - `747-05-reveal`: lib.rs again, `pane: reveal in project panel`: the Project Panel tab forward,
    `src` open and lib.rs selected. REQ-004.
  - `747-06a-dragging`, `747-06-drag`: the Project Panel tab dragged over its own pane's right
    edge: the right half lit; dropped, Project Panel in a pane of its own on the right. REQ-005.
  - `747-07-split-key`: the Git button, then `pane: split down`: Git Panel moved into a new pane
    under lib.rs, no copy left. REQ-005.
  - `747-08-agent-stays`: Ctrl+Alt+B (the Agent Panel in the right dock), then the Files button:
    the Project Panel tab forward, the right dock still on the Agent Panel. REQ-007.
  - `747-09-zed-layout`: `marley: use zed layout`: only the terminal and lib.rs tabs are left.
    REQ-011.
- **Reviewed, not shot:** REQ-009 (a file opened from Files lands where Zed opens it: unchanged
  Zed code, D7); REQ-008's restore of a dock saved open on a moved panel (the same observer path as
  `747-00`, a dock that opens without the focus).
- **Pre-existing, not in scope:** rust-analyzer's "Failed to discover workspace" for the scratch
  crate (it has no `Cargo.toml`).

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Changed: Files, Git and Outline open as tabs); the guide's "Side
  panels as tabs"; the architecture note; the slice line in `workbench-shell.md`; the
  `outline_panel.rs` row of `zed-touchpoints.md` (written before the hunk) describes what shipped.
- **Knowledge appended:** F-claude-747-a-panel-tab-read-its-panel-while-the-panel-updated-001,
  PR-claude-747-an-item-hosting-a-view-keeps-what-the-view-asks-of-it-001,
  F-claude-747-a-panel-hid-the-panes-drop-target-001,
  L-claude-747-probe-a-failing-drag-with-an-ordinary-tab-001.
- **Ticket:** closed; its BACKLOG row left at promotion; #773 carries the restore.
- **Gate:** GATE GREEN [diff], 17 passed, on the tree committed.

### Promotion (2026-10-11, at 50631c0869)
- **Split.** D9's restore after a restart (and REQ-010) goes to TICKET-773, as the drafting agent
  proposed; this ticket's item is not serializable yet.
- **Seams re-checked:** `dock.rs` `PanelHandle` (107-137: `persistent_name`, `set_active`,
  `icon`, `icon_tooltip`, `panel_focus_handle`, `activation_focus_handle`, `to_any`), `Dock`'s
  `is_open` 513, `panel_index_for_persistent_name` 533, `panel_for_id` 545, `active_panel_index`
  574, `set_open` 578, `activate_panel` 938 (its `activate_panel_internal` calls `set_active`
  on the old and new active panel), `active_panel` 970. No `panel_for_index` is public: handles
  come from `PanelAdded`'s view id through `panel_for_id`, or from `active_panel`.
- **Recall:** PR-claude-735 (code in a workspace's update reads it by reference; a modal swap
  defers), F-claude-679 (an open from inside an update defers), L-claude-744 (a key sent from the
  rail misses the workspace), AD-claude-674 (every center tab has a rail row).

## Phase 2 — Code
- **Built:** `panel_tabs.rs` (new, Marley crate): `PanelTab` (an `Item` over the dock panel's
  `Arc<dyn PanelHandle>`, named and iconed from the panel's button, its focus handle kept, drawing
  `panel.to_any()`; `set_active(true)` deferred from its first render in front, `false` from
  `deactivated`; `added_to_workspace` closes a tab dropped in another group); `show_tab` (the
  tab forward anywhere in the workspace, else a new one in the active pane, focused); one
  observer per dock (`dock_changed`): in the Marley layout an open dock on a moved panel goes back
  to the panel it showed before, or closes, and the tab opens when the panel holds the focus;
  `retell` gives the panel back `set_active(true)` when its tab is in front; the split keys'
  capture (`move_split`: a `ClonePane` split on a panel tab becomes `MovePane`); and the switch to
  Zed's layout closes every panel tab. `outline_panel.rs` (Zed crate, its ledger row first):
  `marley_active_editor` at the panel's two reads of the active editor. The in-app page's
  Ctrl+Shift+E row. Not serializable: #773.
- **Found in the visual check and fixed (two real bugs):**
  - The first run panicked ("cannot read OutlinePanel while it is already being updated"): the
    outline panel, inside its own update, asked the active item for its focus handle, and the tab
    answered by reading the panel. The tab now keeps the panel's focus handle from when it was
    made.
  - A tab dragged over a panel tab never split: the project panel's empty space blocks the mouse
    (`block_mouse_except_scroll`), which hides the pane's own drop target (its group's hitbox).
    The pane still tracks the drag's split direction in the capture phase, so the tab draws a drop
    layer of its own over the panel, transparent, coloured only while a tab is dragged over it,
    sized by the pane's `drag_split_direction`, and hands the drop to `Pane::handle_tab_drop`.
    A non-blocking hitbox leaves the panel's own clicks and drags working.
- **Review:** the observer's dock update runs after the update that opened the dock (no
  re-entrancy); the tab and the dock both call `set_active`, and the dock's last word is corrected
  by `retell` while the tab is in front; the Zed layout returns early everywhere.
- **Gate:** red once on seven clippy findings in the new module (`clone_on_ref_ptr` ×3, an
  `unwrap_or` with a call, a doc word needing backticks, two needless `&mut`s), fixed at the
  source. GATE GREEN [diff], 17 passed.

## Phase 3 — Test
- **Scenario:** `script/e2e/747-side-panels-as-tabs.sh`, `compositor sway`: a scratch git repo with
  `src/lib.rs` changed after its first commit. Five runs: the panic above; Ctrl+P not opening a
  file (the file finder is reached through the palette now); a drag dropped in the pane's middle,
  then dropped with one jump (the drag now moves in eight steps, as #613's does); a throwaway probe
  dragging an ordinary `lib.rs` tab the same way split at once, which pointed at the panel tab; the
  fix above; the fifth run exits 0 with no panic.
- **Shots (fifth run):**
  - `747-00-start`: a fresh project with only its terminal; Zed's `starts_open` project panel never
    shows in the dock and makes no tab. REQ-008's rule for a dock opened without the focus.
  - `747-01-files`: the status bar's Files button: a "Project Panel" tab in the active pane with
    the tree, its rail row. REQ-001.
  - `747-02-git`: the Git button: a "Git Panel" tab with lib.rs changed and the commit box. REQ-001.
  - `747-03-again`: Ctrl+Shift+E: the Project Panel tab forward; one Project Panel and one Git
    Panel tab. REQ-002, REQ-003.
  - `747-04-outline`: lib.rs opened, the Outline button: an "Outline Panel" tab, itself in front,
    listing `pub struct Ledger`, `entries`, `impl Ledger`, `pub fn total`, `pub fn open_ledger`.
    REQ-001, REQ-006, REQ-012 (the panel was told active and follows lib.rs).
  - `747-05-reveal`: lib.rs again, `pane: reveal in project panel`: the Project Panel tab forward,
    `src` open and lib.rs selected. REQ-004.
  - `747-06a-dragging`, `747-06-drag`: the Project Panel tab dragged over its own pane's right
    edge: the right half lit; dropped, Project Panel in a pane of its own on the right. REQ-005.
  - `747-07-split-key`: the Git button, then `pane: split down`: Git Panel moved into a new pane
    under lib.rs, no copy left. REQ-005.
  - `747-08-agent-stays`: Ctrl+Alt+B (the Agent Panel in the right dock), then the Files button:
    the Project Panel tab forward, the right dock still on the Agent Panel. REQ-007.
  - `747-09-zed-layout`: `marley: use zed layout`: only the terminal and lib.rs tabs are left.
    REQ-011.
- **Reviewed, not shot:** REQ-009 (a file opened from Files lands where Zed opens it: unchanged
  Zed code, D7); REQ-008's restore of a dock saved open on a moved panel (the same observer path as
  `747-00`, a dock that opens without the focus).
- **Pre-existing, not in scope:** rust-analyzer's "Failed to discover workspace" for the scratch
  crate (it has no `Cargo.toml`).

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Changed: Files, Git and Outline open as tabs); the guide's "Side
  panels as tabs"; the architecture note; the slice line in `workbench-shell.md`; the
  `outline_panel.rs` row of `zed-touchpoints.md` (written before the hunk) describes what shipped.
- **Knowledge appended:** F-claude-747-a-panel-tab-read-its-panel-while-the-panel-updated-001,
  PR-claude-747-an-item-hosting-a-view-keeps-what-the-view-asks-of-it-001,
  F-claude-747-a-panel-hid-the-panes-drop-target-001,
  L-claude-747-probe-a-failing-drag-with-an-ordinary-tab-001.
- **Ticket:** closed; its BACKLOG row left at promotion; #773 carries the restore.
- **Gate:** GATE GREEN [diff], 17 passed, on the tree committed.
