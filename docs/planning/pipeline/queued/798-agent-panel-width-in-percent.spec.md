---
pipeline_id: b980669b-4870-460d-8c46-a03d54b74be3
ticket: docs/planning/tickets/open/TICKET-798-agent-panel-width-in-percent.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: The agent panel's width in percent
type: feature
slice: Cross-cutting (the agent panel)
references:
  - docs/planning/design-notes/agent-components-2026-10-17.md
---

## Title
A button in the agent panel's message editor shows the panel's width as a percent of the window,
and each click steps it to the next size: 33, 50, 66, the whole window, then 33 again.

## Scope
### In
- The button, in the left group of the message editor's bottom row
  (`agent_ui/src/conversation_view/thread_view.rs`, `render_message_editor`, beside the follow
  toggle), shown only while the thread is in the agent panel's dock, not in a center tab (#697,
  #734). Its label is the panel's width over the workspace's, rounded to a whole percent; its
  tooltip names the next size and the action's key.
- A click: from the current width, the next size up in 33, 50, 66 percent sets the panel's size
  through `Workspace::set_panel_size_state::<AgentPanel>`, with the pixel size and the flex
  `Workspace::dock_flex_for_size` gives for it, so the panel keeps the share when the window
  resizes (Zed's flexible size, `agent.flexible`, on by default). Past 66 percent the panel zooms
  (Zed's `ToggleZoom`, the whole window), and a click while zoomed unzooms it at 33 percent.
- A palette action, `marley: cycle agent panel width`, that does the same as a click, with no
  default key.

### Out (explicitly deferred)
- A setting for the sizes.
- The same control for other panels, or for the agent panel docked at the bottom (the agent
  panel cannot dock there: `position_is_valid` refuses `Bottom`).
- A thread in a center tab: its tab's width is the pane's, not the panel's.

## Reference (§20)
Upstream Zed (`workspace`, `agent_ui`): the dock keeps a panel's width as a pixel size or, with
the flexible size on, as a flex share of the center (`dock.rs` `PanelSizeState`,
`workspace.rs` `dock_size` and `dock_flex_for_size`); the agent panel's toolbar has the zoom
button (`agent_panel.rs`, `ToggleZoom`). Marley adds a control over those two; dragging the edge
and zooming keep working as they do.

### Prior art
- The code we ship: `Workspace::set_panel_size_state`, `dock_flex_for_size` and
  `toggle_dock_panel_flexible_size` already turn a pixel width into a flex share and store it per
  workspace (`persist_panel_size_state`); `AgentPanel::set_zoomed` is the whole-window size. The
  message editor's expand button (`toggle-height`, `ExpandMessageEditor`) is the pattern for a
  small icon control with an action tooltip.
- Behavior maps: `docs/zed_architecture/` has no panel-size control; Warp has no side agent
  panel of this kind.
- Published material: none bears on it.

## UI proof
Scenario `script/e2e/798-agent-panel-width-in-percent.sh`, under `compositor sway` (it clicks):
the debug Marley on a scratch project with the agent panel open on a Zed-agent thread.
- `798-01-label`: the button in the bottom row shows the panel's width in percent (REQ-001).
- `798-02-fifty`: after a click from the default width, the panel is half the window and the
  label reads 50% (REQ-002).
- `798-03-sixtysix`: a second click: two-thirds, 66% (REQ-002).
- `798-04-zoomed`: a third click: the panel covers the window (REQ-003).
- `798-05-back`: a fourth click: the panel is a third of the window, 33% (REQ-003).
- `798-06-relaunch`: after quitting and relaunching on the same profile, the panel opens at 33%
  (REQ-004).
- `798-07-tab`: the same thread opened in a center tab shows no button (REQ-005).

## Locked-In Decisions
- **D1 — Percent of the workspace's width.** The label and the sizes are the panel's width over
  the workspace's bounds, the width `dock_size` divides.
- **D2 — Sizes 33, 50, 66 and the whole window, in that order.** A click goes to the smallest
  size larger than the current width by more than one percent, so a width dragged to 45 goes to
  50, and 50 goes to 66.
- **D3 — The whole window is Zed's zoom**, not a 100 percent dock: a dock at the window's width
  leaves no center, and zoom already draws the panel over everything and remembers the size under
  it.
- **D4 — The steps live in `marley_workbench`; Zed gets two small hunks.** `workspace.rs` gains
  `marley_dock_width_share` (the dock's width over the workspace's, through the private
  `dock_size` the layout uses) and `marley_set_dock_width_share` (through the private
  `resize_dock` a drag of the edge uses, so the flex share and the stored size follow as they do
  for a drag). `thread_view.rs` draws the button and dispatches `marley::CycleAgentPanelWidth` by
  name (`cx.build_action`), so `agent_ui` names no Marley crate. The action, the sizes and the zoom
  are `marley_workbench`'s.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a thread is shown in the agent panel, the message editor's bottom row shall show the panel's width in whole percent | shot `798-01-label` |
| REQ-002 | WHEN the button is clicked, the panel shall take the next size in 33, 50, 66 percent and the label shall read it | shots `798-02-fifty`, `798-03-sixtysix` |
| REQ-003 | WHEN clicked at 66 percent or wider, the panel shall zoom to the whole window, and WHEN clicked while zoomed, it shall unzoom at 33 percent | shots `798-04-zoomed`, `798-05-back` |
| REQ-004 | WHEN Marley restarts, the panel shall open at the size the button last set | shot `798-06-relaunch` |
| REQ-005 | WHILE a thread is shown in a center tab, its message editor shall show no width button | shot `798-07-tab` |
| REQ-006 | WHEN the palette action runs, the panel shall step as a click does | the review of the diff (one handler for both) |

## Phase Plan
- **P1 Plan:** this spec; the design in the notes at promotion (where the view learns it is in
  the dock; the action's namespace and `test_action_namespaces`).
- **P2 Code:** the button and the action in `agent_ui`, the touchpoints row; a review of the
  diff; `script/gates.sh --diff` green.
- **P3 Test:** the visual check: the scenario above, every shot read.
- **P4 Complete:** CHANGELOG, `docs/marley/guide.md`, ledger capture, close, archive, commit.
