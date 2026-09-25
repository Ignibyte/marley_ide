# B0a: A Browser tab that shows Marley's own Chromium — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-488-browser-pane.md
- **Pipeline spec:** 488-browser-pane.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-24)
- **Request:** wave 1 of prong 3; Chad's answers on 2026-09-24: Marley starts the Chromium;
  compose only, CJK later; "do your best for decisions".
- **Classification:** feature; a new Marley crate plus a new module in `marley_workbench`.
  Zed paths: the root `Cargo.toml` (a member and a dependency; its ledger row exists and
  grows). No Zed crate's source changes.
- **Recall (§18.3):**
  - PR-claude-callback-stored-inside-owned-resource-captures-weak-001: the frame task and any
    CDP event subscription the view owns hold the view weakly.
  - The #406 lesson (docs/marley_architecture/embedded-browser-model.md): transport failures
    arrive as silence; the tab needs its own liveness (the socket's close fails every waiter
    and draws "the connection closed").
  - L-claude-482-background-work-in-a-marley-crate-is-a-lazy-future-001: blocking work
    (reading `DevToolsActivePort`, decoding JPEG) is
    `background_spawn(futures::future::lazy(…))`.
  - The gate:21 dylint rules in Marley crates: `SharedString::new_static`, no entity update or
    notify in render.
  - Brain consultation 6fc61fddf7124217b31f3ddc9432bf01 (opened for B0; no prior decision on
    the seam).
- **Discovery (this session):** the three Explore reports (gpui frames and input; Zed's items
  and Marley's patterns; the transport, Rusty's units, `marley_mcp`), and the CDP probe's
  captures in the scratchpad.
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.

## Phase 1 — Plan (promoted 2026-09-24)
- **Pre-flight:** cargo idle, no active pipeline, README marker present, #487 landed (the sway
  backend the scenario uses).
- **Seams re-verified:** `marley_workbench::init` calls each module's `init` and is the only
  Marley init `crates/zed` calls; its manifest, its lint table and its `actions!(marley, …)`;
  the root `Cargo.toml`'s `# Marley:` member and dependency blocks (the ledger row counts the
  members); every dependency the crate needs is a workspace dependency already
  (`async-tungstenite` 0.33 with no runtime feature, `base64`, `futures`, `image` with jpeg,
  `log`, `serde`, `serde_json` with `raw_value`, `sha2`, `smol`, `which`, `util`, `paths`,
  `gpui`). `workspace::Item` needs `tab_content_text` and `Event`; `tab_icon`,
  `tab_tooltip_text` and `to_item_events` are defaulted. gpui's `Element` trait
  (`request_layout`, `prepaint`, `paint`), `Window::paint_image(bounds, image_bounds, corners,
  data, frame_index, grayscale)`, `Window::drop_image`, `Window::scale_factor`. The screen-share
  view's N-2 drop in `render` and its `on_release` drops.

### Design
- **`crates/marley_browser`** (new, `MIT OR Apache-2.0`, the workbench's lint table, no views):
  - `service.rs` — the Chromium binary (`MARLEY_CHROMIUM` alone when set, else
    `/usr/lib/chromium/chromium`, else `chromium` or `chromium-browser` from `which`); the unit
    name (`marley-browser-` + 12 hex of SHA-256 of the profile path); the `systemd-run` and
    Chromium argument lists; `devtools_endpoint_in(profile)` reading `DevToolsActivePort`;
    `unit_state` (`systemctl --user is-active`); `start`. Chromium runs with `--headless
    --remote-debugging-port=0 --user-data-dir=<profile> --no-first-run
    --no-default-browser-check --password-store=basic about:blank` (basic: a headless service
    must never wait on a keyring prompt on the user's desktop; the profile sits in the user's
    home either way). The unit: `--user --quiet --collect --unit=… --description=…
    --service-type=exec --property=KillMode=mixed --property=TimeoutStopSec=10`.
  - `cdp.rs` — `Connection` (a cheap clone over one `Arc`): `connect(port, path, executor)`
    opens a smol `TcpStream` to 127.0.0.1, runs `async_tungstenite::client_async`, and spawns
    a reader task (responses to their waiters by id, events to one unbounded channel, every
    waiter failed with the cause when the socket ends) and a writer task; `call(method,
    params, session)` with a 15-second timeout. The tasks live in the shared inner, so the
    socket closes when the last clone drops. `CdpError`: closed, protocol error, timeout.
  - `page.rs` — `Page`: attach to the first `page` target (creating `about:blank` if none)
    with `flatten`; `Page.enable`, focus emulation, `Target.setDiscoverTargets`; viewport,
    screencast start and stop, frame ack; the event payload types (`ScreencastFrame`,
    `FrameMetadata`, `TargetInfo`).
  - `frame.rs` — base64, then JPEG, then BGRA into an `Arc<RenderImage>`.
- **`crates/marley_workbench/src/browser.rs`** (new):
  - `BrowserHub`, one entity per app behind a `Global`: its state (starting, connecting,
    showing, failed with a reason), the page, the newest frame and its metadata, the title and
    URL, the viewport, the number of Browser tabs showing it. `ensure` runs the start path:
    connect through `DevToolsActivePort` if it answers; otherwise, unless the unit is active,
    remove a stale port file and start the unit; then wait up to 15 seconds for the file and a
    connection, failing early when the unit stops. The event loop decodes each frame in the
    background, stores it, acknowledges it and notifies; `targetInfoChanged` for the page
    updates the title and URL (a `HubEvent` the tabs turn into `ItemEvent::UpdateTab`); the
    page's destruction reattaches; the channel's end is "the connection closed". The
    screencast runs while at least one Browser tab exists.
  - `BrowserView`, the Browser tab (`workspace::Item`): observes the hub; draws its state text
    or a `PageElement` with the newest frame, freeing frame N-2 in `render` and both kept
    frames on release; tab text is the page title (else "Browser"), the tooltip its URL, the
    icon a globe.
  - `PageElement`, a small gpui element: fills its space, tells the hub its size and the
    window's scale factor in `prepaint` when they change, and paints the frame with
    `paint_image`.
  - `marley::OpenBrowser` on every workspace: activates the workspace's Browser tab or adds
    one to the active pane, and restarts the hub when it had failed.
- **Manifest:**
  - `crates/marley_browser/{Cargo.toml,src/marley_browser.rs,src/service.rs,src/cdp.rs,src/page.rs,src/frame.rs}` — new Marley crate.
  - `crates/marley_workbench/{Cargo.toml,src/marley_workbench.rs,src/browser.rs}` — Marley crate.
  - `Cargo.toml` (root) — Zed path: a member and a `[workspace.dependencies]` entry in the
    `# Marley:` blocks; its ledger row updated first. `Cargo.lock` follows.
  - `script/e2e/488-browser-pane.sh`, `script/e2e/488-no-chromium.sh` — Test.

### E2E plan
| REQ | Scenario step | Shot / evidence |
|---|---|---|
| REQ-001 | `488-browser-pane.sh` (sway): the palette's "marley: open browser" | `488-01-starting`; the run prints `systemctl --user show` of the unit (command line) and `ss -ltnp` for its port |
| REQ-002, 003, 004 | the stand-in agent (node over the profile's `DevToolsActivePort`) navigates to the fixture page with its cross-site iframe | `488-02-page` |
| REQ-003 | the stand-in highlights the button (`Overlay.highlightNode`) | `488-03-highlight` |
| REQ-005 | the palette's "workspace: toggle right dock" widens the tab | `488-04-wider` (the fixture shows its `innerWidth`) |
| REQ-006 | Ctrl+W closes the tab; "marley: open browser" again | `488-05-reopened`; one unit, still active |
| REQ-007 | `488-no-chromium.sh`: `MARLEY_CHROMIUM` at a missing file | `488-06-no-chromium` |
| REQ-008 | review against the screen-share view | — |

### Risks
- Two start attempts at once (two windows): the second `systemd-run` fails with the unit
  already loaded, which is treated as running.
- A crashed Chromium leaves a stale `DevToolsActivePort`; the start path removes it only after
  the unit is known to be down.
- `--password-store=basic` keeps cookies with Chromium's fixed key: the protection is the home
  directory's permissions and the disk's encryption, as for the keyring's own files.

## Phase 1 — closeout
Phase 1 PASS (autonomous). Checklist: pre-flight ✓, recall ✓, promote ✓, prior art ✓, spec ✓,
design ✓.

## Phase 2 — Code
- **Built:** `crates/marley_browser` (new): `service.rs` (the binary lookup with
  `MARLEY_CHROMIUM`, the unit name, the Chromium and `systemd-run` arguments, `start`,
  `unit_state`, `endpoint_in`, `remove_endpoint_in`), `cdp.rs` (the WebSocket client:
  id-keyed waiters, one event channel, every waiter failed with the reason when the socket
  ends, a 15-second call timeout; the socket closes when the last clone drops), `page.rs`
  (attach to the first page with `flatten`, the viewport, the screencast, the ack; the event
  payload types), `frame.rs` (base64 JPEG to a BGRA `RenderImage`). `marley_workbench`:
  `browser.rs` (the hub as a global entity with its start path and event loop; the Browser tab
  with the N-2 frame drop and its release drops; `PageElement`, which reports the tab's size in
  `prepaint` and paints the frame at its own size from the top left), the `OpenBrowser`
  action and `browser::init`. The root `Cargo.toml` member and dependency, after the ledger row.
- **Deviations:** `Target.targetCreated` is handled beside `targetInfoChanged`: discovery
  reports the pages that already exist as created, so without it the first page's title never
  reached the tab. A restart clears the title and URL. Frames are not filtered by session:
  the connection carries one page session until #493 adds one per tab, which must route by
  session.
- **Review of the diff:** the three gaps above, found by reading the event flow against
  CDP's discovery semantics; clippy's pedantic and nursery findings fixed at the source (doc
  backticks, `Arc::clone`, a lock held across a `match`, `as_chunks_mut`, parameters that
  needed no `&mut`, method references, a field that was not unused). The one `allow` (a cast
  of a length clamped to 1..=16384) carries its reason on its line, as gate:12 requires.
  Re-entrancy: the element updates the hub in `prepaint`, never in `render`, and the hub is not
  being updated then; the tab updates the hub at creation and release only.

## Phase 3 — Test
- **Scenarios:** `script/e2e/488-browser-pane.sh` and `script/e2e/488-no-chromium.sh`, both
  `compositor sway`, over a new shared fixture, `script/e2e/browser-fixture.sh` (loopback web
  servers, a node stand-in agent over the profile's `DevToolsActivePort`, the run's unit name,
  and a teardown that stops the unit and the servers).
- **First run: two reds.** The tab showed the page's URL, not "Marley fixture": Chromium
  reports a URL change as a target change but never the document's title (a probe confirmed
  it), so the hub now asks for the target's info after DOMContentLoaded, load and
  same-document navigations. And the scenario hung on a bare `wait`, which also waited for the
  harness's keyboard holder, until `timeout` sent SIGTERM; the harness's EXIT trap did not run
  on the signal and left the run's sway, Marley and Chromium up (cleaned by hand, then
  F-claude-488-a-signal-skipped-the-e2e-cleanup-001: the harness traps INT, TERM and HUP now,
  and the scenario waits for the agent's pid).
- **Shots (second run):**
  - `488-01-starting` — the Browser tab opened beside the terminal, a globe on its tab,
    showing `about:blank` from the fresh Chromium. The run printed the unit
    `marley-browser-626da090e995: active` with `path=/usr/lib/chromium/chromium --headless
    --remote-debugging-port=0 --user-data-dir=<the e2e profile>/browser/profile --no-first-run
    --no-default-browser-check --password-store=basic`, listening on `127.0.0.1:41135` only.
    (REQ-001)
  - `488-02-page` — after the stand-in agent navigated its own session: the fixture page,
    "Marley fixture" on the tab, "The page is 1100 x 902 CSS pixels." (the tab's size: 1600
    less the 260-pixel rail and the 240-pixel dock, 1000 less the bars), and the green
    cross-site frame from localhost. (REQ-002, REQ-003, REQ-004)
  - `488-03-highlight` — the stand-in's `Overlay.highlightNode` on the button, from its own
    session, drawn in Marley's frame with the box model and the accessibility tooltip
    (button#button, name "A button", role button, keyboard-focusable). (REQ-003)
  - `488-04-wider` — after "workspace: toggle right dock": "The page is 1340 x 902 CSS
    pixels.", laid out again, nothing stretched. (REQ-005)
  - `488-05-reopened` — after Ctrl+W and "marley: open browser": the same page and title;
    "units: 1 running; this run's is active". (REQ-006)
  - `488-06-no-chromium` — with `MARLEY_CHROMIUM` at a missing file: "No Chromium at
    …/no-chromium-here (named by MARLEY_CHROMIUM)" and "Run “marley: open browser” to try
    again.", centered (a first run showed the long path wrapped to the left and literal
    backticks; fixed and run again). No unit was started. (REQ-007)
- **REQ-008 (review):** the tab keeps the frame it drew last and the one before, frees the
  older when a new frame is drawn and both on release, the screen-share view's rule.
- **Run reports:** each run: "hyprland: 0 Marley windows before the run, 0 after; the run added
  no rule and did not reload it" and "sway: stopped, with the run's Marley, pointer and
  keyboard"; no `marley-browser-*` unit after either.
- **Gate:** `just gate-diff` → `16 passed, 0 failed`, `GATE GREEN [diff]`; receipt matches.
- **Pre-existing:** none.

## Phase 4 — Complete
- **Docs (§21):** CHANGELOG (Added: a Browser tab); `docs/marley_architecture/marley_browser.md`
  (new); the Browser tab section in `docs/marley_architecture/marley_workbench.md`; the plan's
  prong 3 slices table (#488 shipped); the root `Cargo.toml` row in
  `docs/marley/zed-touchpoints.md` names the ninth member and `marley_browser` among the
  dependencies, as shipped.
- **Ledger:** F-claude-488-a-signal-skipped-the-e2e-cleanup-001,
  PR-claude-scripts-that-start-detached-processes-trap-the-signals-001,
  L-claude-488-chromium-reports-a-new-url-not-the-documents-title-001,
  L-claude-488-an-overlay-from-any-session-shows-in-every-screencast-001,
  AD-claude-488-marleys-browser-is-a-transient-unit-streamed-into-a-tab-001.
- **Brain:** consultation 6fc61fddf7124217b31f3ddc9432bf01 closed as
  `decisions/marleys-browser-a-transient-chromium-unit-streamed-into-a-gpui-tab-over-cdp`
  (follow-up 2026-10-24).
