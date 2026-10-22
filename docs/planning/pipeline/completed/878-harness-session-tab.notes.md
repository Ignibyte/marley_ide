# A harness session's tab as its board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-878-harness-sessions-and-runs.md
- **Pipeline spec:** 878-harness-session-tab.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #855; split while planning: the session's tab here, the
  rail's Harness section, scheduled runs and the seat form TICKET-886.
- **Classification / tier:** feature, S. Marley crate only.
- **Recall (§18.3):** #534's scenario runs the real `rh` on a scratch root; its tmux is the run's
  own since #880.
- **Discovery:** a probe run of #534: with writes off the tab is its lines alone (the asker's was
  blank, having none yet); with writes on, `render_controls` stacks the question, the Send row,
  the status and the views above the lines.
- **Decisions:** D1, D2 in the spec.

### Design
- `harness.rs`: `render_header` (title, the slot's name, the state chip from `state_word`, and
  the Views chips: kind, Open, Copy, the line as tooltip), `render_question` (a card: the prompt,
  `render_button`s, the first Primary), `render_send` (the field, Primary Send, Quiet Views, the
  status); `render` lays them header, question, lines, send.
- **File manifest:** `harness.rs`; the scenario.

## Phase 2 — Code
- **Built:** `harness.rs` `HarnessView`: `render_controls` gave way to `render_header` (title ·
  the harness's name or "this machine", the state from `state_word`, the rail's words and colours,
  and with writes on each view as a chip: its kind, Open, Copy, the command as tooltip),
  `render_question` (a card in the warning border, `prompts::render_button` per answer, the first
  Primary) and `render_send` (the field, Subtle Views, Primary Send, the status); `render` lays
  them header, question, lines, send.
- **Fix in Test:** Send was a plain filled button; it became #870's primary.
- **Gate:** green on the first run, 17 of 17.
- **Review:** answering, sending, Views and opening a view call what they called before; with
  writes off the tab is the header and the lines (D2).


## Phase 3 — Test
- **Scenario:** `script/e2e/878-harness-session-tab.sh` (sway): #534's scratch harness with
  `worker` and `asker`, `rh mcp --grant write` (the first run, without the grant, had Views refused
  with "session_surface_to_human requires `rh mcp --grant write`", which the status line showed).
- `878-01-asker`: "asker · this machine" and "waiting" in the warning colour; the card "Which base
  branch?" with main (primary) and release; the field, Views and Send at the foot. REQ-001 met.
- `878-02-views`: Views clicked: native and tmux chips, each with Open and Copy, in the header.
  REQ-001 met.
- `878-03-day`: Marley Day, the same. REQ-001 met.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation 7acd569f.
- Ticket closed; pipeline archived; TICKET-886 holds the rail's section, the runs and the form.
