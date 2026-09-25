# `marley_browser`

Marley's browser, prong 3 of `docs/marley/three-prong-plan.md`, written in the fork for B0a
(#488): the Chromium Marley starts for itself, the CDP client that talks to it, and the
decoding of the frames it streams. It holds no views; the Browser tab that draws them is
`marley_workbench`'s (`src/browser.rs`). MIT OR Apache-2.0, with the lint table of the other
Marley crates (CONSTITUTION §14), `future_not_send` and `unused_results` allowed as gpui calls
for.

## The service (`src/service.rs`)

- Marley's Chromium is a transient user unit, `marley-browser-<id>`, where `<id>` is the first
  twelve hex digits of the SHA-256 of the profile's path, so each Marley data directory (an
  e2e run's included) has its own. `systemd-run --user --quiet --collect --service-type=exec`
  starts it, with `KillMode=mixed` and a ten-second stop timeout; the unit outlives the tab,
  the window and Marley, and ends at logout (plan D16).
- The binary is `MARLEY_CHROMIUM` when that is set, and nothing else then; else
  `/usr/lib/chromium/chromium`, the browser behind Arch's and Debian's `/usr/bin/chromium`
  launcher, which would add the user's `chromium-flags.conf` (on Omarchy, three extensions and
  the keyring password store); else `chromium` or `chromium-browser` on the PATH.
- Chromium runs `--headless --remote-debugging-port=0 --user-data-dir=<data dir>/browser/profile
  --no-first-run --no-default-browser-check --password-store=basic about:blank`. Port 0 lets
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

- `Page::attach_first` turns on target discovery, attaches to the browser's first `page`
  target with `flatten` (a new `about:blank` when there is none), enables the Page domain and
  focus emulation. A fresh headless Chromium also lists `browser_ui` and extension targets, so
  only `page` is a tab.
- The viewport is `Emulation.setDeviceMetricsOverride` at the tab's size and the window's
  scale; the screencast is JPEG at quality 85, each frame acknowledged after it is decoded,
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

## What the probe answered (2026-09-24)

A key dispatched over CDP reaches a frame in 6 to 7 ms; frames come at the size the device
metrics set; the Overlay domain's highlights are in every session's frames, the agent's too;
a cross-site iframe renders in the frame; `<select>` popups do not; frames stay at 1× under a
larger emulated scale, so a HiDPI screen needs `--force-device-scale-factor` at the service's
start.
