# The agent panel's width in percent — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-798-agent-panel-width-in-percent.md
- **Pipeline spec:** 798-agent-panel-width-in-percent.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17: "Can we make the agent window able to expand the size in
  percentage. Maybe a toggle at the bottom?" He confirmed the plan's click-to-step button.
- **Discovery (drafted by /spec; re-verified at promotion):**
  - `crates/workspace/src/dock.rs`: `PanelSizeState { size, flex }`, `resize_panel_entry`, and
    `panel_uses_flexible_width` (a horizontal dock with the panel's flexible size on).
  - `crates/workspace/src/workspace.rs`: `set_panel_size_state::<T>`, `persist_panel_size_state`
    (the key-value store, per workspace), `dock_size`, `dock_flex_for_size`,
    `default_dock_flex` (1.0).
  - `crates/agent_ui/src/agent_panel.rs`: `supports_flexible_size` true, `has_flexible_size`
    from `AgentSettings::flexible` (`default.json`'s `agent.flexible: true`), `set_zoomed`,
    `toggle_zoom`.
  - `crates/agent_ui/src/conversation_view/thread_view.rs`: `render_message_editor`'s bottom
    row (left group: add context, follow, fast mode, thinking).
  - `crates/agent_ui/src/conversation_view.rs`: `is_visible_in_agent_panel`, with #734's
    `MarleyThreadHost::shows` for center tabs.
- **Recall (§18.3):** the ledgers hold nothing on dock sizes or the agent panel's width; the
  completed pipelines touch the agent panel only through the rail (#439, #440). The brain was not
  asked: #797's consultation that morning returned only due follow-ups, and this ticket decides
  nothing durable.
- **Promoted 2026-10-17** after #797; every cited seam re-read on the merged tree (`dock_size`,
  `resize_dock`, `dock_flex_for_size` in `workspace.rs`; `toggle_zoom`, `is_zoomed`,
  `visible_conversation_view` in `agent_panel.rs`; the bottom row in `thread_view.rs`).
- **Checklist (no TaskCreate):** pick ✓, pre-flight ✓, recall ✓, promote ✓, prior art ✓, spec ✓,
  design ✓.

### Design
- **`crates/workspace/src/workspace.rs` (Zed, additive, its row):** `marley_dock_width_share`
  (the dock's `dock_size` over `bounds.size.width`, `None` while the dock is closed or the
  workspace has no width) and `marley_set_dock_width_share` (`bounds.size.width * share` through
  the private `resize_dock`, the path a drag takes: it clamps against the other dock and stores the
  flex share and the size for the workspace).
- **`crates/agent_ui/src/conversation_view/thread_view.rs` (Zed, additive, its row):**
  `render_marley_panel_width`, called in the bottom row's left group after the follow toggle. It
  shows only while the workspace's `AgentPanel::visible_conversation_view` is this thread's
  `ConversationView`; the label is 100% while the panel is zoomed, else the share. A click builds
  `marley::CycleAgentPanelWidth` by name and dispatches it on the window.
- **`crates/marley_workbench/src/agent_panel_width.rs` (Marley, new):** the action; `SHARES`
  (1/3, 1/2, 2/3) and `STEP_MARGIN` (0.01); `cycle`: zoomed → `toggle_zoom` out and the
  narrowest share; else the next share past the current one, or `toggle_zoom` in. `init` registers
  it on every workspace from `marley_workbench::init`.

### Visual check plan
- `script/e2e/798-agent-panel-width-in-percent.sh`, `compositor sway`, #685's scripted ACP agent
  as a custom agent server so a thread opens with no model. Shots per the spec's UI proof; the
  click point of the button found from the first run's `798-01-label`. REQ-004 relaunches on the
  same profile (`quit_marley`, `launch_marley`). REQ-006 is the review of the diff: the click
  dispatches the same action the palette runs.

### Risks
- The label could lag a drag of the panel's edge if the thread view is not re-rendered; a drag
  notifies the workspace, whose render draws the dock and its panel again. Seen in the check.
- **Plan revised in Code (2026-10-17):** the first build covered only the Agent Panel's dock. The
  guide (`docs/marley/guide.md`, "Zed's Agent Panel with Marley's tools") and #744's scenario say
  the Marley layout, Chad's (`marley.layout: marley` in his settings), opens every thread in a
  center tab and keeps the dock out of sight. A dock-only button would never show where Chad talks
  to agents, so the button also steps the pane that holds a thread's tab (D4, D5), and its label
  comes from Marley through `MarleyThreadHost::width`. The spec's scope, decisions, criteria and UI
  proof were rewritten to match; gate run 1 checked the dock-only build.

## Phase 2 — Code
- **Built:**
  - `workspace.rs`: `marley_dock_width_share`, `marley_set_dock_width_share` (through `dock_size`
    and `resize_dock`), `marley_pane_width_share` (`None` for a pane alone in its row: no other
    pane overlaps its rows), `marley_set_pane_width_share` (through `center.resize`).
  - `agent_ui.rs`: `MarleyThreadHost::width`.
  - `thread_view.rs`: `render_marley_panel_width` in the bottom row's left group; the click sends
    `marley::CycleAgentPanelWidth { thread: <view id> }` by name.
  - `marley_workbench/src/agent_panel_width.rs` (new): the action (a fields struct for
    `Deserialize`, as `NewAgentThread` has, for `unsafe_derive_deserialize`), `width_label`, the
    place (the pane whose active tab is the thread's, else the Agent Panel while its active thread is
    this one), and the steps; `thread_tab.rs` gives the host `width` and `conversation_view_id`.
- **Deviation from the plan:** the pane as a second place (D4, D5), after the plan's dock-only
  design proved blind to the Marley layout (Phase 1's revision note). The label is text Marley
  writes ("50%"), not a number, so no float-to-integer cast is needed.
- **Review of the diff:** the dock and pane setters take `&mut Workspace` in the action's own
  handler and update the panel or the pane, never the workspace inside them; the label reads the
  workspace, its panes and the panel in render without updating any. A closed dock gives no share
  and the palette's action then does nothing. Each Zed hunk is additive with its row.
- **Gate run 1 (`798-gate-1.log`, the dock-only build):** red on gate:2 (`needless_pass_by_ref_mut`
  on `init`, `derive_partial_eq_without_eq` through `actions!`) and gate:14 (a module doc linking
  the private `SHARES`).
- **Gate run 2 (`798-gate-2.log`):** red on gate:2: `too_long_first_doc_paragraph` on the action
  and `unsafe_derive_deserialize`. Fixed as above; `./script/clippy -p marley_workbench` green.
- **Gate run 3 (`798-gate-3.log`):** **GATE GREEN [diff]**.

## Phase 3 — Test
- **Scenario:** `script/e2e/798-agent-panel-width-in-percent.sh` (`compositor sway`), #744's
  set-up: Marley's own agent on a scripted ACP agent (`MARLEY_ASSISTANT_ADAPTER`).
- **A real bug the check found (fixed in Code):** the first build read a pane's share through
  `PaneGroup::bounding_box_for_pane` in render. The pane axis element holds the `bounding_boxes`
  lock while it lays out the panes (`layout_as_root` inside the guard), and the thread view renders
  inside that layout, so the label's lock deadlocked the app the moment a thread's pane shared its
  row: the frame froze on "Loading…" or on an open palette. The fix reads and writes the row's
  flexes (cloned and unlocked before the layout) through `PaneGroup::marley_row_share` and
  `marley_set_row_share` in `pane_group.rs`, with its touchpoints row; the label became the pane's
  share of its row (D1).
- **Scenario fixes on the way (not Marley's):** the tab menu's Split Right could not be driven by
  keys (Home and Downs dropped some keys; a hover does not choose an entry, and Return with nothing
  chosen closes the menu), and a plain `pane: split right` does nothing for a thread tab, which
  cannot be cloned. The scenario splits the terminal's pane first (Home opens in the new pane,
  #785) and starts the thread there. Its `palette` sends the name, a pause and Return in one
  `wtype`. The panel half runs in Zed's layout, since the Marley layout hides Move Thread to Panel.
- **Shots (`scratchpad/shots-798/`), each read:**
  - `798-01-tab`: terminal left, Home and the thread right; the thread's bottom row shows 50%
    after the follow icon (REQ-001, pane).
  - `798-02-sixtysix`: after the click, the thread's pane is two thirds of the row and reads 67%;
    the tooltip shows "Agent width: click for 33%, 50%, 67%, then the whole window" (REQ-002).
  - `798-02-zoomed`: after the palette's action, the pane covers the workspace and reads 100%
    (REQ-003, REQ-006).
  - `798-03-back`: the next step: the pane is a third of the row and reads 33% (REQ-003).
  - `798-04-lone`: the other panes and tabs closed: the thread alone, its bottom row with no
    width button (REQ-005).
  - `798-05-panel`: Zed's layout, the thread in the Agent Panel docked left: 50% (Zed's flexible
    default, 648 of 1,300 pixels) (REQ-001, dock).
  - `798-06-sixtyseven`, `798-06-zoomed`, `798-06-third`: the palette's action: 67%, the panel
    over the window at 100%, then 33% (REQ-002, REQ-003, REQ-006).
  - `798-07-relaunch`: quit and started again: the panel open on the left at 33%, the width the
    last step set, where Zed's default would be 50% (REQ-004).
- **Gate run 4 (`798-gate-4.log`), after the fix and the scenario:** **GATE GREEN [diff]**.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` ("An agent's width in percent"); `docs/marley/guide.md` ("An
  agent's width in percent (#798)" under Zed's Agent Panel with Marley's tools). Touchpoints rows:
  `workspace.rs` (four share methods), `pane_group.rs` (new), `agent_ui.rs` (`width`),
  `thread_view.rs` (new); gate:16 green.
- **Knowledge appended:**
  - F-claude-798-a-pane-bounds-read-in-render-deadlocked-the-app-001
  - PR-claude-798-read-no-pane-bounds-in-render-001
  - L-claude-798-driving-menus-and-the-palette-in-a-scenario-001
- **Brain:** no decision recorded; the plan's revision is in these notes and the spec.
