# B0b: Typing and clicking in the page — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-489-browser-input.md
- **Pipeline spec:** 489-browser-input.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-24)
- **Request:** wave 1 of prong 3; the input half of B0 (the amendment's questions 1, 2 and 5).
- **Classification:** feature; `marley_browser` (the mappings) and `marley_workbench` (the
  tab's handlers). No Zed path.
- **Recall (§18.3):**
  - L-claude-481-gate-text-by-its-modifiers-not-prefer-character-input-001: on Linux,
    `prefer_character_input` is always false; stopping a key's propagation keeps it from text
    input.
  - PR-claude-raw-input-passthrough-must-filter-platform-chords-001: a new passthrough filters
    the platform chords the other routes filter.
  - The Explore report on gpui's Linux input (compose order, the text step, text-input-v3's
    one-byte commits, `bounds_for_range` in window coordinates).
- **Discovery:** the CDP probe's key, composition, wheel and iframe results.
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.

## Phase 1 — Plan (promoted 2026-09-24)
- **Pre-flight:** no active pipeline before this one; #488 committed (cc5e338ca3); cargo idle.
- **Seams re-verified:** #488's `BrowserHub` (the page, the frame) and `PageElement` (bounds in
  `prepaint`, paint); gpui's `EntityInputHandler` (text_for_range, selected_text_range,
  marked_text_range, unmark_text, replace_text_in_range, replace_and_mark_text_in_range,
  bounds_for_range, character_index_for_point), `ElementInputHandler::new(bounds, view)`,
  `Window::handle_input`, `Window::on_mouse_event` with a `Hitbox` and `DispatchPhase::Bubble`
  (the terminal element's pattern), `Window::focus(&handle, cx)`, `App::read_from_clipboard`
  and `write_to_clipboard`, `ClipboardItem::new_string` and `text`. Marley's log goes to
  `<data dir>/logs/Marley.log`, which the harness deleted with the profile; its stdout is empty.

### Design
- **`marley_browser::input`** (new, pure): `key_press(&Keystroke) -> Option<KeyPress>` (the
  down and up parameters for `Input.dispatchKeyEvent`): nothing for a Super chord; the named
  keys (Enter with the text `\r`, Tab, Backspace, Delete, Escape, Insert, Home, End, Page Up
  and Down, the arrows, Space, F1 to F12) with their DOM key, code and Windows key code; a key
  with text and no Ctrl or Alt as `keyDown` with its text; a Ctrl or Alt chord as `rawKeyDown`
  with the letter's key, code and key code; `autoRepeat` from `is_held`. The US tables give
  `code` and the key code for letters, digits, their shifted symbols and the punctuation
  keys; other text gets an empty code. `modifiers(&Modifiers)` (Alt 1, Ctrl 2, Meta 4,
  Shift 8); `mouse_button(MouseButton)` (name and bit); the mouse and wheel parameters.
- **`Page`** gains `insert_text`, `set_composition`, and `selected_text`, which reads the
  focused field's selection or the document's in an isolated world on the main frame
  (`Page.getFrameTree`, `Page.createIsolatedWorld`, `Runtime.evaluate` there), so the page's
  scripts neither see nor change the read.
- **The hub** sends input in order (each call's message leaves in the order the calls were
  made), keeps the held buttons (a drag that leaves the tab still reaches the page, and its
  release too), and times the first frame after an input: `log::debug!(target:
  "marley_browser", "input to frame: … ms")`. It keeps the newest frame's metadata beside the
  frame.
- **`PageElement`** inserts a hitbox in `prepaint` and in `paint` registers mouse listeners
  (down focuses the tab and records the point for the input method's window; up; move with
  the held buttons; the wheel at 100/3 CSS pixels a line, the sign turned) and the tab's input
  handler. A point maps to CSS pixels through the frame's metadata: its DIP width over the
  width it is drawn at.
- **`BrowserView`**: `on_key_down` sends the keystroke's press and stops the key; Ctrl+V
  inserts the clipboard's text; Ctrl+C and Ctrl+X first read the selection onto the system
  clipboard, then send the key, so the page's own copy and cut still run. As the input handler:
  a preedit becomes the page's composition, a commit or a composed key ends it, and
  `bounds_for_range` is the last press.
- **The harness** copies `<profile>/logs/Marley.log` to `$SHOT_DIR/<scenario>.marley.log`
  before it removes the profile.
- **Manifest:** `crates/marley_browser/src/{input.rs,page.rs,marley_browser.rs}`;
  `crates/marley_workbench/src/browser.rs`; `script/e2e.sh`; `script/e2e/489-browser-input.sh`
  (Test). No Zed path.

### E2E plan
| REQ | Step (`compositor sway`, a fixture page with absolute positions) | Shot / evidence |
|---|---|---|
| REQ-002 | click the field, type "hello world" | `489-01-typed` |
| REQ-002, 003 | five Backspaces, Ctrl+A, "replaced", Home, ">", Enter (the form prints what it submitted) | `489-02-edited` |
| REQ-004 | click the textarea; Multi_key, apostrophe, e | `489-03-composed` |
| REQ-001 | click, double-click and right-click the button; the page prints each with its button and count and marks each press | `489-04-clicks` |
| REQ-007 | click into the cross-site iframe's field, type | `489-06-iframe` |
| REQ-008 | `wl-copy` in the sway, Ctrl+V in the textarea; double-click a word, Ctrl+C, `wl-paste` | `489-07-pasted`; the run log |
| REQ-009 | Super+x; the page's key log | `489-08-super-filtered` |
| REQ-006 | five wheel detents down over the page | `489-05-scrolled` (the page's `scrollY` 500) |
| REQ-010 | `ZED_LOG=marley_browser=debug`; the scenario reads Marley's log | the run's p50 and p95 |
| REQ-005 | not driven (no input method with an engine on the box, Chad's call) | review |

### Risks
- A Zed binding for a key in a context above the tab (Ctrl+W, the palette) wins over the page;
  that is D5, and #490 adds the browser's own context bindings.
- xkb compose needs a compose table for the locale; the run's locale is the user's
  (`en_US.UTF-8`), whose table has `<Multi_key> <apostrophe> <e>`.

## Phase 1 — closeout
Phase 1 PASS (autonomous).

## Phase 2 — Code
- **Built:** `marley_browser::input` (the key mapping with the named-key and US tables, the
  modifier bits, the mouse buttons, the mouse and wheel parameters); `Page::insert_text`,
  `set_composition` and `selected_text` (the focused field's selection or the document's, read
  in an isolated world on the main frame). In `browser.rs`: the hub's `send` (in order; timed
  inputs start the clock the next frame stops, logged as `input to frame: … ms` under
  `marley_browser`), the held buttons, the frame's metadata; the tab's `key_down` (Ctrl+V
  inserts the clipboard's text; Ctrl+C and Ctrl+X read the selection onto the clipboard, then
  send the key; a key with text ends a composition); the tab as `EntityInputHandler`;
  `PageElement`'s hitbox, input handler and mouse listeners, with `PageMapping` from the
  frame's metadata. The harness copies `Marley.log` beside the shots.
- **Deviations:** Ctrl+C and Ctrl+X send their key only after the selection is read, in the
  same task: sent at once, a cut would empty the selection before the read.
- **Review of the diff:** clippy's findings fixed at the source (values passed by value, a
  `map_or`, `&self` where nothing mutates). The order of CDP calls rests on each call sending
  its message on the first poll of its task, and gpui running tasks in the order spawned.

## Phase 3 — Test
- **Scenario:** `script/e2e/489-browser-input.sh` (`compositor sway`, the shared browser
  fixture): a page with every target at a fixed place and a status box reporting its scroll,
  its last keys, its form's submissions and each click with its button and count, with a red
  dot where each press landed. `ZED_LOG=marley_browser=debug` for the latency lines.
- **First run, three reds, none in the input path it tests:**
  - The button logged no click, double-click or context menu. A debug run showed the page
    getting `pointerdown`, `mousedown`, `pointerup`, `mouseup` and `click` from Marley: the
    fixture's own dots, added on each press, sat on top of the button and took the release
    (and so the click) from it. The dots now have `pointer-events: none`.
  - Compose typed `'e`: each `press` was its own `wtype`, and every `wtype` brings its own
    keymap, which gpui answers by dropping the compose state (its log: "Received keymap format
    NoKeymap, expected XkbV1"). The harness gained `press_keys`, several keys in one `wtype`.
  - The latency report's awk failed on a nested ternary; rewritten.
  - Also: the latency lines were missing, because zlog matches a directive against the crate a
    line comes from (`marley_workbench`), not its target; the line moved into `marley_browser`
    (`input::log_latency`).
- **The latency answer:** the first numbers were a median of 114 ms and a 95th percentile of
  160 ms. Timing the decode showed each frame taking 130 ms in the unoptimized debug build, so
  a key waited behind a decode or two. `[profile.dev.package]` now builds `image`,
  `zune-core`, `zune-jpeg` and `marley_browser` at `opt-level = 3` (a Zed-path edit, its ledger
  row first): 11 to 34 ms afterwards, and in the final run **45 inputs, median 20.4 ms, 95th
  percentile 36.1 ms** from dispatch to the decoded frame, against the protocol's own 6 to 7 ms.
- **Shots (final run):**
  - `489-01-typed` — the field reads "hello world"; the key log `h e l l o Space w o r l d`; a
    dot where the click landed, inside the field. (REQ-002)
  - `489-02-edited` — after five Backspaces, Ctrl+A, "replaced", Home, ">" and Enter: the field
    reads ">replaced" and the page says "submitted: >replaced". (REQ-002, REQ-003)
  - `489-03-composed` — the textarea reads "é", once; the key log ends with `é`. (REQ-004)
  - `489-04-clicks` — "click: button 0, count 1", then "count 1" and "count 2" for the double
    click, and "contextmenu: button 2"; a dot on the button under each press. (REQ-001)
  - `489-06-iframe` — the cross-site frame's field reads "in the frame". (REQ-007)
  - `489-07-pasted` — the textarea reads "éfrom the clipboard" after `wl-copy` and Ctrl+V; and
    after a double click on "copy" and Ctrl+C the run printed "the clipboard after Ctrl+C:
    copy". (REQ-008)
  - `489-08-super-filtered` — after Super+x the key log still ends with Ctrl+C's `c`; the word
    "copy" is selected. (REQ-009)
  - `489-05-scrolled` — "scrollY 500" after five wheel detents down. (REQ-006)
- **REQ-005:** not driven: no input method with an engine runs on the dev box (Chad,
  2026-09-24: "Compose only, CJK later"). Reviewed: a preedit calls `imeSetComposition`, a
  commit `insertText`, and a key with text ends a composition first.
- **Run report:** "hyprland: 0 Marley windows before the run, 0 after"; "sway: stopped".
- **Gate:** the first `just gate-diff` was red on gate:14 (the module doc linked the private
  `PageElement`); fixed, and the page's unused `insert_text` and `set_composition` removed (the
  hub sends those calls on its ordered path). Rebuilt, the scenario run again on the final tree
  (the same eight results; latency median 21.6 ms, 95th percentile 35.4 ms over 44 inputs),
  then `just gate-diff` → `16 passed, 0 failed`, `GATE GREEN [diff]`.

## Phase 4 — Complete
- **Docs (§21):** CHANGELOG (Added: typing and clicking in the Browser tab); `marley_browser.md`
  (Input); `marley_workbench.md` (the Browser tab's input); the plan (the latency answer through
  Marley; #489 shipped); the `Cargo.toml` row in `zed-touchpoints.md` names the dev-profile
  opt-levels, as shipped; `script/e2e.sh`'s header names `press_keys`.
- **Ledger:** F-claude-489-the-browser-decoded-frames-unoptimized-001,
  PR-claude-a-per-frame-path-is-optimized-in-the-dev-build-001,
  L-claude-489-each-wtype-resets-gpuis-compose-001,
  L-claude-489-zlog-filters-by-the-crate-a-line-comes-from-001,
  L-claude-489-a-fixture-that-marks-presses-must-not-take-them-001.
- **Brain:** consultation e2a5e63dbb8949369c1ef6ed64761641 closed as
  `decisions/the-browser-tabs-frame-decode-is-optimized-in-marleys-debug-build`.
