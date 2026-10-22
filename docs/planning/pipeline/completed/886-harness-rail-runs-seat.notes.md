# The rail's Harness section, a task's runs and the seat form as the boards draw them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-886-harness-rail-runs-and-seat-form.md
- **Pipeline spec:** 886-harness-rail-runs-seat.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #885, taken whole.
- **Classification / tier:** feature, S. Marley crate only.
- **Recall (§18.3):** #719's and #691's scenarios run a real `rh` on a scratch root.
- **Discovery:** probes of #719 (the rail's tasks, the history tab: a headline, a summary, plain
  rows) and #691 (the form: fields, two filled agent buttons, a filled Create).
- **Decisions:** D1, D2 in the spec.

### Design
- `rail.rs`: an 8 px dot before the header's title, in `says_color`.
- `schedule_runs.rs`: the headline and summary and the runs inside a bordered card; the outcome
  in a chip tinted by its colour.
- `harness_seat.rs`: the two agent buttons in a bordered group; a footer of `render_button`s,
  Cancel (Quiet) and "Create · Enter" (Primary, dimmed while running).
- **File manifest:** `rail.rs`, `schedule_runs.rs`, `harness_seat.rs`, the scenario.

## Phase 2 — Code
- **Built:** `rail.rs`: a 7 px dot before the Harness header, green while the harness answers,
  else in the header's own state colour (D1). `schedule_runs.rs`: `ScheduleView` draws the task,
  its schedule and its runs in one card, each outcome a chip tinted by its colour.
  `harness_seat.rs`: `render_agents` (a framed toggle, Subtle buttons with the accent when
  chosen, only as wide as the two); the footer's Cancel (Quiet) and "Create · Enter" (Primary,
  dimmed with no click while a seat is made, D2).
- **Fixes in Test:** the dot first took the muted "connected" colour (grey), and the toggle's frame
  stretched the form's width; the dot goes green on a live harness and the frame sits in a row of
  its own.
- **Gate:** green, 17 of 17 (clippy first asked for `render` under 100 lines: `render_agents`).


## Phase 3 — Test
- **Scenario:** `script/e2e/886-harness-rail-runs-seat.sh` (sway): #719's scratch harness, two runs
  of `checks`.
- `886-01-rail`: the green dot before "HARNESS connected". REQ-001 met.
- `886-02-runs`: the `checks` tab: a card with "every 1 min · next …", "nothing" and "finding"
  chips, each run's time and first line. REQ-002 met.
- `886-03-seat`: New Harness Seat: the framed Claude Code / Codex toggle, Cancel and "Create ·
  Enter". REQ-003 met.
- `886-04-day`: Marley Day, the form. REQ-003 met.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation 81b2198b.
- Ticket closed whole; pipeline archived.
