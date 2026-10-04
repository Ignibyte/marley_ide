# Codex's state from its own App Server — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-650-codex-app-server-state.md
- **Pipeline spec:** 650-codex-app-server-state.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-03)
- **Request:** Chad, 2026-10-02: "also we need to brain storm integration into claude and codex
  using their tools instead of fighting them as well." On B1 ("should Marley own Codex's App
  Server and join it as a second client?"): "Yes". On B7, versions: "Your install, with a check"
  (#648). The lead's brief splits B1 in two: this ticket is state (#650); approvals and prompts are
  #651. Batch order after #640 to #647: #648 B7, #649 B4, #650, #651, #652 B2, #653 B3.
- **Classification / tier:** feature, prong 2 C1, L. One shippable slice: the server, the client,
  the fold and every reader of a terminal's seat that Codex should now reach. Not split further:
  without the rail, the inbox and the close guard reading the seat, the server and the client
  show nothing; the chip and the token count are a few lines on the same seat. Marley crates:
  `marley_agent`, `marley_workbench`. Zed-side: three files whose `zed-touchpoints.md` rows exist
  (the setting). New dependencies of `marley_workbench`: `smol` and `async-tungstenite`, both
  workspace dependencies already in `Cargo.lock`. No new crate, no new spawn site.
- **Recall (§18.3):**
  - AD-claude-519-claude-codes-hook-events-ride-in-band-into-marley-fleet-001: one `FleetSnapshot`,
    a seat per terminal view, folded by a pure `fold` in `marley_agent`. Codex's seat goes in the
    same snapshot by the same reducer (`marley_fleet::apply`), with its own pure fold beside
    `claude_events`.
  - AD-claude-532-agents-start-with-their-prompts-unless-your-own-settings-say-otherwise-001: the
    chip in the warning colour, "for Claude Code with events, from its reported mode; otherwise
    from its arguments"; Codex gains the reported half. Its rejection of "the arguments alone"
    is this ticket's reason for the chip.
  - AD-claude-550-the-close-guard-asks-in-zeds-own-close-paths-and-holds-the-view-001: "Working"
    is the seat's state for Claude Code, else the quiet timer's. A Codex seat joins the first.
  - AD-claude-508-one-inbox-lists-every-agent-that-waits-on-the-user-001: "Everything else opens
    where it waits"; a Codex wait is a terminal entry that opens its terminal (D9).
  - AD-claude-552-codex-configured-opencode-given-a-file-001: Codex's notifications stay its own
    OSC 9; rejected there, and here, "a live run of the real CLIs in the plan or the scenario".
  - AD-claude-520-each-terminal-names-itself-and-the-bridge-names-the-caller-001: each terminal's
    `MARLEY_TERMINAL_ID`; the gpui entity id is the seat's key and no program sees it. The server
    gets the terminal's id in its environment (D1).
  - AD-claude-632-marley-runs-the-harness-as-a-child-and-says-its-state-001: a program Marley runs
    as a child through its listed spawn site, killed with Marley. Unlike `rh serve`, a stopped
    Codex server is not started again: the TUI on it does not reconnect.
  - AD-claude-583-chromium-on-its-pipe-behind-marleys-relay-001: a 0600 socket in a 0700 folder,
    reached with `cdp::connect_unix`; the shape of this ticket's socket and client.
  - PR-claude-check-a-unix-socket-path-against-sun-path-001 and
    F-claude-513-a-socket-path-too-long-read-as-a-running-marley-001: "too long" is its own case
    with its own message; keep the folder short (the runtime directory).
  - L-claude-531-marley-takes-its-path-from-the-login-shell-so-stand-ins-are-named-001 and
    F-claude-547-a-scenarios-click-ran-the-real-claude-001: Codex 0.155.1 is installed in the
    user's PATH, so the program Marley itself runs is named by `MARLEY_CODEX`, and the launch line
    names the same binary by its full path.
  - L-claude-633-scenarios-copy-the-users-settings-so-defaults-reach-real-services-001: a user who
    turns the switch on would start the real `codex app-server` in every scenario; `e2e.sh` turns
    it off in each copy.
  - L-claude-534-zeds-mcp-client-sees-no-server-exit-001: bound every call (5 s) and treat a closed
    socket as the server gone.
  - BF-claude-blocking-child-reap-in-drop-on-main-thread-001: stopping a server waits for it off
    the main thread.
  - F-claude-594-an-enter-sent-with-a-paste-was-read-as-part-of-it-001 and the design note's
    "every path must check the seat isn't waiting": the paths that already ask
    `agent_events::waiting` hold a paste for a waiting Codex once it has a seat.
  - L-claude-610-a-host-scenario-on-this-machine-sees-its-real-agents-001: the host collector lists
    the user's real Codex processes; this scenario reads rail rows of its own terminals only.
  - Completed pipelines read: 519 and 547 (the seat, `fleet_snapshot`, `no update`), 532 (the chip
    and its scenario's fake `codex`), 550 (the close guard), 552 (Codex's `CODEX_HOME` in a
    scenario), 508 (the inbox), 520 and 575 (terminal identity), 596 (the askpass proxy tied to a
    terminal's release), 632 and 633 (a child program, a stand-in, the setting). Queued 640 (a
    harness seat's `state.source`; Codex's seat here is `protocol` in its terms, and nothing in
    this ticket reads that label) and 643 (a new switch's shape).
  - Brain (`rusty-cli brain search "codex app server"`, read only): research pages on other agent
    workspaces, nothing on Codex's App Server. The Planner's `brain_ask` at promotion is owed.
- **Discovery (two Explore maps, of Marley's Codex touchpoints and of the harness's client, and
  direct reads; Codex's protocol read in its 0.155.1 source and in schemas generated from both
  binaries):**
  - Launch: `crates/marley_agent/src/marley_agent.rs`: `bypass_arguments` (128-139, Codex's
    `--sandbox danger-full-access --ask-for-approval never`), `launch_input` (158),
    `launch_line` (169-187), `resume_line` (198, Claude Code only), `launch_line_after`
    (226-235), `command` (238-247), `quote_argument` (256). Callers:
    `crates/marley_workbench/src/agents.rs` `start_cli_with_prompt` (300-317, the line built before
    the terminal exists), `launch_input` (335-337, launch configs, `launch.rs:433`),
    `start_in_terminal` (363-430: `agent_env` (341-356) for an agent's terminal; the askpass proxy
    tied to the terminal's release at 403-406; the shell's startup handshake, bounded by
    `STARTUP_TIMEOUT`, then `write_init_command_after_startup` at 410-426). Entry points: the
    rail's + (`rail.rs:2472`), the New Agent picker (`agents.rs:695`), worktree agents
    (`worktree_agents.rs:584`). `Launcher.search_path` (`agents.rs:59, 79`) and `installed_clis`
    (105-109, `which::which_in`).
  - Recognition: `agent_bar::agent_in` (`agent_bar.rs:140-145`) from the foreground process's
    name; a hand-typed `codex` and a launched one look alike. `Terminal::marley_terminal_id()`
    (`crates/terminal/src/terminal.rs:2004`) and `marley_foreground_argv()` (3419).
  - The seat: `crates/marley_workbench/src/agent_events.rs`: `AgentEvents` (43-48), `seat`
    (68-72, `Done` filtered), `seat_id` (106-108, the view's entity id), `waiting` (112-116),
    `on_frame` (132-185, gated to Claude Code at 139-144), `end` (508), `forget` (537, on the
    view's release, `notifications.rs:92`). `crates/marley_fleet/src/session.rs:62-87`: `labels`
    and `capabilities`. `mcp.rs:172` serves the snapshot as `fleet_snapshot`.
  - The rail: `crates/marley_workbench/src/rail.rs`: `terminal_snapshot` (6746-6856): the seat for
    Claude Code only (6764-6766), the status (6771-6780), the chip (6781-6789), the line
    (6808-6819), the activity (6829), `reporting` (6840, 6937-6950); `note_claude_code`
    (1973-1985), which ends every seat whose row is not Claude Code's and would end a Codex seat;
    `inbox_entries` (6061-6116, terminal seats at 6086-6104, kind-agnostic); `seat_entry`
    (6205-6222, `agent: "Claude Code"` hard-coded); the terminal entry's icon (1766,
    `IconName::AiClaude` hard-coded); `InboxTarget::Terminal` (716) and `open_inbox_entry` (1685);
    `permission_chip` (7819-7834) and `terminal_marks` (7886). `claude_events.rs`: `AGENT_LABEL`
    (43, `claude-code`), `seat_line` (278-310), `seat_activity` (317-339).
  - The close guard: `crates/marley_workbench/src/close_guard.rs` `working_status` (185-206), the
    seat read for Claude Code only (186-188), the quiet timer at 196-205.
  - The chip: `marley_agent.rs` `MarkSource` (359-364), `tooltip` (387-396), `permission_mark`
    (405-430), `codex_full_access_argument` (459-485).
  - Other seat readers: `send_selection.rs` `target_of` (331-345, kind-agnostic), the Browser
    tab's Send (`browser.rs:5235-5240`, "Claude Code in that terminal waits"), `click_pause.rs:86`
    (Claude Code's permission mode only), `stall.rs` `working_seats` (205-216, every working seat,
    used at 177 and 223).
  - The setting's precedent: `crates/settings_content/src/marley.rs:87` (`embedded_harness`),
    `crates/marley_workbench/src/marley_workbench.rs` (`MarleySettings` 342, `embedded_harness`
    374, `EmbeddedHarness` 462-479, wired 582-584), `assets/settings/default.json:1706`,
    `crates/settings_ui/src/marley_page.rs` `agents_section` (112, 15 items; Codex Permissions at
    389).
  - Spawn and sockets: `process.rs` `follow_with_errors` (73) and `following` (89, kill on drop,
    no folder or environment yet); `marley_browser::cdp::connect_unix` (123-135), `run` (139),
    `dispatch` (214, any message with an id taken as a response); `marley_browser::service::
    socket_fits` (292). No runtime-directory helper exists; `fleet_hosts.rs:190` uses
    `XDG_RUNTIME_DIR` for ssh's control path.
  - `script/e2e.sh:630-640` turns `marley.rusty_tools` off in each profile copy.
  - Codex 0.155.1 (`codex-rs/`): see the spec's Prior art for the cited lines. The ones that shaped
    the design: new threads attach every initialized connection (`app-server/src/lib.rs:1252-1266`);
    `thread/started` and `thread/status/changed` are broadcast
    (`request_processors/thread_processor.rs:1630-1633`, `thread_status.rs:246-251`); server
    requests go to the thread's subscribers (`outgoing_message.rs:168-178`);
    `thread/settings/updated` is experimental (`app-server-protocol/src/protocol/common.rs:1932`);
    the first originating client names the originator and each replaces the user agent's suffix
    (`request_processors/initialize_processor.rs:18, 125-172`); the TUI sends a folder in remote
    mode only with `--cd` (`tui/src/startup_orchestration.rs:209-212`).
  - The harness (read only): `crates/harness-runtime/src/codex/` (stdio JSONL, spawn-and-own,
    synchronous, digest-pinned; no `license`, no repository); `scripts/codex_profile.py` and
    `dependencies/codex.json` (314 stable schemas of 0.158.0, 31 kept); no register for requests
    from Marley; it vendors `marley_fleet`.
- **Decisions:** D1 to D10 in the spec.

### Design
- **`marley_agent::codex_events`** (Marley crate, new module, pure, `serde` and `serde_json`
  only): the read types with `#[serde(rename_all = "camelCase")]` and unknown fields ignored:
  `Thread { id, parent_thread_id, ephemeral, thread_source, created_at, cwd, status }`,
  `ThreadStatus` (tagged by `type`: `NotLoaded`, `Idle`, `SystemError`, `Active { active_flags }`,
  and an `Unknown` for a type it does not know), `Turn { id, status, error: Option<{ message }> }`,
  `TokenUsage { total: { total_tokens }, model_context_window }`, `ResumeAnswer { thread,
  approval_policy, sandbox }` with the policies kept as their wire words (`never`, `on-request`,
  `untrusted`, `granular`; `dangerFullAccess`, `workspaceWrite`, `readOnly`, `externalSandbox`).
  `Notification::decode(method, params)` for the five methods read. `is_lead(&Thread)`.
  `fold(seat, previous, &Input, now_ms) -> Vec<SessionEvent>` (an `Input` is a notification or a
  resume answer), the rules of the spec's fold bullet; labels `agent` (`codex`), `thread`, `turn`,
  `wait`, `error`, `tokens`, `context_window`; capabilities `sandbox`, `approval`. `seat_words`
  for the state's words (`waiting on approval`, `waiting on input`). A `failed` turn label
  (`turn_failed`) keeps the seat failed through the `idle` that follows until `turn/started`.
- **`marley_agent.rs`** (Marley crate): `launch_line_after` and `launch_line` take a
  `Remote<'_> { socket: &Path, folder: &Path }` option (`None` for every agent but Codex) and put
  `--remote 'unix://<socket>' --cd '<folder>'` after the program; the program may be a full path
  (`command` takes it). `MarkSource::Thread { approval }` and its tooltip ("Codex runs with no
  sandbox: its thread reports danger-full-access, approvals <policy>"); `permission_mark` for
  Codex takes the seat's reported sandbox first, as Claude Code's reported mode is. The quiet
  timer's doc says it is now for agents with no seat.
- **`marley_workbench::codex_server`** (Marley crate, new module, the adapter):
  - `CodexServers`, a global: the socket folder (made at first use, removed at quit) and a
    `Server` per terminal view, dropped at the view's release.
  - `prepare(workspace, cx) -> Option<Prepared>`: the switch, a local project, #648's verdict for
    `app_server`, the `codex` path (`MARLEY_CODEX`, else `which::which_in` on the launcher's search
    path), a socket path that fits; `None` with the reason logged otherwise.
  - `Server::start(prepared, folder, env, cx)`: `process::serve` (below), then the follow task:
    wait for the socket; wait for the TUI's connection (`/proc/net/unix`: a line in state `03`
    whose path is the socket, or its link's target); 1 s; connect (`smol::net::unix::UnixStream`,
    `async_tungstenite::client_async("ws://localhost/rpc", …)`); `initialize`, check the
    `userAgent`'s version against the range, `initialized`; `thread/loaded/list` each second until
    a lead; `thread/resume`; then read: each notification and answer through `codex_events::fold`
    into `AgentEvents` (`marley_fleet::apply` on the seat id of the view), `thread/resume` again at
    each `turn/started`, `thread/unsubscribe` for a replaced lead and when Codex leaves the
    foreground. The connection's reader sorts responses (an id and a result or error), server
    requests (an id and a method: logged, unanswered) and notifications (a method).
  - Stop: SIGTERM, a 2 s wait off the main thread, then the kill; the socket path, its link target
    and `.lock` removed if left.
  - `agents::start_in_terminal` gains the server: with a `Prepared` for Codex, the line is built
    with `Remote`, the server starts beside the shell's startup, the line is typed once the socket
    exists (5 s), and the `Server` is tied to the view's release; without the socket in time, the
    plain line is typed and the reason logged. The server's environment:
    `ProjectEnvironment::directory_environment(folder)`, else Marley's own, then `agent_env`'s
    variables and the terminal's `MARLEY_TERMINAL_ID` and `MARLEY_PROJECT`.
- **`process.rs`** (Marley crate): `following` takes a folder and an environment, and `serve`
  starts a long-running program with them (stdin closed, stderr piped for the last error line,
  kill on drop). No new spawn call outside `following`, so `.config/spawn-sites.txt` and the
  gate's pin stay; the file's line names `codex` among the programs.
- **`agent_events.rs`** (Marley crate): `apply_codex(view, events, cx)` for `codex_server`; `seat`
  unchanged; `waiting` unchanged (kind-agnostic).
- **`rail.rs`** (Marley crate): `terminal_snapshot` reads the seat when its `agent` label is the
  row's kind (`claude-code` or `codex`); for Codex the line is `codex_events::seat_words` plus
  `· {fleet::compact(tokens)} tokens`, #547's `no update` rule shared; the activity is the error
  for a failed seat; the chip from `permission_mark` with the seat's `sandbox`.
  `note_claude_code` ends a seat whose `agent` is not the row's agent. `seat_entry` takes the
  agent's name and the ask from the seat (`wait`), and the terminal entry's icon follows the seat's
  agent (`AiClaude`, `AiOpenAi`).
- **`close_guard.rs`** (Marley crate): `working_status` reads the seat for Claude Code and Codex.
- **`stall.rs`** (Marley crate): `working_seats` keeps seats whose `agent` is `claude-code`.
- **`send_selection.rs`, `browser.rs`** (Marley crate): the waiting sentence names the seat's agent.
- **`marley_workbench.rs`** (Marley crate): `MarleySettings::codex_app_server: CodexAppServer`,
  `CodexAppServer { On, Off }` with `from_setting` (`Some(true)` is `On`).
- **`Cargo.toml` of `marley_workbench`** (Marley crate): `smol.workspace = true`,
  `async-tungstenite.workspace = true`.
- **`marley_agent::versions`** (Marley crate, #648's module): the `Integration` row
  `codex_app_server` (Codex, from 0.155.1, before 0.158.1, what it rests on); `codex_server` asks
  #648's `verdict` for it at `prepare` and again with the server's `userAgent` version.
- **Zed crates (additive, rows that exist):**
  - `crates/settings_content/src/marley.rs`: `codex_app_server: Option<bool>` with `/// Default:
    false`. Extends its row's list: "`codex_app_server: Option<bool>`, whether each Codex Marley
    launches runs against an App Server of its own that Marley joins (#650)".
  - `crates/settings_ui/src/marley_page.rs`: the Agents section's Codex App Server toggle after
    Codex Permissions (`agents_section` grows to 16), and the Agent Versions section's (#648) App
    Server on Untested Codex item (`marley.allow_untested_versions.codex_app_server`). Extends its
    row: "the Agents section's Codex App Server toggle (`marley.codex_app_server`) and the Agent
    Versions section's App Server on Untested Codex (#650)".
  - `assets/settings/default.json`: `marley.codex_app_server: false` with its comment. Extends its
    row: "`marley.codex_app_server: false` with its comment (#650)".
  - `project` (`ProjectEnvironment`) and `terminal` (`marley_terminal_id`): read through their
    public functions, unchanged.
- **Test-harness file:** `script/e2e.sh`: `marley.codex_app_server` false in each profile copy,
  beside `rusty_tools`; no `codex_app_server` key in #648's allow map.
- **File manifest.** Marley: `crates/marley_agent/src/marley_agent.rs`,
  `crates/marley_agent/src/codex_events.rs` (new), `crates/marley_workbench/src/codex_server.rs`
  (new), `process.rs`, `agents.rs`, `agent_events.rs`, `rail.rs`, `close_guard.rs`, `stall.rs`,
  `send_selection.rs`, `browser.rs`, `marley_workbench.rs`, `crates/marley_workbench/Cargo.toml`,
  `crates/marley_agent/src/versions.rs` (#648's, one row); `script/e2e.sh`;
  `script/e2e/650-codex-app-server-state.sh` (Test phase). Zed:
  `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs`,
  `assets/settings/default.json`. `marley_fleet`, `marley_rail`, `marley_mcp` unchanged.
- **The stand-in** (written by the scenario, Test phase): `$E2E_WORK/bin/codex`, Python standard
  library only. `--version`: the version file's line. `app-server --listen unix://P`: binds `P`,
  answers the HTTP upgrade (`Sec-WebSocket-Accept` from SHA-1 and base64), reads masked client
  frames and writes unmasked text frames, one JSON message each, no `jsonrpc` field, an
  `emittedAtMs` on each notification; `initialize` answers `userAgent`
  `codex-tui/<version> (stand-in)` (the real shape, recorded in P2), `codexHome`,
  `platformFamily`, `platformOs`; tracks which connections are initialized, attaches them to each
  thread it creates, broadcasts `thread/started` and `thread/status/changed`, and sends a thread's
  other notifications to its subscribers; logs `<connection's clientInfo.name> <method>` to
  `$E2E_WORK/server-<pid>.log`; on SIGTERM removes `P` and exits. `--remote unix://P …`: prints
  its arguments, connects, initializes as `codex-tui` with the experimental API, `thread/start`
  with its `--cd`, `--sandbox` and `--ask-for-approval`, starts an ephemeral side thread with a
  `Feature` source, and sends each typed line as `standIn/cue` (a method only the stand-in knows,
  from the TUI's connection, which Marley never sends). Without `--remote`: #532's fake.

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | the switch off (the run's copy as `e2e.sh` leaves it); a Codex from the +; 3 s quiet | `650-01-off` |
| REQ-002 | `codex_app_server` set on by `profile_setting`; a Codex (A) from the + | `650-02-launched` (the stand-in's printed arguments: `--remote unix://…/codex/<id>.sock --cd …`) |
| REQ-003, REQ-004 | `work` typed in A; `settle 5` with no output | `650-03-working` (`working · 12k tokens`) |
| REQ-005, REQ-007, REQ-008 | a second Codex (B); `approve` in A | `650-04-approval` (A `waiting on approval`, B `idle`; the inbox's Codex entry for A) |
| REQ-006, REQ-007 | `input` in B | `650-05-input` |
| REQ-008 | B's row clicked, then A's inbox entry clicked | `650-06-opened` (A's terminal in the center) |
| REQ-009, REQ-004 | `done` in A | `650-07-done` (`idle · 15k tokens`; no entry for A) |
| REQ-010 | `work`, then `fail` in A | `650-08-failed` |
| REQ-011 | `work` in A, `settle 5`, A's close button | `650-09-close-asks` |
| REQ-012 | Cancel; `full`, then `work` in A; the pointer on A's chip | `650-10-full` |
| REQ-013 | `codex_permissions` `full_access` for the repository through `agent_permissions_by_project`; a Codex (C); `scoped`, then `work` in C | `650-11-scoped` (C's chip gone; the stand-in's arguments still show `--sandbox danger-full-access`) |
| REQ-014 | `crash` in B | `650-12-server-gone` |
| REQ-015 | `quit_marley`; the version file at `codex-cli 0.150.0`; `launch_marley`; a Codex from the +; the pointer on #648's chip | `650-13-outside` (no `--remote`; the quiet timer; #648's chip and its tooltip) |
| REQ-019 | the Settings window's Marley page, scrolled to Agents and Agent Versions | `650-14-page` |
| REQ-016, REQ-017, REQ-018 | after the run: every `server-*.log` shows `codex-tui initialize` before `marley initialize`; Marley's lines name only the four methods; no stand-in server process is left (`pgrep -f` on the stand-in's path, with the pattern split so it does not match itself) | the review, and the logs read in the notes |

Rows' places come from the first run's shots, as #532's did (`PLUS_X`, `PLUS_Y`, the + menu's
steps to Codex). What no scenario can reach: a real Codex turn (network and an account, and the
brief forbids it) and the user's own Codex. P2 records the real 0.155.1 server's `initialize`
answer once with the network off and no thread, so the stand-in's handshake is the real one; the
turn notifications follow the generated schema. The Phase 3 entry says which parts are recorded
and which are schema-shaped.

### Risks
- **#648 is queued, not built.** Its `Integration` row, `verdict`, agent-bar chip, allow map
  (`marley.allow_untested_versions`) and `MARLEY_CODEX` are used as its spec drafts them; the
  promotion re-reads them as shipped. The releases between 0.155.1 and 0.158.0 were not generated
  (neither is on this box); the range takes the two ends' agreement for them.
- **Joining second rests on timing.** The TUI's connection on the socket, then 1 s, is taken as
  "the TUI has initialized". A TUI that connects and stalls longer before `initialize` would let
  Marley's go first and make `marley` the originator of that server's threads. The stand-in's log
  proves the order in the scenario only. If the promotion's source read finds a firmer signal (a
  line on the server's stderr at each `initialize`, say), the follow waits for that instead.
  Asking OpenAI for a non-originating capability is in Out.
- **The user agent's suffix names Marley.** With Marley joined, Codex's requests upstream carry
  `(marley; <version>)` where they carried `(codex-tui; <version>)`. True, and said in the
  setting's description, but it is a change in what Codex tells OpenAI; it is one reason the
  switch is off by default.
- **Two servers on one `CODEX_HOME`.** Every Marley-owned server shares the user's home, as
  embedded TUIs always have. Orca locks its probe per home against a single-use refresh token
  spent twice. The promotion checks Codex's token refresh across processes in the 0.155.1 source;
  if it is not safe, this ticket narrows to one server per `CODEX_HOME` per Marley instance and
  maps threads by `--cd` and `createdAt`, which D1 rejected for its ambiguity.
- **The thread's source reads `vscode`.** `codex app-server` records its threads with the
  `vscode` session source (as Codex's own daemon does); the promotion checks that `codex resume`
  lists them.
- **Policies lag a turn.** Without the experimental API the chip learns a `/permissions` change
  at the next turn's start. The change applies from that turn, so the chip is right about every
  turn that runs; between the change and the turn it shows the old policy.
- **The approval requests Marley receives are never answered.** First answer wins and the TUI's
  answer resolves them (`serverRequest/resolved`). A TUI that exits with a request open leaves it
  pending on Marley's connection until the terminal closes; the seat then still reads waiting
  until Codex leaves the foreground, which ends it.
- **Codex's commands run under the project's environment, not the shell's.** An environment the
  user sets by hand in that shell before typing `codex` would not reach Codex's commands. It is
  the same as Codex's own daemon, and Zed's `directory_environment` includes direnv; said in D4.
- **The socket's link in 0.158.0.** `/proc/net/unix` names the link's target, not the path
  Marley passed; the follow resolves the link before it reads. 0.155.1 binds the path itself.
- **`/proc/net/unix` is Linux's.** Elsewhere the switch stays off with the reason logged.
- **Cuts in Codex's TUI startup.** `--remote` refuses `--worktree` (Marley passes none) and other
  local-only options a user might add to the line from history; those fail in Codex with its own
  message, as they would without Marley.

### Checklist (no TaskCreate in this harness)
- [x] Read CONSTITUTION.md §3, §7, §14, §18, §19, §20, the templates, and #633's and #640's pairs
      for shape.
- [x] Read the design note in full (the fights table, the tools, B1 to B7, Chad's answers, the
      harness's side) and the lead's brief, with the coordinator's correction (Codex's source at
      the 0.155.1 clone; `terminalSequence` documented, which this ticket does not touch).
- [x] Recall (§18.3): the four ledgers grepped (Codex, quiet, socket, child, stand-ins, inbox,
      close guard) and the completed pipelines 508, 519, 520, 532, 547, 550, 552, 575, 596, 632,
      633; queued 640 and 643; a brain search.
- [x] Discovery: an Explore map (§18.2) of Marley's Codex touchpoints and one of the harness's
      client; direct reads of `marley_agent.rs`, `agents.rs`, `agent_events.rs`,
      `close_guard.rs`, `rail.rs`'s rows and inbox, `process.rs`, `marley_browser::cdp`.
- [x] Protocol: `codex --help`, `codex app-server --help`; JSON Schema generated offline from
      0.155.1 and 0.158.0 and compared; Codex 0.155.1's source read at the cited lines; the
      published App Server page.
- [x] Decided the hard part: one server per terminal, so the socket maps a thread to its
      terminal; Marley joins second; its own client, no request to the harness.
- [x] Reference (§20) and Prior art's three legs filled; Warp's source not read.
- [x] Spec, notes and ticket doc written; no other file touched, no cargo run, no agent turn.

### Reconciliation, 2026-10-03
- From #651's draft: the server takes the first response to a server request and reads a
  client's error or malformed body as decline, an empty grant or a denial, so Marley's client
  never replies to a server request on its own, not even "method not found"; requests it does
  not handle are left unanswered for the TUI.
- From #651's draft: #651 needs `item/started` for file-change paths, so this client opts out of
  item deltas but never out of `item/started`.
- Prompts are B1 part 3 and resume B1 part 4 (split out of #651), not #651; where this spec's Out
  says #651 covers prompts, read B1 part 3.
- `terminalSequence` is documented in Claude Code's hooks reference (#648's finding).

### Promotion (2026-10-04)
- Promoted into `active/`; the BACKLOG row removed; the ticket in-progress. Pre-flight green, no
  other active pipeline; #640 to #649 landed (the last 308ba7584b).
- **Brain:** `brain ask` (consultation `c60abfdf55ae40d999b172d6e29c9aef`) returned due follow-ups
  on other work only; nothing on Codex's App Server. A repeat ask (`39a739a2…`) was closed with
  `brain no-decision` as a duplicate.
- **Seams re-read** (an Explore pass over the fourteen areas the design names). The shapes hold,
  with these corrections:
  - `agent_versions`: `AgentVersions`, its `check`, the found path and `verdict(integration, cx)`
    are private; `prompt_reading` is the only crate-visible reader. The design adds
    `pub(crate) fn is_on(integration, cx) -> bool` and `pub(crate) fn program(kind, cx) ->
    Option<PathBuf>` (the path the check found). `INTEGRATIONS` grows to two; the chip, the
    reasons and `allowed_in` take the row with no other change.
  - `MARLEY_CODEX` reaches only #648's check; a launch types the bare `codex`. With the server, the
    line names the checked path, so the server, the TUI and the check are one binary.
  - `ProjectEnvironment::directory_environment(abs_path, cx)` returns `None` for a local path in
    no worktree, and no Marley crate calls it yet (`acp_thread/src/terminal.rs:624-628` is the
    pattern). `None` falls back to Marley's own environment.
  - `marley_workbench` depends on `marley_browser` but not on `smol` or `async-tungstenite`
    (workspace 2.0 and 0.33); the two are added.
  - `process.rs`: `output` alone takes a folder and an environment; `following` takes neither.
    `following` gains both, so the spawn count stays 7.
  - No `XDG_RUNTIME_DIR` helper; `fleet_hosts.rs:195` reads it inline. The askpass socket sits in
    a `tempfile` folder. `on_app_quit` hooks exist in `mcp.rs`, `clients.rs`, `browser.rs`.
  - `MarleySettings` holds three bools under pedantic clippy: the switch is an enum
    (`CodexAppServer { On, Off }`, `EmbeddedHarness`'s `const fn from_setting`).
  - `tooltip` is `PermissionMark`'s, not `MarkSource`'s. `seat_entry` (rail.rs:6504) and the
    terminal entry's icon (:1890) hard-code Claude Code; `note_claude_code` (:2097) ends every seat
    whose row is not Claude Code's; `terminal_snapshot` reads the seat for Claude only
    (:7055-7057); `close_guard::working_status` likewise (:186-188); `stall::working_seats` and
    `send_selection::target_of` take any seat; `browser.rs:5235-5237` names Claude Code.
  - `marley_page.rs`: `agents_section` is 15 items (Codex Permissions the 12th, Agent Prompts in a
    Tab last); `agent_versions_section` is 2.
- **Codex's source** (an Explore pass over the 0.155.1 clone and 0.158.0 at its tag, with both
  generated schema sets). What it changed:
  - **`thread/resume` renames the thread's client** to the resuming connection's until the TUI's
    next `turn/start` (`thread_processor.rs:4412-4417`, `turn_processor.rs:580-588`), and Codex
    offers plugin installs only to `codex-tui` (`request_plugin_install.rs:139-151`). The
    design's resume at every `turn/started` would land inside the TUI's turn: dropped (D3).
  - **Every initialized connection is attached to each thread made while it is joined**, the
    TUI's ephemeral title and recap threads included (`lib.rs:1274-1292`), and a thread unloads
    only with no subscriber after 60 s idle (`thread_lifecycle.rs:55-62, 416-456`). Marley
    unsubscribes from each thread it was attached to that is not the lead.
  - **`thread/settings/updated` goes to subscribers of the experimental API only**
    (`transport.rs:104-127`), sent when a turn's settings change (`bespoke_event_handling.rs:1234-
    1250`). Its `threadSettings.approvalPolicy` and `sandboxPolicy` agree in the 0.155.1 and
    0.158.0 bundles. Marley initializes with `experimentalApi: true`, as the TUI does, and reads
    it (D3).
  - **`thread/status/changed` and `thread/started` are broadcast** to every initialized connection
    (`thread_status.rs:246-251`, `thread_processor.rs:1646-1648`), so the state needs no
    subscription; `turn/*` and `thread/tokenUsage/updated` go to subscribers only.
  - **`thread/loaded/list` answers ids only**; `thread/read {threadId}` gives a thread with its
    live status without subscribing (`thread_processor.rs:860-880`). REQ-017's list gains it.
  - **Never answer a server request**, not even with an error: an error reply resolves an approval
    as denied (`bespoke_event_handling.rs:2026-2032`), and Codex's own remote client answers
    unknown requests with -32601 (`remote.rs:895-905`), so no ready-made client is safe here.
  - **Tokens:** `thread/resume` with `excludeTurns` does not replay token use; the count shows from
    the next turn.
  - **The socket:** 0.155.1 binds the path and chmods its parent to 0700; 0.158.0 binds under
    `/tmp/codex-daemon-<uid>/<sha256>` and links the path to it once listening; both remove what
    they made on a clean stop. SIGTERM starts a drain that waits for running turns; a second
    SIGTERM or the kill ends it. A bare `unix://` is Codex's shared daemon: Marley always names a
    path.
  - **Originator and user agent:** first-wins and replaced per initialize, by name only
    (`initialize_processor.rs:19, 131, 166-194`); no capability opts out. No signal tells another
    connection that the TUI initialized; with `RUST_LOG` unset the server logs nothing then. D10's
    wait (the TUI's connection, then 1 s) stands.
  - **Token refresh across processes:** each server has its own auth manager with an in-process
    lock only; a guarded re-read of `auth.json` before refreshing narrows, but does not close, a
    window where two servers spend one refresh token, the loser reading "refresh token was
    already used" until a reload sees the winner's tokens (`login/src/auth/manager.rs`). 0.155.1's
    embedded TUIs carry the same risk today; 0.158.0's default shares one daemon. A risk, kept
    with D1; the switch is off by default.
  - **Session source:** `codex app-server` records `vscode`; `codex resume` and `--last` list
    `cli` and `vscode` alike (`tui/src/lib.rs:764-773`).
  - **The TUI in remote mode:** initializes as `codex-tui` with `experimentalApi`, sends
    `approvalPolicy` always and a sandbox unless its profile is external, takes the folder only
    from `--cd`, and starts ephemeral side threads (`system` in 0.155.1; `thread_title` and
    `system` in 0.158.0).
  - **Framing:** no `jsonrpc` field; notifications carry `emittedAtMs`; a server request has an
    integer id and a method. A connection whose 32,768-message queue fills is dropped;
    `capabilities.optOutNotificationMethods` suppresses notifications by exact name.

### Design, as revised at promotion
These replace the matching parts of the Design above; the rest stands.
- **`codex_server`'s follow:** after `initialize` (`experimentalApi: true`; the opt-outs are the
  `*Delta` notification methods the 0.155.1 bundle lists, `item/started` kept) and `initialized`:
  `thread/loaded/list`, then `thread/read` for each id not yet known, each second until a lead is
  found. State comes from `thread/status/changed` for the lead whether or not Marley subscribed.
  A lead seen first in `thread/started` after the join is subscribed already; a lead found by the
  list is resumed (`excludeTurns: true`) at the first moment its status is `idle`, never while a
  turn runs. Each `thread/started` that is not the lead (ephemeral, a `parentThreadId`, another
  source) is unsubscribed at once. No resume at `turn/started`.
- **Policies:** the resume's `approvalPolicy` and `sandbox`, then `thread/settings/updated`'s
  `threadSettings.approvalPolicy` and `sandboxPolicy`. `codex_events` decodes both.
- **`agent_versions`:** `pub(crate) fn is_on(integration, cx) -> bool` (the private `verdict`'s
  answer) and `pub(crate) fn program(kind, cx) -> Option<PathBuf>` (the path the check found:
  `MARLEY_CODEX`, else the launcher's search path).
- **`process.rs`:** `following` takes an optional folder and an environment to add; `serve`
  (new) calls it with stdin closed and stderr piped, kill on drop. Still one spawn call.
- **The server's environment:** `directory_environment(folder)` awaited (bounded by the 5 s the
  socket wait has), else Marley's own; then `agent_env`'s variables, `MARLEY_TERMINAL_ID` and
  `MARLEY_PROJECT`.
- **The stand-in server** keeps the real rules Marley depends on: it attaches every initialized
  connection to a thread at its start, broadcasts `thread/started` and `thread/status/changed`,
  sends turn, token and approval messages to subscribers only and `thread/settings/updated` to
  experimental subscribers only, answers `thread/read` without subscribing, makes the caller of
  `thread/resume` a subscriber and logs the thread's client name it now holds, and logs every
  method with its connection's `clientInfo.name` and whether the thread's status was `active`
  at a resume.
- **Added risk:** a lead from before the join that never goes idle (a launch with a first prompt
  whose turn runs long) has its state from the broadcast but no tokens, failure text or policies
  until its first `idle`.

## Phase 2 — Code (2026-10-04)
- **Built.**
  - `marley_agent`: `codex_events` (new, pure): the read types (`Thread` with `is_lead`,
    `ThreadStatus`, `Turn`, `Policies`), `Notification::decode` for the seven methods followed,
    `read_answer`, `resume_answer`, the `Input` enum, `fold`, `seat_words`, `seat_activity`,
    `tokens`, `is_codex`. `Remote` and `launch_line_after`'s `remote` (the program by its quoted
    path, `--remote` and `--cd`); `thread_mark` and `MarkSource::Thread`; `agent_kind_of` reads a
    quoted program path. `versions::CODEX_APP_SERVER`, from 0.155.1 before 0.158.1.
  - `marley_workbench::codex_server` (new, the adapter): `CodexAppServer`, `prepare`, `start`
    (the folder made 0700, the environment, `process::serve`, the socket wait), the process task
    (stderr's last line, the exit, SIGTERM then the kill), the follow (the TUI's connection in
    `/proc/net/unix`, a second, `initialize` with the experimental API and the delta opt-outs, the
    server's version through #648's verdict, the lead found and resumed only while idle, side
    threads unsubscribed), the client (one channel for responses, notifications and the server's
    end; server requests dropped unanswered), the quit hook.
  - `process.rs`: `following` takes the outputs and an optional folder and environment; `serve`.
    One spawn call still; `.config/spawn-sites.txt`'s line names `rh` and `codex`.
  - `agents.rs`: `agent_line` builds both lines for a Codex that joins; `start_in_terminal` takes
    the `Joining`, starts the server after the terminal, and types the plain line when it does not
    come up; `launch_input` (launch configs) and `start_cli_with_prompt` use it.
  - `agent_events`: `seat_agent`, `seat_of`, `fold_codex`, `heard_from`, `waiting_agent`; Claude
    Code's fold starts no seat from a Codex one. `agent_versions`: `is_on`, `program`, and the
    chip lists the App Server row only while the switch is on.
  - Readers: the rail's row (the seat by its agent, `codex_line`, `agent_mark`, the activity),
    `note_claude_code` ends a seat whose agent left the foreground, the inbox entry and its icon
    by the seat's agent (risk and route marks for Claude Code's waits only); the close guard;
    the stall watch (Claude Code's seats only); the Browser tab's Send sentence.
  - Zed: `settings_content`'s field, `default.json`'s key and allow-map row, the page's two items;
    the three ledger rows widened first. `script/e2e.sh` turns the switch off in each copy.
- **Deviations from the plan, and why.**
  - The policies ride as the labels `sandbox` and `approval`, not capabilities: no fleet event
    sets a seat's capabilities.
  - `agent_kind_of` trims quotes from the first word: the launch line names Codex by its quoted
    path, and #551's long-command watch and #572's error marks would otherwise read the session as
    a command.
  - #648's chip lists `codex_app_server` only while `marley.codex_app_server` is on, so a user who
    never turned the switch on sees no chip about it.
  - The server's environment adds the worktree's port variables (#590) and the terminal
    settings' `env` to the design's list, as Zed's terminal builder does; the folder's
    environment waits at most 3 s.
  - The row's name is "App Servers", so #648's tooltip line reads "App Servers are off: …".
  - At quit the servers are killed at once (gpui gives the quit hooks 200 ms), and the whole
    `marley-<pid>` folder goes.
- **Review.**
  - Re-entrancy: the follow folds through `cx.update`, outside any entity update; the terminal's
    release only removes the server from a global, whose drop detaches the process task.
  - Errors reach the user: a server that does not come up leaves the plain launch (logged); a
    server that stops makes the row read failed with its last stderr line; an untested server
    version leaves the row on the quiet timer and the chip says why.
  - Codex's turn is never touched: no resume while the lead is active, no answer to any server
    request.
  - The socket's folder is the user's only (0700); the program, the socket and the folder must
    hold no whitespace (the TUI's line and `/proc/net/unix` split on it).
- **Offline recording** (the plan's P2, scratchpad only: it names this machine): the installed
  0.155.1 server under a scratch `CODEX_HOME`, network off (`unshare -rn`). The `userAgent` reads
  `codex-tui/0.155.1 (…) (codex-tui; 0.155.1)` and becomes `(marley; …)` after a second client's
  `initialize`; `/proc/net/unix` lists the listening socket with state 01 and each accepted
  connection under the same path with state 03; `thread/read` of an unknown id answers -32600; a
  SIGTERM removes the socket.
- **Clippy rounds:** a field named after its struct (`thread_source`), four first doc paragraphs,
  two `map_or_else`, an unused import, a dead `waiting`, `serve` at 102 lines (the socket wait
  moved out), a value parameter not consumed, `&mut AsyncApp` where `&` serves.
- **Gate:** `just gate-diff` green after the manifest order fix (`async-tungstenite` after `askpass`, gate:17).

## Phase 3 — Test (2026-10-04)
- **Scenario:** `script/e2e/650-codex-app-server-state.sh`, `compositor sway`: one stand-in
  `codex` (`--version`, the App Server with Codex's routing rules, the remote TUI, and #532's
  plain fake), copies `0.155.1` and `0.150.0` behind the link `bin/codex` named by
  `MARLEY_CODEX`. Every check passes: no `--remote` with the switch off; the line, the server,
  Marley joining after the TUI; C's arguments still asking for full access; B's server and socket
  gone when B closed idle; the untested Codex without `--remote`; Marley's requests only the five,
  no request answered, no resume of an active thread; A's server and the socket folder gone after
  the quit.
- **Shots of the second run, each read:**
  - `650-01-off`: the plain stand-in, `codex started with:` and nothing; the row `Codex ·
    waiting`. REQ-001.
  - `650-02-launched`: A's line `'…/bin/codex' --remote unix:///run/user/1000/marley-<pid>/codex/1.sock
    --cd …/repo`; the row `idle`. REQ-002.
  - `650-03-working`: A `working · 12k tokens`, first in the rail. REQ-003, REQ-004.
  - `650-04-approval`: B `waiting on approval`, A `working · 12k tokens`; the inbox's `Codex ·
    repo`, "Waits on an approval", with Codex's icon. REQ-005, REQ-007, REQ-008.
  - `650-05-input`: A `waiting on input · 12k tokens`, B unchanged; two inbox entries. REQ-006,
    REQ-007.
  - `650-06-opened`: the first inbox entry clicked from A: B's terminal in front. REQ-008.
  - `650-07-done`: B `idle · 15k tokens`, its entry gone, A's left. REQ-009, REQ-004.
  - `650-08-failed`: B `failed · 12k tokens`, "stream disconnected before co…". REQ-010.
  - `650-09-close-asks`: "Close Codex in repo? It is working." with `repo · Codex · working`.
    REQ-011.
  - `650-10-full`: B's `full access` chip; its tooltip "Codex runs with no sandbox: its thread
    reports dangerFullAccess, with approvals on request", B started with no sandbox argument.
    REQ-012.
  - `650-11a-full-from-start`, `650-11-scoped`: C, started with full access, shows the chip;
    after `scoped` and `work` the chip is gone while its line still reads `--sandbox
    danger-full-access`. REQ-013.
  - `650-12-server-gone`: C `failed · 12k tokens`, "Codex's App Server stopped: st…". REQ-014.
  - `650-13-outside`: the 0.150.0 Codex typed with no `--remote`, its row on the quiet timer; the
    bar's "Untested Codex 0.150.0" chip, its tooltip "App Servers are off: …", the path found, the
    range, the setting. REQ-015.
  - `650-14-page`, `650-15-versions`: Codex App Server in the Agents section (on, as the run set
    it; the default is `default.json`'s `false`) and App Server on Untested Codex, off. REQ-019.
- **Fixed in Test:**
  - The first run's quit check found `marley-<pid>` left behind: the hook removed only its `codex`
    folder. It removes the whole folder now.
  - The first run's clicks on rail rows missed, since the rail sorts rows by what each agent
    needs; those steps use the palette (`pane: activate previous item`, `pane: close active
    item`), and the chip's place came from the shots.
- **Not reached by a scenario:** the real Codex and a real turn (an account and the network); the
  0.158.0 link socket (the stand-in binds as 0.155.1 does; `remove_leftovers` by review); the
  token-refresh risk across servers (Codex's own, by review); zsh and fish are not involved.

## Phase 4 — Complete (2026-10-04)
- **Docs:** `CHANGELOG.md` (Added, Codex's state from its own App Server); `docs/marley/guide.md`
  (the paragraph after the permission chip's, the settings page's list, the settings example);
  `docs/marley_architecture/marley_agent.md` (Codex's App Server, the surface, the versions row)
  and `marley_workbench.md` (Codex's own App Server, the agent versions' two readers); the plan's
  C1 row; the design note's B1 part 1 marked done; the three `zed-touchpoints.md` rows checked
  against what shipped.
- **Knowledge:** AD-claude-650-marley-joins-a-codex-app-server-of-its-own-per-terminal-001,
  F-claude-650-the-quit-hook-left-marleys-socket-folder-001,
  L-claude-650-a-codex-thread-resume-renames-its-client-001,
  L-claude-650-proc-net-unix-shows-who-joined-a-socket-001,
  L-claude-650-the-rail-sorts-rows-so-a-scenario-uses-the-palette-001,
  L-claude-650-e2e-runs-end-without-marleys-quit-hooks-001.
- **Brain:** consultation `c60abfdf55ae40d999b172d6e29c9aef` closed with
  `decisions/marley-joins-a-codex-app-server-of-its-own-per-codex-terminal-behind-a-switch-650`
  (follow-up 2026-11-04).
- **Closed:** TICKET-650 in `tickets/closed/`; the pair archived to `completed/`.
