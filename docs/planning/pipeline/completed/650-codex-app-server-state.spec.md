---
pipeline_id: 9b8291bc-2e89-4f11-9b37-fa1ed3c6e3a5
ticket: docs/planning/tickets/closed/TICKET-650-codex-app-server-state.md
status: Phase 4 — Complete PASS
title: "Codex's state from its own App Server"
type: feature
slice: prong 2, C1 (a terminal's own agent state, #519's line, now for Codex); design note B1, part 1
references: [docs/planning/design-notes/claude-and-codex-on-their-own-tools-2026-10-02.md, docs/planning/pipeline/completed/519-claude-code-events-in-the-rail.spec.md, docs/planning/pipeline/completed/532-agent-permission-modes.spec.md, docs/planning/pipeline/completed/550-ask-before-ending-a-working-agent.spec.md, docs/planning/pipeline/completed/552-codex-and-opencode-notifications.spec.md]
---

## Title
With `marley.codex_app_server` on (off by default), each Codex Marley launches runs against an
App Server of its own: Marley starts `codex app-server --listen unix://<socket>` for the
terminal, types `codex --remote unix://<socket>` into it, and joins the same server as a second
client once the TUI has joined. The server belongs to one terminal, so every thread it loads is
that terminal's. Marley follows the TUI's thread and folds its typed state into the terminal's
seat, the one `AgentEvents` keeps for Claude Code since #519: the rail's Codex row, the approvals
inbox, the rail's attention order and the close guard read the thread's status instead of 2 s of
quiet output, the row shows the thread's token use, and the full access chip follows the
thread's sandbox instead of the arguments. The integration registers Codex 0.155.1 to 0.158.0
with #648's tested-version table and stays off outside it. Marley sends Codex nothing that acts
in this slice.

## Scope
### In
- **The switch:** `marley.codex_app_server: Option<bool>`, off by default (D4):
  `MarleySettingsContent`, `MarleySettings::codex_app_server` resolved as `CodexAppServer { On,
  Off }` (#632's `EmbeddedHarness` shape), `default.json`'s `false` with its comment, and a
  Codex App Server toggle in the Marley page's Agents section after Codex Permissions, whose
  description says what changes (a server per Codex terminal; Codex's requests name Marley in
  their user agent, D10). It applies at the next launch; turning it off stops no server a
  running Codex uses.
- **The version check:** a row in #648's `marley_agent::versions::INTEGRATIONS`: id
  `codex_app_server`, Codex, from 0.155.1, before 0.158.1, resting on the App Server's thread,
  status, turn, token and policy types (D5). Outside it, or when #648's check reads no version,
  Codex starts as today and #648's chip in the agent bar under its terminal says why, unless the
  user's `marley.allow_untested_versions` allows `codex_app_server`; the Marley page's Agent
  Versions section (#648) gains the row's item, App Server on Untested Codex. The server's own
  version (the `userAgent` text after its first `/`) goes through the same verdict at
  `initialize`; a server it refuses is closed and its terminal's Codex reads the quiet timer,
  with the reason logged.
- **One server per Codex terminal (D1):** when Marley launches Codex in a local project (every
  caller of `agents::start_in_terminal` with Codex: the rail's +, New Agent, a worktree agent, a
  launch config's agent item) with the switch on and the version in range, it starts
  `<codex> app-server --listen unix://<socket>` through `process.rs` while the terminal's shell
  starts: in the terminal's folder, with the environment Zed gives that folder's processes
  (`ProjectEnvironment::directory_environment`, else Marley's own when the folder is in no
  worktree), `agent_env`'s variables, and the terminal's
  `MARLEY_TERMINAL_ID` and `MARLEY_PROJECT`. Once the socket is there (bounded at 5 s) it types
  the launch line. The server and its socket last exactly as long as the terminal, as #596's
  askpass proxy does; Marley's quit ends them.
- **The socket:** `$XDG_RUNTIME_DIR/marley-<pid>/codex/<8 hex>.sock`, else under Zed's
  `paths::temp_dir()`, in a folder Marley makes 0700 (Codex requires a parent only the user can
  write) and removes at quit. A path that does not fit `sun_path`
  (`marley_browser::service::socket_fits`) launches as today, the reason logged. Codex 0.155.1
  binds the path itself and removes it on a clean stop; 0.158.0 makes the path a link to a socket
  under `/tmp/codex-daemon-<uid>/` with a `.lock` beside it. A server is stopped with SIGTERM,
  then killed after 2 s, off the main thread, and Marley removes whatever of the path, its target
  and the `.lock` is left.
- **The launch line (D6):** `marley_agent`'s launch lines take an optional App Server address and
  folder, put as `--remote unix://<socket> --cd <folder>` (each one quoted word) after the
  program; Codex's own arguments (#532's `--sandbox` and `--ask-for-approval`, which the TUI sends
  in `thread/start` in remote mode, a first prompt, a setup command) stay as they are. The server
  and the line name the same `codex` by its full path: `MARLEY_CODEX`, else the search path's, the
  binary #648's check reads (L-claude-531).
- **The client (D2):** a new adapter module `marley_workbench::codex_server`: JSON-RPC over a
  WebSocket on the Unix socket (`smol`'s `UnixStream`, `async_tungstenite::client_async` at
  `ws://localhost/rpc`, as `marley_browser::cdp::connect_unix` reaches the relay). Messages carry
  no `jsonrpc` field and notifications an `emittedAtMs`; a message with both an id and a method is
  a server request, which this slice reads and never answers, not even with an error, since any
  subscriber's reply resolves it and an error reads as a denial (D9). A call waits at most 5 s.
- **Joining second (D10):** the client opens no connection until the TUI's own connection shows on
  the server's socket (an accepted connection under its path in `/proc/net/unix`, checked every
  500 ms while the terminal lives) and 1 s has passed, so the TUI's `initialize` names the
  server's originator. Then `initialize` with `clientInfo` `marley` and Marley's version, the
  experimental API as the TUI asks for it (D3), `optOutNotificationMethods` for the item deltas
  Marley does not read (the server drops a connection whose queue fills; never `item/started`,
  which #651 reads), and `initialized`.
- **Following the thread (D3):** `thread/started` and `thread/status/changed` reach every
  initialized connection, so the state needs no subscription. Once joined, `thread/loaded/list`
  and a `thread/read` of each id find a lead the TUI started before Marley joined, each second
  until there is one; afterwards new threads come by `thread/started`. The lead is the newest
  thread that is not ephemeral (the TUI's side threads, for a title or a recap, are), has no
  `parentThreadId`, and whose `threadSource` is `user` or absent. A lead made after Marley joined
  is subscribed already, since the server attaches every initialized connection to each thread it
  makes; one made before is subscribed with a bare `thread/resume { threadId, excludeTurns: true }`
  only while its status is `idle`, since a resume renames the thread's client until the TUI's next
  `turn/start` and Codex offers plugin installs only to `codex-tui`. Its answer gives the approval
  policy and sandbox; `thread/settings/updated` gives them afterwards. Every other thread Marley
  was attached to (side threads, subagents), a lead replaced (Codex's `/new`), and a lead left
  when Codex leaves the terminal's foreground is unsubscribed (`thread/unsubscribe`), since a
  subscriber keeps a thread loaded.
- **The fold (D7), pure, in `marley_agent::codex_events`:** the read types (thread, status, turn,
  token use, thread settings, the resume answer) and a `fold` from a notification or an answer to
  `marley_fleet` events for the terminal's seat, as `claude_events::fold` does. `active` with no
  flag is working; `waitingOnApproval` and `waitingOnUserInput` are waiting, with a `wait` label
  (`approval`, `input`); `idle` is idle; `systemError` is failed; a `turn/completed` whose status
  is `failed` is failed with the turn's error message, and stays failed through the `idle` that
  follows until the next turn starts; `thread/closed` or `notLoaded` for the lead ends the seat.
  `thread/tokenUsage/updated` gives the labels `tokens` (the thread's total) and `context_window`;
  the policies give the capabilities `sandbox` and `approval`. The seat carries `agent: codex`
  and `thread`. A value that does not parse leaves the seat as it was, logged.
- **The rail:** `terminal_snapshot` reads a seat for Codex as for Claude Code, when the seat's
  `agent` is the row's agent. The row's line is the state's words (`working`, `waiting on
  approval`, `waiting on input`, `idle`, `failed`, or #547's `no update in N m`) and `· 41k tokens`
  once tokens are known (`fleet::compact`); a failed seat's next line is its error. The seat's
  events place the row in the rail's attention order as Claude Code's do (#542).
  `note_claude_code` ends a seat whose row's agent is no longer the seat's, so a Codex seat ends
  when Codex leaves the foreground, and no longer merely because its row is not Claude Code's.
- **The inbox:** a waiting Codex seat is a terminal entry named Codex with Codex's icon, asking
  "Waits on an approval" or "Waits on an answer"; clicking it shows its terminal (#508's terminal
  entry). It answers nothing (#651).
- **The close guard:** `working_status` reads a Codex seat as it reads Claude Code's, so a quiet
  working Codex asks before it closes.
- **The chip (D8):** `permission_mark` for Codex with a seat reads the seat's `sandbox`: full
  access when the thread reports `dangerFullAccess`, none otherwise, whatever the arguments say;
  the tooltip names the thread as the source, with its approval policy. Without a seat the
  arguments decide, as #532 does.
- **Holding a paste:** the paths that already hold a paste while a seat waits (send selection,
  the Browser tab's Send) hold it for a waiting Codex too; their sentence names the seat's agent.
- **Claude Code's own readers stay Claude Code's:** the stall watch (#569), which samples every
  working seat, skips seats whose `agent` is not `claude-code`; the stop kind (#566) and per-turn
  commits (#509) are reached only from Claude Code's frames and stay so.
- **A server that stops:** a client whose server exits or whose socket closes ends its follow;
  a seat it had reads failed with "Codex's App Server stopped" and the server's last error line.
  Marley does not start it again: `--remote` never falls back or reconnects.
- **Scenarios stay off the user's Codex:** `script/e2e.sh` sets `marley.codex_app_server` false in
  each run's profile copy (L-claude-633), and adds no `codex_app_server` key to the allow map it
  writes for #648, so the version check stays live; a scenario that wants the server sets the
  switch back, with a stand-in named by `MARLEY_CODEX` (#648's variable).
- `script/e2e/650-codex-app-server-state.sh`, with a stand-in `codex` (server and TUI).

### Out (explicitly deferred)
- **#651, B1 part 2:** answering Codex's approvals from the inbox (the server sends each request
  to every subscriber and the first answer wins), the request's detail on the entry and the
  inbox's risk and route marks for it, and prompts through `turn/start`, `turn/steer` and
  `thread/queue/add` (send selection, send block, review notes, rich input) in place of the typed
  paste.
- The items: the tool in flight, the prompt and the last message on the row (`item/*`), diffs and
  plans in the Agent tab (`turn/diff/updated`, `turn/plan/updated`), per-turn commits for Codex,
  and a subagent count on the row.
- Notifications from Codex's typed state (#538's banner, #535's push): Codex's own OSC 9 (#552)
  stays the way Codex notifies.
- Resume through `thread/resume` or `codex resume` after a restart (B5).
- A Codex the user types by hand, or one pointed at another server: it uses Codex's own shared
  daemon or an embedded server, so the quiet timer, as today. Remote projects and terminals.
- Codex's hooks and plugins (the server's environment carries the terminal's identity, which a
  later hook ticket can use; not checked here). Codex's shared daemon and `codex agents`.
- A Codex capability for a client that should not name the originator or the user agent: worth
  asking OpenAI for (D10), not this ticket's.
- **A client shared with rustal-harness: not requested.** The harness speaks JSONL over stdio to
  private servers it spawns and owns, polls synchronously, accepts one binary by its digest, sits
  in an unlicensed crate with no repository, and handles neither `thread/started` nor
  `thread/status/changed` (Prior art). Marley writes its own small client, and no MREQ-style
  request goes to the harness. If the harness ever joins a shared server, a Marley-owned crate it
  vendors, as it vendors `marley_fleet`, is the route.

## Reference (§20)
N/A — Marley-specific. The behaviour is Codex's own: the App Server its TUI already runs against,
read by a second client over the protocol Codex documents (learn.chatgpt.com/docs/app-server) and
generates as JSON Schema. Warp's agent notifications show working, blocked, completed and errored
per tab (docs.warp.dev, as #519 cites; no Warp code read), and Marley's row already shows those
states for Claude Code; this ticket gives the same row Codex's typed state. Upstream Zed is used,
not changed in behaviour: `project`'s `ProjectEnvironment` for the server's environment,
`util::command` through `process.rs` for the spawn, the terminal builder's identity variables
(#520); three Zed-side files whose `zed-touchpoints.md` rows Marley already owns gain the setting.

### Prior art
- **Behavior maps.** The design note: B1, the "fights" table (the quiet timer at
  `marley_agent.rs:589-599`, `close_guard.rs:204`, the argv chip at `marley_agent.rs:405-499`),
  "Their tools" (`--remote` never falls back; hooks run in the server's process with the starting
  terminal's environment, herdr #4859; a `-c` or `--profile` sends the TUI to a private server)
  and Chad's answers (B1 "Yes", B7 "Your install, with a check"). Orca's map,
  `docs/orca_architecture/01-agents-and-sessions.md`: Orca runs `codex app-server` for its
  structured sessions with `CODEX_HOME` pinned, over JSONL (lines 456-467), and locks its probe's
  server per home because two processes refreshing one home's token would spend a single-use
  refresh token twice (lines 623-627). Orca's MIT source, `src/main/codex/` (read):
  `codex-structured-thread-facts.ts` reads the thread and turn ids and the turn's status in both
  shapes Codex has used and treats `idle` and `systemError` as not running; nothing in it joins a
  TUI's server (`--remote` appears only in `src/cli/codex-command-classification.ts`'s list of
  flags). `docs/warp_architecture/` and `docs/zed_architecture/`: nothing on Codex.
- **Published material.** learn.chatgpt.com/docs/app-server (fetched 2026-10-03): `--listen
  unix://PATH` is "WebSocket connections over ... a custom Unix socket path, using the standard
  HTTP Upgrade handshake"; "The server rejects any request on that connection before this
  handshake"; `optOutNotificationMethods`; `thread/loaded/list`; `thread/unsubscribe` and the
  30 minutes a thread stays loaded past its last subscriber; `thread/status/changed`'s payload;
  `codex --remote` takes `unix://PATH`. It does not say which notifications reach a connection
  that did not start the thread, nor how server requests route between clients. Codex's source at
  `rust-v0.155.1`, the installed release (Apache-2.0; a shallow clone in the scratchpad, paths
  under `codex-rs/`): the server attaches every initialized connection to each thread it creates
  (`app-server/src/lib.rs:1252-1266`), so such a connection gets that thread's turns, items and
  approval requests (`outgoing_message.rs:168-178`); `thread/started` and
  `thread/status/changed` go to every initialized connection (`thread_processor.rs:1630-1633`,
  `thread_status.rs:246-251`, `outgoing_message.rs:745-778`, `transport.rs:220-232`);
  `thread/settings/updated` is experimental (`app-server-protocol/src/protocol/common.rs:1932`),
  so only a client of the experimental API receives it; the first originating client names the
  server's originator and each one replaces the user agent's client suffix, except
  `codex_app_server_daemon` and `codex-backend`
  (`request_processors/initialize_processor.rs:18, 125-172`); the TUI initializes as `codex-tui`
  (`tui/src/lib.rs:447`), goes to a server only with `--remote` or to a daemon already running,
  else embeds one (`tui/src/lib.rs:927-955`), sends a folder in remote mode only with `--cd`
  (`tui/src/startup_orchestration.rs:209-212`, `app_server_session.rs:2202-2213`), always sends its
  approval policy and sandbox (`app_server_session.rs:2048-2076`), refuses `--worktree` with
  `--remote` (`startup_orchestration.rs:17-20`), rejoins a loaded thread with a bare
  `thread/resume` (`app_server_session.rs:2096`), and makes ephemeral side threads with a
  `Feature` source (`temporary_structured_request.rs:143-144`); the socket is 0600, its parent is
  made 0700, a live one refuses a second server, a clean stop removes it, and the client's
  upgrade URL is `ws://localhost/rpc` (`app-server-transport/src/transport/unix_socket.rs:28,
  107, 158-188, 253-263`; `app-server-client/src/remote.rs:69`). Against 0.158.0 (read through
  `gh api` at `rust-v0.158.0`, and the harness's notes): the socket path becomes a link into
  `/tmp/codex-daemon-<uid>/`, a plain `codex` starts the shared daemon itself when none runs, and
  the side threads' source reads `thread_title`; nothing Marley reads moved. `codex --help` and
  `codex app-server --help` (0.155.1). JSON Schema bundles generated offline into the scratchpad,
  with a scratch `CODEX_HOME`, from the installed 0.155.1 (312 files) and the harness's pinned
  0.158.0 binary (314): `Thread`, `ThreadStatus`, `ThreadActiveFlag`, `TurnStatus`, `Turn`,
  `ThreadTokenUsage`, `AskForApproval` and `SandboxPolicy` agree in both on every field Marley
  reads, the basis of D5.
- **Code we already ship.** Marley: `marley_browser::cdp` (`connect_unix`, `run`, `dispatch`: a
  JSON-RPC-shaped client over a WebSocket on a Unix socket with `async-tungstenite` 0.33 and
  `smol`, which the new client follows; its `dispatch` takes a message with an id for a response,
  so a server request would be lost, which is one reason the client is its own);
  `marley_browser::service::socket_fits`; `process.rs`'s `following` (kill on drop), used by #632
  for `rh serve`; `agents::start_in_terminal` (the shell's handshake before the line is typed; the
  askpass proxy tied to the terminal's release); `agent_events::AgentEvents` and
  `marley_fleet::apply`, the seat every reader already takes; `claude_events::{fold, seat_line,
  seat_status}`; `marley_agent::{permission_mark, quote_argument, launch_line_after}`;
  `fleet::compact`; #632's `EmbeddedHarness` switch and #633's page toggle. Zed: `project`'s
  `ProjectEnvironment::directory_environment` (the environment Zed's tasks and language servers
  get, direnv included); `context_server`'s JSON-RPC client is stdio and HTTP only and its
  `Client` is `pub(crate)`, `rpc` is protobuf, `lsp` is Content-Length over stdio: none owns this
  seam. `Cargo.lock`: `async-tungstenite` 0.33 (tungstenite 0.28), `smol`, `serde_json`, `which`;
  nothing new. rustal-harness (Ignibyte's own, read): its `codex` module
  (`crates/harness-runtime/src/codex/{session.rs, wire.rs}`, JSONL over stdio only), the
  `initialize` it sends and checks (`userAgent` as `<client>/<version> ...`), its schema pinning
  (`scripts/codex_profile.py`: `generate-json-schema` with the network off, inventories compared)
  and its Python peers (`scripts/fixtures/codex_*_peer.py`), the pattern for the stand-in. Ideas
  reused, no code taken.

## UI proof
`script/e2e/650-codex-app-server-state.sh` (`compositor sway`: the + menu, a chip's tooltip and a
row's close button are clicks). Setup: a scratch repository opened with `open_path`; a stand-in
`codex`, a Python program the scenario writes, named by `MARLEY_CODEX` and first on the
terminal's PATH (its `.bashrc` through `terminal_env`), with `CODEX_HOME` a scratch folder. It
answers `--version` from a file the scenario writes (`codex-cli 0.155.1`). As `app-server
--listen unix://P` it serves the WebSocket upgrade and the JSON-RPC on `P` as the 0.158.0 source
does (`initialize` with a `userAgent` in the real shape, `thread/start`, `thread/loaded/list`,
`thread/read`, `thread/resume`, `thread/unsubscribe`; `thread/started` and status to every initialized
connection; a new thread's subscribers are the connections initialized at its start), logs each
method with the client that sent it, and plays recorded notifications, in the 0.155.1 schema's
shapes, to the thread's subscribers. As `--remote unix://P` it is the TUI: it prints its
arguments, initializes as `codex-tui`, starts a non-ephemeral thread and an ephemeral side
thread for its folder, and turns each line typed into it into a cue the server plays (`work`:
`turn/started`, `active`, token use 12,400, then silence; `approve`: `waitingOnApproval` and an
approval request; `input`: `waitingOnUserInput`; `done`: `turn/completed`, `idle`, 15,800 tokens;
`fail`: a failed `turn/completed` with an error message, then `idle`; `full` and `scoped`: the
thread's sandbox moved to `dangerFullAccess` and to `workspaceWrite`, sent as
`thread/settings/updated` to the thread's subscribers that asked for the experimental API;
`crash`: the server exits).
Without `--remote` it prints its arguments and reads its input, as #532's fake does. Shots:
- `650-01-off`: the switch off: the terminal shows `codex` started with no `--remote`; its row
  reads the quiet timer's `Codex · waiting`;
- `650-02-launched`: on, a Codex (A) from the +: the terminal shows `--remote unix://…/codex/…`
  and `--cd`, and its row reads `idle`;
- `650-03-working`: `work` typed in A, then 5 s of silence: A's row reads `working · 12k tokens`;
- `650-04-approval`: a second Codex (B) launched; `approve` in A: A's row reads `waiting on
  approval`, B's `idle`; the inbox lists A as Codex's, "Waits on an approval";
- `650-05-input`: `input` in B: B's row reads `waiting on input`, A's is unchanged;
- `650-06-opened`: B's terminal shown, A's inbox entry clicked: A's terminal shows;
- `650-07-done`: `done` in A: `idle · 15k tokens`, and A's inbox entry gone;
- `650-08-failed`: `work` then `fail` in A: `failed`, the error on the next line;
- `650-09-close-asks`: `work` in A, 5 s of silence, A's close button: the close guard's question
  names Codex and `working`;
- `650-10-full`: the question cancelled; `full`, then `work`, in A (started without full access):
  the `full access` chip on A's row, its tooltip under the pointer naming the thread;
- `650-11-scoped`: a Codex (C) launched with `codex_permissions: full_access` for the repository:
  its chip shows; `scoped`, then `work`, in C: the chip gone while its arguments still ask for it;
- `650-12-server-gone`: `crash` in B: B's row reads `failed` with "Codex's App Server stopped";
- `650-13-outside`: Marley restarted with the version file at `codex-cli 0.150.0`: a new Codex
  starts with no `--remote`, its row reads the quiet timer, and #648's chip in its agent bar,
  under the pointer, names the part, the version found and the range;
- `650-14-page`: the Marley page: the Agents section's Codex App Server toggle, off, and the Agent
  Versions section's App Server on Untested Codex.
After the run the scenario checks the stand-in's logs: Marley's `initialize` came after each
TUI's, its methods are only the five of REQ-017, and no stand-in server outlived its terminal.

## Locked-In Decisions
- D1 — One App Server per Codex terminal Marley launches, owned by Marley and living exactly as long
  as the terminal. The socket is the map from thread to terminal, so nothing is guessed; the server
  is never one another terminal started, and never updates itself. Rejected: one server per Marley
  instance or per `CODEX_HOME` (a thread then has to be matched to its terminal by `cwd` and
  `createdAt`, whole seconds and the same folder for two agents in one project, and nothing in
  `thread/start` that a launcher can set reaches `thread/started`); Marley calling `thread/start`
  itself and launching `codex --remote … resume <id>` (Marley would choose the thread's settings and
  be its first client); a relay socket per terminal that reads the TUI's own traffic for its
  `thread/start` answer (Marley in the middle of the user's live session); Codex's shared daemon,
  which a plain `codex` uses when one runs and, since about 0.157, starts (another terminal's
  environment, its own updates, the same matching problem, and Marley's `initialize` would rename
  the user agent of every Codex session on it).
- D2 — Marley's own small client: the pure fold in `marley_agent::codex_events` beside
  `claude_events`, the IO in `marley_workbench::codex_server`, shaped as `marley_browser::cdp` but
  reading server requests too, since #651 answers them. The harness's client is not shared (Out).
- D3 — Marley follows what reaches it and subscribes for the rest, without touching the TUI's
  turn: `thread/started` and `thread/status/changed` come to every initialized connection; a lead
  made before Marley joined (D10) is found with `thread/loaded/list` and `thread/read` and
  subscribed with `thread/resume` only while idle, since a resume renames the thread's client
  until the TUI's next `turn/start`. The policies come from that answer and then from
  `thread/settings/updated`, which Codex sends only to clients of the experimental API; Marley
  asks for it, as the TUI does, and reads only `threadSettings.approvalPolicy` and
  `sandboxPolicy`, whose shapes agree in both ends' bundles. Rejected at promotion: a
  `thread/resume` at each `turn/started` (inside the TUI's turn) and the stable API alone (the
  chip would lag a turn).
- D4 — Off by default, applied at the next launch: it changes how Codex starts (two arguments, a
  server Marley owns, Codex's commands under the project's environment rather than the shell's,
  the user agent's suffix, D10), behaviour a user already has. Turning it off never stops a
  running server, which would cut a live TUI.
- D5 — The tested range is from 0.155.1 and before 0.158.1, closed as #648 asks of a protocol
  generated per release: the two ends, whose generated schemas agree on every field Marley reads.
  #648's table holds it and decides, and its allow map can turn it on outside. The server's
  `userAgent` goes through the same verdict, since it is the version that answers.
- D6 — One binary and one folder: the server and the TUI are the same `codex`, named by its full
  path, and the line passes `--cd` with the folder the server starts in, since a remote TUI sends
  no folder otherwise and the thread would take the server's.
- D7 — `thread/status/changed` decides the state; a failed turn adds its text and stays failed
  until the next turn starts; token use and policies are labels and capabilities on the seat,
  which `fleet_snapshot` serves as it serves Claude Code's (#547).
- D8 — The chip keys on the sandbox alone, as #532's argument rule does: `dangerFullAccess` is
  full access whatever the approval policy, and the tooltip says both.
- D9 — Marley answers nothing: a waiting Codex enters the inbox as a terminal entry that shows its
  terminal, and the approval request that reaches Marley's connection is left to the TUI, whose
  answer wins. Answering in place is #651.
- D10 — Marley joins second. A server's first initializing client names the originator of its
  threads, and each initializing client replaces the client suffix of the user agent Codex sends
  upstream; only Codex's own `codex_app_server_daemon` and `codex-backend` are exempt, and Marley
  does not borrow either name. Marley waits for the TUI's connection on the server's socket, then
  1 s, so the originator stays `codex-tui`; the suffix then names Marley (`marley; <version>`),
  which the setting's description says.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `marley.codex_app_server` is off, the system shall start Codex with no `--remote` and no App Server, and its row shall read the quiet timer as today. | Shot `650-01-off` |
| REQ-002 | WHEN Marley launches Codex in a local project with the switch on and the installed Codex in the tested range, the system shall start an App Server for that terminal and type a launch line naming it with `--remote unix://` and `--cd`. | Shot `650-02-launched` |
| REQ-003 | WHILE the thread's status is `active` with no flag, the Codex row shall read `working`, also after its terminal has shown no output for more than 2 s. | Shot `650-03-working` |
| REQ-004 | WHEN the thread reports token use, the Codex row shall show its total as `· <n>k tokens` after the state's words. | Shots `650-03-working`, `650-07-done` |
| REQ-005 | WHILE the thread is `active` with `waitingOnApproval`, the row shall read `waiting on approval`. | Shot `650-04-approval` |
| REQ-006 | WHILE the thread is `active` with `waitingOnUserInput`, the row shall read `waiting on input`. | Shot `650-05-input` |
| REQ-007 | WHILE two Codex terminals run, each row shall show only its own terminal's thread state. | Shots `650-04-approval`, `650-05-input` |
| REQ-008 | WHILE a Codex seat waits, the approvals inbox shall list it as Codex's with what it waits on, and WHEN the entry is clicked the system shall show its terminal. | Shots `650-04-approval`, `650-06-opened` |
| REQ-009 | WHEN the thread's status becomes `idle`, the row shall read `idle` and its inbox entry shall go. | Shot `650-07-done` |
| REQ-010 | WHEN a turn completes as `failed`, the row shall read `failed` with the turn's error message on its next line until the next turn starts. | Shot `650-08-failed` |
| REQ-011 | WHEN the user closes a terminal whose Codex seat is working or waiting, the close guard shall ask first, naming Codex and its state. | Shot `650-09-close-asks` |
| REQ-012 | WHEN the thread's sandbox is `dangerFullAccess`, the row shall carry the `full access` chip, whose tooltip names the thread as its source, whatever Codex's arguments say. | Shot `650-10-full` |
| REQ-013 | WHEN the thread's sandbox becomes any other policy, the system shall remove the chip, even while the arguments ask for full access. | Shot `650-11-scoped` |
| REQ-014 | IF a terminal's App Server exits while its Codex runs, THEN its row shall read `failed` with "Codex's App Server stopped". | Shot `650-12-server-gone` |
| REQ-015 | IF the installed Codex, or the server's reported version, is outside 0.155.1 to 0.158.0 and `marley.allow_untested_versions` does not allow `codex_app_server`, THEN the system shall start Codex as with the switch off and #648's chip shall say the part is off and why. | Shot `650-13-outside`; review |
| REQ-016 | WHEN a terminal with a Codex App Server closes, or Marley quits, the system shall end that server and remove its socket. | Review; the scenario's check that no stand-in server is left |
| REQ-017 | The system shall send a Codex App Server no request other than `initialize`, `thread/loaded/list`, `thread/read`, `thread/resume` and `thread/unsubscribe`, and answer none of its requests. | Review; the stand-in's method log |
| REQ-018 | The system shall send its `initialize` to a Codex App Server only after the terminal's TUI has connected to it. | Review; the stand-in's log of `initialize` order |
| REQ-019 | The Marley page shall offer the Codex App Server toggle in its Agents section, off by default, and App Server on Untested Codex in its Agent Versions section. | Shot `650-14-page` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes. At promotion: #648 must have shipped (its
  `Integration` row, `verdict`, chip, allow map and `MARLEY_CODEX`, as drafted); settle in
  Codex's source what is still open (Risks: `thread/settings/updated`'s gate, the environment
  hooks and commands get, two servers refreshing one home's token, whether `codex resume` lists a
  thread whose source is `vscode`); `brain_ask`.
- **P2 Code** — record the installed server's `initialize` answer and framing once, offline (a
  scratch `CODEX_HOME`, the network off as the harness's profile script runs it, no thread, no
  turn), so the stand-in answers in its shape; the switch and its page item; the socket folder;
  `codex_server` (the spawn through `process.rs`, the join, the client, the follow, the stop);
  `codex_events`; the launch line's address and folder; the rail, the inbox, the close guard, the
  chip and the paste guard; the version row; `e2e.sh`'s default; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write the stand-in and the scenario, run it, read every shot and
  the stand-in's logs.
- **P4 Complete** — CHANGELOG; `docs/marley_architecture/marley_agent.md` and
  `marley_workbench.md` (the fold, the client, the socket folder); the guide's agents section;
  the plan's C1 row; `zed-touchpoints.md`'s three rows checked; ledger capture (§19); close the
  ticket, archive, commit.
