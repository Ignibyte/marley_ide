# B5: Record what already happened in the Browser tab — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-499-flight-recorder.md
- **Pipeline spec:** 499-flight-recorder.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-25)
- **Request:** wave 2 of prong 3, pillar C (`browser-handoff.md`).
- **Classification:** feature; `marley_browser` (the recorder's rings and the recording's
  format, pure), `marley_workbench` (feeding it from the input path and the frames, Record this,
  the tools), `marley_mcp` (two rows). No Zed path expected.
- **Recall (§18.3):** #492's rings and `redact_url`; `L-claude-492-chromium-shows-a-password-by-length-001`
  (a snapshot writes no values); `F-claude-489-the-browser-decoded-frames-unoptimized-001` (the
  frames' cost in the debug build: the recorder keeps the JPEG bytes and decodes nothing).
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.

## Phase 1 — Plan (promoted 2026-09-25)
- **Pre-flight:** #498 committed (0201f81e37); no other active pipeline; cargo idle; the README
  marker present.
- **Recall:**
  - The brain (consultation `2f652b6749ac4eff961b8f7d51ef6c12`): nothing on this seam.
  - #492's rings: `ConsoleEntry` carries `time_ms` (Unix), `NetworkEntry` its request id and a
    browser-clock start that is private; `redact_url` hides query values and credentials;
    `snapshot::render` writes no field value. `L-claude-492-chromium-shows-a-password-by-length`.
  - #489's input path: every user press, release, move, wheel and key reaches the page through
    the hub's `mouse_press`, `mouse_release`, `mouse_move`, `wheel`, `send_key`, `insert_text`
    and `copy_selection`; a `KeyPress`'s `down` holds CDP's `key`, `text` and `modifiers`.
  - The frame loop decodes each frame's base64 JPEG off the main thread, then `show` keeps it.
- **Seams re-verified:** `paths::data_dir()` is where the browser's profile lives already
  (`browser/profile`), and the e2e profile stands in for it; `Workspace::show_toast` with
  `NotificationId` as `mcp.rs` uses; `chrono` is in the workspace's dependencies;
  `IconName::Circle` exists.

### Design
- **`marley_browser::recorder` (new, pure but for its files):** `Entry` (`click` with its place,
  button and count; `scroll`; `key` by name or chord; `typed` with a count; `navigation`;
  `console`; `request` with its id, method, redacted URL, then status and failure; `snapshot`;
  `agent`), each with the `Instant` it came. `Recorder` keeps the entries and the frames (the
  base64 JPEG as `Arc<str>`) of the last 60 seconds (`WINDOW`), a frame only 500 ms after the
  last (`FRAME_GAP`), 16 MiB of frames at most (`FRAME_BYTES`); consecutive `typed` entries add
  up, and so do scrolls within half a second. `Recorder::take(now)` gives a `Recording` (times
  in milliseconds from the minute's start, each frame an entry of its own) and its frames.
  Files: `save_in(dir, recording, frames)` writes `<dir>/<id>/timeline.json` and
  `<dir>/<id>/frames/NNNN.jpg`; `list_in(dir)`, `read_in(dir, id)` and `frame_in(dir, id,
  index)` read them back; an id outside `[0-9A-Za-z-]` is refused.
- **The hub:** a page's `recorder`, fed only while a tab draws the page: presses (not moves or
  releases), wheels, keys (a key with text and no Ctrl, Alt or Meta counts as one typed
  character; any other key by its name, a chord as `Ctrl+A`), inserted text as typed
  characters, the agent's actions as the Agent chip ends them, console entries as they come,
  requests as they start (their status added at `take`), the main frame's navigations, the
  main frame's snapshot after each load, and frames from `show`, which the frame loop now
  hands the base64 back to. `record(target, dir)` takes the minute, adds a snapshot of the
  moment, and writes it off the main thread.
- **The tab:** a red dot button (`IconName::Circle` in the error color, so the tab shows the
  recorder runs) and `marley::RecordThis`; a toast in the tab's workspace names the recording
  or says why it failed.
- **For agents:** `browser_recordings` (each recording's id, tab, URL, title, time, length,
  frames and entries) and `browser_recording {id, frame?}` (the timeline, and frame `frame`,
  from 1, as the answer's image), both read from the files off the main thread, before the
  browser needs to show.
- **Manifest:** `crates/marley_browser/src/recorder.rs` (new), `marley_browser.rs`;
  `crates/marley_workbench/src/browser.rs`, `browser_tools.rs`, `marley_workbench.rs` (the
  action), `Cargo.toml` (`chrono`); `crates/marley_mcp/src/registry.rs`;
  `script/e2e/499-flight-recorder.sh`, `script/e2e/browser-fixture.sh` (the agent's
  `recordings`, `recording`). No Zed path.

### E2E plan
| REQ | Step (`compositor sway`, offline) | Shot or log |
|---|---|---|
| REQ-004 | a click at a marked spot, then 65 seconds | the run log: the timeline starts after the click, frames at least 500 ms apart |
| REQ-001 | an e-mail and a password typed, Sign in (a fetch with `?token=`), a scroll, `marley: record this` | `499-01-recorded`: the toast |
| REQ-003 | the stand-in agent's `browser_recordings`, `browser_recording` with frame 2 | the run log, and the frame saved and read |
| REQ-002 | a grep of the recording's files for the e-mail, the password and the token | the run log: none |
| (regression) | #489's scenario (the input path), #492's (the rings) | their shots |

### Risks
- A page streaming at 60 fps hands its frames through the hub as before; the recorder keeps one
  every half second, a pointer copy.
- The minute is in memory per page; a page no tab draws records nothing (D4), so an agent's
  background tab has no minute.
- A snapshot after each load is one more CDP call per navigation of a drawn page.

### Checklist (no TaskCreate in this harness)
pick ✓, pre-flight ✓, recall ✓, promote ✓, prior art ✓, spec ✓, design ✓.

## Phase 1 — closeout
Phase 1 PASS (autonomous).

## Phase 2 — Code (2026-09-25)
- **Built:**
  - `marley_browser::recorder` (new): `Entry` (click, scroll, key, typed, navigation, console,
    request, snapshot, agent, frame), `TimedEntry` (serialized with its `at_ms` beside the
    entry's own fields), `Recording`, `RecordingError`; `Recorder` with `push` (typing adds up,
    and so do wheel turns within half a second), `request_ended`, `wants_frame`, `push_frame`
    (the 500 ms gap and the 16 MiB cap), the 60-second trim, and `take` (entries and frames in
    time order, frames numbered from 1); `save_in` (a `-2` suffix for a taken id), `list_in`,
    `read_in`, `frame_in`, which refuse an id outside letters, digits and dashes.
    `ConsoleLog::apply` now answers the entry it kept.
  - The hub: a page's `recorder`, fed by `record_entry` only while a tab draws the page, from
    `mouse_press`, `wheel`, `send_key`, `copy_selection` (now `&mut self`), `insert_text`,
    `agent_ended`, `observed` (console entries, requests and their ends, `record_request`), the
    main frame's `frameNavigated` (`record_navigation`), each load (`record_snapshot`, spawned
    so the event loop does not wait on the tree), and `show`, which the frame loop hands the
    base64 back to; `record(target, dir)`; `key_entry` (a key with text, or a single character
    that is not a letter or digit, which is how AltGr's characters come, counts as typed; any
    other key by its name, a shortcut with its modifiers); `snapshot_text`;
    `recordings_dir()` under `paths::data_dir()`.
  - The tab: the red dot (`IconName::Circle` in the error color) and `marley::RecordThis`,
    whose toast names the recording or the failure.
  - The tools: `browser_recordings` and `browser_recording {id, frame?}`, answered off the main
    thread before the browser needs to show; their registry rows and schemas. `chrono` and
    `base64` join the workbench's dependencies (both in the lockfile).
  - The stand-in agent's `recordings` and `recording`.
- **Deviations from the design:** the request statuses are set as their responses come
  (`request_ended`), not looked up at `take`, which keeps the recorder whole on its own.
- **Review against the criteria:**
  - REQ-001: the save runs off the main thread, and the toast reports either way.
  - REQ-002: no path puts a typed character into an entry: keys with text and AltGr characters
    count, `insert_text` counts, the snapshot writes no value, URLs go through `redact_url`,
    and requests keep no headers or bodies. An agent's typing reaches the timeline only as its
    Agent chip text, which gives a length.
  - REQ-003: the tools read the files, so a recording outlives the page and the launch.
  - REQ-004: the trim runs on every push and `take` filters by the minute again; frames are
    kept only half a second apart and only while a tab draws the page.
  - An agent's own clicks go to the page directly, not through the hub's input calls, so they
    are not recorded as the user's.
- **Checks:** clippy on the three crates clean; `cargo fmt` clean.
- **Checklist (no TaskCreate in this harness):** recorder.rs ✓, observe.rs ✓, hub ✓, tab ✓,
  tools ✓, registry ✓, action ✓, fixture ✓, review ✓.

## Phase 2 — closeout
Phase 2 PASS (autonomous).

## Phase 3 — Test (2026-09-25)
- **Scenario:** `script/e2e/499-flight-recorder.sh` (`compositor sway`, offline Chromium): a
  loopback sign-in page (an e-mail and a password field, Sign in, an "Early" box) that logs to
  the console, and on Sign in fetches `/api/session?token=s3cr3t-token-42&page=2` and warns
  "signing in". Steps: a click on Early, 65 seconds, a click in the e-mail field and
  `chad@example.com`, a click in the password field and `hunter2-secret`, Sign in, three wheel
  detents, `marley: record this`; the stand-in agent's `recordings` and `recording <id> 2`; a
  listing of the recording's files and a `grep -rlF` for the e-mail, the password and the token.
- **Shots and files (in the scratchpad, `e2e-499/`), each read:**
  - `499-01-recorded` (REQ-001): the toast "Saved this page's last minute as recording
    20260925-191409; agents read it with browser_recording." at the bottom right; the page
    scrolled; the toolbar's red dot beside the pencil and the crosshair.
  - `499-frame-2.jpg` (REQ-003): the recording's second frame, the sign-in page with the
    e-mail field focused, as the agent got it.
- **The run log:**
  - REQ-003: `recording 20260925-191409: 'Sign in' at http://127.0.0.1:…/index.html, 10.0 s,
    7 frames, 16 entries`, then its timeline: `click (290, 52)`, `typed 16 characters`,
    `click (290, 112)`, `typed 14 characters`, `click (220, 180)`, `request GET
    …/api/session?token=…&page=2, status 404`, `console warning "signing in"`, `console error
    "Failed to load resource: … 404 (File not found)"`, `scroll dy 300`; the snapshot at the
    save, 3 lines; frame 2 saved.
  - REQ-004: the early click, the navigation and the page's first console message, all older
    than a minute at the save, are gone (the timeline starts at the e-mail click, 10 seconds);
    7 frames, the least gap between two 501 ms.
  - REQ-002: the files are `timeline.json` and `frames/0001.jpg` to `0007.jpg`; "none of the
    e-mail, the password or the token is in the recording". The typing is counted (16 and 14
    characters, the lengths of what was typed).
- **Regressions:** `489-browser-input.sh` (its eight shots as before, the input latency median
  21.3 ms), `492-browser-tools.sh` (its three shots and log as before; 20 tools listed).
  `496-element-picker.sh` clicked where its pick button was before #499's red dot moved the
  toolbar's buttons left again, so `PICK_X` is now 1288; rerun, its four shots and log are as
  before (`L-claude-498-a-new-toolbar-button-moves-older-scenarios-clicks-001` foretold it).
- **Focus report:** every run in the headless sway; "hyprland: 0 Marley windows before the run,
  0 after; the run added no rule and did not reload it".
- **Gate:** the first `just gate-diff` went red in clippy and rustdoc with no finding in the
  code: other cargo runs had started on the box during it (a workspace `cargo check` on this
  repository and a workspace clippy), and its build failed writing into the shared target
  ("could not parse/generate dep info", "failed to write", "No such file or directory"). After
  `just idle` (rustal's own test run, in its own target directory, went on meanwhile and was
  left alone) the gate ran again: 16 passed, 0 failed, `GATE GREEN [diff]`; the receipt matches
  the tree.
- **Pre-existing, not in scope:** none.
- **Checklist (no TaskCreate in this harness):** REQ-001 ✓, REQ-002 ✓, REQ-003 ✓, REQ-004 ✓,
  489 ✓, 492 ✓, 496 ✓, gate ✓.

## Phase 3 — closeout
Phase 3 PASS (autonomous).

## Phase 4 — Complete (2026-09-25)
- **Documented:** `CHANGELOG.md` (#499 under Added); `docs/marley_architecture/marley_browser.md`
  (The flight recorder), `marley_workbench.md` (The flight recorder; the two tools),
  `marley_mcp.md` (ten read tools); the plan's B5 row shipped. No path outside the Marley-owned
  set changed.
- **Knowledge appended:** `L-claude-499-a-screencast-sends-frames-only-when-the-page-changes-001`,
  `AD-claude-499-the-flight-recorder-keeps-a-drawn-pages-minute-in-memory-001`. The first gate's
  red came from other cargo runs writing into the shared target, not from the change, so it has
  no `F-` block; the #496 scenario's moved button is what
  `L-claude-498-a-new-toolbar-button-moves-older-scenarios-clicks-001` already says.
- **Brain:** consultation `2f652b6749ac4eff961b8f7d51ef6c12` closed with
  `decisions/marleys-flight-recorder-keeps-a-drawn-pages-last-minute-in-memory`.
- **Ticket:** closed; the BACKLOG row went at promotion.
