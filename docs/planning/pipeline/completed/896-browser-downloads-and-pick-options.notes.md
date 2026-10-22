# The Browser's download bar, and a pick's options — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-896-browser-downloads-and-pick-options.md
- **Pipeline spec:** 896-browser-downloads-and-pick-options.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row.
- **Classification / tier:** feature, S–M. Marley crate only.
- **Recall (§18.3):** #767 tracks downloads in the hub but no view draws them; #879 made the
  picks panel; #892 made `send_line` and `console_line`; Send Block starts a new agent with
  `start_cli_with_prompt`. The brain had nothing on this seam.
- **Decisions:** D1–D4 in the spec.

### Design
- `BrowserView`: `download_closed: Option<String>` (the guid closed), `goes_with: GoesWith`
  (screenshot, source, console), `to_new_agent: bool`.
- `render_download_bar`: the page's newest `Download`; in progress, done (the folder with `~`),
  canceled; Show reveals the file (`cx.reveal_path`); the close records its guid.
- `render_tray`'s foot when an unsent pick shows: `render_goes_with` (three `Checkbox`es),
  `render_to` (two choices; the terminal's label from `agent_targets` when it is an agent's),
  Send N picks.
- `send_picks(ids)`: each pick's line (`pick_line` with what goes with it), then `console_line`
  when ticked; the hub records each pick sent with its options (`Pick.with_screenshot`,
  `with_source`); To: `send_line`, or `start_cli_with_prompt(Claude, line)` in the workspace.
- `browser_tools::pick_for_agents`: no crop without the screenshot; no component and no
  listener places without the source.
- **File manifest:** `browser.rs`, `browser_tools.rs` (Marley crate), the scenario.

### Visual check plan
| REQ | Setup and action | Shot |
|---|---|---|
| REQ-001 | a served page's download link clicked | `896-01-download`, `896-04-day` |
| REQ-002 | Save picked | `896-02-options`, `896-04-day` |
| REQ-003 | screenshot unticked, console ticked, Send 1 pick, the terminal | `896-03-sent` |
- Not shot: A new agent (it starts Claude Code, which the run has no account for); Show (it opens
  the desktop's file manager).

## Phase 2 — Code
- Built as designed, with two changes the first shots asked for:
  - #767's "Downloaded" toast covered the new bar's Show and close. The bar now takes over while
    a Browser tab holds the page, with the toast's Open beside Show (D5).
  - The per-pick Send went: the board sends from the foot. A Note's Enter still sends its pick
    (D4).
- Review: starting the new agent is deferred with `window.defer`, since the new terminal's pane
  may hold this tab, which opening it updates (the same reason `send_line` defers). `Pick`
  records what went with it; `pick_for_agents` drops the crop, the component's source and the
  listeners' original places when unticked.
- Gate: the first `just gate-diff` failed clippy (more than three bools in `Pick` and
  `BrowserView`; `BrowserView::new` and `render` over 100 lines). Fixed at the source: `Pick`
  carries one `GoesWith`, To is a `PickTo` enum, the tab's subscriptions moved to `subscribe`, and
  `render_under_page` holds the download bar and the drawer. The scenario ran again with the same
  shots; the second gate run was 17/17 green.

## Phase 3 — Test
Scenario `script/e2e/896-browser-downloads-and-pick-options.sh` (sway, the browser fixture,
downloads into the run's own folder through `XDG_DOWNLOAD_DIR`).
- `896-01-download`: the bar under the page: "report.txt saved to …/downloads", Open, Show, the
  close; no toast (REQ-001).
- `896-02-options`: Pick 1 with its Note; Goes with it with the screenshot and source ticked and
  the console not; To with "The terminal you used last" chosen and "A new agent"; Send 1 pick
  (REQ-002).
- `896-03-sent`: the terminal holds `[browser pick 1: button "Save changes" on …/card.html;
  browser_pick id 1, without its screenshot] [browser console: 2 errors in 2 messages on …;
  browser_console tab …]` (REQ-003).
- `896-04-day`: Marley Day, Pick 2 unsent above Pick 1 sent, the foot keeping the tab's choices
  (screenshot off, console on), the bar at the foot (REQ-001, REQ-002).
- Not shot: A new agent (the run has no Claude Code account); Open and Show (they hand off to
  the desktop); `browser_pick`'s answer without the crop (the line names it; the code drops it).

## Phase 4 — Complete
- CHANGELOG (Added), `marley_workbench.md` (the Browser's download bar and pick options).
- Brain decision recorded on consultation 3b751417. No F/L/PR codes: nothing broke.
