# The Browser toolbar as its board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-850-browser-tab-and-tools.md
- **Pipeline spec:** 850-browser-toolbar.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #849; split while planning: the toolbar here, the
  Browser's other parts TICKET-879.
- **Classification / tier:** feature, S. Marley crate only.
- **Recall (§18.3):** the browser fixture (`script/e2e/browser-fixture.sh`: `offline_chromium`,
  `serve_site`, `browser_teardown`).
- **Discovery:** a probe run: the toolbar has back, forward, reload, the address box, and icon
  buttons for scripts, Pick, Annotate and Record.
- **Decisions:** D1 in the spec.

### Design
- `browser.rs`: the address box an `h_flex` of the project's name (muted, small), the editor, and
  `place_of(address)`; Pick, Annotate and Record as `Button`s with their icons and labels in a
  bordered group, their states (lit while picking or annotating, disabled without a page) kept.
- **File manifest:** `browser.rs`; the scenario.

## Phase 2 — Code
- **Built:** `browser.rs` `render_toolbar`: the address box an `h_flex` of the project's name
  (from `hub.project_of(target)`, since the view keeps only the project's key), the address editor
  and `place_of(address)` (D1); Pick, Annotate and Record as `Button`s with their icons and labels
  in one bordered group, their disabled and lit states kept.
- **Deviation:** the plan read the name from the view; the view holds the key, so the hub's
  `BrowserProject` gives it, and a tab still attaching shows no name.
- **Gate:** the first run failed clippy (`render_toolbar` at 109 lines; an `if let` in
  `place_of`); `render_address` and `render_page_tools` split off, `map_or_else` used; the second
  run green, 17 of 17, and the scenario re-run on it.
- **Review:** the buttons' actions, tooltips and states are unchanged; only their faces moved.
  The buttons are wider, so five older browser scenarios that click them at fixed points
  (496, 505, 508, 518, 523) need their points measured again at the regression; written into
  TICKET-879.


## Phase 3 — Test
- **Scenario:** `script/e2e/850-browser.sh` (sway): an offline Chromium, a page served in the run.
- `850-01-toolbar`: back, forward, reload; the address box reads "repo", the address, and "port
  N · this machine" at its right; the scripts button; Pick, Annotate and Record (its red dot)
  labelled in one framed group. REQ-001 met.
- `850-02-day`: Marley Day, the same on the light toolbar. REQ-001 met.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation fd9bc1dc.
- Ticket closed; pipeline archived; TICKET-879 holds the rest of the Browser's boards.
