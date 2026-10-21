# The rail's view switch as icons — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-873-view-switch-icons.md
- **Pipeline spec:** 873-view-switch-icons.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-21, while #841 was planned: it waits in `pipeline/queued/`.
- **Classification / tier:** feature, S. `rail.rs` only.
- **Recall (§18.3):** #832 made the switch, #868 the Files button beside it.
- **Decisions:** D1 in the spec.

### Design
- `render_view_switch`: the `view_button` closure makes an `IconButton` (`IconSize::Small`,
  `toggle_state`, `Tooltip::text`) in place of the `Button` with an XSmall label; the dots stay as
  they are.
- **File manifest:** `crates/marley_workbench/src/rail.rs`; the scenario.
- **Visual check plan:** the header at start, Threads by a click, Sessions by a click in Day.
  Positions from a probe shot.

## Phase 2 — Code
- **Built:** `render_view_switch`'s `view_button` makes an `IconButton` (`IconSize::Small`,
  `toggle_state`, `Tooltip::text` with the view's name) from an `IconName`: Folder, Thread,
  TerminalAlt; the dots and the click handlers are unchanged.
- **Review:** no other code reads the words; the walkthrough's three mentions are updated at
  Complete.


## Phase 3 — Test
- **Scenario:** `script/e2e/873-view-switch-icons.sh` (sway); a probe run (`STOP_AT_START=1`)
  placed the icons at x 15, 39 and 63 on y 17.
- `873-01-projects`: the header's folder (lit, pressed), thread and terminal icons at the left,
  then Files, Add Project and the panel toggle; the header fits its row. REQ-001 met.
- `873-02-threads`: the thread icon pressed, the rail listing conversations with its search
  field. REQ-001, REQ-002 met.
- `873-03-day`: Marley Day, the terminal icon pressed, the Sessions view ("No host is looked on").
  REQ-001, REQ-002 met.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md` (the rail header), the walkthrough's three mentions.
- Knowledge: none new (the change follows #868's button). Brain: `brain decide` on consultation
  7c557f67.
- Ticket closed; pipeline archived.
