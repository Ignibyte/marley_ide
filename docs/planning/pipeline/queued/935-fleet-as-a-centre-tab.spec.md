---
pipeline_id: 319e502f-5724-4a46-8ef1-40290c0354dc
ticket: docs/planning/tickets/open/TICKET-935-fleet-as-a-centre-tab.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: The Fleet tab with the snapshot beside the list
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/F-tools.md, docs/planning/pipeline/completed/848-fleet.spec.md, docs/planning/pipeline/completed/748-more-side-panels-as-tabs.spec.md]
---

## Title
The Fleet as the Fleet board draws it: in its centre tab, the hosts and their agents on the left
and the selected agent's snapshot as a column on the right, under a "Fleet" title with the
summary beside it. A narrow Fleet keeps the snapshot under the list.

## Scope
### In
All in `crates/marley_workbench/src/fleet.rs` (Marley crate). No Zed crate changes.
- **The tab stays as it is.** In the Marley layout Fleet already opens as a centre tab, #748's
  `PanelTab` titled "Fleet", from every entry (`marley: toggle fleet`, the status bar's Panels
  menu, `marley panel fleet`). The dock panel stays registered so those entries and Zed's layout
  keep working. Nothing changes in `panel_tabs.rs`.
- **Two arrangements** (`FleetPanel::render`). A new `Arrangement` enum, `Beside` and `Under`,
  chosen from the body's measured width (D2). `FleetPanel` gains a `body_width:
  Rc<Cell<Option<Pixels>>>` that a `canvas` behind the body fills at prepaint, as the rail strip
  measures its search (`rail_strip.rs:284`). When the measured width gives another arrangement
  than the one drawn, the panel redraws once.
- **Beside** (`FleetPanel::render`, a new `render_beside`). An `h_flex` of two columns that share
  the width equally, each scrolling on its own: the list (`render_list`, unchanged) and, after a
  vertical border, the snapshot column on the editor background (`render_selected`, unchanged).
  With no agent selected, the column holds the muted line "Select an agent to see its snapshot
  here." The drag handle is not drawn.
- **Under** (`FleetPanel::render`, the current body moved into `render_under`). The list over the
  snapshot with #608's draggable split and `snapshot_ratio`, as now.
- **The header** (`FleetPanel::render`). "Fleet" as a `Headline` (`HeadlineSize::Small`, as MCP
  Servers and Agent Activity title their pages), then the summary (`summary`, unchanged) muted
  beside it, in place of the XSmall "FLEET" label. Both arrangements draw it.

### Out (explicitly deferred)
- The snapshot's CPU and memory chart over 30 minutes, its events, every phase's name under the
  strip, the work item as a chip and the tokens meter: #848 Out ("the agent snapshot pane keeps
  #608's look"). The Agent tab (`agent_tab.rs`) shows the chart and the events.
- The board's snapshot header (a lettered tile, the name over "runtime · model · working for … ·
  host"): the same #848 deferral.
- "Harness sessions, in the rail" under the list, with the session quota: #848 Out (TICKET-878)
  and #886 Out ("the rows carry no quota today").
- The bottom row for the Fleet ("Fleet", "3 agents · 2 hosts", "review-1 waits", "Harness
  connected"): the build order's item 2; #909, the bottom row for the other screens, owns it.
- Turning the dock panel into a workspace item of its own (no `Panel`): not needed. #748's tab is
  the centre tab the board draws, and removing the panel would break `marley panel fleet`, the
  Panels menu and Zed's layout.

## Reference (§20)
N/A — Marley-specific: the Fleet is Marley's own surface (#607, #608, #848) over Marley's fleet
contract (`marley_sdk`), shown as a tab through Marley's panel tabs (#747, #748). The two
arrangements follow the Fleet board; no Zed or Warp view lays out a list and its detail this way.

### Prior art
- The review's main gap does not hold at HEAD, a win to record: `panel_tabs.rs:41-48` `MOVED`
  lists `MarleyFleetPanel`, so in the Marley layout every way the dock would open on Fleet ends
  in a `PanelTab` in the active pane (`dock_changed`, `panel_tabs.rs:132`), titled from the
  panel's tooltip "Fleet" (`fleet.rs:1603`, `panel_tabs.rs:428`). `fleet_shows`
  (`fleet.rs:330-351`) already counts that tab as a place the Fleet shows. The review judged from
  `impl Panel` (`fleet.rs:1570`) alone; its shot `tour-18-fleet.png` missed the screen.
- The gap that holds: `FleetPanel::render` (`fleet.rs:1488-1559`) stacks the list over the
  snapshot in a `v_flex` with a drag handle (`render_handle`, `fleet.rs:910`) and
  `snapshot_ratio` (`SNAPSHOT_RATIO`, `fleet.rs:47`), whatever its width. The header is an XSmall
  muted "FLEET" (`fleet.rs:1512-1516`) with `summary` (`fleet.rs:1377`).
- `render_list` (`fleet.rs:1416`) and `render_selected` (`fleet.rs:1453`) return the two columns'
  contents as they are; host cards, rows with the runtime tile and state chips are #848's and
  match the board.
- Measuring a width in gpui: `canvas(prepaint, paint)` filling a `Cell` with its bounds, as
  `rail_strip.rs:284-290` and `:423-430` do. gpui has no container query; a `flex_wrap` row with
  two 520 px bases (the board's own CSS) was weighed and rejected (D2).
- Page titles in tabs: `Headline::new(…).size(HeadlineSize::Small)` in `mcp_servers.rs:376` and
  `agent_activity.rs:445`.
- Entries that open the Fleet: `ToggleFleet` (`marley_workbench.rs:447-449`, registered in
  `fleet::init`, `fleet.rs:299-313`); `marley panel fleet` (`commands.rs:97`); the Panels menu
  (`panel_group.rs:97`). No keymap binds it (grepped `assets/keymaps`); Home and the rail have no
  Fleet entry.
- `agent_tab.rs` (#609): the full Agent tab, opened by Open, Enter or a double-click
  (`agent_tab::open_later`, `agent_tab.rs:31`); unchanged.
- Specs: #607, #608 (the split; its scenario drags the handle), #748 (Fleet as a tab), #848 (the
  board's list).

## UI proof
Scenario `script/e2e/935-fleet-as-a-centre-tab.sh` (sway). Setup: a scratch repository and the
pseudo provider (`marley.fleet` `{"providers": [{"kind": "pseudo"}], "hosts": []}`), as #848's
scenario sets it. No agent CLI and no Rusty.
- `935-01-beside`: `marley: toggle fleet` in a full-width pane, build-1 clicked: the "Fleet" title
  with its summary, the host cards on the left, build-1's snapshot in the right column (REQ-001,
  REQ-003).
- `935-02-none`: before any click: the right column's "Select an agent…" line (REQ-002).
- `935-03-under`: `pane: split right` with the Fleet tab in front, so it holds half the width: the
  snapshot under the list with the split's handle (REQ-004).
- `935-04-day`: `935-01-beside`'s view in Marley Day (REQ-001, REQ-003).

## Locked-In Decisions
- D1 — The centre tab is #748's panel tab. The Fleet stays a `Panel` the tab draws, so every
  entry, `fleet_shows`, the `Wanted` selection and Zed's layout keep working unchanged.
- D2 — The arrangement follows the Fleet's own width, measured each frame: `Beside` at 760 px
  or wider, `Under` below. A full-width tab is beside; the 320 px dock and a half-width pane are
  under, where two columns would be too narrow to read. Rejected: choosing by place (tab or
  dock), since a tab can sit in a narrow pane; and a `flex_wrap` row as the board's CSS has, since
  wrapped it loses the draggable split and scrolls the list and the snapshot as one.
- D3 — Beside, the columns share the width equally, as the board's two 520 px columns do, with
  no drag handle. Under keeps #608's split and the ratio the user dragged to.
- D4 — With nothing selected, the right column says so in one muted line. Nothing is selected for
  the user: a selection reads the agent's full detail (AD-claude-608).
- D5 — The header is the board's in both arrangements: the "Fleet" headline and the summary.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the Fleet is 760 px wide or wider and an agent is selected, the Fleet shall draw the hosts and their agents on the left and the selected agent's snapshot in a column on the right, in Marley Night and Marley Day. | `935-01-beside`, `935-04-day` |
| REQ-002 | WHILE the Fleet is 760 px wide or wider and no agent is selected, the right column shall say "Select an agent to see its snapshot here." | `935-02-none` |
| REQ-003 | The Fleet shall title itself "Fleet" as a headline with its summary of agents, hosts and providers beside it. | `935-01-beside`, `935-04-day` |
| REQ-004 | WHILE the Fleet is narrower than 760 px, the system shall draw the selected agent's snapshot under the list with the draggable split. | `935-03-under` |
| REQ-005 | `marley: toggle fleet`, `marley panel fleet` and the Panels menu shall still open the Fleet, and the Fleet shall stay a panel the tab draws. | the review of the diff; `935-01-beside` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `fleet.rs` only; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
