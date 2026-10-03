---
pipeline_id: b9cf18ea-194e-4c29-af68-b23a7a537160
ticket: docs/planning/tickets/open/TICKET-653-marley-as-claude-codes-ide.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Marley as Claude Code's IDE: the link, diagnostics, selection and open file"
type: feature
slice: prong 2, Claude Code on its own tools (design note B3, "Yes, with version checks"); #549's send selection
references: [docs/planning/design-notes/claude-and-codex-on-their-own-tools-2026-10-02.md, docs/planning/pipeline/completed/549-selection-to-the-agent.spec.md, docs/planning/pipeline/completed/633-rusty-tools-for-zeds-agents.spec.md, docs/planning/design-notes/warp-second-pass-2026-09-25.md]
---

## Title
With `marley.claude_code_ide` on, Marley serves Claude Code's IDE MCP server for each open
project: a WebSocket server on a loopback port the system picks, a lock file `<port>.lock` in
Claude Code's `ide` directory, and that port in the environment of the project's new terminals,
so `claude` started in a Marley terminal connects to its own project's Marley. Claude Code then
gets the editor's selection and open file with each prompt, reads Zed's language-server
diagnostics through `mcp__ide__getDiagnostics`, and takes send selection's reference as an IDE
mention instead of typed text. Every part the docs do not name is checked against the Claude Code
versions in #648's table and stays off outside them, with the reason shown. Diffs come later.

## Scope
### In
- **The server** (`marley_mcp::ide`, the pure core, and `marley_mcp::transport::ide`, the
  sockets): one per open project (Zed's `Project`), bound to `127.0.0.1:0`. The upgrade request
  is read with the HTTP transport's pre-auth bounds (#524); the token in
  `X-Claude-Code-Ide-Authorization` is compared in constant time; a non-loopback `Origin` and a
  `Host` other than the server's own address are refused; the `mcp` subprotocol the client asks
  for is echoed and no extension is accepted. Frames go through `tungstenite` 0.28's sync API on
  `marley_mcp`'s own threads. Several clients at once, at most 8. JSON-RPC: `initialize`,
  `notifications/initialized`, `ide_connected`, `tools/list`, `tools/call`, `prompts/list` (empty)
  and `ping`; the server pings every 30 s and closes a client that misses two pongs.
- **The lock file**: `$CLAUDE_CONFIG_DIR/ide/<port>.lock`, else `~/.claude/ide/<port>.lock`; the
  directory made 0700 when it is missing; the file written 0600 through a temporary file and a
  rename. Fields: `pid` (Marley's), `workspaceFolders` (the project's visible worktree roots,
  rewritten when one is added or removed), `ideName` `"Marley"`, `transport` `"ws"`,
  `authToken` (a fresh 128-bit token per start, from `marley_mcp::secret::mint_secret`) and
  `runningInWindows` `false`. Removed when the project closes, the switch turns off or Marley
  quits. When a server starts, Marley removes any lock file in that directory whose `ideName` is
  `Marley` and whose `pid` is no live process, and touches no other.
- **Which Marley**: each new local terminal of a project that has a server carries
  `CLAUDE_CODE_SSE_PORT=<that project's port>` and `CLAUDE_CODE_AUTO_CONNECT_IDE=true`, set before
  the settings' `terminal.env` so the user's own values win (`crates/project/src/terminals.rs`).
  A Claude Code started outside Marley reaches it with `/ide` or `--ide`, through
  `workspaceFolders`.
- **Diagnostics**: `getDiagnostics` with an optional `uri`: that file's diagnostics from Zed's
  language servers, or, with none, every file of the project that has any. Each entry has
  `message`, `severity` (`Error`, `Warning`, `Information`, `Hint`), `range` (0-based lines,
  UTF-16 characters, as the LSP counts them), `source` and `code`.
- **Selection and open file**: `selection_changed` to every connection, 100 ms after the
  selection or the active file editor of the project's workspace changes, and once after a client
  says `ide_connected`: `text`, `filePath`, `fileUrl`, and `selection` with `start`, `end` and
  `isEmpty`; a caret alone sends the open file with an empty selection. The editor is the last file
  editor that was the workspace's active item: a terminal or another item taking the focus leaves
  it as it was. The pull side the CLI uses: `getCurrentSelection`, `getLatestSelection` and
  `getOpenEditors`; and `getWorkspaceFolders`, which answers the lock file's folders.
- **Send selection** (`send_selection.rs`'s `send`): for a Claude Code target whose connection is
  known and in the mention row's range, `at_mentioned` (`filePath`, `lineStart`, `lineEnd`, 0-based)
  to that one connection, the terminal revealed and focused, nothing typed. Rich input open on the
  terminal, and the refusal while the seat waits (#549), come first as today; every other case
  types `@path#La-b` as #549 does.
- **A connection's terminal**: `ide_connected`'s `pid`, matched to the terminal whose foreground
  process group is that pid, or whose shell is an ancestor of it (`/proc/<pid>/stat` read as
  `marley_agent::stall` reads it).
- **Version checks**, three rows in #648's table (`marley_agent::versions`'s `INTEGRATIONS`):
  `claude_ide_connection` (the lock file's fields, `CLAUDE_CODE_SSE_PORT`, the `mcp`
  subprotocol, `ide_connected`, `getWorkspaceFolders`), `claude_ide_selection`
  (`selection_changed`, `getCurrentSelection`, `getLatestSelection`, `getOpenEditors`) and
  `claude_ide_mention` (`at_mentioned`), each with its item in the Marley page's Agent Versions
  section and its key in the e2e harness's allow map, as #648's Out asks of B3. While #648's
  verdict for the installed Claude Code leaves the connection row off: no server, no lock file,
  no variable, and #648's chip under a Claude Code terminal says why. A connecting client's own
  version (`initialize`'s `clientInfo`, else its `User-Agent`) is judged by the same range test
  and the same allow map: outside the selection or the mention row, that part is not sent to it
  and its tools are not listed to it, and the log says so; `getDiagnostics`, which the docs name,
  is still answered.
- **The switch**: `marley.claude_code_ide: Option<bool>`, off by default, with a toggle in the
  Marley page's Agents section. A change takes effect without a restart: servers start or stop,
  and new terminals get the variables or not.
- **Scenarios stay off the user's Claude Code**: `script/e2e.sh` writes
  `marley.claude_code_ide: false` into each run's copy of the user's settings, as #633 did for
  Rusty, and adds the three rows' ids to #648's allow map there; a scenario that wants the link
  turns it back on with a scratch `CLAUDE_CONFIG_DIR`.
- `script/e2e/653-marley-as-claude-codes-ide.sh`, with a stand-in IDE client named `claude` and a
  stand-in language server.

### Out (explicitly deferred)
- **Diffs**, the next slice of B3: `openDiff` into Zed's diff view with accept and reject, with
  `close_tab` and `closeAllDiffTabs`, its own row in #648's table. Until then edits show in Claude
  Code's terminal.
- `openFile`, `checkDocumentDirty`, `saveDocument` and `executeCode` (Jupyter): not listed; a call
  gets an MCP tool error naming the tool.
- Terminals opened before the switch turned on, and the shells of remote terminals: no variable
  (a remote shell cannot reach Marley's loopback, and its environment leaves the machine); `/ide`
  from a local one still finds Marley.
- Sessions Claude Code hosts in its background service (agent view, `/bg`): their connection
  comes from the service's process (anthropics/claude-code#98858), whose pid no terminal holds, so
  send selection types for them; they get the selection like any connection.
- A link indicator in the rail row or the terminal's tab: Claude Code's `/ide` and its prompt
  footer show the link.
- Codex, Gemini CLI and OpenCode: none of them speaks this protocol.
- The by-hand check of a later Claude Code minor or major: the ticket that moves the rows'
  `before` (#648's D3). This ticket records the first one (P3).

## Reference (§20)
N/A — Marley-specific: no Warp or Zed behavior to match. Warp's maps describe no Claude Code IDE
server, and Zed reaches Claude Code through ACP in its Agent Panel, not this protocol. The reference
is Claude Code's own published IDE integration, its VS Code extension and JetBrains plugin
(code.claude.com/docs/en/vs-code, "The built-in IDE MCP server"; /docs/en/jetbrains), which Marley
matches on the wire. Upstream Zed is read, unchanged: the diagnostics its language servers report
(`project`'s `lsp_store`), the selection `editor` holds, and send selection's #549 route, kept as
the fallback.

### Prior art
- **Behavior maps.** `docs/warp_architecture/`, `docs/zed_architecture/` and
  `docs/orca_architecture/` hold nothing on this protocol (searched for `claude/ide`,
  `selection_changed`, `at_mentioned`, `getDiagnostics`, `SSE_PORT`), nor does Orca's checkout.
  The Warp second pass (`warp-second-pass-2026-09-25.md:40-47`) named this route "L and uncertain"
  and #549 left it Out; the design note's B3 and Chad's answer scope it to what is here.
- **Published material.**
  - Claude Code's docs. `vs-code`: the server is named `ide` and hidden from `/mcp`; it binds
    `127.0.0.1` on a random port in 10000 to 65535 over unencrypted `ws://`; a fresh token per
    activation is written to `~/.claude/ide/<port>.lock` and presented as
    `X-Claude-Code-Ide-Authorization`; the file is 0600 in a 0700 directory, or under
    `$CLAUDE_CONFIG_DIR/ide/`; "a dozen tools", two visible to the model, `getDiagnostics`
    ("optionally scoped to one file") and `executeCode`, the rest "internal RPC … such as opening
    diffs, reading selections, and saving files"; the selection and the active file's path go with
    each prompt, shown as `⧉ Selected N lines from <file>`; a `Read` deny rule keeps a file out.
    `jetbrains`: an OS-assigned port; one model-visible tool, covering one file; `/ide` from an
    external terminal, confirmed as `Connected to IntelliJ IDEA.`; "launched `claude` from the
    IDE's integrated terminal" for the automatic link. `env-vars`: `CLAUDE_CODE_AUTO_CONNECT_IDE`
    (`true` forces an attempt "when auto-detection fails, such as when tmux obscures the parent
    terminal"), `CLAUDE_CODE_IDE_SKIP_VALID_CHECK` ("validation of IDE lockfile entries"),
    `CLAUDE_CODE_IDE_HOST_OVERRIDE`. `settings-reference`: `autoConnectIde` (default `false`) and
    `diffTool`. `cli-reference`: `--ide`, "if exactly one valid IDE is available". No page names
    `CLAUDE_CODE_SSE_PORT`, `ENABLE_IDE_INTEGRATION`, the lock file's fields or any notification.
  - Public implementations, read for facts; no code taken. coder/claudecode.nvim (MIT, active in
    2026-09): its `PROTOCOL.md`, reverse-engineered from the VS Code extension, gives the lock
    file's fields, `CLAUDE_CODE_SSE_PORT` and `ENABLE_IDE_INTEGRATION` set when it launches
    `claude`, the header, `selection_changed` and `at_mentioned`, and the twelve tools with their
    answers; its source answers `initialize` with MCP `2024-11-05`, sends mention lines 0-based,
    also sets `FORCE_CODE_TERMINAL=true`, adds loopback to `no_proxy` (its issue #70), pings every
    30 s, and writes the lock by a temporary file and a rename. stevemolitor/monet (MIT, Emacs):
    one server per session and `ide_connected` handled. iansinnott/obsidian-claude-code-mcp (0BSD):
    `ide_connected` carries Claude Code's `pid`; its note that lock files moved to
    `~/.config/claude/ide/` at 1.0.30 is not what the docs say now, and Marley follows the docs.
    jiahaoxiang2000/claude-code-zed (MIT, a Zed extension in Rust behind an LSP bridge): its
    `REVERSE.md` capture of 1.0.44 shows `User-Agent: claude-code/1.0.44`, `Sec-WebSocket-Protocol:
    mcp` asked for and required in the answer, and `permessage-deflate` offered.
    manzaltu/claude-code-ide.el (GPL-3.0): its README only (one server per Claude Code instance;
    `⧉ In file.el`); its source is not read, since Marley's crates are MIT OR Apache-2.0. Copies of
    Claude Code's own leaked source turn up in code search; none was opened.
  - anthropics/claude-code issues: #96742 (the VS Code extension sets `CLAUDE_CODE_SSE_PORT` for
    its terminals, and the CLI's lock-file discovery matches it), #67502 (the JetBrains plugin sets
    both variables per project), #79806 (2.1.216: `/ide` fails when the cwd is a subdirectory of
    the only workspace folder; a lock file shows `runningInWindows: false`), #98858 (2.1.287: a
    background-hosted session and its terminal front end both connect; VS Code keeps only the newest
    client).
  - The installed Claude Code: `claude --version` is 2.1.288 (it moved from the brief's 2.1.287);
    `--help` lists `--ide`. A string search of its binary, for names only, finds
    `CLAUDE_CODE_SSE_PORT`, `selection_changed`, `at_mentioned`, `ide_connected`,
    `workspaceFolders`, `ideName`, `runningInWindows` and `FORCE_CODE_TERMINAL`, and no
    `ENABLE_IDE_INTEGRATION`.
  - RFC 6455 (the upgrade, frames, close, ping); the MCP specification (`initialize`'s
    `clientInfo`).
- **The code we already ship.** Does a crate we build own this seam? `marley_mcp` owns all of it
  but the WebSocket: the bounded pre-auth reader (`transport.rs:611-630`, #524), `auth::ct_eq`
  (`:53`) and `origin_allowed` (`:40`), `secret::mint_secret` (`:44`),
  `discovery::write_endpoint_file_in` (`:38`, 0600 through the open handle), `jsonrpc`
  (`:55`, `:74`, `:82`) and the deferred app call (`ask_app`, `transport.rs:462-481`, 30 s).
  `tungstenite` 0.28.0, in `Cargo.lock` under `async-tungstenite` 0.33 (MIT OR Apache-2.0), owns
  the frames: `WebSocket::from_raw_socket`, `WebSocketConfig`'s caps and
  `handshake::derive_accept_key`. `marley_browser::relay` is Marley's own WebSocket server with a
  token at the handshake (`relay.rs:473-514`: `Host`, `Origin`, the token; `same_token` `:517`; a
  10 s handshake `:41`, `:531`), the model for the checks. Zed's agent `DiagnosticsTool`
  (`crates/agent/src/tools/diagnostics_tool.rs:92-257`) shows the calls for one file and for the
  project; its body is not carried (§20). `go_to_line`'s `cursor_position.rs:268-292` shows the
  active editor and `SelectionsChanged` with a debounce. `project/src/terminals.rs:417-442` is
  where a local terminal's environment is final, with Marley's variables already set there, and
  `marley_terminal::shell_integration` (`:142-162`) holds a value the workbench sets and each
  terminal reads. `Terminal::pid` (`terminal.rs:3627`) and `ProcessIdGetter::fallback_pid`
  (`pty_info.rs:26`) give the foreground group and the shell; `marley_agent::stall`
  (`stall.rs:17-18`) reads `/proc/<pid>/stat` with `procfs_core`. `claude_plugin::init`
  (`claude_plugin.rs:111-117`) resolves Claude Code's config directory; `MARLEY_CLAUDE` (`:122`)
  names the `claude` Marley runs.

## UI proof
`script/e2e/653-marley-as-claude-codes-ide.sh` (`compositor sway`; keys, and the picker's keys).
No real Claude Code runs and no model is called.

Setup: a scratch repository with `src/auth.txt` (ten numbered lines), `src/main.rs` and a
`Cargo.toml`, opened with `open_path`; a HOME whose `.bashrc` puts the scenario's `bin` first on
the PATH (`terminal_env HOME`); `CLAUDE_CONFIG_DIR` exported to a scratch directory, so Marley and
its terminals use the scenario's `ide` directory, and `MARLEY_CLAUDE` naming the stand-in, the
`claude` whose version #648 reads (F-claude-547); `marley.allow_untested_versions` set back to
`{}` in the run's settings, so #648's verdict is the version's own. In that `ide` directory, a
stale `1.lock` (`ideName` `Marley`, the pid of a process the setup started and ended) and a
foreign `2.lock` (`ideName` `Neovim`).

- **The stand-in `claude`** (Python, standard library only): `--version` prints the version in
  `$E2E_WORK/claude-version` with ` (Claude Code)`, the version the three rows hold until
  `653-13`; the other subcommands Marley runs exit 0. Run
  bare, it prints `port:` (`CLAUDE_CODE_SSE_PORT` or `unset`), `auto-connect:`, the lock file's
  fields, its mode and its directory's, then connects as Claude Code does (the token header,
  `Sec-WebSocket-Protocol: mcp`, `User-Agent: claude-code/<version>`, `initialize` with
  `clientInfo`, `notifications/initialized`, `ide_connected` with its pid) and prints the answer's
  subprotocol, `serverInfo` and tool names; then each notification on one line
  (`selection_changed <path> <start>-<end> <text>`, `at_mentioned <path> <a>-<b>`, `closed`). Each
  line typed at it prints `typed: <line>`, except `diag`, `diag <path>`, `latest` and `badtoken`,
  which call the tool or try a wrong token and print the answer. `--offline` skips the connection;
  `--as <version>` claims another version.
- **The stand-in language server** (Python), named in `lsp.rust-analyzer.binary.path`, publishes
  one error on line 3 of `src/main.rs` when the file opens; `session.trust_all_worktrees` is true
  in the run's settings.

Shots:
- `653-01-off`: the runner's copy has the switch off; `claude` in a terminal prints `port: unset`
  and `lock: none`.
- `653-02-lock`: the switch set on in the run's settings; a new terminal; `claude` prints the port,
  `auto-connect: true`, the lock's fields with `ideName Marley` and the repository as the folder,
  `0600` and `0700`; its listing shows the port's lock and `2.lock`, and no `1.lock`.
- `653-03-refused`: `badtoken`: `401`, no upgrade.
- `653-04-connected`: subprotocol `mcp`, `serverInfo`, the five tool names.
- `653-05-selection`: lines 2 to 4 of `src/auth.txt` selected in the editor:
  `selection_changed …/src/auth.txt` with a range from line 1 (0-based) and the three lines.
- `653-06-open-file`: `src/main.rs` opened, a caret only: `selection_changed …/src/main.rs` with an
  empty selection.
- `653-07-latest`: the terminal focused; `latest`: `src/main.rs`'s caret, the last file editor's.
- `653-08-diagnostics`: `diag`: `src/main.rs` with the stand-in's error at line 2 (0-based);
  `diag src/auth.txt`: an empty list.
- `653-09-mention`: lines 2 to 4 of `src/auth.txt` again and `ctrl->`: `at_mentioned
  …/src/auth.txt 1-3`, no `typed:` line, the terminal focused.
- `653-10-fallback`: a second terminal running `claude --offline`; `ctrl->`, the picker's second
  row: `typed: @src/auth.txt#L2-4` in it.
- `653-11-old-client`: a third terminal, `claude --as 9.9.9`; a new selection in the editor:
  nothing in it; `diag`: answered.
- `653-12-off-again`: the switch set off: each connected stand-in prints `closed`; the listing
  holds `2.lock` alone.
- `653-13-untested`: the version file set to `9.9.9`, the switch on, Marley restarted; a new
  terminal, `claude`: `port: unset`, `lock: none`; the pointer on #648's chip under it
  (`Untested Claude Code 9.9.9`): its tooltip names the IDE link among what is off.

Machine checks: the run log holds `at_mentioned` once and `typed: @src/auth.txt#L2-4` once; after
`quit_marley` the `ide` directory holds `2.lock` alone.

## Locked-In Decisions
- D1 — **The server lives in `marley_mcp`, beside the HTTP server, not inside it.** A second
  server kind, `marley_mcp::ide` (the methods, the lock file's JSON, the versions read from a
  client) and `marley_mcp::transport::ide` (the listener and frames), because the crate already
  holds every guard this needs (the bounded pre-auth read, `ct_eq`, the loopback bind, the token,
  the 0600 file, the deferred app call) and §14 puts sockets in `marley_mcp::transport`. It is a
  separate listener with its own tool set: Marley's own families (`terminal_*`, `browser_*`) are
  never listed on it, since any Claude Code of the user's that reads the lock file reaches this
  port and Marley's grants do not apply there. Rejected: a smol server in `marley_workbench`, as
  the relay runs (it would put a socket outside `marley_mcp::transport` and copy its guards).
- D2 — **One server per project; the port in the project's terminals picks it.** VS Code keeps
  one per window and JetBrains one per project (#67502); a Marley window holds several projects
  (`MultiWorkspace`), so the project is the unit. `CLAUDE_CODE_SSE_PORT` names the project's port,
  which is how a Claude Code in a Marley terminal connects to its own project's Marley when two
  windows show the same folder; `CLAUDE_CODE_AUTO_CONNECT_IDE=true`, the documented variable,
  asks Claude Code to connect at start from a terminal it does not know as an IDE's.
  `ENABLE_IDE_INTEGRATION` is not set (2.1.288 has no such name), nor `FORCE_CODE_TERMINAL`
  (undocumented, purpose unknown) unless the by-hand check shows the link needs it.
- D3 — **`tungstenite` for the frames, `marley_mcp` for the handshake.** The upgrade request is
  read by the transport's bounded reader, so every read before the token is checked keeps #524's
  limits (PR-claude-bound-every-read-before-auth-in-size-and-time-001); the answer's
  `Sec-WebSocket-Accept` comes from `tungstenite::handshake::derive_accept_key`; then
  `WebSocket::from_raw_socket` with 1 MiB frame and message caps. One thread per connection reads
  with a short timeout and writes its queue between reads, as `marley_mcp`'s threads do now.
  `tungstenite` joins `[workspace.dependencies]` at the version the tree builds, as `procfs-core`
  did (#503). Rejected: frames written by hand (the tree already builds a tested implementation).
- D4 — **Three rows in #648's table, checked twice.** #648's verdict on the installed version
  gates the connection row before anything is written, and the link follows it when it changes;
  each client's own version gates the selection and mention rows for that client, since a session
  started before an update runs its old version (#648 leaves the running session's version Out).
  Outside a row, its parts are not sent or listed; #648's chip says why for the install, the log
  for a client. `getDiagnostics`, documented, needs only the connection row. Bounds follow #648's
  D3: `from` the version the by-hand check passed on at P3 (2.1.288 on 2026-10-03), `before`
  2.2.0, so later 2.1 releases count as tested. If the check does not pass, the rows ship with no
  range Marley accepts, and only `marley.allow_untested_versions` turns the link on.
- D5 — **The selection is the last file editor's.** Claude Code's terminal is often the focused
  item in the Marley layout; VS Code keeps its last text editor active while the terminal has the
  focus, and claudecode.nvim ignores its Claude terminal. Debounced 100 ms (Zed's cursor position
  uses 50 ms; claudecode.nvim debounces too); local files only.
- D6 — **Send selection's link route is narrow.** Only a Claude Code target, only with a known
  connection in the mention row's range, after rich input and the waiting refusal (#549's D-rules
  unchanged). A mention types nothing into the pty, so it avoids #594's paste wait; the refusal
  stays until a check shows what a mention does while Claude Code asks a question.
- D7 — **Off by default** under `marley.claude_code_ide`. On, Marley writes a file every Claude
  Code of the user's can find (`/ide` in any terminal on the machine lists Marley's projects whose
  folders hold its cwd), adds two variables to every new terminal, sends the selection and the open
  file with each prompt, and changes send selection's route: all behavior a user does not have
  today.
- D8 — **Only what is in scope is listed.** `getDiagnostics`, `getWorkspaceFolders`,
  `getCurrentSelection`, `getLatestSelection`, `getOpenEditors`; any other call gets an MCP tool
  error naming the tool. The answers take the shapes claudecode.nvim documents for the VS Code
  extension (a text item holding JSON).
- D9 — **Several clients per server.** #98858 shows two connections for one backgrounded session
  and VS Code dropping the older one; Marley keeps each, up to 8.
- D10 — **The lock file is Claude Code's directory, so Marley is careful in it.** Created 0700
  only when missing, an existing directory's mode left alone; Marley's own file written through a
  temporary file and a rename; only lock files naming `Marley` with a dead `pid` are ever removed;
  every path through `*_in(dir)` (§14), so a scenario's `CLAUDE_CONFIG_DIR` keeps it off the
  user's.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `marley.claude_code_ide` is not set, the system shall write no lock file and leave `CLAUDE_CODE_SSE_PORT` out of a new terminal's environment. | Shot `653-01-off` |
| REQ-002 | WHEN the switch is on and the installed Claude Code is in the connection row's range, the system shall listen on a loopback port for each open project and write `<port>.lock`, 0600 in a 0700 directory, holding `pid`, `workspaceFolders`, `ideName` `Marley`, `transport` `ws` and `authToken`. | Shot `653-02-lock` |
| REQ-003 | WHEN a local terminal of that project opens, the system shall set `CLAUDE_CODE_SSE_PORT` to that project's port and `CLAUDE_CODE_AUTO_CONNECT_IDE` to `true` in its environment. | Shot `653-02-lock` |
| REQ-004 | WHEN a server starts, the system shall remove each lock file in the directory whose `ideName` is `Marley` and whose `pid` is no live process, and leave every other. | Shot `653-02-lock` |
| REQ-005 | IF an upgrade request does not carry the lock file's token in `X-Claude-Code-Ide-Authorization`, THEN the system shall answer 401 and upgrade nothing. | Shot `653-03-refused` |
| REQ-006 | WHEN a client with the token asks for the `mcp` subprotocol, the system shall upgrade with `mcp`, answer `initialize`, and list `getDiagnostics`, `getWorkspaceFolders`, `getCurrentSelection`, `getLatestSelection` and `getOpenEditors`. | Shot `653-04-connected` |
| REQ-007 | WHEN the selection in a file editor of the project changes, the system shall send each connection in the selection row's range `selection_changed` with the file's path, the selected text and its 0-based range. | Shot `653-05-selection` |
| REQ-008 | WHEN a file editor holding a caret only becomes the workspace's active item, the system shall send `selection_changed` with its path and an empty selection. | Shot `653-06-open-file` |
| REQ-009 | WHILE an item that is not a file editor holds the focus, the system shall answer `getLatestSelection` with the last file editor's selection. | Shot `653-07-latest` |
| REQ-010 | WHEN `getDiagnostics` is called without a `uri`, the system shall answer every file of the project that has diagnostics, each with its message, severity, 0-based range and source. | Shot `653-08-diagnostics` |
| REQ-011 | WHEN `getDiagnostics` is called with a file's `uri`, the system shall answer that file's diagnostics alone. | Shot `653-08-diagnostics` |
| REQ-012 | WHEN send selection targets a Claude Code whose connection is known and in the mention row's range, the system shall send `at_mentioned` with the file and its 0-based lines to that connection alone, focus its terminal, and type nothing. | Shot `653-09-mention` |
| REQ-013 | WHERE the targeted Claude Code has no known connection, the system shall type `@path#La-b` at its prompt as #549 does. | Shot `653-10-fallback` |
| REQ-014 | IF a connecting client's version is outside the selection row's range, THEN the system shall send it no `selection_changed` and still answer its `getDiagnostics`. | Shot `653-11-old-client` |
| REQ-015 | WHEN the switch turns off, the system shall close the project's connections and remove its lock file. | Shot `653-12-off-again` |
| REQ-016 | WHILE #648's verdict for the installed Claude Code leaves `claude_ide_connection` off, the system shall start no server, write no lock file and set no variable, and #648's chip shall name the IDE link among what is off. | Shot `653-13-untested` |
| REQ-017 | WHEN a project closes or Marley quits, the system shall remove that project's lock file. | The run's closing check; review |
| REQ-018 | WHEN a folder is added to or removed from the project, the system shall rewrite the lock file's `workspaceFolders`. | Review |
| REQ-019 | The system shall bound every read before the token is checked in bytes per line, in header lines and in total time, cap a frame and a message at 1 MiB, and hold at most 8 connections per server. | Review |
| REQ-020 | WHERE a Claude Code version becomes a bound of these rows, each row's doc comment in #648's table shall name the by-hand check it passed on that version: `/ide` names Marley connected, a selection made in Marley shows `⧉ Selected N lines from <file>`, send selection puts the mention into Claude Code's prompt, and an edit proposal still shows in the terminal. | Review of the rows after the check (P3, with Chad) |
| REQ-021 | The change shall pass `script/gates.sh --diff`. | Gate |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes. At promotion: re-read #648 as it shipped
  (its `Integration` rows, its verdict and how a change of it is observed, its chip, its Agent
  Versions section, the harness's allow map) and fit the three rows to it; confirm that
  `lsp.rust-analyzer.binary.path` starts a stand-in in a scenario and what trust it waits on; check
  that Zed restores terminals after the servers start; ask the brain.
- **P2 Code** — `tungstenite` in the workspace; `marley_mcp::ide` and `transport::ide`; the
  per-project ports in `marley_terminal` and their read in `project/src/terminals.rs`;
  `marley_workbench::claude_ide` (lifecycle, lock file, selection, tools, the connection's
  terminal, the rows, holding the box's Claude Code version as the candidate); send selection's
  link route; the switch and its toggle; `script/e2e.sh`'s copy; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot. Then, with Chad's
  go-ahead and in a session of his, the by-hand check on the installed Claude Code (REQ-020). The
  candidate version stays in the rows only if every step passes; otherwise it comes out, the rows
  ship empty, and the gate runs again before the commit.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
