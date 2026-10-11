---
pipeline_id: 41b67bff-4204-4bcd-acad-edd792ed8981
ticket: docs/planning/tickets/closed/TICKET-773-panel-tabs-after-a-restart.md
status: Phase 4 — Complete PASS
title: Panel tabs come back after a restart
type: feature
slice: the Marley layout (docs/marley/workbench-shell.md), tabs everywhere (docs/planning/design-notes/tabs-everywhere-2026-10-11.md)
references:
  - docs/planning/pipeline/completed/747-side-panels-as-tabs.spec.md
  - docs/planning/pipeline/completed/748-more-side-panels-as-tabs.spec.md
  - docs/planning/pipeline/completed/736-thread-tabs-after-a-restart.spec.md
---

## Title
Panel tabs come back after a restart. #747 and #748 draw Files, Git, Outline, Collab, Fleet,
Knowledge and Containers as center tabs in the Marley layout, but the tab is not a serializable
item, so a restart loses every one of them while the editors beside them come back. This ticket
is #747's D9.

## Scope
### In
- `PanelTab` as a Zed `SerializableItem`: a row per tab in a table of its own, the restore, the
  refusals and the cleanup.
- The tab built in one place (`PanelTab::new`) for both a fresh open and a restore.
- Found at Test: the Outline tab follows the editor used last while any non-editor item is
  active (the outline hunk), and the dock observer re-tells the dock's previous panel that its
  tab is in front.

### Out (explicitly deferred)
- The panel's own state (folders expanded, scroll, the git panel's selection): each panel keeps
  what Zed's own serialization keeps for it, which this ticket leaves alone.
- A group whose workspace does not open at launch: Zed opens only the window's active workspace
  (L-claude-507-a-window-restores-only-its-active-workspace-001), and the others restore their
  items, panel tabs among them, when they open.
- Zed's layout: it has no panel tabs (#747's D10) and restores none.

## Reference (§20)
Upstream Zed, `workspace`: an item survives a restart by implementing `SerializableItem` and being
registered with `register_serializable_item`; the pane layout records each item's kind and id,
`SerializedPane::deserialize_to` (`persistence/model.rs` 355) asks each kind to rebuild its item
and adds the ones that succeed, and `delete_unloaded_items` drops the rows of items no longer
alive. Behavior kept: the tab comes back in its pane and its place, as an editor does; a tab that
fails to restore leaves nothing, with Zed's log line. What Marley adds: the item it rebuilds draws
a dock panel the workspace already has.

### Prior art
- **The code we ship:**
  - Marley's thread tab (#736, `thread_tab.rs`): its own table `marley_thread_tabs` keyed by
    workspace and item, rows read once at init into a global, the restore waiting in its own task
    for the Agent Panel every 100 ms for up to 10 s. This ticket takes the same shape.
  - The Graph tab (#657, `rusty/graph_tab.rs`) and the Browser tab (`browser.rs`): the same
    pattern, with the refusal when Rusty is off (the Graph tab) as the model for refusing in
    Zed's layout.
  - Zed's item registration fires `enqueue_item_serialization` when an item joins a pane
    (`item.rs` 761), so a tab whose state never changes is written once without an event.
  - `Dock::panel_index_for_persistent_name` and `panel_for_id` (`dock.rs` 533, 545) find a panel
    by its persistent name, the key the row keeps.
- **No crate owns a panel drawn as an item:** Zed restores a panel only into its dock
  (`Dock::restore_state`), and no dependency in `Cargo.lock` touches the workspace's items.
- **Lessons in the ledger:** L-claude-736-a-restored-item-waits-for-its-workspaces-panels-001
  (Zed adds panels and restores items in one load, no order), PR-claude-a-row-a-restore-reads-
  is-read-before-any-cleanup-001 (read the rows at init), L-claude-601 and L-claude-670 (a
  relaunch with a path restores that workspace's items but not the window's state).

## UI proof
Scenario `script/e2e/773-panel-tabs-after-a-restart.sh` (`compositor sway`): a scratch git repo
with a Rust file; Files, Git, Outline and Containers opened as tabs, Files split to the right;
Marley quits and starts again three times.

| Shot | Shows | Criteria |
|---|---|---|
| `773-01-before` | the tabs at the quit: Files alone in the right pane; Git, lib.rs, Outline, Containers on the left | setup |
| `773-02-restored` | after the restart, the same tabs in the same panes, each drawing its panel | REQ-001 |
| `773-03-outline` | the restored Outline tab brought forward by its button, listing lib.rs's symbols | REQ-002, REQ-003 |
| `773-04-again` | the Files button after the restart: the restored Files tab forward on the right, no second; the Outline tab still listing | REQ-002, REQ-003 |
| `773-05-disabled` | a start with Containers turned off: Files, Git and Outline back, no Containers tab, the Outline tab listing from the start | REQ-004, REQ-002 |
| `773-06-zed-layout` | a start in Zed's layout: no panel tab in the center | REQ-005 |

## Locked-In Decisions
- **D1 — `PanelTab` is a `SerializableItem`** of kind `MarleyPanelTab`, registered in
  `panel_tabs::init`. One Zed hunk, found at Code: `Dock::panel_for_persistent_name`, since the
  dock hands out a panel only by type or entity id.
- **D2 — Its row** is (workspace id, item id, the panel's persistent name) in a table of its own,
  `marley_panel_tabs`, keyed by workspace and item with no unique item id (ids repeat across
  launches, #576), cascading on the workspace's delete. The rows are read once at init into a
  global, and a restore reads that global, never the table
  (PR-claude-a-row-a-restore-reads-is-read-before-any-cleanup-001). The row is written when the
  tab joins a pane; `should_serialize` answers false, since nothing about the tab changes.
- **D3 — The restore refuses at once** in Zed's layout, for an item with no saved row, and for a
  name outside the moved list. Otherwise it waits in its own task for a panel of that name in any
  of the workspace's docks, every 100 ms for up to 10 s, as the thread tab waits for the Agent
  Panel; it refuses a panel that never arrives, a disabled one (`Panel::enabled`: Containers
  hidden, Knowledge with Rusty off), and a panel whose tab the group already shows (#747's D4,
  one tab per panel per group).
- **D4 — A refusal leaves no tab**: Zed logs it, and the cleanup (`delete_unloaded_items`) drops
  every row whose item is not alive, the refused ones among them.
- **D5 — One constructor.** `PanelTab::new(panel, workspace, window, cx)` takes the name, icon
  and focus handle from the panel; `show_tab` and the restore both call it, so a restored tab is
  the tab a click opens (PR-claude-restore-is-a-second-constructor-001).
- **D6 — Where the tab comes back is Zed's**: the pane layout's own record places it in its pane
  at its index. Marley adds nothing to the placing.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley starts again in the Marley layout after a quit with panel tabs open, the system shall restore each panel tab in the pane it was in, drawing its workspace's panel. | shot `restored` |
| REQ-002 | WHILE a restored Outline tab is in front, the system shall list the symbols of the editor last active in its group, whichever item has the focus. | shots `outline`, `again`, `disabled` |
| REQ-003 | WHEN a panel whose tab was restored is asked for, the system shall bring the restored tab forward and open no second one. | shots `outline`, `again` |
| REQ-004 | WHEN a saved panel tab's panel is disabled at the start, the system shall not restore that tab, and its row shall be dropped. | shot `disabled`; the row's absence after the next quit |
| REQ-005 | WHEN Marley starts in Zed's layout with panel tabs saved, the system shall restore none of them. | shot `zed-layout` |
| REQ-006 | IF a saved row names a panel outside the moved list, or its panel never arrives, THEN the system shall restore no tab for it. | review of the diff (no scenario writes such a row) |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `panel_tabs.rs`: `PanelTab::new`, the table, the saved-rows global, the
  `SerializableItem` impl, the registration; a review of the diff; `script/gates.sh --diff`
  green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
