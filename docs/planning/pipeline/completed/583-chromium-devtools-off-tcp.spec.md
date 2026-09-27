---
pipeline_id: a3d5afff-fdef-4aa1-ac1e-372600b510b7
ticket: docs/planning/tickets/open/TICKET-583-chromium-devtools-off-tcp.md
status: Phase 4 — Complete PASS
title: "Chromium's DevTools off TCP, behind Marley's relay"
type: feature
slice: prong 3, B8b (slice 2 of #524)
references: [docs/planning/pipeline/completed/524-trusted-outside-browser-access.notes.md, docs/planning/pipeline/completed/507-browser-context-per-project.notes.md, docs/planning/pipeline/completed/523-saved-playwright-scripts.notes.md]
---

## Title
Each project's Chromium listens for DevTools on a TCP port on 127.0.0.1 with no credential, and
a loopback port checks no user, so every process on the machine, other users' included, can
drive the browser and read its cookies. Chromium now runs on its debugging pipe as the child of a
relay in the same unit, the relay being a hidden mode of Marley's own executable. The relay gives
each client a browser session of its own over the one pipe, and it serves Marley over a Unix
socket in the project's 0700 folder, and Playwright clients over a loopback WebSocket that takes a
token it mints at each start. Chromium itself listens on no port.

## Scope
### In
- `crates/marley_browser/src/relay.rs` (new): `marley --browser-relay`, the relay. It starts
  Chromium with `--remote-debugging-pipe` (fds 3 and 4 through `command-fds`), reads and writes
  NUL-framed JSON on the pipe, and multiplexes clients: each client gets a browser session
  (`Target.attachToBrowserTarget`), its command ids are rewritten and restored, a command with no
  session goes to its browser session and the answers and events come back without it, a command
  names only a session the client owns, events go to the owner of their session, and a client
  that leaves has its sessions detached. It serves the same WebSocket frames on a 0600 Unix socket
  and on `127.0.0.1:0`, the latter with the token as `Authorization: Bearer`, a non-loopback
  `Origin` or `Host` refused, and the handshake bounded in time. It writes the loopback endpoint
  and the token to a 0600 file once it listens. On SIGTERM it closes Chromium with `Browser.close`
  and waits; when Chromium exits it removes its socket and file and exits.
- `crates/marley_browser/src/service.rs`: `chromium_args` with `--remote-debugging-pipe`; the
  unit's command is Marley's executable with `--browser-relay`, the socket, the endpoint file and
  then Chromium's command line; the paths of the relay's socket and endpoint file in the project's
  folder, their length checked against `sun_path`; `endpoint_in` kept for the Chromiums of earlier
  builds.
- `crates/marley_browser/src/cdp.rs`: a connection over a Unix socket, the same frames.
- `crates/marley_workbench/src/browser.rs`: the hub connects to the relay's socket, waits for it
  as it waited for `DevToolsActivePort`, and closes a unit an earlier build started (a Chromium on
  a port) over its port before starting the relay; `stop_chromium` works through the relay.
- `crates/marley_workbench/src/playwright_scripts.rs`, `playwright/run.mjs`,
  `playwright/template.mjs`: the runner's command names the relay's endpoint file
  (`MARLEY_CDP_FILE`), never a token; `run.mjs` reads it, attaches with the token as a header,
  and hands `MARLEY_CDP_URL` and `MARLEY_CDP_TOKEN` to what the script starts.
- `crates/marley_workbench/src/marley_workbench.rs` and `crates/zed/src/main.rs`: the relay mode
  runs before anything else in `main`, as `--askpass` and `--crash-handler` do (a ledger row).
- `script/e2e/browser-fixture.sh` (`agent` and `browser_close` through the relay's endpoint file
  and token), `script/e2e/488-browser-pane.sh` (its port checks), `script/e2e/507-…` (its
  project units closed through the relay; its legacy Chromium stays on a port on purpose),
  `script/e2e/583-chromium-devtools-off-tcp.sh`.

### Out (explicitly deferred)
- CDP for #524's outside clients: CDP can evaluate script, which no Marley tool does (plan D15,
  AD-claude-492); they keep the MCP browser tools. Slice 3 (#584) is other machines.
- `--service-type=notify` readiness (no `sd-notify` in the tree): Marley polls the socket.
- A relay that outlives a crash of its own Chromium by starting it again: the unit ends, and the
  next Browser tab starts it, as today after a crash.

## Reference (§20)
N/A — Marley-specific: no reference product relays a browser's DevTools pipe to several clients.
The pieces follow published behavior: Chromium's `--remote-debugging-pipe` (JSON messages ended by
`\0`, commands on fd 3, answers and events on fd 4, as Playwright's pipe transport uses it) and
CDP's flat sessions with `Target.attachToBrowserTarget`. Upstream Zed: the relay is a hidden mode
of the app's executable, as Zed's own `--askpass` and `--crash-handler` modes are
(`crates/zed/src/main.rs`), and the child's extra fds are mapped as `util::shell_env` maps them.
Warp: N/A.

### Prior art
- **Behavior maps and reports.** Orca report 04 §2.13 (a path by path table of who reaches what;
  Orca leaves its browser's CDP to its own process). #524's notes ("The split"): slice 2's shape.
- **Published material.** Chromium's pipe transport as Playwright ships it
  (`playwright-core/lib/coreBundle.js`: `PipeTransport` writes `JSON.stringify(message)` and
  `"\0"`, splits what it reads on `"\0"`, and launches Chromium with
  `stdio: ["ignore","pipe","pipe","pipe","pipe"]`, so fds 3 and 4). CDP's `Target` domain:
  `attachToBrowserTarget` ("only uses flat sessionId mode"), `setDiscoverTargets`,
  `setAutoAttach`, `detachFromTarget`. Playwright 1.63's `connectOverCDP` takes `headers` for the
  WebSocket upgrade (`types.d.ts`, `ConnectOverCDPOptions.headers`), and Playwright MCP 0.0.80
  takes `--cdp-header`.
- **A probe** (2026-09-27, a scratch Chromium 152 on a pipe from a Python script in the
  scratchpad): two `attachToBrowserTarget` sessions are distinct; `setDiscoverTargets` on one
  sends `targetCreated` to that one alone; a target the other creates answers on its session;
  `attachToTarget` with `flatten` on one gives a page session whose answers and events carry it;
  `setAutoAttach` on the other announces `attachedToTarget` on that one alone; a browser session
  detaches; `Browser.close` from a browser session ends Chromium with code 0.
- **The code we already ship.** `async-tungstenite` 0.33 (`client_async`, `accept_hdr_async`
  over any futures-io stream, so the same frames run over a Unix socket) and smol's `UnixStream`
  and `UnixListener`; `command-fds` 0.3.2 with its precedent in `util::shell_env`
  (`std::io::pipe`, `fd_mappings`, `smol::process::Command::from`); `async-signal` 0.2.13 for
  SIGTERM on smol; Zed's hidden modes in `main.rs` and `get_shell_safe_zed_path`
  (`util::util.rs`), which strips `" (deleted)"` from `current_exe()`; `marley_mcp`'s
  `mint_secret` and `ct_eq`, whose way the relay follows for its token and its compare
  (`marley_browser` does not depend on `marley_mcp`); `service::make_private_dir` (0700). No CDP crate is in the
  lock (chromiumoxide, headless_chrome and fantoccini are absent). Does a crate we build own the
  seam? `marley_browser` owns the service and the CDP client, so the relay is a module of it.

## UI proof
UI-AFFECTING (the Browser tab and a script's run go through the relay; nothing new is drawn).
`script/e2e/583-chromium-devtools-off-tcp.sh` (`compositor sway`): the browser on a local page, a
shot of the tab (`583-01-tab`); the unit's command line; `ss -ltnp` over the unit's processes:
Chromium on no TCP port, the relay on one; the relay's WebSocket refused without the token, with
a wrong one and with a web page's `Origin`; the stand-in agent through the relay with the token;
two clients' sessions kept apart (a call on the other's session refused); Marley quits and
starts again and the tab shows the same page (`583-03-restored`); the unit's Chromium closed from
the agent, the relay's socket and file gone and the unit inactive. A Playwright script's run is
523's, which gains a check that the typed command names no token. The golden set carries the
rest: every browser scenario runs through the relay.

## Locked-In Decisions
- D1: The relay is a hidden mode of Marley's own executable (`marley --browser-relay …`), run
  before anything else in `main` as Zed's `--askpass` and `--crash-handler` are: no second binary
  to build, install or find. The unit runs `current_exe()` with `" (deleted)"` stripped.
- D2: Chromium runs with `--remote-debugging-pipe` as the relay's child in the same unit, fds 3
  and 4 mapped with `command-fds`; nothing in it listens on a port. The unit still outlives
  Marley's quit.
- D3: Each client gets a browser session of its own (`Target.attachToBrowserTarget`), which keeps
  every client's discovery and auto-attach apart (the probe). The relay rewrites command ids per
  client, routes by flat session id, refuses a command on a session its client does not own, and
  detaches a leaving client's sessions.
- D4: Marley reaches the relay over a Unix socket at mode 0600 in the project's 0700 folder
  (`<data>/browser/projects/<key>/relay.sock`), outside `profile/`, which #581 deletes, with the
  same WebSocket frames, so `cdp.rs` changes only how it connects. A path too long for
  `sun_path` is its own error.
- D5: Playwright clients reach it over a WebSocket on `127.0.0.1:0` that takes a token the relay
  mints at each start, sent as `Authorization: Bearer`; the relay refuses a web page's `Origin`
  and a non-local `Host`, and bounds the handshake at 10 seconds (#524's PR-claude-bound-every-
  read-before-auth-in-size-and-time-001). The endpoint and the token go in
  `<data>/browser/projects/<key>/relay.json` (0600), written once the relay listens.
- D6: A token never goes into a terminal: the runner's command names the file
  (`MARLEY_CDP_FILE`); `run.mjs` reads it and connects with the header.
- D7: Stop keeps #507's order: Marley sends `Browser.close` through the relay and waits for the
  unit to stop; on SIGTERM the relay closes Chromium itself before it exits (`KillMode=mixed`
  sends SIGTERM to it alone), so a stop keeps the cookies Chromium set last
  (F-claude-507). When Chromium exits, the relay removes its socket and file and exits.
- D8: A unit an earlier build started (a Chromium on a port, no relay) is closed over its port
  before the relay starts; the pre-#507 legacy profile's move keeps its port-based close. The
  port reader stays for those two alone.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley starts a project's browser, the system shall run Chromium with no TCP port open, behind the relay. | The log: the unit's command; `ss -ltnp` over its processes |
| REQ-002 | WHILE a project's browser runs, the system shall show its pages in Browser tabs and carry every browser tool through the relay. | Shot `583-01-tab`; the golden set |
| REQ-003 | WHEN a client connects to the relay's WebSocket without the token, with a wrong one, or with a web page's `Origin`, the system shall refuse it. | The log: three refusals |
| REQ-004 | WHEN a Playwright script runs on a Browser tab, the system shall attach it through the relay with the token, and the command typed into the terminal shall hold no token. | 523 in the golden set; its log |
| REQ-005 | WHILE two clients use one browser, the system shall keep their sessions apart: a command on a session another client owns is refused. | The log |
| REQ-006 | WHEN Marley quits and starts again, the system shall keep the project's Chromium running under its relay and attach the tab to the same page. | Shot `583-03-restored`; the log |
| REQ-007 | WHEN Marley stops a project's browser, the system shall close Chromium through the relay, keep the cookies it set, and stop the unit. | The golden set: 507 and 581 |
| REQ-008 | WHEN Marley finds a project's unit that an earlier build started, the system shall close its Chromium over its port and start the relay in its place. | The log: a unit started with the old command line |
| REQ-009 | WHEN Chromium exits, the system shall end the relay and remove its socket and endpoint file. | The log |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes; the probe.
- **P2 Code:** the relay, the service's unit, the Unix connection, the hub's start and migration,
  the runner, the fixture; the Zed hunk with its ledger row first; fmt, clippy and dylint clean;
  a review of the diff with a security read of every path that takes bytes before the token.
- **P3 Test:** write and run the scenario and read every shot; the golden set with 583 added;
  `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `marley_browser.md`, `marley_workbench.md`, `docs/marley/guide.md`;
  the plan's D15 and D16 (where a CDP client finds Marley's browser now); the ledger; close,
  archive, commit and push.
