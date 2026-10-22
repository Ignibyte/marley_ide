# The Browser's picks as a panel of cards — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-879-browser-tools-and-panels.md
- **Pipeline spec:** 879-picks-panel.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #878; split while planning: the picks panel here, the
  console drawer and the other tools TICKET-887.
- **Classification / tier:** feature, S–M. Marley crate only.
- **Recall (§18.3):** #496's scenario (the tray's rows, `page_point` counting tray rows): its
  points are stale since #850 and change again here; written into TICKET-887's notes.
- **Discovery:** the tray is a full-width strip above the page, at most 9 rem, a 36 px row per
  pick; the board puts the picks in an aside at the page's right.
- **Decisions:** D1, D2 in the spec.

### Design
- `browser.rs`: in `render`, the page area becomes an `h_flex` of the page and the picks panel;
  `render_tray` draws the aside (header with the count, the error, the cards); `render_pick_row`
  a card: the head (Pick N, summary, Discard), the place, then "Note" and the field with a
  primary Send, or the sent line; Check beneath.
- **File manifest:** `browser.rs`; the scenario.

## Phase 2 — Code
- **Built:** `browser.rs`: in `render` the scripts tray stays above the page and the page area
  sits in an `h_flex` with the picks panel at its right; `render_tray` draws the 320 px aside
  ("Picks for the agent" and the count, the error, the cards); `render_pick_row` a card (the head
  with Pick N, the summary and Discard; the listener's place; the body; Check and a primary Send);
  `render_pick_body` the Note field (its `SendPick` key) or the sent line.
- **Review:** clippy asked for the card's body in its own function (`map_or_else`) and the caption
  field made before the colours borrow; every pick action keeps its id, key and tooltip (D2).
- **Gate:** green on the first run, 17 of 17.


## Phase 3 — Test
- **Scenario:** `script/e2e/879-picks-panel.sh` (sway): Ctrl+Shift+C and a click on Save changes.
- `879-01-picked`: the page narrowed, the panel at its right: "Picks for the agent 1", the card
  "Pick 1 button “Save changes”", Note with its placeholder, Check and Send in the accent.
  REQ-001 met.
- `879-02-sent`: after Enter (the pick typed into the terminal, which Marley shows) and back on
  the Browser's tab: "✓ Sent: Make this button green", no Send. REQ-001 met.
- `879-03-day`: Marley Day, the same card. REQ-001 met.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation 9f64d7aa.
- Ticket closed; pipeline archived; TICKET-887 holds the console drawer and the other tools.
