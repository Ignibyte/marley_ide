# The Browser's console drawer, and its tools' buttons — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-887-browser-console-and-tools.md
- **Pipeline spec:** 887-console-drawer.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #886.
- **Classification / tier:** feature, M. Marley crate only.
- **Recall (§18.3):** the hub keeps each page's console and network (#492) for the agents' tools;
  the tab shows neither.
- **Discovery:** the BrowserTools board's other parts exist and are close (held click, the page's
  dialog, the scripts tray, take over, clearing data); their buttons are Zed's.
- **Decisions:** D1 to D3 in the spec.

### Design
- `browser.rs`: `drawer: Option<Drawer>`; `render_console_button` (before Scripts, the count);
  `render_drawer` (tabs with counts, Close; `console_row`, `network_row`), under the page and its
  picks; the pause bar's and `render_dialog`'s buttons as `render_button`s.
- **File manifest:** `browser.rs`; the scenario.

## Phase 2 — Code
- **Built:** `browser.rs`: `Drawer { Console, Network }` and the view's `drawer`;
  `render_console_button` (before Scripts, "Console N" in the error colour while the page has
  errors or exceptions, D1); `render_drawer` (220 px under the page and its picks, tabs with their
  counts, Close; `console_row` with `time_of_day`, `network_row`); the held click's "Refuse · Esc"
  and "Allow · Enter" and the page dialog's Cancel and OK as `prompts::render_button`s.
- **Review:** `time_of_day` checks the range before its one cast, under a narrow `allow` as
  `harness.rs` does; the keys of the held click and the dialog are unchanged.
- **Gate:** green on the first run, 17 of 17.


## Phase 3 — Test
- **Scenario:** `script/e2e/887-console-drawer.sh` (sway): an offline Chromium, a page that logs,
  warns, throws and fetches a missing file; a second page that asks `confirm()`.
- `887-01-console`: the toolbar's "Console 3" (two 404s and the exception); the drawer's Console 5:
  log, warning, two errors, the exception with its stack, each with its time and place.
  REQ-001 met.
- `887-02-network`: Network 3: GET 200 index.html, GET 404 verdicts.json and favicon.ico in red,
  each with its time. REQ-001 met.
- `887-03-day`: Marley Day, Network. REQ-001 met.
- `887-04-dialog`: "127.0.0.1:<port> says", the question, Cancel and a primary OK. REQ-002 met.
- Not shot: the held click's buttons (an agent's click held by System One, #571's setup); the same
  `render_button` draws them.


## Phase 4 — Complete
- CHANGELOG (Added), `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation fad6d30f.
- Ticket closed; pipeline archived; TICKET-892 holds the drawer's Recordings and Send, a pick's
  options and the download bar, which are new behaviour.
