---
pipeline_id: b980669b-4870-460d-8c46-a03d54b74be3
ticket: docs/planning/tickets/closed/TICKET-798-agent-panel-width-in-percent.md
status: Phase 4 — Complete PASS
title: The agent panel's width in percent
type: feature
slice: Cross-cutting (the agent panel)
references:
  - docs/planning/design-notes/agent-components-2026-10-17.md
---

## Title
A button in an agent thread's message editor shows the width of the thread's place in percent, and
each click steps it to the next size: a third, half, two thirds, the whole window, then a third
again. The place is the Agent Panel's dock (its share of the window), or, in the Marley layout,
where threads open in center tabs (#734), the pane that holds the thread's tab (its share of its
row).

## Scope
### In
- The button, in the left group of the message editor's bottom row
  (`agent_ui/src/conversation_view/thread_view.rs`, `render_message_editor`, beside the follow
  toggle). Its label is the place's width over the workspace's, rounded to a whole percent, and
  100% while the place is zoomed; its tooltip names the sizes it steps through. It shows:
  - while the thread is the Agent Panel's (the dock's width);
  - while the thread's tab is in front in a pane that shares its row with another pane (the pane's
    width). A pane alone in its row has nothing to share its width with and shows no button.
- A click runs `marley::CycleAgentPanelWidth` for that thread: from the current width, the next
  size up in a third, half and two thirds; past two thirds the place zooms (Zed's `ToggleZoom`, the whole
  window); a click while zoomed unzooms it at 33 percent.
  - The dock's share goes through the workspace's dock sizing, the path a drag of its edge takes,
    so with Zed's flexible size on (`agent.flexible`) it keeps the share as the window resizes,
    and the workspace keeps it across a restart.
  - A pane's share goes through the center's pane resizing, the path a drag of a pane's edge
    takes, taking the width from its neighbor in the row.
- The palette's `marley: cycle agent panel width` does the same for the thread in front: the
  active pane's thread tab, else the Agent Panel's thread. No default key.

### Out (explicitly deferred)
- A setting for the sizes.
- Panels and tabs that are not agent threads.
- A pane's height, in a column of panes.

## Reference (§20)
Upstream Zed (`workspace`, `agent_ui`): the dock keeps a panel's width as a pixel size or, with
the flexible size on, as a flex share of the center (`dock.rs` `PanelSizeState`, `workspace.rs`
`dock_size` and `dock_flex_for_size`); the center keeps a row of panes as flex shares that a drag
of a pane's edge changes (`pane_group.rs` `PaneAxis::resize`); a panel and a pane both zoom over
the workspace (`ToggleZoom`). Marley adds a control over those; dragging an edge and zooming keep
working as they do.

### Prior art
- The code we ship: `Workspace::resize_dock` and `dock_flex_for_size` turn a pixel width into a
  dock's flex share and store it per workspace (`persist_panel_size_state`);
  `PaneGroup::bounding_box_for_pane` and `PaneGroup::resize` give and change a pane's width in its
  row; `AgentPanel::toggle_zoom` and `Pane::toggle_zoom` give the whole-window size.
  `MarleyThreadHost` (#734) is the way `agent_ui` already asks Marley about a thread's tab. The
  message editor's expand button (`toggle-height`) is the pattern for a small control in the row.
- Behavior maps: `docs/zed_architecture/` has no size control for a panel or a pane; Warp's
  agent mode has no such control in its docs.
- Published material: none bears on it.

## UI proof
Scenario `script/e2e/798-agent-panel-width-in-percent.sh`, under `compositor sway` (it clicks):
the debug Marley on a scratch project, a thread of #685's scripted ACP agent (a custom agent
server in the run's settings) so no model is needed.
- `798-01-tab`: the terminal's pane split right (Home opens in the new pane, #785) and the thread
  started there: the button shows the thread pane's share of its row, 50% (REQ-001).
- `798-02-sixtysix`: a click steps the pane to two thirds of its row, 67% (REQ-002);
  `798-02-zoomed`: `marley: cycle agent panel width` zooms it over the window, 100% (REQ-003,
  REQ-006).
- `798-03-back`: the next step: the pane at a third of its row, 33% (REQ-003).
- `798-04-lone`: the thread's tab alone in its row: no button (REQ-005).
- `798-05-panel`: in Zed's layout (the Marley layout hides Move Thread to Panel), the thread moved
  to the Agent Panel: the button shows the dock's share, 50%, Zed's flexible default (REQ-001).
- `798-06-sixtyseven`, `798-06-zoomed`, `798-06-third`: the palette's action steps the dock to 67%,
  zooms it, and brings it back at 33% (REQ-002, REQ-003).
- `798-07-relaunch`: after quitting and relaunching on the same profile, the panel comes back open
  at 33% (REQ-004).

## Locked-In Decisions
- **D1 — A dock's share of the window, a pane's share of its row.** The dock's share is the width
  the dock sizing divides. A pane's share is its flex in the nearest row of the center that holds
  it, the value a drag between two panes changes; its share of the window would need the panes'
  bounding boxes, which are locked while the panes lay out and so cannot be read in render.
- **D2 — Sizes a third, half, two thirds (labelled 33%, 50%, 67%) and the whole window, in that
  order.** A click goes to the smallest size larger than the current width by more than one
  percent, so a width dragged to 45% goes to 50%, and 50% goes to 67%.
- **D3 — The whole window is Zed's zoom**, for a dock and a pane: zoom already draws the place
  over everything and keeps its size under it.
- **D4 — The steps live in `marley_workbench`; Zed gets small additive hunks.** `workspace.rs`
  gains `marley_dock_width_share` and `marley_set_dock_width_share` (through the private
  `dock_size` and `resize_dock`), and `marley_pane_width_share` and `marley_set_pane_width_share`
  (through the private `center`'s new `marley_row_share` and `marley_set_row_share` in
  `pane_group.rs`). `agent_ui.rs`'s
  `MarleyThreadHost` gains `width`, the label Marley gives for a thread. `thread_view.rs` draws the
  button and dispatches `marley::CycleAgentPanelWidth` with the thread's view id by name
  (`cx.build_action`), so `agent_ui` names no Marley crate.
- **D5 — A pane alone in its row shows no button**: its width is the center's, and nothing beside
  it can give or take width.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a thread is the Agent Panel's, or its tab is in front in a pane that shares its row, the message editor's bottom row shall show that place's share in whole percent | shots `798-01-tab`, `798-05-panel` |
| REQ-002 | WHEN the button is clicked, the place shall take the next size in a third, half and two thirds, and the label shall read it | shots `798-02-sixtysix`, `798-06-sixtyseven` |
| REQ-003 | WHEN clicked at two thirds or wider, the place shall zoom over the window, and WHEN clicked while zoomed, it shall unzoom at 33 percent | shots `798-02-zoomed`, `798-03-back`, `798-06-zoomed`, `798-06-third` |
| REQ-004 | WHEN Marley restarts, the Agent Panel shall open at the width the button last set | shot `798-07-relaunch` |
| REQ-005 | WHILE a thread's tab is alone in its row, its message editor shall show no width button | shot `798-04-lone` |
| REQ-006 | WHEN the palette action runs, the thread in front shall step as a click does | shots `798-02-zoomed`, `798-06-*`; the review of the diff (one handler for both) |

## Phase Plan
- **P1 Plan:** this spec and the design in the notes.
- **P2 Code:** the workspace and `agent_ui` hunks with their rows, the action in
  `marley_workbench`; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test:** the visual check: the scenario above, every shot read.
- **P4 Complete:** CHANGELOG, `docs/marley/guide.md`, ledger capture, close, archive, commit.
