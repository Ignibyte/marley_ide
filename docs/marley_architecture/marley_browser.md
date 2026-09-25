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

## What the probe answered (2026-09-24)

A key dispatched over CDP reaches a frame in 6 to 7 ms; frames come at the size the device
metrics set; the Overlay domain's highlights are in every session's frames, the agent's too;
a cross-site iframe renders in the frame; `<select>` popups do not; frames stay at 1× under a
larger emulated scale, so a HiDPI screen needs `--force-device-scale-factor` at the service's
start.
