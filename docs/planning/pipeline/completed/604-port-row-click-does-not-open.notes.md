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

### Promotion (Opus, 2026-09-30, back-to-back run)
- **Pre-flight:** no active pipeline, cargo idle, README marker present, the branch level with its
  origin after #603.
- **Recall (§18.3):**
  - AD-claude-453: the cursor is the rail's one selection while the rail holds focus, and focus
    leaving drops it. A click that marks a row must leave the focus in the rail.
  - AD-claude-453's "Enter runs the row's click handler" is out of date: `confirm` calls
    `open_row(selection)`, whose `Selection::Port` arm opens the URL. Enter keeps opening with no
    change.
  - Brain (consultation c31343ae): nothing on this seam.
- **Seams re-verified:** `render_port_row` (the row's `on_click` calls `open_port`; the three
  buttons stop propagation), `confirm`, `open_row`, `move_cursor`, and the rail's focus-out
  handler.

### Design
- **Approach:**
  - The port row's `on_click` reads `ClickEvent::click_count()`. A second click opens, as the
    Open button does (`open_port`). A single click runs `mark_row(Selection::Port(port, pid))`,
    which focuses the rail's `focus_handle` (so the cursor counts), sets `cursor` and refreshes;
    nothing opens.
  - The row's tooltip gains a last line, "Double-click, or Enter, to open in a Browser tab."
  - Nothing else changes.
- **File manifest:** `crates/marley_workbench/src/rail.rs` (Marley);
  `script/e2e/604-port-row-click-does-not-open.sh` (Test). No Zed path.
- **Visual check plan** (sway):
  - A `python3 -m http.server 38605 --bind 127.0.0.1` typed into the project's terminal gives the
    row.
  - Single-click the row: `single.png`, with the row marked and the center still the terminal
    (REQ-001).
  - Enter: `enter.png`, a Browser tab on the URL (REQ-002).
  - Close the tab, click back in the rail, and double-click the row: `double.png`, the tab again
    (REQ-003).
  - REQ-004, the Open button, is unchanged: a review.

## Phase 2 — Code (2026-09-30)
- **Built:**
  - In `rail.rs`, the port row's `on_click` reads `click_count()`: two or more clicks run
    `open_port`, and one runs the new `mark_row`, which focuses the rail, sets `cursor` and
    refreshes.
  - The row's tooltip gains "Double-click, or Enter, to open in a Browser tab.", and
    `render_port_row`'s doc comment says what a click does now.
- **Deviations:** none.
- **Review of the diff:**
  - REQ-001: one click opens nothing, and the cursor is the rail's selection while it holds the
    focus, which `mark_row` gives it.
  - REQ-002: Enter goes through `confirm` to `open_row`, whose port arm opens the URL, unchanged.
  - REQ-003: the second click of a double-click opens.
  - REQ-004: the Open button's handler is untouched and still stops propagation.
  - `mark_row` runs in the rail's own listener and updates nothing else.
- **Gate:** `just gate-diff` 17 PASS, 0 FAIL (`GATE GREEN [diff]`).

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/604-port-row-click-does-not-open.sh` (sway). The scratch project
  `repo` has an `index.html`, and `python3 -m http.server 38605 --bind 127.0.0.1` is typed into
  its terminal. `double_click` presses twice within about 0.2 s. It passed on the first run.
- **Shots, each read:**
  - `single.png` (REQ-001): the `:38605 python3` row is marked (the selected card), and its tooltip
    ends "Double-click, or Enter, to open in a Browser tab." The center still shows the terminal,
    and the tab bar holds only the terminal: nothing opened.
  - `enter.png` (REQ-002): after Enter, a "Port page" Browser tab on `http://127.0.0.1:38605/`
    shows "Served from repo". The rail lists the tab's row, with the port row below it.
  - `terminal.png`: a click on the terminal's row shows the terminal, with "Port page" behind it.
  - `double.png` (REQ-003): the double-click on the port row brought the same "Port page" tab
    forward, with no second tab.
- **Not reachable by a shot:** REQ-004, the Open button, is unchanged in the diff; covered by the
  review.
- **Focus:** its own headless sway; Hyprland had 0 Marley windows before and after.
- **Fixes:** none needed; the Phase 2 gate stands.

## Phase 4 — Complete (2026-09-30)
- **Documented:**
  - `CHANGELOG.md` (Changed).
  - `docs/marley_architecture/marley_workbench.md` (port rows: the click, `mark_row`, Enter).
  - `docs/marley/workbench-shell.md` (the slice line), `docs/marley/guide.md` (port rows) and
    `docs/marley/walkthrough.md` (2.8).
  - The guide page's `port-rows` article, rebuilt from the scratch parts.
  - No Zed path touched.
- **Knowledge:**
  - L-claude-604-the-rails-enter-opens-through-open-row-not-a-rows-click-001;
  - AD-claude-604-a-port-row-marks-on-one-click-and-opens-on-two-001.
  - No bug was found.
  - Brain: consultation c31343ae closed as
    `decisions/marleys-port-rows-mark-on-one-click-and-open-on-two`.
