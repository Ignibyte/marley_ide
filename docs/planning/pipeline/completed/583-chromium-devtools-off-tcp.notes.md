# Chromium's DevTools off TCP, behind Marley's relay — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-583-chromium-devtools-off-tcp.md
- **Pipeline spec:** 583-chromium-devtools-off-tcp.spec.md

## Phase 1 — Plan
- **Request:** slice 2 of #524 (its notes, "The split"), filed at #524's Complete on
  2026-09-27 and put at the top of the Queue: each project's Chromium listens on a loopback TCP
  port with no credential, open to every local process.
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (no active pipeline, a clean tree at
  1cdd5a929e; cargo busy with the release install, which Plan does not need); recall ✓; mint ✓
  (a new pair, the backlog row removed, the ticket in progress); discovery ✓ (an Explore agent);
  the prior-art sweep ✓ with a probe; the spec ✓; the design ✓.
- **Classification / tier:** feature, prong 3, B8b. Marley crates, the fixture and one Zed hunk
  in `crates/zed/src/main.rs` (a ledger row). Size L: it stays one slice because every consumer
  of the port (Marley, #523's runner, the fixture's agent) has to move in the same change, or the
  port stays open.
- **Recall (§18.3):**
  - AD-claude-488-marleys-browser-is-a-transient-unit-streamed-into-a-tab-001: the unit runs the
    browser binary itself with `--remote-debugging-port=0` and the endpoint from
    `DevToolsActivePort`; a fixed port was rejected. The relay keeps the unit and replaces the
    port.
  - PR-claude-close-chromium-over-cdp-before-stopping-its-unit-001 (F-claude-507): `Browser.close`
    before the unit stops, or the last 30 seconds of cookies are lost. The relay closes Chromium
    itself on SIGTERM (D7).
  - PR-claude-check-a-unix-socket-path-against-sun-path-001 (F-claude-513): a socket path under a
    folder the user chooses can pass 108 bytes; that is its own error (D4).
  - PR-claude-bound-every-read-before-auth-in-size-and-time-001 (#524): the relay's WebSocket
    accept is bounded before the token is checked (D5).
  - Brain: consultation 183f21b8ec5449fd820aef202043cf6f, nothing on this seam.
- **Discovery (at 1cdd5a929e, an Explore agent):**
  - `crates/marley_browser/src/service.rs`: `find_binary` (188, `MARLEY_CHROMIUM` read by Marley,
    `browser.rs:3078`), `unit_name` (220, `marley-browser-` and 12 hex of SHA-256 of the
    profile), `chromium_args` (232, `--remote-debugging-port=0` at 237), `systemd_run_args` (249:
    `--service-type=exec`, `KillMode=mixed`, `TimeoutStopSec=10`), `start` (283, "already" means
    `AlreadyThere`), `stop` (305), `unit_state` (349), `ENDPOINT_FILE`/`Endpoint`/`endpoint_in`/
    `remove_endpoint_in` (31, 372, 385, 413), the project paths (43 to 161), `make_private_dir`
    (172, 0700).
  - `crates/marley_browser/src/cdp.rs`: `connect(port, path, executor)` (96) opens
    `smol::net::TcpStream` and `async_tungstenite::client_async`; the reader and writer tasks
    (108 to 142); `dispatch` (176) works on a `&str`; `call` (218) numbers ids from 1 and adds
    `sessionId` for a session. `client_async` takes any futures-io stream, and smol's
    `UnixStream` is one.
  - `crates/marley_browser/src/page.rs`: nothing assumes a port; browser-level calls go
    session-less (`discover`, `page_ids`, `create`, `close`, `target_info`, `fit_window`), page
    calls on the flat session from `Target.attachToTarget`.
  - `crates/marley_workbench/src/browser.rs`: `open_browser` (3067: `try_connect`, else a stale
    `DevToolsActivePort` removed, `find_binary`, `service::start`, then 150 polls of 100 ms with
    `unit_state` each second), `try_connect` (3143, the one Rust reader of `DevToolsActivePort`),
    `stop_chromium` (3107: `Browser.close`, `CLOSE_WAIT`, `service::stop`), its callers
    `stop_browser` (1157), `clear_browser_data` (1178), `move_legacy_profile` (7163, with
    `remove_endpoint_in` at 7182); the quit (7078) stops no browser.
  - `crates/marley_workbench/src/playwright_scripts.rs`: `run_on` (268, `endpoint_in` at 280),
    `run_command` (312, the `ws://` URL at 321), `command` (207, `MARLEY_CDP_URL` and `MARLEY_TAB`
    typed into a terminal); `playwright/run.mjs` (`connectOverCDP(endpoint)` at 40);
    `template.mjs` (2 to 3) promises `MARLEY_CDP_URL` to what a script starts.
  - `script/e2e/browser-fixture.sh`: `browser_close` (79, node over `DevToolsActivePort`),
    `agent` (126) and `write_agent` (384, the same); `browser_teardown` (289) stops units by name.
    Scenarios on the port: 488 (62, 66), 489, 490, 493 (`agent`), 507 (its legacy Chromium on a
    port on purpose, 53 to 61, and `browser_close`), 523 (`MARLEY_CDP_URL`).
  - `crates/zed/src/main.rs`: `--askpass` (217) and `--crash-handler` (223) run before paths and
    the single-instance hand-off (362 to 394); `Args` (1764); `sandbox::run_sandbox_launcher_if_invoked`
    (208).
  - Cargo: `async-tungstenite` 0.33 (tungstenite 0.28), smol 2; `command-fds` 0.3.2 with its use
    in `crates/util/src/shell_env.rs:181`; `async-signal` 0.2.13; no CDP crate. `clippy.toml`
    disallows `std::process::Command::spawn` and `smol::Timer::after`; the Marley crates deny
    `unsafe_code`.
- **Decisions:** D1 to D8 in the spec.

### Design
- **The relay (`marley_browser::relay`).** `relay_args()` spots `--browser-relay` as the first
  argument; `run(args) -> ExitCode` reads `--socket <path> --endpoint-file <path> -- <chromium>
  <args…>` and runs on `smol::block_on`.
  - *Chromium.* Two `std::io::pipe()`s; `command_fds` maps the command pipe's reader to fd 3 and
    the answer pipe's writer to fd 4; stdin and stdout `/dev/null`, stderr inherited (the unit's
    journal); spawned through `smol::process::Command::from`. The relay writes each message and
    `\0` to fd 3's pipe and splits what fd 4's pipe gives on `\0` (`smol::Unblock` over the std
    pipe ends). It waits for `Browser.getVersion` to answer before it binds anything.
  - *The switch.* One task owns the state: the clients (each an outgoing channel, its browser
    session, the sessions it owns), the relay's pending ids (relay id to the client, its own id,
    and whether its session was added), and the owner of each session. A client's command
    without `sessionId` gets its browser session and a relay id; one with a `sessionId` it does
    not own is answered at once with a CDP error (`-32001`, "No session with given id"). An answer
    goes back under the client's id, without the session when the relay added it; an answer to
    `Target.attachToTarget` records the returned session as the client's. An event goes to the
    owner of its session, without the session when it is the owner's browser session;
    `Target.attachedToTarget` records the new session as the owner's, and
    `Target.detachedFromTarget` forgets it. A new client's first commands wait until its
    `Target.attachToBrowserTarget` answers. A leaving client's sessions are detached
    (`Target.detachFromTarget`), children before its browser session, and its pending answers
    dropped. A message from Chromium is read into a small head (`id`, `method`, `sessionId`) and
    rewritten; what it carries passes through.
  - *Listeners.* The Unix socket: a stale file removed, bound, set 0600; every connection is a
    client after the WebSocket handshake (`accept_async`). The loopback listener on
    `127.0.0.1:0`: `accept_hdr_async` inside a 10-second timeout, refusing a request whose
    `Authorization` is not `Bearer <token>` (401, `ct_eq`), whose `Origin` is not loopback or
    whose `Host` is not `127.0.0.1` or `localhost` with the port (403). The token is minted at
    each start (`marley_mcp`'s `mint_secret` is not reachable from `marley_browser`, so 16 bytes
    of `/dev/urandom` as hex, the same way). Once both listen, `relay.json` (`{"url":
    "ws://127.0.0.1:<port>/devtools/browser", "token": "<hex>"}`) is written at 0600 through a
    renamed file.
  - *Ending.* SIGTERM (`async-signal`): `Browser.close` on the root, up to 5 seconds for Chromium
    to exit, then a kill. Chromium's exit, or the answer pipe's end: the socket and `relay.json`
    removed, exit code 0 (Chromium's code otherwise). A client's `Browser.close` closes Chromium
    for everyone, as it does today.
- **The service.** `chromium_args` with `--remote-debugging-pipe`; `relay_socket_in(project_dir)`
  and `relay_endpoint_file_in(project_dir)`; `socket_fits(path)` (under 108 bytes);
  `relay_command(executable, socket, endpoint_file, binary, profile)`, the unit's argv after
  `--`; `systemd_run_args` takes it; `RelayEndpoint { url, token }` and
  `read_relay_endpoint_in(project_dir)`. `Endpoint` and `endpoint_in` stay for D8.
- **The client.** `cdp::connect_unix(socket, executor)` opens `smol::net::unix::UnixStream` and runs
  the same handshake and tasks as `connect`, which moves its body into a function generic over
  the stream.
- **The hub.** `try_connect(profile)` connects to the project folder's `relay.sock` when it is
  there. `open_browser`: the unit up with no relay socket and a `DevToolsActivePort` is an
  earlier build's: close it over its port (`cdp::connect`, `Browser.close`, the unit's stop as
  `stop_chromium` waits), then start the relay unit; the executable is `current_exe()` with
  `" (deleted)"` stripped (as Zed's `get_shell_safe_zed_path` does), found off the main thread
  with the browser binary. A socket path that does not fit fails the start with its own words.
  `stop_chromium`, `stop_browser`, `clear_browser_data` and the quit keep their shape.
  `move_legacy_profile` keeps the port for the pre-#507 unit.
- **The runner.** `command` types `MARLEY_CDP_FILE=<relay.json> MARLEY_TAB=<tab> node …`;
  `run.mjs` reads the file, attaches with `connectOverCDP(url, { headers: { Authorization:
  "Bearer " + token } })`, and sets `MARLEY_CDP_URL` and `MARLEY_CDP_TOKEN` in its own
  environment; `template.mjs` says so.
- **`main`.** `crates/zed/src/main.rs`: before `Args::parse`, `if let Some(code) =
  marley_workbench::run_browser_relay_if_invoked() { std::process::exit(code) }`, where the
  workbench forwards to `marley_browser::relay`; a row in `docs/marley/zed-touchpoints.md` first.
- **File manifest.** Marley crates: `crates/marley_browser/src/relay.rs` (new), `service.rs`,
  `cdp.rs`, `marley_browser.rs`, `Cargo.toml` (`command-fds`, `async-signal`);
  `crates/marley_workbench/src/browser.rs`, `playwright_scripts.rs`, `marley_workbench.rs`,
  `playwright/run.mjs`, `playwright/template.mjs`. Zed crate: `crates/zed/src/main.rs` (one
  call). Scripts: `script/e2e/browser-fixture.sh`, `script/e2e/488-browser-pane.sh`,
  `script/e2e/507-browser-context-per-project.sh`, `script/e2e/523-saved-playwright-scripts.sh`
  (a check that the typed command holds no token), `script/e2e/583-chromium-devtools-off-tcp.sh`,
  `script/e2e/golden`.
- **Ledger rows.** `crates/zed/src/main.rs`: the relay mode.

### E2E plan
`compositor sway`, the offline Chromium, a loopback page; before Marley starts, the scenario
starts the repository's project unit the way an earlier build did (`systemd-run` with the unit
name Marley uses and `--remote-debugging-port=0`).

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-008 | Marley opens the repository; `marley: open browser` on the old unit | the log: the old unit's port closed, the unit's new command holds `--browser-relay` |
| REQ-001 | the unit's `ExecStart`; its processes (`cgroup.procs`); `ss -ltnpH` over them | the log: `--remote-debugging-pipe`, no `--remote-debugging-port`; Chromium's processes on no port, the relay on one |
| REQ-002 | the page in the tab; `mcp_agent tabs` | `583-01-tab`; the log |
| REQ-003 | python against the relay's port: no token, a wrong token, the token with `Origin: http://example.com` | the log: 401, 401, 403 |
| REQ-005 | `agent` (the stand-in, through `relay.json`) attaches to the page; a second agent sends a call on the first's page session | the log: the first answers, the second is refused |
| REQ-004 | (523 in the golden set) a script run on the tab; its block's command | 523's shots; the log: `MARLEY_CDP_FILE=`, no token, no `ws://` |
| REQ-006 | `quit_marley`; the unit still active; `launch_marley` | `583-03-restored`; the log: `active` |
| REQ-009 | `agent close-browser` (a `Browser.close` through the relay) | the log: the unit inactive, `relay.sock` and `relay.json` gone |
| REQ-007 | 507 and 581 in the golden set | the golden set |

`583-02-script` in the spec's UI proof is 523's run instead: 523 already installs Playwright from a
loopback registry and runs a script on a tab, and it gains the check on the typed command.

### Risks
- The switch rewrites every message; a screencast frame (tens of kilobytes of base64, many a
  second) is read into a head and passed through. If profiling in Test shows the relay costs
  frames, parse with `serde_json::value::RawValue` (the `raw_value` feature) rather than a full
  `Value`.
- A CDP shape the switch does not know (a non-flat attach answering through
  `Target.receivedMessageFromTarget`) still routes by the session it arrives on.
- A relay from an older build keeps running until its unit stops; a newer Marley speaks the same
  frames to it.
- `KillMode=mixed` sends SIGTERM to the relay alone; if the relay dies without closing Chromium,
  systemd kills what is left when the unit stops. Whether Chromium exits by itself when its pipe
  closes is not settled; the relay's exit ends the unit either way.

## Phase 2 — Code
- **Checklist** (no task tool): the ledger row ✓ (`crates/zed/src/main.rs`'s row extended);
  `relay.rs` ✓; `service.rs` ✓; `cdp.rs` ✓; the crate root and manifest ✓; `browser.rs` ✓; the
  runner ✓; the workbench's entry and `main` ✓; the scripts ✓ (after the release install's golden
  set, which ran them from the checkout); check, fmt, clippy, dylint ✓; the review ✓.
- **Built.**
  - `marley_browser::relay` (new): `RELAY_FLAG`, `run_if_invoked` (`smol::block_on`), `Args`;
    `spawn_chromium` (`command_fds` maps two `std::io::pipe`s to fds 3 and 4, the command
    dropped at once so the relay sees the answers' end; `smol::Unblock` over the ends); `ready`
    (`Browser.getVersion` within 15 s); `mint_token`, `bind_unix` (a stale socket removed, 0600),
    `write_endpoint_file` (0600 through a renamed file); `read_chromium`, `watch_signals`
    (`async-signal`, SIGTERM and SIGINT), `accept_unix` and `accept_tcp` (10-second handshakes;
    the loopback one through `Admission`, tungstenite's `Callback`, and `refusal`: 403 for a web
    page's `Origin` or a foreign `Host`, 401 for a missing or wrong token, compared in constant
    time); `serve_client`; and `Switch` (clients, pending ids, session owners; `joined`,
    `client_command`, `gone`, `chromium_message`, `browser_session_made`, `own`).
  - `service.rs`: `--remote-debugging-pipe`; `relay_socket_in`, `relay_endpoint_file_in`,
    `socket_fits`, `relay_executable`, `relay_command`; `systemd_run_args(unit, command)` and
    `start(unit, command)`; `endpoint_in` kept for the old units.
  - `cdp.rs`: `connect_unix`; `connect` and it share `run`, generic over the stream.
  - `browser.rs`: `open_browser` checks the socket's length, closes an earlier build's unit that
    answers on its port (`try_connect_port`, then `stop_chromium`), and starts the relay's
    command; `try_connect` goes through the relay's socket; `stop_chromium` without a connection
    tries the relay, then the port (the legacy profile's move keeps its polite close).
  - The runner: `command(endpoint_file, …)` types `MARLEY_CDP_FILE`; `run.mjs` reads the file,
    connects with the header, and sets `MARLEY_CDP_URL` and `MARLEY_CDP_TOKEN`; `template.mjs`
    says so.
  - `marley_workbench::run_browser_relay_if_invoked` and the call in `main`, before
    `Args::parse`, beside Zed's sandbox launcher check.
  - Scripts: the fixture's `agent` and `browser_close` through `relay.json` with the token (else
    the port), with `hold-session`, `use-session` and `close-browser`; 488 reads the relay's
    port; 523 checks the typed command; `583-chromium-devtools-off-tcp.sh`.
- **A fix outside the relay.** The release install of #524 (1cdd5a929e) failed its golden set on
  523: the failed run's recording held its end and not its start. `record_script` kept an entry
  only while a tab drew the page, and the run's terminal takes the tab's place in its pane
  before it moves beside it; a release build records the start inside that moment. It now
  records whether or not a tab draws. Nothing was installed; F-claude-583 at Complete.
- **Deviations.** The relay mints its token itself (`marley_browser` does not depend on
  `marley_mcp`). No Rust reads `relay.json`: Marley connects through the socket, and the runner
  and the fixture read the file in node. The handshake check is a `Callback` struct rather than a
  closure: clippy's `result_large_err` flags a closure that returns tungstenite's
  `ErrorResponse`, and a trait's fixed signature is not flagged.
- **Review.**
  - A setup failure after Chromium started (the socket, the listener, the file) left Chromium
    running; the relay now kills it and removes the socket. A broken pipe under a live Chromium
    left the relay waiting on its exit; the relay kills it after the switch ends.
  - A client's own `Target.attachToBrowserTarget` or a detach naming another client's session
    goes to Chromium on the client's own browser session, which knows no such session; the relay
    records only `attachToTarget` answers and `attachedToTarget` events as ownership.
  - `Target.closeTarget` and `Browser.close` from a token holder close pages and the browser, as
    any CDP client could before; the token keeps other users out.
  - The socket's mode is set after the bind, inside the 0700 project folder.
  - Provenance: the pipe framing and fds follow Playwright's published transport, read as
    behavior; the executable lookup is Marley's own few lines, not Zed's `get_shell_safe_zed_path`
    body.
- **Checks.** `cargo check -p marley_browser -p marley_workbench --all-targets` ✓ (after an
  argument `close` takes in async-tungstenite 0.33 and two unused imports); `cargo check -p zed
  --bin marley` ✓; `cargo fmt` ✓; `./script/clippy -p marley_browser -p marley_workbench` ✓
  (after backticks, long first doc paragraphs, `from_*` names, a needless `collect`, a `const fn`,
  a borrowed argument and the closure's large `Err`); `cargo dylint` ✓; `shellcheck` ✓.

## Phase 3 — Test
- **Checklist** (no task tool): REQ-001 to REQ-009 ✓; 488, 489, 490, 493 ✓; the golden set ✓;
  the gate ✓.
- **The scenario.** `script/e2e/583-chromium-devtools-off-tcp.sh` (`compositor sway`, the offline
  Chromium, one loopback page). Its setup starts the repository's project unit the way a build
  before #583 did (`systemd-run` with the unit name Marley uses, Chromium with
  `--remote-debugging-port=0`), and waits for its `DevToolsActivePort`.
- **Runs.** The first (`e2e1.log`) passed every check through "the page in the tab", then ended
  in "what listens": `grep -c` exits 1 when it counts none, which is the passing count, and the
  runner's errexit ended the step. Both `ss` pipelines take `|| true`. The second (`e2e2.log`)
  passed all 17 checks:
  - REQ-008: the unit an earlier build started (on port 33093) gave way: the unit's command holds
    `--browser-relay`, `--socket`, `--endpoint-file` and `--remote-debugging-pipe`, and no
    `--remote-debugging-port`; its `DevToolsActivePort` is gone; `relay.json` and `relay.sock`
    are 600.
  - REQ-002: `browser_tabs` through Marley's own bridge names "Behind the relay".
  - REQ-001: the relay (its pid) listens on `127.0.0.1:<port>` alone; the unit's other processes,
    Chromium's, on none (`ss -ltnpH` over the unit's `cgroup.procs`).
  - REQ-003: no token `HTTP/1.1 401 Unauthorized`, a wrong token 401, the token with `Origin:
    http://example.com` `HTTP/1.1 403 Forbidden`.
  - REQ-005: one agent holds a page session; another agent's call on it gets `Session with given
    id not found.`
  - REQ-006: after `quit_marley` the unit is still active; after `launch_marley` the tab is back
    on its page.
  - REQ-009: the agent's `Browser.close` through the relay: the unit inactive within the wait,
    `relay.sock` and `relay.json` gone.
- **Shots** (scratchpad `583/shots`, only Marley in each):
  - `583-01-tab` (REQ-002): the Browser tab "Behind the relay" on `127.0.0.1:<port>/index.html`,
    "A page behind Marley's relay", its row in the rail.
  - `583-03-restored` (REQ-006): after the relaunch, the same tab on the same page, the rail row
    back.
  - `583-04-closed` (REQ-009): after the client's `Browser.close`, the page gone and so its tab,
    as a page that goes always takes its tab; the terminal remains.
- **Focus.** Headless sway: `hyprland: 0 Marley windows before the run, 0 after; the run added
  no rule and did not reload it`; sway stopped with the run's Marley, pointer and keyboard.
- **The scenarios on the agent and the port.** 488, 489, 490 and 493, which print rather than
  check, exited 0: 488's unit runs `marley --browser-relay … --remote-debugging-pipe`, `listening:
  127.0.0.1:<port>` (the relay's), `agent: navigated to …` and `agent: highlighted #button`, and
  its shot `488-03-highlight` shows the agent's highlight, drawn in its own session through the
  relay, in Marley's screencast; 489 and 490 `agent: navigated to …`; 493 `agent: closed the page
  at …/agent.html`.
- **REQ-004 and REQ-007** are the golden set's: 523 (the runner through `relay.json`, and its new
  check on the typed command) and 507 and 581 (stops through the relay).
- **The golden set** (34 with 583, `regress/20260927-092509`, the debug build): all 34 passed,
  every browser scenario through the relay: 492 (the agent tools), 494 (restore), 504 (the rail),
  505, 506, 507 (per-project Chromiums, the legacy profile's move closed over its port, stops
  through the relay), 518, 523 (the runner through `relay.json`, and its new check that the
  typed command holds no token), 524, 561, 574, 576, 579, 581 (Clear Browser Data through the
  relay), 582 and 583. The release race in 523 (F-claude-583) is the next release install's to
  prove, since a debug build records the start after the tab is drawn again.
- **The gate.** The first `script/gates.sh --diff` (`gate.log`) was red on gate:20 alone: semgrep's
  `command-injection-risk` flagged `std::process::Command::new(program)` in `spawn_chromium`, a
  program that is not a literal. Before #583 the path was an argument of the literal
  `systemd-run`. The relay now runs the fixed `/bin/sh` with the literal script `exec "$0" "$@"`
  and Chromium's command as its positional parameters, each one word, nothing read as a name or as
  shell; the exec keeps fds 3 and 4, as the fixture's offline wrapper already relied on. `env` was
  not taken, since it would read a path holding `=` as a variable. After the change, on a fresh
  build: 583 (17 checks), 488, 507 (27) and 523 (13) passed again, and the second run of the gate
  (`gate2.log`) passed rustfmt, clippy (every target), cargo-audit, cargo-deny, cargo-shear,
  gitleaks, shellcheck, no-suppressions, source-bans, docs, zed-ledger, manifests, spelling,
  semgrep and dylint, and the receipt: `GATE GREEN [diff]`.
- **Not reached by a scenario:** none of the plan's criteria; the release race of F-claude-583
  waits for the next release install, whose golden set runs 523.
- **Pre-existing:** nothing.
- **Verdict:** PASS.

## Phase 4 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented.** `CHANGELOG.md`: Changed (the browser listens on no network port) and Fixed (a
  failed Playwright run's recording keeps its start). `docs/marley_architecture/marley_browser.md`:
  the unit's relay, Chromium's flags and the relay's files, the client over the Unix socket, and a
  section on the relay. `marley_workbench.md`: the hub's start and stop through the relay, the
  runner's `MARLEY_CDP_FILE`, and the script entry kept whatever the view. `docs/marley/guide.md`:
  the browser's paragraph (a profile per project, no port), how a CDP tool attaches, and the
  files table. `docs/marley/three-prong-plan.md`: D15 and D16, row B8b shipped (size L).
  `docs/marley/zed-touchpoints.md`: `crates/zed/src/main.rs`'s row, written before the hunk,
  matches what shipped.
- **Knowledge appended:** F-claude-583-a-script-runs-start-was-lost-while-no-tab-drew-the-page-001,
  PR-claude-an-event-a-command-caused-is-kept-whatever-the-view-001,
  L-claude-583-one-pipe-many-clients-through-browser-sessions-001,
  L-claude-583-grep-counting-none-ends-a-step-under-errexit-001,
  L-claude-583-a-program-marley-starts-by-path-runs-through-a-fixed-shell-exec-001,
  AD-claude-583-chromium-on-its-pipe-behind-marleys-relay-001. The brain: consultation
  183f21b8ec5449fd820aef202043cf6f closed with
  `decisions/marleys-chromium-on-its-debugging-pipe-behind-a-relay-marley-583`, follow-up by
  2026-10-27.
- **Closed:** `tickets/closed/TICKET-583-chromium-devtools-off-tcp.md`; its backlog row went at the
  mint.
