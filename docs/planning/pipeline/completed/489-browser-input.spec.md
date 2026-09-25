---
pipeline_id: 89bf9612-89ac-4235-8c8b-84187a76c7c1
ticket: docs/planning/tickets/open/TICKET-489-browser-input.md
status: Phase 4 — Complete PASS
title: "B0b: Typing and clicking in the page"
type: feature
slice: prong 3 B0b
references: [docs/marley/three-prong-plan.md, docs/planning/pipeline/queued/488-browser-pane.spec.md, docs/planning/intake/embedded-agent-browser-chromium-cdp.md]
---

## Title
The Browser tab takes the mouse, the wheel and the keyboard, including composed and
input-method text and the clipboard, and passes them to the page over CDP.

## Scope
### In
- **Mouse.** Left, middle and right presses and releases with gpui's click count; moves, with
  the held buttons during a drag; the wheel. Each goes to `Input.dispatchMouseEvent` at the
  point's CSS pixels, taken from the newest frame's metadata (the frame's DIP size against
  the size it is drawn at), never from hand-kept state. gpui's wheel lines count 100/3 CSS
  pixels each, so one detent (three lines) scrolls 100 pixels; pixel deltas pass through; the
  sign flips (gpui's positive y is up, CDP's is down). A press in the tab focuses it.
- **Keys** while the tab has the focus, after Zed's own bindings:
  - a key with text and no modifier but Shift: `keyDown` with its text, then `keyUp`, so the
    page sees keydown, keypress, input and keyup as from a keyboard;
  - Enter (with the text `\r`, which submits forms), Tab, Backspace, Delete, Escape, the
    arrows, Home, End, Page Up and Down, Insert and F1 to F12: `rawKeyDown` and `keyUp`;
  - Ctrl and Alt chords: `rawKeyDown` with the modifiers, which run Blink's editing
    commands (select all, undo, word moves);
  - `code` and `windowsVirtualKeyCode` inferred for the US layout, since gpui carries no
    physical key code;
  - Super chords never reach the page (PR-claude-raw-input-passthrough-must-filter-platform-chords-001).
- **Composed and input-method text** through gpui's input handler, registered while the tab
  is focused: a compose sequence (the Compose key, dead keys) reaches the page once, as its
  character; an input method's preedit shows as the page's composition
  (`Input.imeSetComposition`), and its commit lands as text (`Input.insertText`); the
  candidate window anchors at the last point clicked in the tab.
- **Clipboard.** Ctrl+V inserts the system clipboard's text; Ctrl+C and Ctrl+X put the page's
  selection on the system clipboard, read in an isolated world (`Page.createIsolatedWorld`,
  so the page's scripts can neither see nor change it); Ctrl+X also cuts in the page.
- **Focus.** Focus emulation keeps the page focused while the tab is (caret and `:focus`).
- **Latency.** The time from each input to the next frame is logged at debug level
  (`marley_browser`); the scenario reports its median and 95th percentile.

### Out (explicitly deferred)
- Japanese or Chinese through a live input method: the path is built, the test waits (Chad,
  2026-09-24, "Compose only, CJK later").
- The page's cursor shape (headless Chromium reports none over CDP), right-click menus
  (Chromium draws none headless), file drag and drop, touch, pinch.
- Browser keys such as Ctrl+L and Ctrl+R (#490's key context).

## Reference (§20)
N/A — Marley-specific: no Warp or Zed counterpart. The behavior matched is Chromium's own
handling of the same input: every event is one of Chromium's input events, sent over CDP as
DevTools' screencast view and Puppeteer send them.

### Prior art
- **Published material.** CDP's Input domain (Chromium 152): `dispatchMouseEvent` (x and y in
  CSS pixels relative to the main frame's viewport, `buttons` as a bitmask, `clickCount`,
  `deltaX` and `deltaY` in CSS pixels), `dispatchKeyEvent` (`keyDown`, `keyUp`, `rawKeyDown`,
  `char`; `text`, `key`, `code`, `windowsVirtualKeyCode`; modifiers Alt 1, Ctrl 2, Meta 4,
  Shift 8), `insertText`, `imeSetComposition` (152 has no `imeCommitComposition`;
  `insertText` commits). The W3C UI Events tables for `code` values and legacy key codes.
- **Observed (the CDP probe, 2026-09-24).** `keyDown` with text types; `rawKeyDown`
  Backspace deletes; Ctrl+A (`rawKeyDown`, code 65, modifiers 2) selects the field;
  `insertText("é")` replaces the selection; a composition shows underlined and `insertText`
  commits over it; keys reach a cross-site iframe through the main page's session once it
  has focus; a wheel `deltaY` of 120 scrolls 120 CSS pixels; twenty wheel steps give twenty
  frames.
- **Code we already ship.** gpui's Linux key path (the Explore report): bindings, then
  `key_down` listeners, then the platform's text step, which runs only for a key that still
  propagates, has at most Shift and has `key_char`. A compose sequence sends
  `replace_and_mark_text_in_range` before its keys and ends with a key carrying the composed
  `key_char`; stopping that key's propagation leaves the marked text for the handler to clear.
  Wayland's text-input-v3 commits one byte as a synthesized key and longer text through
  `replace_text_in_range`. `EntityInputHandler`, `ElementInputHandler` and
  `window.handle_input`; the terminal's handler returns `Some(0..0)` from
  `selected_text_range` so input methods can place their window. `ScrollDelta::pixel_delta`.
  gpui's clipboard (`read_from_clipboard`, `write_to_clipboard`). The rich input's lesson
  (L-claude-481-gate-text-by-its-modifiers-not-prefer-character-input-001).

## UI proof
UI-AFFECTING. `script/e2e/489-browser-input.sh` with `COMPOSITOR=sway` (#487). A fixture
page with a text input, a textarea, a button that prints "clicked: button N, count M", a
dot drawn where each press landed, a key log, a 3000-pixel list that prints its scroll
offset, a paragraph to select, and a cross-site iframe with its own input. A stand-in agent
(node, over the profile's `DevToolsActivePort`) navigates to the fixture; the scenario then
clicks, types, scrolls and uses the clipboard (`wl-copy` and `wl-paste` against the sway).
`ZED_LOG=marley_browser=debug` makes the latency lines. Shots: `489-01-typed`,
`489-02-edited`, `489-03-composed`, `489-04-clicks`, `489-05-scrolled`, `489-06-iframe`,
`489-07-pasted`, `489-08-super-filtered`.

## Locked-In Decisions
- D1 — A printable key is sent as a key event with text, not as `insertText`: pages that
  listen for keydown (shortcuts, games, search-as-you-type) see a keyboard.
  `insertText` is for input-method commits and the clipboard.
- D2 — `code` and key codes are inferred for the US layout. `key` and the text are always
  exact; `event.code` on other layouts is best effort until gpui carries scancodes.
- D3 — One wheel detent scrolls 100 CSS pixels.
- D4 — Marley bridges the clipboard; headless Chromium's own clipboard is not the system's.
- D5 — Zed's bindings keep priority over the page (Ctrl+W closes the tab, Ctrl+Shift+P opens
  the palette). The browser's own keys arrive with #490's key context.
- D6 — The input method's candidate window anchors at the last click in the tab: without
  script, CDP reports no caret rectangle.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user presses and releases a mouse button in the Browser tab, the page shall receive both at the same point, with the button and the click count. | Shot `489-04-clicks`: the dot under each click, "clicked: button 0, count 2" after a double click, button 2 after a right click |
| REQ-002 | WHEN the user types into a focused field, the text shall appear as typed; Backspace, Delete, the arrows, Home, End and Enter shall act as in Chromium. | Shots `489-01-typed`, `489-02-edited` |
| REQ-003 | WHEN the user presses a Ctrl chord Blink edits with (Ctrl+A, Ctrl+Z), the page shall run it. | Shot `489-02-edited` |
| REQ-004 | WHEN a compose sequence completes, the composed character shall appear in the page once. | Shot `489-03-composed`: "é" once |
| REQ-005 | WHEN an input method sends a preedit and then commits, the page shall show a composition and then the committed text. | Not driven: no input method with an engine runs on the dev box (Chad's call); recorded, with the path reviewed |
| REQ-006 | WHEN the user turns the wheel over the page, the page shall scroll 100 CSS pixels per detent in the wheel's direction. | Shot `489-05-scrolled`: the list's offset after five detents down |
| REQ-007 | WHEN the user clicks into a cross-site iframe and types, the iframe's field shall receive the text. | Shot `489-06-iframe` |
| REQ-008 | WHEN the user presses Ctrl+V, the system clipboard's text shall be inserted; WHEN the user presses Ctrl+C over a selection, the selection shall be on the system clipboard. | Shot `489-07-pasted`; the run prints `wl-paste` after Ctrl+C |
| REQ-009 | WHEN the user presses a Super chord in the tab, the page shall receive nothing. | Shot `489-08-super-filtered`: the key log is unchanged |
| REQ-010 | WHEN an input reaches the page, the time to the next frame shall be logged, and the run shall report the median and 95th percentile. | The run's latency report |

## Phase Plan
- **P1 Plan** — this spec, and the design and E2E plan in the notes.
- **P2 Code** — the mouse and wheel mapping, the key mapping and its US tables, the input
  handler, the clipboard bridge, focus emulation, the latency log; fmt and clippy clean; a
  review of the diff.
- **P3 Test** — write and run `489-browser-input.sh`, read every shot; `script/gates.sh --diff`
  green.
- **P4 Complete** — CHANGELOG, `marley_browser.md`, the plan's answers to the spike's
  questions, ledger capture, close, archive, commit.
