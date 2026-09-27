# `marley_browser`

Marley's browser, prong 3 of `docs/marley/three-prong-plan.md`, written in the fork for B0a
(#488): the Chromium Marley starts for itself, the CDP client that talks to it, and the
decoding of the frames it streams. It holds no views; the Browser tab that draws them is
`marley_workbench`'s (`src/browser.rs`). MIT OR Apache-2.0, with the lint table of the other
Marley crates (CONSTITUTION §14), `future_not_send` and `unused_results` allowed as gpui calls
for.

## The service (`src/service.rs`)

- Each project's Chromium is a transient user unit, `marley-browser-<id>`, where `<id>` is the
  first twelve hex digits of the SHA-256 of its profile's path, so each project in each Marley
  data directory (an e2e run's included) has its own (#507). `systemd-run --user --quiet
  --collect --service-type=exec` starts it, with `KillMode=mixed` and a ten-second stop
  timeout; the unit outlives the tab, the window and Marley, and ends at logout or when its
  project is removed (plan D16).
- A project's folder is `<data dir>/browser/projects/<key>/` (`project_dir_in`), its profile
  `profile/` in it (`profile_in`). `project_key` is the first sixteen hex digits of the SHA-256
  of the project's main worktree paths, sorted, each followed by a newline, then of the host
  and a newline for a remote project: nothing in it changes at a restart. `write_project_file_in`
  makes the folder, which only its owner reads, and writes `project.json` once, through a
  temporary name: the paths, the host and when it was made (Unix seconds), for anyone reading
  the folder. `move_legacy_profile_in` renames `<data dir>/browser/profile`, the profile every
  build before #507 used, to a project's folder, unless a project has a profile already
  (`LegacyMove`: moved, none, or kept); the caller stops its Chromium first. `stop` runs
  `systemctl --user stop`, which answers once the unit has stopped; a unit that is not loaded
  is stopped already. `remove_profile_in` (#581) deletes a project's `profile/`, keeping
  `project.json`, after its Chromium has stopped; a profile that is not there is deleted
  already.
- The binary is `MARLEY_CHROMIUM` when that is set, and nothing else then; else
  `/usr/lib/chromium/chromium`, the browser behind Arch's and Debian's `/usr/bin/chromium`
  launcher, which would add the user's `chromium-flags.conf` (on Omarchy, three extensions and
  the keyring password store); else `chromium` or `chromium-browser` on the PATH.
- Chromium runs `--headless --remote-debugging-port=0 --user-data-dir=<project's profile>
  --no-first-run --no-default-browser-check --password-store=basic --no-startup-window`: it
  opens no page of its own (#494), and Marley opens the pages its tabs and agents ask for; without
  the flag or a URL, headless Chromium opens `chrome://newtab/`. Port 0 lets
  Chromium pick the port, which it writes with the browser's WebSocket path to
  `DevToolsActivePort` in the profile; `endpoint_in` reads it, for Marley and for any other
  CDP client. The basic password store keeps a headless service from ever waiting on the
  desktop keyring's unlock prompt.
- `unit_state` reads `systemctl --user is-active`; `remove_endpoint_in` removes the file a
  Chromium that is gone left behind.

## The client (`src/cdp.rs`)

- One WebSocket to the browser's endpoint on 127.0.0.1 (`async_tungstenite::client_async` over
  a smol `TcpStream`) carries every session. `Connection::call` sends a request with an id
  and, for a page, its flat session, and waits up to fifteen seconds for the answer; events go
  to one unbounded channel with their session.
- When the socket ends, every waiting call fails with the reason and the event channel
  closes, so a browser that went away reads as such (the #406 lesson: failures arrive as
  silence unless something turns them into a state). The reader and writer tasks live in the
  connection's shared inner, so the socket closes when the last clone drops.

## The page (`src/page.rs`) and the frames (`src/frame.rs`)

- The page's life (#493): `Page::discover` turns on target discovery, `page_ids` lists the
  browser's `page` targets, `create` opens one at a URL and answers with its id (its
  `targetCreated` comes first), `attach` attaches to one with `flatten`, enables the Page domain,
  focus emulation and the observers before it hands the page back, and `close` closes one by id,
  attached or not. A fresh headless Chromium also lists `browser_ui` and extension targets, so
  only `page` is a tab; it keeps running with no page at all, and `create` works then too.
- The viewport is `Emulation.setDeviceMetricsOverride` at the tab's size and the window's
  scale, after `Browser.setWindowBounds` gives the page's own headless window the same size
  (#494): a page made by `Target.createTarget` keeps the window of its first size, and laid out
  wider than that, a page with a cross-site iframe can stop sending screencast frames after a
  resize. The screencast is JPEG at quality 85, each frame acknowledged after it is decoded,
  which paces Chromium to Marley. `target_info` reads the page's title: Chromium reports a new
  URL as a target change, with the URL as the title, but never the title the document sets.
- `frame::decode` turns a frame's base64 JPEG into a BGRA `RenderImage`.

## Input (`src/input.rs`, #489)

- `key_press` maps a gpui keystroke to the down and up halves of `Input.dispatchKeyEvent`: a
  key with text and no Ctrl or Alt as `keyDown` with its text (the page sees keydown,
  keypress, input and keyup), Enter with the text `\r` so a keypress submits a form, the named
  keys and the Ctrl and Alt chords as `rawKeyDown`, which run Blink's editing commands, and
  nothing for a Super chord. gpui carries no physical key code, so `code` and the Windows key
  code come from the US layout's tables; `key` and the text are always what was typed.
- `mouse_event` and `wheel_event` build `Input.dispatchMouseEvent`'s parameters;
  `modifier_bits` counts Alt 1, Ctrl 2, Meta 4, Shift 8; `log_latency` logs the time from an
  input to the frame that showed it (`ZED_LOG=marley_browser=debug`).
- `Page::selected_text` reads the focused field's selection, or the document's, in an isolated
  world on the main frame, where the page's scripts neither see the read nor change it.
- The dev profile builds `image`, `zune-core`, `zune-jpeg` and this crate at `opt-level = 3`:
  unoptimized, a 1100 by 900 frame took 130 ms to decode, and a key waited behind it.

## Navigation and dialogs (`src/address.rs`, `src/page.rs`, #490)

- `address::url_for` turns what the address bar holds into a URL: text with a scheme Chromium
  navigates to (`http`, `https`, `file`, `about`, `data`, `chrome`, `view-source`) as typed; a
  host with its port and path (`localhost` or a name under it, an IPv4 address, a bracketed IPv6
  address, or a name with a dot whose last label is not a number) over `http` when it is
  loopback and `https` otherwise; anything else as `https://duckduckgo.com/?q=` and the
  form-encoded text. A typed `javascript:` URL is no known scheme, so it is searched for.
- `Page` navigates (`Page.navigate`, whose `errorText` becomes an error; the call answers at the
  commit, so a slow page holds it), reloads, stops, reads the session history
  (`NavigationHistory`, whose `entry_at(offset)` names the entry back or forward) and moves to an
  entry, and answers a JavaScript dialog (`Page.handleJavaScriptDialog`, with a prompt's text).
- `JavaScriptDialog` is `Page.javascriptDialogOpening`'s event: the frame's URL, the message,
  the kind (`alert`, `confirm`, `prompt`, `beforeunload`) and a prompt's default; `origin()` is
  the host and port the Browser tab's card names as asking. Headless Chromium draws no dialog,
  and the page's script waits until one is answered.
- A failed load commits Chromium's error page, whose `frameNavigated` URL is
  `chrome-error://chromewebdata/`; the URL that failed is `unreachableUrl`, which the tab shows.
- `address::local_url` (#503) reads an http or https URL on this machine: its host `localhost` or
  a name under it, a `127.0.0.0/8` address, `::1`, or the unspecified `0.0.0.0` and `::`, which
  become `127.0.0.1` and `::1` in the URL to open. It gives that URL, the `host:port` label a
  terminal's footer shows and the port, the scheme's own when none is named.
  `printed_local_urls` finds them in a terminal's line: each `http://` or `https://` up to a
  space, a quote or an angle bracket, less the sentence punctuation after it and a closing
  bracket it did not open (`(http://0.0.0.0:8000/)` from Python's `http.server`).

## Listening ports (`src/ports.rs`, #503)

- `listening_ports_in(dir)` reads `dir/tcp` and `dir/tcp6` (`/proc/net` on the machine) through
  `procfs-core`, which the tree already builds for `crashes`, and keeps the ports of the
  sockets in `LISTEN` bound to a loopback or unspecified address, IPv4-mapped ones included:
  those a local URL reaches. It fails only when neither table reads, since a machine without
  IPv6 has no `tcp6`. The workbench runs it off the main thread.

## Page icons (`src/favicon.rs`, #504)

- CDP has no favicon event, so the workbench reads a page's icon after its load.
  `Page::favicon_href` runs `FAVICON_HREF` in the isolated world: the first
  `link[rel~="icon" i]` whose href is http, https or `data:image/` (not `data:,`), else
  `/favicon.ico` at an http or https origin, else nothing, as Orca's `browser-favicon-url.ts`
  picks one.
- `format(bytes)` names the image format from its first bytes (PNG, ICO, GIF, JPEG, WebP, SVG),
  for gpui's `Image::from_bytes`; a server's content type is not trusted. `data_url_bytes`
  decodes a `data:` URL, base64 or percent-encoded, and `Page::load_bytes` loads any other; both
  stop at `MAX_BYTES`, 256 KiB.

## Titles (`src/title.rs`, #582)

- Chromium sends no CDP event for a title a script sets after the load:
  `Target.targetInfoChanged` carries a page's first title and not a later one. `WATCHER`, run in
  the isolated world `WORLD` (`marley-title`) of the page's main frame, keeps the last title it
  reported and calls the binding `BINDING` (`marleyTitle`) with `document.title` whenever a
  change in the document leaves it different. It reports from the top frame only, makes the title
  well formed (`toWellFormed`) and cuts it at 1,000 characters, never inside a surrogate pair,
  since half of one fails the whole CDP message (#518). A `MutationObserver` watches the whole
  document until `DOMContentLoaded` and the head alone after.
- `Page::watch_title(session)` sets it up as `watch_actions` does: the binding for the world,
  the script for each new document in the world, and a world in the main frame's loaded
  document.

## For agents (`src/snapshot.rs`, `src/observe.rs`, #492)

- `snapshot::render` writes one or more frames' accessibility trees (`AxNode`, from
  `Accessibility.getFullAXTree`) as text: by default each interactive node (buttons, links,
  fields, boxes, options, tabs and the like) on a line with its role, its trimmed name, its states
  and a ref (`e3`); with `full`, every node indented, unnamed containers folded, and since #498
  every element the tree names gets a ref too (text and the document do not). A cross-site
  iframe's nodes follow under a line that names its URL. A `RefTarget` says what each ref names:
  the session, the iframe's frame, the DOM node, the role and the name. No field's value is
  written: Chromium's value for a password field is a bullet a character. The text stops at 30,000
  characters, with a note.
- `observe` keeps the newest 200 console messages, uncaught errors and browser log entries
  (`ConsoleLog`) and requests (`NetworkLog`: method, URL, type, status, duration, failure; no
  headers, no bodies). `redact_url` drops a URL's user and password and hides the values of query
  and fragment parameters whose names hold `token`, `key`, `secret`, `password`, `auth`, `code`,
  `sig` or `session`.
- `Page` gains what the tools call: `call_in` (another session: a cross-site iframe's),
  `observe` (Runtime, Network, Log and auto-attach, turned on before the Browser tab shows the
  page, since a navigation sent sooner loads before the network is watched), `screenshot`,
  `viewport`, `frames`, `accessibility_tree`, `scroll_into_view`, `box_center`, `frame_origin`
  (an iframe's owner element's content box, which places a point in the iframe on the page) and
  `focused_element` (read in an isolated world; a password field's value never).
- `input::char_press` types a character as the keyboard does, and `input::chord` turns `Enter`,
  `Ctrl+A` or `Shift+ArrowLeft` into a key press; `address::agent_url` lets an agent open `http`
  and `https` URLs only.

## Select lists (`src/select.rs`, #495)

- Headless Chromium opens a `<select>`'s popup in a widget no frame shows. `LISTENER`, a script
  run in the isolated world `WORLD` of each frame, takes the press that would open a single,
  enabled select (a primary-button `mousedown`, and Alt+Down, Alt+Up, F4 or Space on a focused
  one), cancels it, which keeps the popup shut, gives the select the focus, and reports it
  through the binding `BINDING` as a `SelectRequest`: the select's box and the press in its
  frame's viewport, the selected index, and the options with text, disabled and group. The
  page's own listeners still get the event; a closed, focused select takes the arrows in the
  page as Chromium's does.
- `Page::watch_selects(session)` adds the binding for the world
  (`Runtime.addBinding` with `executionContextName`), the listener for each new document
  (`Page.addScriptToEvaluateOnNewDocument` with `worldName`), and, for each frame the session
  shows already, a new world with the listener evaluated in it. `Page::choose_option(session,
  context, index)` sets the select's index in the world that reported it and dispatches `input`
  and `change` when the choice differs; the events are not trusted.

## Picking (`src/pick.rs`, #496)

- `Page::set_inspect(on)` turns Chromium's inspect mode on (`DOM.enable`, `Overlay.enable`,
  `Overlay.setInspectMode` with `searchForNode` and a highlight that shows its tooltip and the
  accessibility info) or off (`none`). The highlight is in the frames, so the Browser tab draws
  nothing; a click in the page raises `Overlay.inspectNodeRequested` with the node's
  `backendNodeId` and does not reach the page.
- `Page::watch_scripts` turns on the Debugger domain with every pause skipped
  (`Debugger.setSkipAllPauses`), so a page's `debugger` statement never stops it; its
  `Debugger.scriptParsed` events name each script's URL and source map, the scripts loaded
  already included.
- `Page::capture_pick(backend_node_id, scripts)` reads the pick into a `PickBundle` in one pass:
  the node resolved and walked up, through open shadow roots, to its nearest interactive
  ancestor (a control, a link, an element with a tab index, an `onclick` or an interactive role),
  else its own element; `DESCRIBE`'s tag, text (a field's value never, a password's included;
  an `<input>` button's value is its label), locators (a test id, an id, the text, a CSS path,
  each marked when it finds the element alone in its document), box in the page through
  same-origin frames, and blockers (`pointer-events: none` on it or an ancestor, visibility,
  `display`, opacity, `disabled`, and the element on top at its middle); role and name from
  `Accessibility.getPartialAXTree`; and the listeners of the element, its ancestors, the
  document and the window (`DOMDebugger.getEventListeners` on each), at most 24, with their
  script's URL and source map from `scripts`. `PickBundle::summary` names it as the tray does:
  its role and name, else its tag and text.
- `Page::crop(page_box)` is the box with a 16-pixel margin, at one pixel to the CSS pixel, as a
  base64 JPEG at 80. Since #505 it captures the whole viewport (`Page.captureScreenshot` as a
  PNG, with `Page.getLayoutMetrics` giving the visual viewport's place and size) and cuts the
  box out on `smol::unblock`'s pool (`cut`), scaling by the capture's pixels to the CSS pixel.
  A capture with a clip can reach the page's running screencast as a frame of the clip alone,
  which the tab then draws in place of the page until the page next changes. The box must lie
  in the viewport, and a margin past its edge is left out.

## The fuller bundle (`src/pick.rs`, #518)

- `DESCRIBE` takes `observe::SECRET_NAMES` as its argument (`call_on_with` passes it), and in
  the same call, in the page's main world where React keeps its fiber, also reads:
  - `html`: the element cloned; its scripts removed; the `value` of every `input` but button,
    submit, reset and image, and every `textarea`'s text, removed; an attribute whose name holds a
    secret name, or whose value holds one of Orca's secret patterns, set to `[redacted]` (the
    value check skips `type`, `autocomplete`, `inputmode`, `role`, `id`, `name`, `for` and
    `class`, which name things); each URL attribute (`href`, `src`, `action`, `formaction`,
    `poster`, `cite`, `data`, `ping`, `xlink:href`, each `srcset` candidate) cut at `?` or `#`
    and stripped of a user and a password, a `data:` URL as `data:…`, and any scheme but http,
    https, file, about, mailto and tel as `[redacted]`. The walk stops after 20,000 elements.
  - `styles`: sixteen computed properties, Orca's list.
  - `nearby_text`: up to ten of the siblings' texts, before and after in turn; `selected_text`:
    the page's selection, none while a field has the focus.
  - `react`: the fiber under `__reactFiber$…` (or `__reactInternalInstance$…`) walked up
    `.return` for 35 levels: six component names with Orca's skip list, the first `_debugSource`
    on a fiber or its `_debugOwner`, and, when there is none, the host fiber's
    `_debugStack.stack`.
  Each read is its own `try`, so a failure empties one field and the pick still comes. Each
  text is made well formed, and each cut steps back from a lone surrogate, since one in the
  answer fails the parse of the whole CDP message. The page caps each text for the trip only,
  marked ` (truncated)`: the HTML at 65,536 characters, a text at 4,096, a stack at 4,000, a
  debug source and a style's value at 500, a component's name at 200.
- `PickBundle` gains `html`, `styles`, `nearby_text`, `selected_text` and `component`
  (`Component { chain, source }`, `ComponentSource { from, source, file, line, column }`, whose
  `from` is `debug source` or `debug stack`). `trip` holds each field to its cap again on
  arrival. `within(text, budget)` and `HTML_BUDGET` (4,096), `TEXT_BUDGET` (200) and
  `SELECTION_BUDGET` (500) are the cuts the workbench's `browser_pick` makes after its redaction.
  `stack_frames` parses V8's `at` lines into `StackFrame`s (lines and columns from 1), and
  `StackFrame::is_reacts` names React's own (`jsxDEV`, `jsx`, `jsxs`, `createElement`, the stack's
  bottom frame).

## Checking a pick (`src/pick.rs`, #505)

- The page functions share pieces of JavaScript through `macro_rules!` spliced in with `concat!`:
  `interactive!`, the test of an element a user acts on and the step up through shadow roots, which
  `INTERACTIVE_ANCESTOR` and `REFIND` take; and `generated_name!`, the test of an id a framework
  made (React's `useId` as `:r1:`, `«r1»`, `_r_1_` or `_R_1_`, `radix-…`, `headlessui-…`,
  `react-aria…`, `mui-7`, `ember12`, or 12 or more characters with a digit and a capital), which
  `DESCRIBE` takes. `DESCRIBE`'s id locator leaves such an id out, and its CSS path walks past one
  to the next ancestor's.
- `Page::refind(bundle)` finds the pick's element in the page's main document by the first kind
  of locator that finds anything, in `REFIND_ORDER`: the test id, the id, the role and name
  (`Accessibility.queryAXTree` on the document's object, each node resolved and its box read with
  `PAGE_BOX`), the text, the CSS path. `REFIND` runs the selector kinds and the text on the
  document. For a text, it takes each element whose text reads the pick's, walked up to its
  interactive ancestor, the innermost kept. Of several matches, the one whose center lies nearest
  the old box's wins. It answers a `Refound { found_by, backend_node_id }`, or none.
- `changes(before, after)` says what changed, a line each: the box moved or resized by at least a
  CSS pixel, each computed style that differs, the text, the role and the name, and, when nothing
  else did, `its HTML changed`. `CHANGE_BUDGET` (420) is what an agent gets of each line.

## Boxes (`src/page.rs`, #498)

- `Page::border_box(session, backend_node_id)` gives a node's border box in its frame's
  viewport (left, top, width, height), read from `DOM.getBoxModel`'s border quad; the content
  quad that `box_center` uses comes from the same read.

## The flight recorder (`src/recorder.rs`, #499)

- `Recorder` keeps a page's last minute (`WINDOW`, 60 seconds): `Entry`s (a press with its
  place, button and count; a scroll; a key by its name or a shortcut; typing as a count; a
  navigation; a console entry; a request with its method, redacted URL, status and failure; a
  snapshot; an agent's action; since #523 a Playwright script's start and its end with its exit
  code, `Script { name, exit_code }`), each with the `Instant` it came, typing added up and wheel
  turns within half a second added up; and frames as the base64 JPEGs Chromium sent, one at
  most every 500 ms (`FRAME_GAP`), 16 MiB of them at most (`FRAME_BYTES`). `request_ended`
  adds a status or a failure. `take(now)` gives the minute in time order, each frame an
  `Entry::Frame` numbered from 1, with its JPEGs and its length.
- `save_in(dir, recording, frames)` writes `<dir>/<id>/timeline.json` (the `Recording`: id,
  tab, URL, title, time, length, frame count, entries with their `at_ms`, and the snapshot at
  the save) and `<dir>/<id>/frames/NNNN.jpg`, a suffix on an id another recording has.
  `list_in`, `read_in` and `frame_in` read them back and refuse an id that is not letters,
  digits and dashes.
- `ConsoleLog::apply` answers the entry it kept, which the recorder copies.
- Since #506 the page's own session also runs an action listener (`LISTENER`, `WORLD`
  `marley-record`, `BINDING` `marleyRecord`), which `Page::watch_actions` sets up as
  `watch_selects` does: the binding, the script for new documents in the world, and a world in
  the main frame's loaded document. It works in the top frame only, at the window, in the
  capture phase, on trusted events:
  - a primary-button `pointerdown`, walked up to the interactive ancestor;
  - an `input` on a fillable field, with the text capped at 4,096 for the trip, or no text for
    a secret field (a password or hidden input, or an `autocomplete` of `current-password`,
    `new-password`, `one-time-code` or `cc-…`);
  - Enter, Tab or Escape pressed, on the focused element.
  Each report carries the target's locators, most durable first, each with `unique`: the test id
  and its attribute; the role (explicit, else ARIA in HTML's implicit one) and the accessible
  name; the label; the placeholder; the text; the CSS path. It also carries the page's URL and,
  for a fill, the field's `name` or its id when that was not generated. The listener takes its
  JavaScript pieces (`interactive!`, `generated_name!`, `css_path!`) from `pick.rs`, which
  re-exports them with `pub(crate) use`.
- `ReportedAction::parse` and `into_entry` make an `Entry::Action { action, locators, url, text,
  secret, key, field }`: the URL through `redact_url`, each text held to its trip cap, no text for
  a secret field whatever the page sent, and only Enter, Tab and Escape as keys.
  `Recorder::push` merges a fill into the last action when that is a fill with the same first
  locator, keeping its first time. `Entry::Navigation` gains `within` for a move within the
  document, and `Recording` gains `project`, the root of the tab's project.

## Drafting a Playwright test (`src/playwright.rs`, #506)

- `draft(timeline) -> Result<Draft { text, env, skipped, start }, DraftError>`, pure, over a saved
  timeline. It fails with no action to replay, or with a first action's page that is not http
  or https.
- The test imports `@playwright/test`. When a secret field is filled, it adds a `secret(name)`
  helper that throws `set <NAME> to run this test` for an unset variable. It sets `test.use({
  baseURL })` to the first action's origin and runs one test, named for the recording's title
  and id: `page.goto` to the first action's path, then a step per action.
- Each step uses the first locator marked `unique` that Playwright can name: `getByTestId`
  (`data-testid`; a `locator('[attr="…"]')` for the other test attributes), `getByRole` with
  the exact name, `getByLabel`, `getByPlaceholder`, `getByText` (each `exact: true`), or
  `locator(<css>)`. A fill writes its text, or `secret('<NAME>')` named for the field's name, id
  or label in upper snake case. A press with no locator presses on the page's keyboard.
- After each action that a navigation followed, it writes `expect(page).toHaveURL(<path>)`, or a
  `RegExp` on the path when the recorder hid a value in the URL. An action it cannot replay
  becomes a `// Skipped …` comment and a line in `skipped`.

## Source maps (`src/source_map.rs`, #497)

- `map_location(script_url, source_map_url)` decodes a `data:` map (base64 or percent-encoded)
  as `Inline`, and joins any other URL to the script's as `Remote`. `Page::load_resource(url)`
  loads a remote map as Chromium's own tools do, through `Page::load_bytes(url, cap)`:
  `Network.loadNetworkResource` in the page's main frame (its frame id is the target's), its
  stream read with `IO.read` to the end and closed, and past `cap` an error (#504). A map is 32
  MiB at most and must be UTF-8, and a failed load names its status.
- `SourceMap::parse(text, base)` takes a version 3 map: its `sources` joined to `sourceRoot` and
  resolved against the map's URL (the script's for an inline map), its `mappings` kept as text,
  and one level of an index map's `sections`, each with its offset. `SourceMap::original(line,
  column)` finds the section, then scans the base64 VLQ segments to the place: the last mapping
  at or before it, where a one-value segment maps to nothing. Lines and columns count from 0.
- `source_path(source)` gives the path a source names for a lookup: the scheme and host taken
  off (`webpack://app/`, `http://localhost:5173/`), a `file:` URL and Vite's `/@fs/` marked
  absolute, each component percent-decoded, `.` dropped and `..` applied.
- Since #497 a pick's `Listener` counts its line and column from 1, and carries `original`, a
  `SourcePosition` (the source, the file in the user's project when the workbench finds one, the
  line and the column, from 1).

## What the probe answered (2026-09-24)

A key dispatched over CDP reaches a frame in 6 to 7 ms; frames come at the size the device
metrics set; the Overlay domain's highlights are in every session's frames, the agent's too;
a cross-site iframe renders in the frame; `<select>` popups do not; frames stay at 1× under a
larger emulated scale, so a HiDPI screen needs `--force-device-scale-factor` at the service's
start.
