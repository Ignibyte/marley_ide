---
pipeline_id: c2d53f3d-0e43-493a-a7a0-0ee72e26458c
ticket: docs/planning/tickets/closed/TICKET-747-side-panels-as-tabs.md
status: Phase 4 — Complete PASS
title: Side panels open as tabs - the mechanism, Files, Git and Outline
type: feature
slice: the Marley layout (docs/marley/workbench-shell.md), tabs everywhere (docs/planning/design-notes/tabs-everywhere-2026-10-11.md)
references:
  - docs/planning/pipeline/completed/697-an-agent-thread-in-a-center-tab.spec.md
  - docs/planning/pipeline/completed/736-thread-tabs-after-a-restart.spec.md
  - docs/planning/pipeline/completed/739-rustal-icons-in-the-status-bar.spec.md
---

## Title
In the Marley layout the project panel, the git panel and the outline panel open as tabs of the
center, in the active pane of the group you are looking at, instead of in the right dock. Chad,
2026-10-11: "I wanted everything to open in the main panel now with a tab. THe includes everything
in the bottom right. With the ability to split pane it and drag it all", then "everything opens in
the main pane now then the user can just split panel it out themselves. that way its consistent".
This ticket builds the tab and moves these three panels; #748 moves the rest of the right dock's.

## Scope
### In
- A center item that hosts a dock panel's own entity. The panel stays registered in its dock, so
  Zed's code still finds it (`workspace.panel::<P>()`, `focus_panel`, `open_panel`); the dock never
  draws it. Its tab shows the panel's button icon and tooltip name (Project Panel, Git Panel,
  Outline Panel), and it gets its rail row as any center tab does (AD-claude-674).
- Every way Zed shows one of the three panels opens its tab in the active pane of the workspace
  whose dock holds the panel, or brings that tab forward where it is, focused: the status-bar
  button, the panel's `ToggleFocus` and `Toggle` actions and their keys (Ctrl+Shift+E,
  Ctrl+Shift+G, Ctrl+Shift+B), `workspace::ToggleRightDock`, and Zed's own calls (`focus_panel`,
  the panel's `PanelEvent::Activate`, as Reveal in Project Panel sends).
- One tab per panel per group. A drag, the tab bar's Split menu and the split keys move the tab;
  nothing copies it.
- In the Marley layout a dock never shows one of the three. A dock that opens on one closes before
  the next frame; when it showed another panel just before (the Agent Panel), it goes back to that
  panel and stays open. A dock restored open from the last session stays closed.
- The panel's `set_active` follows its tab: true while the tab is its pane's front item, false
  when it goes behind or closes.
- The outline panel keeps following the editor while its own tab is the active item: a small hunk
  in `crates/outline_panel/src/outline_panel.rs`, with its ledger row.
- The Zed layout keeps its docks: switching to it closes every panel tab, and a saved panel tab is
  not restored there.
- `docs/marley/guide.md` and `docs/marley/walkthrough.md`: side panels open as tabs.

### Out (explicitly deferred)
- A panel tab after a restart: #773 (D9), split off at promotion.
- The collab panel, Fleet, Knowledge and Containers, and the Debug button: #748, on this tab.
- The Agent Panel. Its status-bar button is #744's (the New Agent picker); the panel itself stays
  in the right dock (#748 says so and asks Chad).
- Another place for a file opened from a Files tab that sits in a split of its own: Zed's rule
  holds here (D7).
- A toggled look on a status-bar button whose tab is in front: Zed's `PanelButtons` draws it only
  for an open dock (`dock.rs` 1424).
- A Files tab at a project's first open (`project_panel.starts_open`): D8.

## Reference (§20)
Upstream Zed, `workspace`: the `Item` and `Pane` model (tabs, drag, split, `can_split`), and the
`Dock`, `Panel` and `PanelButtons` model. Behavior kept: each panel's own view, keys, actions and
settings; the status bar's buttons, their order and their right-click menus; the panes' drag and
split; Zed's layout as it is. What Marley adds, in its layout only: the panel drawn by a center item
instead of its dock.

### Prior art
- **The code we ship:**
  - `workspace::item::Item`: `can_split` is false by default (`item.rs` 264), and Zed moves an
    item that cannot split wherever a split would clone it: the tab drag (`pane.rs` 3914-3938) and
    `move_item_to_pane_at_index` / `move_item_to_pane_in_direction` (`workspace.rs` 5705, 5981).
    The default `pane::SplitRight` and its siblings clone (`SplitMode::ClonePane`, `pane.rs`
    216-224), and `split_and_clone` does nothing for such an item (`workspace.rs` 6355); the tab
    bar's Split menu sends the `MovePane` twins once the pane holds two items (`pane.rs`
    4359-4418). Hooks: `deactivated` (`item.rs` 216), `on_removed` (218), `pane_changed` (220),
    `added_to_workspace` (362); `Pane::activate_item` calls `deactivated` on the item it hides
    (`pane.rs` 1490) and emits `ActivateItem` (1500).
  - `workspace::dock`: `Dock::render` draws only its visible entry, the active panel while the
    dock is open (`dock.rs` 1271-1373); `set_open_internal` tells the active panel
    `set_active(open)` (585-594); `activate_panel_internal` tells the old and the new active panel,
    open or not (945-963); `PanelEvent::Activate` opens the dock, activates and focuses the panel
    (761-771); a saved dock waits for a panel of its saved name and then opens
    (`restore_serialized_state`, `replay_pending_serialized_state`, 828-889); `set_open` drops a
    pending restore when the open state changes (578-583). `PanelButtons` sends a panel's
    `toggle_action`, or the dock's toggle for the open dock's active panel (1391-1453); the right
    dock's buttons are reversed (1563-1565).
  - `workspace.rs`: the three `PanelButtons`, the left dock's a left status item, the right and
    bottom docks' right items (2031-2043); `focus_or_unfocus_panel` activates, opens and focuses
    the panel (4856-4899); `open_panel` opens without focus (4902-4911); `toggle_dock` (4626);
    `capture_dock_state` saves each dock's open state and active panel (2559-2600) and
    `load_workspace` hands them back (8045-8057); `recent_active_item_by_type` (3038).
  - The panels. `ProjectPanel` (`project_panel.rs` 8209-8283): `starts_open` (8257), `set_active`
    preloads icons (8270), Reveal in Project Panel emits `PanelEvent::Activate` (757-767), files
    open through `open_path_preview` with no pane (946-957), which takes
    `last_active_center_pane` (`workspace.rs` 5170-5178). `GitPanel` (`git_panel.rs` 9488-9560):
    no `set_active`; its Close action emits `PanelEvent::Close` (2082-2084); the commit modal opens
    it behind itself and restores the dock after (`commit_modal.rs` 158-171). `OutlinePanel`
    (`outline_panel.rs` 5588-5690): `set_active` (5644-5681); it follows `workspace.active_item()`
    and clears when that is no editor (1090-1114, `workspace_active_editor` 5564-5573).
  - None of the three zooms: only the Agent, Debug and Terminal panels emit `PanelEvent::ZoomIn`.
  - Marley's own: `ThreadTab` (#697) hosts the Agent Panel's `ConversationView` in a center tab
    and keeps it drawn in one place; its restore (#736) waits for the workspace's panel
    (`thread_tab.rs`, `PANEL_WAITS`); the Graph tab keeps its rows in a table of its own
    (`rusty/graph_store.rs`); the layout switch keeps a dock memory from outside the `workspace`
    crate with Zed's public dock API (AD-claude-456, `marley_workbench.rs` 1099-1213); `routing.rs`
    catches actions at the workspace's root in the capture phase (32-58).
  - Zed's own test item `ThreadViewItem` hosts a panel's view in an item
    (`agent_ui/src/conversation_view.rs` 7906).
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` §1.6 (`Dock`,
  `Panel`, `PanelHandle`) and the item model. No Warp or Orca map has a side panel in a tab.
- **Published material:** none. Upstream Zed has no center place for a panel
  (`settings_content::DockPosition` is Left, Bottom, Right).

## UI proof
Scenario `script/e2e/747-side-panels-as-tabs.sh`, under `compositor sway`. A scratch repository
with a Rust file and an uncommitted change. The first launch runs in Zed's layout
(`marley.layout` "zed") with the right dock open on the project panel, then quits; the profile is
switched to the Marley layout and Marley starts again.

Shots:
- `restored`: the restored window, no right dock (REQ-008).
- `files`, `git`, `outline`: each status-bar button clicked: its tab in the repo's active pane,
  focused, no dock (REQ-001); `outline` lists the open file's symbols (REQ-006).
- `key`: Ctrl+Shift+E from the editor brings the Files tab forward (REQ-002).
- `again`: the Git button clicked while Git's tab is behind another: the same tab forward, one Git
  tab (REQ-003).
- `reveal`: the editor's Reveal in Project Panel: the Files tab forward with the file selected
  (REQ-004).
- `drag`: the Outline tab dragged to the pane's right edge: a split with Outline on the right and
  no Outline tab left in the first pane (REQ-005).
- `split-key`: Ctrl+\ on the Git tab: Git moves to a new pane (REQ-005).
- `outline-used`: a symbol clicked in the split Outline tab: the editor scrolls there and the
  outline still lists the file's symbols (REQ-006).
- `open-file`: a file double-clicked in the Files tab: its editor in front in the Files tab's pane
  (REQ-009).
- `agent-stays`: the Agent Panel open in the right dock, then the Files button: the Files tab
  opens and the dock stays open on the Agent Panel (REQ-007).
- `after-restart`: Marley quit and started again: the three tabs where they were (REQ-010).
- `zed-layout`: `marley: use zed layout`: no panel tab; the Files button then opens the right dock
  (REQ-011).

## Locked-In Decisions
- **D1 — A Marley item hosts the panel's own entity**, the panel staying registered in its dock.
  Zed's code looks panels up in the docks, so removing a panel from its dock would break Rename,
  the commit modal, the agent's and the rail's calls. Rejected: a Zed hunk that adds a center
  `DockPosition` (a change across `workspace`, every panel's `position_is_valid` and the settings
  schema); a second panel entity made for the tab (two project panels on one project, each with its
  own selection and state).
- **D2 — One catch for every path: a dock observer.** In the Marley layout each workspace observes
  its docks. When a dock is open and its active panel is one of the moved panels, the observer
  closes it, gives the dock back the panel it showed before when that was another one (and keeps it
  open), and, when the moved panel holds the focus (its activation focus handle is the window's
  focus, as `focus_or_unfocus_panel` and `PanelEvent::Activate` leave it), opens or brings forward
  its tab. Observers run in the effect flush of the update that opened the dock, before the frame
  draws, so the dock never shows the panel. An open that leaves the focus elsewhere (a restored
  dock, `starts_open`, `open_panel`, the commit modal) only closes. Rejected: catching each panel's
  `ToggleFocus` and `Toggle` in the capture phase (it needs `project_panel` and `outline_panel` as
  dependencies, still misses `focus_panel`, `PanelEvent::Activate` and `ToggleRightDock`, and
  becomes a second path beside the observer).
- **D3 — The moved panels are named by persistent name**: `Project Panel`, `Git Panel`,
  `Outline Panel` here; #748 adds the rest. Handles come from the docks (`Dock::active_panel`,
  `panel_for_id` with the `workspace::Event::PanelAdded` view's id), so `marley_workbench` gains
  no crate dependency.
- **D4 — One tab per panel per group; a split moves it.** The opener looks through the workspace's
  panes for a tab of that panel and brings it forward. The item keeps `can_split` false, so a drag
  and the Split menu move it; the split keys (`pane::SplitLeft`, `SplitRight`, `SplitUp`,
  `SplitDown`, whose default mode clones and does nothing for such an item) are caught in the
  capture phase while a panel tab is the active item and sent as their `SplitAndMove` twins. A tab
  that lands in another workspace closes itself (`added_to_workspace`); the rail moves only
  terminals between groups (#613).
- **D5 — `set_active` has one source: the tab.** Zed's dock still calls it on its active panel
  when the dock opens, closes or changes its active panel, even closed (`dock.rs` 945-963). After
  each change of a dock the observer re-tells the dock's previous and current active panels what
  their tabs show, where the dock's call said otherwise.
- **D6 — The outline follows the editor, not its own tab.** A hunk in `outline_panel.rs` at its two
  reads of the active item (the `ActiveItemChanged` subscription, 1097, and `set_active`, 5644):
  when the active item is the one that hosts the panel (its focus handle is the panel's), the panel
  keeps the editor it follows, or takes `recent_active_item_by_type::<Editor>` when it follows
  none yet. About ten lines, marked `// Marley:`, with a row in `docs/marley/zed-touchpoints.md`.
  Rejected: the tab answering `act_as::<Editor>` with the last editor (the outline would store the
  tab as the item to activate, and every other `act_as` reader would see an editor that is not
  there).
- **D7 — A file opened from the Files tab opens where Zed opens it**: `last_active_center_pane`,
  which is the Files tab's own pane once it has been used. In one pane the file comes forward over
  Files; with Files split out, it opens beside Files in that pane. Rejected for now: steering it to
  another pane (a hook in `project_panel.rs`'s `open_path_preview` call). That rule is Chad's to
  pick.
- **D8 — A tab opens only when asked for.** A panel that starts open (`project_panel.starts_open`,
  on by default) or a restored dock gives no tab; the terminal stays the first thing a fresh
  project shows (#455).
- **D9 — A panel tab comes back after a restart (#773, split off at promotion).** The item is serializable; its row (workspace
  id, item id, persistent name) sits in a small table of its own, as the Graph tab's do. The
  restore waits for the workspace's panel of that name, as the thread tab waits for the Agent
  Panel; it refuses in Zed's layout, for a name outside the moved list, and for a panel that never
  arrives, and the cleanup drops the row.
- **D10 — The Zed layout has no panel tabs.** The switch to it closes them; the docks are Zed's
  again. The switch back to the Marley layout opens no tab.
- **D11 — The panel's own Close event (`PanelEvent::Close`, the git panel's Close action) does not
  close the tab**; the tab closes as any tab does.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the project, git or outline panel's status-bar button is clicked in the Marley layout, the system shall open that panel as a tab in the active pane of the shown group, focused, with the right dock closed. | shots `files`, `git`, `outline` |
| REQ-002 | WHEN a panel's key or palette action (Ctrl+Shift+E, Ctrl+Shift+G, Ctrl+Shift+B) runs in the Marley layout, the system shall open or bring forward that panel's tab. | shot `key` |
| REQ-003 | WHEN a panel whose tab is open in the group is asked for again, the system shall bring that tab forward where it is and open no second one. | shot `again` |
| REQ-004 | WHEN Zed's own code shows one of the three panels (Reveal in Project Panel), the system shall open or bring forward its tab instead of the dock. | shot `reveal` |
| REQ-005 | WHEN a panel tab is dragged to a pane's edge or split with a split key, the system shall move it into the new pane and leave no copy. | shots `drag`, `split-key` |
| REQ-006 | While its own tab is the active item, the outline panel shall keep listing the symbols of the editor it follows. | shots `outline`, `outline-used` |
| REQ-007 | WHEN one of the three panels is asked for while a dock is open on another panel, the system shall open the tab and leave that dock open on the panel it showed. | shot `agent-stays` |
| REQ-008 | WHEN a window restores a session whose dock was open on one of the three panels, the system shall keep the dock closed and open no tab. | shot `restored` |
| REQ-009 | WHEN a file is opened from the Files tab, the system shall open it in the pane Zed's rule names, the Files tab's own pane once used. | shot `open-file` |
| REQ-011 | WHEN the layout switches to Zed's, the system shall close every panel tab, and each panel's button shall open its dock as Zed does. | shot `zed-layout` |
| REQ-012 | The system shall tell a moved panel `set_active(true)` while its tab is its pane's front item and `set_active(false)` otherwise. | review of the diff; shot `outline-used` (the outline updates) |
| REQ-013 | The change shall pass `script/gates.sh --diff`. | the gate's exit code |

## Phase Plan
- **P1 Plan** — this spec; at promotion the design in the notes, every cited seam checked again.
- **P2 Code** — README marker present (§3); the ledger row for `outline_panel.rs` first
  (`enforce-zed-ledger.sh`); a new `panel_tabs.rs` in `marley_workbench` (the item, its table, the
  dock observer, the split keys, the layout switch), its `init` in `marley_workbench.rs`, the
  outline hunk; the guide and walkthrough lines; a review of the diff; `script/gates.sh --diff`
  green.
- **P3 Test** — the visual check: write and run `script/e2e/747-side-panels-as-tabs.sh`, read every
  shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21: `marley_workbench.md`,
  `workbench-shell.md`, the ledger row checked), ledger capture (§19), close the ticket, archive,
  commit.
