---
pipeline_id: 4abba6cb-8b6c-4aca-9556-8d309cac3348
ticket: docs/planning/tickets/open/TICKET-488-browser-pane.md
status: Phase 4 — Complete PASS
title: "B0a: A Browser tab that shows Marley's own Chromium"
type: feature
slice: prong 3 B0a
references: [docs/marley/three-prong-plan.md, docs/marley/browser-handoff.md, docs/planning/intake/embedded-agent-browser-chromium-cdp.md, docs/marley_architecture/embedded-browser-model.md]
---

## Title
Marley starts a Chromium of its own and shows its page in a Browser tab, drawn from CDP
screencast frames at the tab's size. The rendering half of the B0 spike; input is #489.

## Scope
### In
- A new crate, `crates/marley_browser` (`MIT OR Apache-2.0`), with no gpui views:
  - **The service.** It finds the Chromium binary: `MARLEY_CHROMIUM` when set, else
    `/usr/lib/chromium/chromium` (the browser behind Arch's and Debian's `/usr/bin/chromium`
    launcher, which would add the user's `chromium-flags.conf`), else `chromium` or
    `chromium-browser` on PATH. It starts it with `systemd-run --user --quiet --collect
    --unit=marley-browser-<id> --service-type=exec` and
    `--headless --remote-debugging-port=0 --user-data-dir=<data dir>/browser/profile
    --no-first-run --no-default-browser-check about:blank`, where `<id>` is the first twelve
    hex digits of the SHA-256 of the profile path, so each Marley data directory (an e2e run's
    included) has its own unit. An existing unit is reused: Marley reads the profile's
    `DevToolsActivePort` and connects before starting anything.
  - **The CDP client.** JSON-RPC over a WebSocket (`async_tungstenite::client_async` over a
    smol `TcpStream` to 127.0.0.1), requests keyed by id with a timeout, events routed by
    `sessionId`, and every waiter failed with the cause when the socket closes. Connecting
    retries every 100 ms for up to 10 s and gives up early when the unit has failed.
  - **The page session.** The first `page` target (a new `about:blank` when there is none),
    attached with `flatten`; Page enabled; the device metrics set to the tab's size in logical
    pixels at the window's scale factor; focus emulation on; the screencast started (JPEG,
    quality 85); each frame acknowledged after it is decoded, which paces Chromium to Marley.
  - **Frames.** Decoded off the main thread: base64, then JPEG, then BGRA for a `RenderImage`.
- `crates/marley_workbench/src/browser.rs`, the Browser tab (a `workspace::Item`):
  - `marley::OpenBrowser` (the palette's "marley: open browser") opens it in the active pane,
    or activates the one already open in the workspace;
  - it draws the newest frame to fill the tab and frees each frame from the GPU atlas two
    paints later, as Zed's screen-share view does;
  - the tab shows the page's title (from `Target.targetInfoChanged`), a web icon and the URL
    as its tooltip;
  - it draws its states in place: "Starting Chromium…", "Connecting…", and the reasons it
    stopped (no Chromium found, with the paths tried; the unit failed; the connection
    closed), with `marley: open browser` as the way to try again;
  - when the tab's size changes, the device metrics follow (once per new size).
- Chromium keeps running when the tab closes and when Marley quits; the next
  `marley: open browser` attaches to the same page.

### Out (explicitly deferred)
- Any input into the page (#489), navigation chrome and dialogs (#490), tabs as targets and
  restore (#493), the agent tools (#492).
- HiDPI frames (`--force-device-scale-factor` at the service's start; the dev box is at 1×).
- Stopping or restarting the service from Marley; audio, downloads, file choosers.

## Reference (§20)
N/A — Marley-specific: neither Warp nor Zed has a browser. What the tab shows is Chromium's
own compositor output, unmodified, so the behavior reference is the page in Chromium. The
embedding follows Chrome DevTools' screencast, which shows and drives a remote device's page
the same way (`Page.startScreencast`, `Input.dispatch*`).

### Prior art
- **Published material.** CDP 1.3 as Chromium 152 serves it at `/json/protocol` (read this
  session): `Page.startScreencast` (format, quality, maxWidth/maxHeight, everyNthFrame),
  `Page.screencastFrame` with `ScreencastFrameMetadata` (offsetTop, pageScaleFactor,
  deviceWidth and deviceHeight in DIP, scrollOffsetX and Y in CSS pixels, timestamp),
  `Page.screencastFrameAck`; `Target.getTargets`, `attachToTarget` with `flatten`,
  `targetInfoChanged`; `Emulation.setDeviceMetricsOverride` and
  `setFocusEmulationEnabled`. With `--remote-debugging-port=0` Chromium chooses a port,
  listens on 127.0.0.1 only and writes the port and the browser's WebSocket path to
  `<profile>/DevToolsActivePort`. systemd-run(1) transient user units.
- **Observed (the CDP probe, 2026-09-24).** Frames arrive at exactly the size the device
  metrics set (900×560); a key reaches a frame in 6 to 7 ms; the Overlay highlight and a
  cross-site iframe render in frames; `<select>` popups do not; frames stay at 1× under an
  emulated larger scale. A fresh headless Chromium lists `browser_ui` and extension targets
  beside its page, so only `type == "page"` is a tab. `/usr/bin/chromium` on the dev box adds
  Omarchy's three extensions and the keyring password store from `chromium-flags.conf`.
- **Code we already ship.** Zed's screen-share view
  (`livekit_client/src/remote_video_track_view.rs`) streams `RenderImage`s on Linux and drops
  frame N-2 while painting N; the atlas never evicts on its own, and freeing the frame on
  screen can panic on a re-present. `repl/src/outputs/image.rs` decodes base64 JPEG to a BGRA
  `RenderImage`. `async_tungstenite::client_async` is compiled with no runtime feature and
  takes a smol `TcpStream` (`dap` connects smol streams already). `dap/src/transport.rs`'s
  pending-request map and its connect-retry loop, and `context_server`'s request with a
  timeout, are the client's models. `util::command::new_command` runs `systemd-run`. Rusty
  starts its agent hosts as `systemd-run --user --quiet --collect --unit=rusty-agent-<id>
  --service-type=exec`. `workspace::Item` needs only `tab_content_text` and an event type;
  `theme_preview.rs` is the smallest model, and `ItemEvent::UpdateTab` retitles a tab. No
  crate in `Cargo.lock` speaks CDP (no `chromiumoxide` or `headless_chrome`); the client is
  small enough to own.

## UI proof
UI-AFFECTING. `script/e2e/488-browser-pane.sh` with `COMPOSITOR=sway` (#487). Fixtures: two
loopback HTTP servers (python3's `http.server`), one serving a page titled "Marley fixture"
that shows its own `innerWidth` and embeds an iframe from the other (a different site); a
stand-in agent, a small node script, that reads `$E2E_PROFILE/browser/profile/DevToolsActivePort`,
attaches to the same browser, navigates the page and highlights its button with
`Overlay.highlightNode`. `teardown` stops the e2e run's `marley-browser-*` unit and the
servers. Shots: `488-01-starting`, `488-02-page`, `488-03-highlight`, `488-04-narrower`,
`488-05-reopened`, `488-06-no-chromium`.

## Locked-In Decisions
- D1 — Marley owns the Chromium (Chad, 2026-09-24, "Marley starts it"): a transient user unit
  started on first use, one per data directory; it outlives the tab, the window and Marley.
- D2 — The browser binary itself, never the distribution's launcher: Marley's browser gets
  only Marley's flags, and never the user's extensions.
- D3 — Port 0 and `DevToolsActivePort`: profiles never collide over a port, and any CDP client
  finds the endpoint in the profile.
- D4 — Screencast frames into `RenderImage`s (plan D12), JPEG at quality 85, acknowledged
  after decoding; each frame freed two paints after it was first drawn.
- D5 — The viewport follows the tab through the device-metrics override at the window's
  scale factor; at 1× the frame is drawn pixel for pixel.
- D6 — One shared page: the tab shows the browser's first page target, the page an agent
  attached to the same browser sees and drives.
- D7 — `marley_browser` holds no views; the tab is `marley_workbench`'s, and
  `marley_workbench` is still the only Marley crate `crates/zed` calls.
- D8 — The connection and the page session live in one app-wide entity, not in the tab: the
  tab draws from it, and the agent tools (#492) and later tabs (#493) share it. Closing the
  tab stops the screencast, not the connection.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user runs `marley: open browser` and no Marley Chromium runs for this data directory, the system shall start one as a transient user unit from the Chromium binary itself, with its profile under Marley's data directory and CDP on 127.0.0.1 only, and open a Browser tab in the active pane. | Shot `488-01-starting`; the run prints the unit's command line and the listening address |
| REQ-002 | WHILE the tab is shown, it shall draw Chromium's frame of the page at the tab's size, with the page's title on the tab. | Shot `488-02-page` |
| REQ-003 | WHEN another CDP client attached to the same browser navigates the page or highlights an element, the tab shall show it. | Shots `488-02-page` (navigated by the stand-in) and `488-03-highlight` |
| REQ-004 | WHEN the page embeds a cross-site iframe, the tab shall show the iframe's content. | Shot `488-02-page` |
| REQ-005 | WHEN the tab's size changes, the page shall be laid out again at the new size. | Shot `488-04-narrower`: the fixture's reported width matches the narrower tab, and nothing is stretched |
| REQ-006 | WHEN the tab is closed and `marley: open browser` runs again, the tab shall show the same page from the same Chromium. | Shot `488-05-reopened`; the run shows one unit, still active |
| REQ-007 | WHEN no Chromium binary is found, the tab shall say so and name the paths it tried. | Shot `488-06-no-chromium` (a relaunch with `MARLEY_CHROMIUM` pointing at a missing file and no running unit) |
| REQ-008 | Frames shall not accumulate in the GPU atlas: each is freed two paints after it was drawn. | Review of the diff against Zed's screen-share view |

## Phase Plan
- **P1 Plan** — this spec, and the design and E2E plan in the notes.
- **P2 Code** — `marley_browser` (service, CDP client, page session, frames); the Browser tab
  and `marley::OpenBrowser`; the workspace member and dependency (the `Cargo.toml` ledger
  row); fmt and clippy clean; a review of the diff.
- **P3 Test** — write and run `488-browser-pane.sh`, read every shot; `script/gates.sh --diff`
  green.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_browser.md`, the plan's
  status, ledger capture, the brain decision (consultation 6fc61fddf7124217b31f3ddc9432bf01),
  close, archive, commit.
