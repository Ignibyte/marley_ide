# The panel buttons become one labeled group — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-862-the-panel-buttons-become-a-labeled-group.md
- **Pipeline spec:** 862-panel-buttons-as-a-labeled-group.spec.md

## Phase 1 — Plan
- **Request:** the Queue's next row (#890, #895 wait on Chad; #900 and #860 need a stand-in).
- **Classification / tier:** feature, M. Two small additions in `workspace::dock`, one Marley file.
- **Recall (§18.3):** Zed's three `PanelButtons` items, left, bottom and right, draw unlabeled
  icons; `Dock` lists no panels publicly; `panel_tabs` keeps moved panels as center tabs. The
  brain had nothing.
- **Decisions:** D1, D2 in the spec.

## Phase 2 — Code
- `workspace::dock`: `Dock::panel_handles`; `MarleyPanelButtons`, whose yes makes `PanelButtons`
  draw an empty row.
- `panel_group.rs`: `init` sets the global to `marley_layout`; `PanelGroup` observes the three
  docks and the settings, and is told of each active item (a tab's opening moves it); a button per
  panel but Debug (its own button, #774): icon, the board's name (`BOARD`, in the board's order)
  from 1200 px wide, filled while it shows, an accent dot while a `PanelTab` draws it; a click
  focuses the dock and dispatches the panel's toggle, or the dock's for the one showing, as Zed's
  button does.
- `status_buttons.rs` adds it before Home (the right items draw from the end); `panel_tabs`'s
  `PanelTab::panel_id`.
- Changed after the first shot: the group came after Home and in dock order (Files, Agent,
  Changes, …); it is ordered as the board lists the panels, Home first.
- Gate: `just gate-diff` 17/17 green.

## Phase 3 — Test
Scenario `script/e2e/862-panel-buttons-as-a-labeled-group.sh` (sway).
- `862-01-group`: the row's right: Home, Files, Changes, Outline, Fleet, Collab, Agent, each
  named; the left end holds Debug, what has the focus and search, no other panel buttons
  (REQ-001).
- `862-02-open`: after `git panel: toggle focus`, Changes carries the accent dot (REQ-002).
- `862-03-day`: the same row in Marley Day (REQ-001).

## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, the ledger row, look-plan W1.4.
- Brain decision recorded on consultation 40c98317.
