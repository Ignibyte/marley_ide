---
pipeline_id: 975013ba-2fe6-4687-8b74-4a5c9b80c6b1
ticket: docs/planning/tickets/closed/TICKET-456-docks-across-a-layout-switch.md
status: Phase 4 — Complete PASS
title: Each dock as it was across a layout round trip
type: bug
slice: workbench shell W6g
references: [docs/marley/workbench-shell.md, docs/planning/pipeline/completed/451-marley-layout-presets.spec.md, docs/planning/pipeline/completed/438-marley-layout-and-rail.notes.md]
---

## Title
A layout switch moves the Agent Panel between docks, and Zed's dock code then forgets what the
dock that took it over was showing. With the right dock open on another panel and the Agent
Panel open on the left, a switch to the Marley layout and back left the right dock closed, with
its panel lost (#438 inspect S9). The switch now remembers what the Agent Panel displaced, per
workspace, and gives it back when the Agent Panel leaves again.

## Scope
### In
- **The memory** (`marley_workbench`, in the layout switch). When the layout setting changes,
  before any dock moves, the switch notes each workspace's three docks: open or not, their
  active panel's persistent name, and which one holds the Agent Panel. After the docks have
  moved, a deferred pass settles each workspace:
  - **Remember.** The dock the Agent Panel entered, having shown another panel before,
    remembers that panel and whether it was open, whether or not the move made the Agent Panel
    its shown panel.
  - **Give back.** The dock the Agent Panel left, when it remembers something and the Agent
    Panel was the panel it showed, activates the remembered panel. It opens when it was open
    before the trip and still was as the Agent Panel left. Otherwise the memory is dropped: the
    user chose another panel in between, and that choice stands.
- The memory lives in the switch's global, keyed by workspace, and is pruned when a workspace
  is gone.

### Out (explicitly deferred)
- Keeping the memory across a restart: a window saved in one layout and restored in the other
  (the sweep's adjacent risk) is its own ticket if it bites.
- The Agent Panel's own focus: the move keeps Zed's behavior.
- Any dock change not caused by the Agent Panel moving (Zed's Panel Layout presets are already
  answered with a toast in the Marley layout, #451).

## Reference (§20)
- **Upstream Zed:** the dock move (`crates/workspace/src/dock.rs`).
  - Each panel's `SettingsStore` observer moves it when its position changes. A panel that was
    visible opens its new dock and becomes its active panel (`:638-705`).
  - `Dock::remove_panel` closes a dock whose active panel leaves, keeping no record of it
    (`:895-926`).

  Marley keeps Zed's move and adds the memory around it, from outside the crate, with the dock
  API Zed exposes:
  - `Dock::is_open`, `active_panel`, `panel_index_for_persistent_name`, `activate_panel`,
    `set_open` and `has_agent_panel`;
  - `Workspace::dock_at_position`.
- **Warp:** N/A. The Marley layout's docks are Zed's.

### Prior art
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` (docks and
  panels).
- **Published material:** none.
- **Code we already ship.**
  - The move itself: the Agent Panel's position is `AgentSettings::dock`
    (`crates/agent_ui/src/agent_panel.rs:5046-5048`). The Marley switch patches it in
    `apply_defaults` (`crates/marley_workbench/src/marley_workbench.rs:277-290`), called from
    the layout observer `layout_setting_changed` (`:255-275`).
  - Global observers run in the order they were registered, and the switch's observer is
    registered at init, before any dock's. So the docks have not moved when it runs, and a
    `cx.defer` from it runs after the move.
  - The move schedules `serialize_workspace` behind its 200 ms throttle
    (`dock.rs:699-703`, `workspace.rs:177`), so the state the deferred pass leaves is the one
    saved.
  - Zed keeps no memory of a displaced panel. The nearest mechanisms,
    `last_open_dock_positions` and `PreviousWorkspaceState`, do not cover this.
  - `DockData { visible, active_panel, zoom }` names panels by persistent name
    (`crates/workspace/src/persistence/model.rs:203-208`), so the memory does too.
- **The ledger:** `L-claude-449-driving-keys-and-docks-in-a-gpui-test-001` (`TestPanel` as a
  stand-in panel, and docks sorting panels by activation priority).

## UI proof
UI-AFFECTING.
- **Driven tests:** a window in the Zed layout with the Agent Panel open on the left and a
  `TestPanel` open on the right.
  - A switch to the Marley layout and back leaves the right dock open on the `TestPanel` and the
    left open on the Agent Panel.
  - With the right dock closed before the trip, it ends closed with the `TestPanel` active.
  - A panel chosen on the right while in the Marley layout stays chosen.
  - With the right dock closed during the trip, it stays closed.
  - With the Agent Panel hidden before the trip, no dock changes.
- **Live drive:** the reported round trip; screenshot before and after. It needs clicks, so it
  runs only while Chad is away from the desk; otherwise the Test phase records why.

## Locked-In Decisions
- D1 — Zed's move stays as it is. Marley remembers what it displaced and gives that back from
  outside the `workspace` crate, with Zed's public dock API: no touchpoint.
- D2 — Before the move, the switch's own observer notes the docks. The restore runs in a
  `cx.defer` after the move, so it is the state saved.
- D3 — Something is given back only while the Agent Panel is the panel the dock shows. A choice
  the user made in between stands, and a dock the user closed stays closed.
- D4 — The memory is per workspace, in memory only, and names panels by persistent name, as
  Zed's saved dock state does.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the layout switches to the Marley layout and back with the Agent Panel visible, each dock shall be open or closed as before the trip, showing the panel it showed | driven tests |
| REQ-002 | WHEN the user shows another panel in the dock the Agent Panel took over, before switching back, that dock shall keep the user's panel | driven test |
| REQ-003 | WHEN the user closes that dock before switching back, it shall stay closed, with the panel it showed before the trip active | driven test |
| REQ-004 | WHEN the layout switches with the Agent Panel hidden, no dock shall change beyond the Agent Panel's own move | driven test |
| REQ-005 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — the capture, the deferred settle and the memory in the layout switch; fmt and
  clippy clean; a review of the diff.
- **P3 Test** — write and run the tests, with negative checks; the live drive;
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the
  ticket, archive, commit.
