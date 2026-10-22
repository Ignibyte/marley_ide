# The Browser drawer's Recordings, and Send to the agent — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-892-browser-recordings-send-and-downloads.md
- **Pipeline spec:** 892-drawer-recordings-send.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row; the download bar and a pick's options split to TICKET-896.
- **Classification / tier:** feature, M. Marley crate only.
- **Recall (§18.3):** #887's drawer; #496's Send and its waiting-agent guard (#508).
- **Decisions:** D1, D2 in the spec.

### Design
- `browser.rs`: `Drawer::Recordings`; `recordings: Option<Vec<RecordingRow>>` read by
  `read_recordings` (`recorder::list_in` on the background executor) when the tab is chosen and
  after Record This; `send_line(line, window, cx) -> Result<(), SharedString>` from `send_pick`'s
  terminal half, which `send_pick` now calls; `drawer_error` for its refusals; the header's Send to
  the agent on Console; a Send per recording row.
- **File manifest:** `browser.rs`; the scenario.

## Phase 2 — Code
- **Built:** `browser.rs`: `Drawer::Recordings`; `RecordingRow::of` (id, title, URL, `recorded_at`
  in local time from its Unix seconds, length from its float seconds); `read_recordings`
  (`recorder::list_in` off the main thread, newest first) when the tab is chosen; `drawer_rows`;
  `sendable` and `send_line` from `send_pick`'s terminal half (which `send_pick` now calls);
  `send_from_drawer` with `drawer_error`; the Console's Send to the agent (`console_line`) and a
  Send per recording (`recording_line`).
- **Fix in Test:** the first run's rows had no time and "0 s": `recorded_at` is Unix seconds and
  `seconds` a float; both are read as such now.
- **Review:** clippy asked for `Value` unqualified, `send_line` and `recording_row` as associated
  functions, `&Context` in `read_recordings`, and `render_drawer` under 100 lines (`drawer_rows`,
  the error count inside `console_line`).
- **Gate:** green, 17 of 17.


## Phase 3 — Test
- **Scenario:** `script/e2e/892-drawer-recordings-send.sh` (sway): #887's page; the terminal used
  first; `marley: record this`.
- `892-01-recordings`: the drawer's "Recordings 1": "Oct 22, 05:57:59", Feature Map over its
  address, "17 s", Send. REQ-001 met.
- `892-02-sent`: the Console's Send to the agent: the terminal shows "[browser console: 3 errors in
  5 messages on http://127.0.0.1:<port>/index.html; browser_console tab <id>]". REQ-002 met.
- `892-03-day`: Marley Day, Recordings. REQ-001 met.


## Phase 4 — Complete
- CHANGELOG (Added), `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation 8729c76b.
- Ticket closed; pipeline archived; TICKET-896 holds the download bar and a pick's options.
