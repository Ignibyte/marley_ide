# A click on a port row opens nothing — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-604-port-row-click-does-not-open.md
- **Pipeline spec:** 604-port-row-click-does-not-open.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad chose "Row click doesn't open" from the options offered after he asked why
  Playwright's server opened in a Browser tab.
- **Evidence (2026-09-30):** Marley's log shows a page opening at 07:42:29, 07:46:52 and 07:58:31
  with no agent browser call; `stop_port` opens nothing, and the row's three buttons stop
  propagation, so the opens were clicks on the row's body.
- **Discovery:** `render_port_row` (`crates/marley_workbench/src/rail.rs:3995-4078`): the three
  buttons each call `cx.stop_propagation()`; the row's `on_click` calls `open_port`. The rail's
  Enter runs the row's click handler (AD-claude-453), so Enter needs the open path of its own once
  the click no longer opens.

### Visual check plan
- As in the spec; a `python3 -m http.server` in the scratch project for the row.

### Risks
- A click that marks a port row moves the rail's single highlight to it; confirm the highlight
  rules (#453, #542) accept a port row as the cursor's row.
