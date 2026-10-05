# Marley as Claude Code's IDE: the link, diagnostics, selection and open file — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-653-marley-as-claude-codes-ide.md
- **Pipeline spec:** 653-marley-as-claude-codes-ide.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-03)
- **Request:** Chad, 2026-10-02: "also we need to brain storm integration into claude and codex
  using their tools instead of fighting them as well." On the design note's B3 (Marley as Claude
  Code's IDE: connection, diagnostics, selection, open file; undocumented parts checked per
  supported version; diffs later): "Yes, with version checks". On B7, which the checks rest on:
  "Your install, with a check". This batch's build order puts #653 last, after #648 (B7) to #652.
- **Classification / tier:** feature, medium. The design note sized selection, open file and
  diagnostics at M; send selection's mention route and the connection-to-terminal match add a
  little. One shippable slice: the second slice of B3 is diffs (`openDiff`), in Out with its own
  scope. If promotion finds it too large, the natural cut is send selection's mention route with
  the connection's terminal (D6, REQ-012, REQ-013) as a slice of its own after the rest. Zed-side
  changes are additive and each widens an existing row: `crates/project/src/terminals.rs` (two
  variables), `crates/settings_content/src/marley.rs` (a field), `assets/settings/default.json`
  (its default), `crates/settings_ui/src/marley_page.rs` (a toggle), the root `Cargo.toml` (one
  `[workspace.dependencies]` line) and `Cargo.lock` (generated). No new external crate:
  `tungstenite` 0.28.0 is already built.
- **Recall (§18.3):**
  - AD-claude-491-marleys-mcp-server-runs-in-the-app-behind-a-stdio-bridge-001: `marley_mcp` in
    the app, loopback, a per-boot bearer, a 0600 file, app answers as deferred calls with the lock
    released. It rejected an environment variable for the endpoint because only Marley's terminals
    would have it; for Claude Code's IDE link that is the point (D2).
  - BF-mcp-pre-auth-body-alloc-001, F-claude-524-the-mcp-servers-read-before-auth-had-no-bound-001,
    F-claude-524-a-431-closed-with-the-request-unread-001 and
    PR-claude-bound-every-read-before-auth-in-size-and-time-001: the upgrade request is read by the
    same bounded reader, a refusal drains before closing, and frames are capped (D3, REQ-019).
  - L-claude-524-a-restarted-mcp-server-listens-on-a-new-port-001: each start is a new port, so a
    new lock file, and a crash leaves a stale one; hence the sweep (REQ-004).
  - The completed #549 (spec and notes): the typed route, its D1 to D3, the picker, the refusal
    while a seat waits, `send` at `send_selection.rs:350-358`; this ticket keeps all of it as the
    fallback and puts the link route inside `send`.
  - F-claude-594-an-enter-sent-with-a-paste-was-read-as-part-of-it-001: typing into an agent needs
    `paste_then`'s 200 ms wait; a mention types nothing (D6).
  - F-claude-596-a-lookup-inside-a-window-update-lost-that-windows-terminals-001: the connection's
    terminal is found across windows before any window is updated.
  - AD-claude-633-rustys-server-is-offered-where-installed-as-a-default-001 and
    L-claude-633-scenarios-copy-the-users-settings-so-defaults-reach-real-services-001: a switch
    observed through the settings store; the runner turns it off in each copy.
  - F-claude-547-a-scenarios-click-ran-the-real-claude-001: the scenario names its stand-in with
    `MARLEY_CLAUDE` and keeps a scratch `CLAUDE_CONFIG_DIR`, so neither Marley nor a terminal
    reaches the user's Claude Code or its `ide` directory.
  - L-claude-619-a-scenario-that-sets-terminal-settings-cannot-use-terminal-env-001: the scenario
    sets no other terminal setting, so `terminal_env` stays usable.
  - AD-claude-587-marley-brings-claude-codes-trust-question-to-the-user-001: the real Claude Code
    asks its trust question before anything runs; the stand-in avoids it, and the by-hand check
    (REQ-020) answers it in Chad's own session.
  - AD-claude-583-chromium-on-its-pipe-behind-marleys-relay-001: Marley's one WebSocket server so
    far, a token minted per start and kept in a 0600 file, the shape of the lock file's token.
  - Completed pipelines read: 549, 633 (spec and notes), and the shape of 643's queued notes.
    Queued #648 (drafted beside this one, its spec read on 2026-10-03): the table in
    `marley_agent::versions`, its verdicts, its chip, its allow map and its D3 range policy, which
    the three rows here follow; its Out asks B3 for its rows, Settings items and allow keys.
  - Brain: `rusty-cli brain search "Claude Code IDE lock file websocket"` found nothing on this
    seam.

### How the protocol was found
What the docs name, and what they do not, as of 2026-10-03:

| Part | Where it is stated | Row in #648's table |
|---|---|---|
| Server on `127.0.0.1`, unencrypted `ws://`, port not configurable | docs: vs-code, jetbrains | none needed |
| Lock file at `~/.claude/ide/<port>.lock`, or `$CLAUDE_CONFIG_DIR/ide/`; 0600 in 0700 | docs: vs-code | none needed |
| A fresh token per start, sent as `X-Claude-Code-Ide-Authorization` | docs: vs-code, jetbrains | none needed |
| `mcp__ide__getDiagnostics`, optionally for one file | docs: vs-code, jetbrains | none needed |
| Selection and active file's path sent with each prompt | docs (the behavior, not the wire) | none needed |
| `CLAUDE_CODE_AUTO_CONNECT_IDE`, `--ide`, `/ide`, `autoConnectIde` | docs: env-vars, cli-reference, commands, settings-reference | none needed |
| Lock fields `pid`, `workspaceFolders`, `ideName`, `transport`, `authToken`, `runningInWindows` | claudecode.nvim `PROTOCOL.md`; monet; #79806 | `claude_ide_connection` |
| `CLAUDE_CODE_SSE_PORT` picks the lock file | claudecode.nvim, monet, #96742, #67502 | `claude_ide_connection` |
| `Sec-WebSocket-Protocol: mcp`, required in the answer | claude-code-zed `REVERSE.md` (1.0.44) | `claude_ide_connection` |
| `ide_connected` with Claude Code's `pid` | obsidian-claude-code-mcp; monet | `claude_ide_connection` |
| `getWorkspaceFolders` and its answer | claudecode.nvim | `claude_ide_connection` |
| `selection_changed` (selection, and the open file with an empty one) | claudecode.nvim; claude-code-ide.el's README | `claude_ide_selection` |
| `getCurrentSelection`, `getLatestSelection`, `getOpenEditors` | claudecode.nvim | `claude_ide_selection` |
| `at_mentioned` with 0-based `lineStart`, `lineEnd` | claudecode.nvim, monet | `claude_ide_mention` |
| The client's version: `User-Agent: claude-code/<v>`, `initialize`'s `clientInfo` | claude-code-zed's capture; the MCP spec | read only, no row |

So the answer to "notifications or internal tools" is both: the editor pushes `selection_changed`
and `at_mentioned`, and the CLI can also pull with `getCurrentSelection` and `getLatestSelection`.
The docs' own wording ("internal RPC … reading selections") matches the pull half.

**How Claude Code picks an IDE.** From the docs: it connects at start when launched inside a
supported IDE's integrated terminal, or when `CLAUDE_CODE_AUTO_CONNECT_IDE=true` (documented as the
way to force an attempt when detection fails), or with `--ide` when exactly one valid IDE is
available, or with `autoConnectIde` from an external terminal; otherwise `/ide` lists the IDEs and
connects to one. A lock entry is "validated" (`CLAUDE_CODE_IDE_SKIP_VALID_CHECK`). From public
implementations and issues: `CLAUDE_CODE_SSE_PORT` in the terminal's environment names the lock
file to use (VS Code sets it for its own terminals, the JetBrains plugin per project); the
validation compares `workspaceFolders` with the cwd, strictly enough that a subdirectory failed on
2.1.216 under WSL (#79806); `pid` presumably names a live IDE process. `ENABLE_IDE_INTEGRATION=true`
is set by the JetBrains plugin, claudecode.nvim and monet, but a string search of 2.1.288 finds no
such name. **How Marley gets the right window:** one server per project, its port in that
project's terminals with the documented variable to ask for the connection (D2), and
`workspaceFolders` naming the project's own folders, so `/ide` from elsewhere lists only the
projects whose folders hold the cwd.

- **Discovery:**
  - `crates/marley_mcp/src/transport.rs`: `spawn` `:182`, `TcpListener::bind("127.0.0.1:0")`
    `:187`, one accept thread `:200-204` and one per connection `:224-240`, `serve_connection`
    `:244-323` (bounds, Origin, bearer, session, dispatch), the bounds `:611-630` (1 MiB body,
    8 KiB lines, 100 headers, 10 s), `ask_app` `:462-481`, `read_entropy` `:162`. No WebSocket code.
  - `crates/marley_mcp/src/marley_mcp.rs`: `MCP_PROTOCOL_VERSION` `:91`, `APP_CALL_TIMEOUT_SECONDS`
    `:94`, `Outgoing` `:126`, `AppCall` `:167`, `AppCaller` `:255`. `jsonrpc.rs` `:55`, `:66`,
    `:74`, `:82`; `dispatch.rs` `:19`, `:41-59`, `:107-113`; `auth.rs` `:10`, `:40`, `:53`;
    `secret.rs` `:11`, `:44`; `discovery.rs` `:24`, `:38`, `:59`, `:69`. `Cargo.toml`: MIT OR
    Apache-2.0, Rustal's lint table, no async runtime.
  - `crates/marley_workbench/src/mcp.rs`: `start` `:75-160` (from `crates/zed/src/main.rs:900`),
    `write_endpoint` `:365-380`, the app's tool match `:470-505`, `terminals` `:719-747`.
  - `crates/marley_workbench/src/send_selection.rs`: `init` `:100-132`, `send_selection`
    `:150-197`, `selection_of` `:201-219`, `line_span` `:223-234`, `reference` `:239-263`,
    `agent_targets` `:268-301`, `target_of` `:331-347` (the seat by view id, `waiting`), `send`
    `:350-358` (the selection's own path; `english.rs:279`, `:290` and `send_block.rs:102`, `:113`
    call `send_text` and stay as they are), `send_text` `:363-410` (rich input `:373-379`, the
    refusal `:380-390`, the paste `:405`).
  - `crates/marley_workbench/src/claude_plugin.rs:111-117` (`CLAUDE_CONFIG_DIR` or `~/.claude`),
    `:122` (`MARLEY_CLAUDE`); `agent_bar.rs:140-145` (`agent_in`); `agent_events.rs:68-72`
    (`seat`), `:112-116` (`waiting`).
  - `crates/project/src/terminals.rs`: `create_terminal_shell_internal` `:343-`, the environment
    merged at `:417-442` (ports `:420`, `settings.env` `:421`, `marley_extra_env` `:423`,
    `MARLEY_PROJECT` `:427-433`, the restored id `:437-442`); the project's entity id as the
    builder's `window_id` `:260`, `:484`. Row `docs/marley/zed-touchpoints.md:81`.
  - `crates/terminal/src/terminal.rs`: `insert_zed_terminal_env` `:752-761` (`TERM_PROGRAM=zed`),
    Marley's variables `:1289-1343`, `pid` `:3627`, `pid_getter` `:3634`;
    `crates/terminal/src/pty_info.rs:26` (`fallback_pid`), `:33-39` (`tcgetpgrp`).
  - `crates/marley_terminal/src/shell_integration.rs:142-162` (`BROWSER_OPENER`, set from
    `mcp.rs:343-348`); `ports.rs:52` (`variables`).
  - `crates/marley_agent/src/stall.rs:17-18` (`procfs_core::process::Stat` over `/proc`).
  - Zed: `Workspace::active_item` `workspace.rs:4392`, `Event::ActiveItemChanged` `:1552`;
    `EditorEvent::SelectionsChanged` `editor.rs:12451`, `selections` `:976`,
    `selections_collection.rs:309` (`newest`); `cursor_position.rs:268-292`;
    `Project::diagnostic_summaries` `project.rs:5237`, `Event::DiagnosticsUpdated` `:385`,
    `open_buffer` `:3276`, `absolute_path` `:5263`, `project_path_for_absolute_path` `:5372`;
    `buffer.rs:5431` (`diagnostic_groups`), `:5356` (`diagnostics_in_range`);
    `diagnostics_tool.rs:92-257`; `MultiWorkspace` `multi_workspace.rs:318`, `workspaces()`
    `:1300`; `lsp_store.rs:725` (a `binary.path` setting starts that program for any adapter, after
    worktree trust); `default.json:2830` (`session.trust_all_worktrees`).
  - Settings: `settings_content/src/marley.rs:88-92` (`rusty_tools`, the pattern), `default.json`
    `:1664` (the `marley` block), `marley_workbench.rs:342`, `:544-`, `:585`; `marley_page.rs`
    `agents_section` `:112`, the Rusty toggle `:344-364`; `script/e2e.sh:625-642` (the copy and
    its Python edit).
  - `Cargo.toml:578` (`async-tungstenite = "0.33"`); `Cargo.lock:19658` (`tungstenite` 0.28.0:
    `data-encoding`, `http`, `httparse`, `sha1`); `marley_browser/src/relay.rs:41`, `:124`,
    `:291`, `:458`, `:473-517`, `:531`.
  - The installed tool: `claude --version` 2.1.288; `~/.claude/ide/` exists, 0700, empty.
- **Decisions:** D1 to D10 in the spec. The ones with a road not taken: D1 (`marley_mcp`, not a
  smol server in the workbench), D2 (per project, not per window or per terminal: a server per
  terminal, as claude-code-ide.el runs one per instance, would put a lock file for every shell into
  every other Claude Code's `/ide`), D3 (`tungstenite`, not hand-written frames), D6 (the refusal
  kept on the mention route until checked).

### Design
- **`marley_mcp::ide`** (pure, no IO): `IdeRequest` parsing of the upgrade's headers into what the
  checks need (`Host`, `Origin`, the token header, `Upgrade`, `Sec-WebSocket-Version: 13`,
  `Sec-WebSocket-Key`, the subprotocols, `User-Agent`); `admit(&request, &expected) ->
  Result<Accept, Refusal>` with `Refusal::{Unauthorized, Forbidden, BadRequest}`; the lock file's
  JSON (`LockFile { pid, workspace_folders, ide_name, transport, auth_token,
  running_in_windows }` serialized with the wire's camelCase names); `client_version(user_agent,
  client_info) -> Option<Version>`; `handle(state, message) -> IdeHandled` over `initialize`,
  `notifications/initialized`, `ide_connected`, `tools/list` (the tools the client's rows allow),
  `tools/call` (each listed tool as `Outgoing::Deferred`, every other an MCP tool error naming
  it), `prompts/list`, `ping`; builders for `selection_changed` and `at_mentioned`. The JSON-RPC
  parse and reply builders are `jsonrpc.rs`'s.
- **`marley_mcp::transport::ide`** (`transport/ide.rs`, declared from `transport.rs`):
  `spawn_ide(caller: AppCaller, events: Sender<IdeEvent>) -> io::Result<IdeServerHandle>`; the
  listener on `127.0.0.1:0`; an accept thread refusing a ninth connection; per connection, the
  upgrade read with the transport's bounded reader and its 10 s deadline, `admit`, a 101 with
  `derive_accept_key` and `Sec-WebSocket-Protocol: mcp` (a refusal written, then drained, as #524
  does), then `WebSocket::from_raw_socket(stream, Role::Server, config)` with 1 MiB caps; a loop
  that reads with a 50 ms timeout, hands each message to `ide::handle`, sends deferred calls through
  `ask_app` and writes the connection's queue between reads; a ping every 30 s. `IdeServerHandle`:
  `port()`, `token()`, `notify_all(method, params, row)`, `notify(connection, method, params)`,
  `close_all()`, and `Drop` closing the listener. `IdeEvent::{Connected { connection, version },
  Identified { connection, pid }, Closed { connection }}` go to the app.
- **`marley_terminal::ide`** (new, small): the variable names (`CLAUDE_CODE_SSE_PORT`,
  `CLAUDE_CODE_AUTO_CONNECT_IDE`) and a `RwLock<HashMap<u64, u16>>` of project entity id to port,
  with `set_port(project, Option<u16>)` and `variables(project) -> Vec<(String, String)>`, the
  shape of `shell_integration`'s `BROWSER_OPENER`, since `project` depends on `marley_terminal`
  and not on the workbench.
- **`project/src/terminals.rs`** (Zed): in `create_terminal_shell_internal`, for a local terminal
  of a local project, `marley_terminal::ide::variables(cx.entity_id().as_u64())` read before the
  future and extended into `env` after the port variables and before `settings.env`, so the
  user's `terminal.env` wins; nothing for a remote terminal. `create_terminal_task` is left alone.
- **`marley_workbench::claude_ide`** (new):
  - `init`: a `SettingsStore` observer and an observer of each window's `MultiWorkspace` (its
    `WorkspaceAdded` and `WorkspaceRemoved`), as `rusty.rs` does for its switch. For each project
    with the switch on and #648's verdict for `claude_ide_connection` `On` or `Allowed`: sweep
    stale lock files, `spawn_ide`, write the lock file into the config directory's `ide` folder,
    `set_port`; on off, a verdict that turns the row off, close, or quit (`on_app_quit`):
    `close_all`, remove the file, `set_port(None)`. A verdict's change is observed the way #648
    lets a gate observe it (its windows refresh on a changed verdict, D6 there).
  - The lock file goes through `discovery`'s 0600 writer, by a temporary name and a rename, into a
    directory taken as a parameter (`write_lock_in(dir, …)`), the directory from
    `claude_plugin`'s `config_dir`; a project's `Event::WorktreeAdded` or `WorktreeRemoved`
    rewrites it.
  - Selection: per workspace, a subscription to `ActiveItemChanged` that, when the item is a file
    editor (`act_as::<Editor>` with a local file), moves a `SelectionsChanged` subscription to it
    and keeps it as the last file editor; a 100 ms debounce on the background executor; then
    `notify_all("selection_changed", …, selection row)`. On `Identified`, the current one to that
    connection.
  - Tools: `getDiagnostics` from `Project::diagnostic_summaries`, `open_buffer` for a named file
    not open, and `diagnostic_groups(None)`' primary entries, ranges as UTF-16 points;
    `getCurrentSelection` and `getLatestSelection` from the last file editor;
    `getOpenEditors` from the workspace's file editors; `getWorkspaceFolders` from the lock file's
    folders. Each answer is a text item holding JSON, as `marley_mcp::ToolAnswer` carries it.
  - The connection's terminal: on `Identified { pid }`, the terminals of every window found first
    (F-claude-596), then the one whose `Terminal::pid()` is the pid, else whose
    `pid_getter().fallback_pid()` is an ancestor (parents read from `/proc/<pid>/stat` through
    `procfs_core`, at most 32 steps). Kept as connection to terminal view id; dropped on `Closed`.
  - `link_for(view, cx) -> Option<(IdeServerHandle, ConnectionId)>`, for send selection.
  - A client's own version, from `IdeEvent::Connected`, judged with #648's range test and its
    allow map against `claude_ide_selection` and `claude_ide_mention`; the answer is kept per
    connection and logged (`claude ide: client 2.1.288 on port <n>: selection on, mention on`).
- **`send_selection.rs`**: in `send`, before `send_text`: when the target is Claude Code, rich
  input is not open on it, the seat does not wait, and `claude_ide::link_for` answers, `notify`
  that connection with `at_mentioned` (the absolute path, `lines` minus one, none for a caret),
  then activate, reveal and focus as `send_text` does; otherwise `send_text` unchanged.
- **The rows** (`marley_agent::versions`, #648): `CLAUDE_IDE_CONNECTION`, `CLAUDE_IDE_SELECTION`
  and `CLAUDE_IDE_MENTION` in `INTEGRATIONS`, agent Claude, each `rests_on` naming its parts, each
  doc comment naming the by-hand check and the version it passed on; their items in the Agent
  Versions section (`marley_page.rs`): IDE Link, IDE Selection and IDE Mentions on Untested Claude
  Code.
- **The switch**: `marley.claude_code_ide: Option<bool>` (`settings_content/src/marley.rs`),
  `"claude_code_ide": false` (`default.json`), `MarleySettings::claude_code_ide`, and a "Claude Code
  IDE Link" toggle in the Agents section (`marley_page.rs`), its description naming the lock file
  and `/ide`.
- **`script/e2e.sh`**: `settings["marley"]["claude_code_ide"] = False` beside `rusty_tools` in the
  copy's Python edit, and the three ids added to the `allow_untested_versions` map #648 writes
  there.

### File manifest
- Marley: `crates/marley_mcp/src/ide.rs` (new), `crates/marley_mcp/src/transport/ide.rs` (new),
  `crates/marley_mcp/src/transport.rs` (the module, the bounded reader shared with it),
  `crates/marley_mcp/src/marley_mcp.rs` (the module and its exports),
  `crates/marley_mcp/src/discovery.rs` (the lock writer), `crates/marley_mcp/Cargo.toml`
  (`tungstenite`); `crates/marley_terminal/src/ide.rs` (new), `marley_terminal.rs`;
  `crates/marley_workbench/src/claude_ide.rs` (new), `send_selection.rs`, `marley_workbench.rs`
  (the setting, `init`), `crates/marley_workbench/Cargo.toml` (`procfs-core`, if not already
  reached through `marley_agent`); `crates/marley_agent/src/versions.rs` (#648's table: three
  rows); `script/e2e.sh`; `script/e2e/653-marley-as-claude-codes-ide.sh` (new) with its stand-ins
  under `script/e2e/`.
- Zed, each extending its row in `docs/marley/zed-touchpoints.md`:
  - `crates/project/src/terminals.rs` (row `:81`): the IDE variables for a local terminal of a
    local project, after the port variables and before `settings.env`. Why: the project is where a
    terminal's environment is final and knows its own entity. Merge: keep them before
    `settings.env`, local terminals only.
  - `crates/settings_content/src/marley.rs` (row `:59`): `claude_code_ide: Option<bool>`.
  - `assets/settings/default.json` (row `:67`): `"claude_code_ide": false` in the `marley` block.
  - `crates/settings_ui/src/marley_page.rs` (row `:63`): the toggle in the Agents section, and
    the three rows' items in #648's Agent Versions section.
  - `Cargo.toml` (row `:47`): `tungstenite = { version = "0.28", default-features = false,
    features = ["handshake"] }` in `[workspace.dependencies]`, the version the lock already holds,
    as `procfs-core` (#503). Merge: keep it at the version `async-tungstenite` takes.
  - `Cargo.lock` (row `:48`): `marley_mcp`'s new edge; generated.
  - `crates/project/Cargo.toml` (row `:82`): already depends on `marley_terminal`; no change
    expected.

### Visual check plan
The scenario, its setup and stand-ins are in the spec's UI proof. One row per criterion:

| REQ | What the scenario does | Shot |
|---|---|---|
| REQ-001 | The runner's copy (switch off); `claude` in the first terminal | `653-01-off` |
| REQ-002, REQ-003, REQ-004 | `set_setting marley.claude_code_ide true`; settle; a new terminal; `claude` | `653-02-lock` |
| REQ-005 | `badtoken` typed at the stand-in | `653-03-refused` |
| REQ-006 | The stand-in's connection report | `653-04-connected` |
| REQ-007 | The editor focused; `src/auth.txt` opened; lines 2 to 4 selected; settle 1 | `653-05-selection` |
| REQ-008 | `src/main.rs` opened through the file finder; a caret only | `653-06-open-file` |
| REQ-009 | The terminal focused; `latest` | `653-07-latest` |
| REQ-010, REQ-011 | `diag`; `diag src/auth.txt` | `653-08-diagnostics` |
| REQ-012 | `src/auth.txt`, lines 2 to 4; `ctrl->` | `653-09-mention` |
| REQ-013 | A second terminal, `claude --offline`; `ctrl->`; `down`, Return in the picker | `653-10-fallback` |
| REQ-014 | A third terminal, `claude --as 9.9.9`; a new selection; `diag` | `653-11-old-client` |
| REQ-015 | `set_setting marley.claude_code_ide false`; settle; `ls` in a shell terminal | `653-12-off-again` |
| REQ-016 | `9.9.9` into the version file; the switch on; `quit_marley`, `launch_marley`; a new terminal, `claude`; `pointer_to` #648's chip | `653-13-untested` |
| REQ-017 | `quit_marley`; `expect` the `ide` directory holds `2.lock` alone | the run's output |

Not reached by the scenario, and why: the real Claude Code (no agent turn and no user session in
a scenario; REQ-020's by-hand check covers it, with Chad); a project closed while Marley runs
(REQ-017's other half, review); a folder added to a project (REQ-018, review); the bounds and caps
(REQ-019, review); two projects in one window each with its own port (the design's keying by
project entity, review; the Test phase adds a shot if a second project opens in the run).

### Risks
- **`openDiff` with no `openDiff` listed.** With an IDE connected and `diffTool` at its default
  `auto`, Claude Code opens an edit's diff in the IDE. Whether it falls back to the terminal when
  the IDE lists no `openDiff` is unknown (claude-code-zed stubs every tool; claudecode.nvim
  implements it). An edit that waits on a missing diff would stop a session. REQ-020's by-hand
  check includes one edit proposal; until it passes, the connection row holds no version and the
  link stays off on a real install. If it fails, `openDiff` moves into this ticket or the link
  waits for the diffs slice.
- **The automatic link may need more than the documented variable.** claudecode.nvim also sets
  `FORCE_CODE_TERMINAL=true`; if the by-hand check shows no connection at start, Marley adds it
  under the connection row (D2).
- **`workspaceFolders` against the cwd.** #79806 shows a subdirectory refused on 2.1.216 under
  WSL; Claude Code in `src/` of a Marley project may not connect. The by-hand check runs once from
  the root and once from a subdirectory and records both; the guide says what it found.
- **Restored terminals.** Zed restores a project's terminals when the window opens; if they spawn
  before the project's server starts, they miss the variables (they can still use `/ide`).
  Promotion checks the order; the server starts in the project's first observer, before the
  terminal panel loads, if the order allows.
- **Two config directories.** Marley writes into the `ide` folder of the `CLAUDE_CONFIG_DIR` it was
  started with; a shell profile that sets another one for its terminals sends Claude Code to a
  folder with no Marley lock. The guide names it.
- **Proxies.** claudecode.nvim's issue #70: with `http_proxy` set and no loopback in `no_proxy`,
  Claude Code sent its `ws://127.0.0.1` connection through the proxy. Marley does not edit the
  user's proxy variables; the guide names the fix.
- **The client's version.** Neither `clientInfo` nor `User-Agent` is documented; if 2.1.288 sends
  neither, the per-client check falls back to the installed version (#648) and the by-hand check
  records which one it saw.
- **`tungstenite` with read timeouts.** A read that times out returns `WouldBlock` with any partial
  frame kept in its buffer; the Code phase confirms 0.28 does so, else the reader runs on a
  non-blocking socket with a poll.
- **Privacy.** Any process of the user's that can read the 0600 file can connect and receive the
  selection's text, which is the model Claude Code's docs describe. Off by default; the toggle's
  description and the guide say so.
- **#648 not landed.** Its draft (read 2026-10-03: `Integration` rows with `from` and `before`,
  verdicts `On`, `Allowed`, `Off`, the agent bar's chip, `marley.allow_untested_versions`, the
  harness's allow map) is what this plan is written against. Promotion re-reads it as shipped. Its
  chip draws under a terminal whose foreground agent has a row off, so REQ-016's shot needs the
  stand-in running when the pointer reaches the chip. Until #648's first check ends its rows are
  off, so the link starts a moment after Marley does, and a terminal restored in that moment
  misses the variables (the next risk).
- **The scenario's stand-in language server.** `lsp.rust-analyzer.binary.path` starts a program
  for any adapter once the worktree is trusted (`lsp_store.rs:725`); if the Rust adapter checks
  for a toolchain first, the scenario names another built-in adapter.
- **The receipt.** The stand-ins and the scenario are fingerprinted (`script/e2e*`); a change after
  the Code phase's green needs `--diff` again before the commit.

### User-facing docs to update at Complete (P4)
- `docs/marley/guide.md`, "Agent CLIs in terminals" (`:574`): a part on Claude Code's IDE link
  (the switch, which terminals get it, `/ide` and `--ide` from elsewhere, what Claude Code
  receives, the versions it is checked on and what turns off outside them, the subdirectory,
  config directory and proxy notes); the guide has no send selection article today, so the mention
  route goes into the same part.
- `docs/marley/walkthrough.md` §5.12 (`:889`, send selection): the mention route when the link is
  up; the key tables `:1570`, `:1591` unchanged.
- Architecture (§21): `docs/marley_architecture/marley_mcp.md` (a section on the IDE server beside
  "The transport" `:128` and "In the app" `:188`); `docs/marley_architecture/marley_workbench.md`
  (a `claude_ide` section; send selection's `:1124` part gains the route);
  `docs/marley/three-prong-plan.md`, prong 2 (B3's first slice shipped); the design note's B3 line;
  the zed-touchpoints rows checked against what shipped; `CHANGELOG.md` (Added: Marley as Claude
  Code's IDE, off by default).

### Checklist (no TaskCreate in this harness)
- [x] Read the brief (both parts), CONSTITUTION §3, §7, §14, §18, §19, §20, and the ticket and
      pipeline templates.
- [x] Read the design note in full: the fights table, the tools, B1 to B7, Chad's answers and the
      harness's side.
- [x] Read Claude Code's docs: vs-code (the IDE MCP server section), jetbrains, env-vars,
      settings-reference, cli-reference, commands, agent-view; `claude --version` and `--help`.
- [x] Read the public implementations for facts, with licences: claudecode.nvim (MIT), monet (MIT),
      obsidian-claude-code-mcp (0BSD), claude-code-zed (MIT), claude-code-ide.el (GPL-3.0, README
      only); no leaked Claude Code source opened; anthropics/claude-code #79806, #96742, #98858,
      #67502, #84462.
- [x] Recall: the knowledge ledgers (AD-491, AD-583, AD-587, AD-633, BF-mcp, F-524 twice, PR-524,
      L-524, F-547, F-594, F-596, L-619, L-633), completed 549 and 633, a read-only brain search.
- [x] Discovery with file:line, through an Explore pass and direct reads: `marley_mcp`, `mcp.rs`,
      `send_selection.rs`, the terminal environment in `project` and `terminal`, the settings and
      the Marley page, Zed's selection and diagnostics APIs, `Cargo.lock`'s WebSocket crates, the
      relay, the e2e runner and #549's scenario, the touchpoint rows.
- [x] Decided where the server lives (D1) and how a terminal reaches its own project's server
      (D2).
- [x] Spec: scope, Reference (§20), Prior art, UI proof, D1 to D10, twenty-one EARS rows, phase
      plan.
- [x] Design: approach, file manifest by crate, the touchpoint rows widened, the visual check plan,
      risks, the docs for Complete.
- [x] Docs only: the ticket doc and this pair; no BACKLOG.md, no `active/`, no source, no cargo.

### Promotion (2026-10-04)
- Promoted into `active/`; the BACKLOG row removed; the ticket in-progress. Pre-flight green, no
  other active pipeline; #648 to #652 landed (the last 0187ee407a).
- **Brain:** `brain ask` (consultation `a4c7036a0a7d40e5aeb2a0a7fb9c510a`) returned due follow-ups
  on other work only.
- **#648 as shipped** (an Explore pass, read only):
  - `Integration` is `{ id, agent, name, off_means, setting, tested: Range }`; there is no
    `rests_on` field, so each row's doc comment names the parts it rests on, as the two rows do.
    `INTEGRATIONS` is a `static [Integration; 2]` and becomes `[_; 5]`.
  - `verdict(integration, found, allowed)` gives `On` in range, `Allowed` when the id is allowed
    (even before the first check ends), else `Off`. `agent_versions::is_on` judges the installed
    version; `codex_server::server_tested` judges a live peer's version with `verdict` directly,
    which is the model for a client's own version here.
  - Nothing publishes a change of verdict: the check runs at `init` and when an agent bar draws
    after 10 s, and `store` refreshes windows when verdicts move. The link needs to follow it, so
    `agent_versions` gains an observer hook over its global.
  - The chip lists the rows `wanted` passes; `wanted` hard-codes #650's switch, and gains the
    same clause for the three IDE rows (shown only while `marley.claude_code_ide` is on). The
    tooltip's words come from each row's `name`, `off_means` and `setting`.
  - The Agent Versions items are hand-written, one per row (`[SettingsPageItem; 3]` becomes 6),
    each title equal to its row's `setting`. The toggle follows "Codex App Server" in the Agents
    section (`[SettingsPageItem; 16]` becomes 17).
  - `allow_untested_versions` is `Option<BTreeMap<String, bool>>`; `default.json` lists each id
    as `false` so the Settings toggle reads a default, and gains the three ids.
  - `MarleySettings` already holds three plain bools, so the switch is an enum
    (`ClaudeCodeIde { Off, On }` with `from_content`), as `CodexAppServer` is; `from_settings` is
    at the 100-line cap, so the field is one line.
  - `script/e2e.sh` sets `codex_app_server` false and leaves its id out of the allow map, so a
    scenario that turns the feature on meets the check; #653 does the same with
    `claude_code_ide`.
- **Zed's seams** (an Explore pass, read only):
  - `create_terminal_shell_internal` (`terminals.rs:343`) runs in `Context<Project>`; the
    project's entity id is `cx.entity_id()` (the builder's `window_id`, :484). The variables are
    read synchronously beside `first_project_directory` (:355) and merged after the ports (:420)
    and before `settings.env` (:421). `create_terminal_task` (:64) is not touched, and the
    builder empties nothing new: the variables are added only for a local project, and a remote
    one (`remote_client`) gets none.
  - Order: Marley's `observe_new::<Workspace>` runs before any restored terminal spawns its
    shell (`Workspace::new_local` opens items in a later update; `marley_workbench::init` registers
    before zed's panel loader). A server bound synchronously there is in place for restored
    terminals, once the version verdict allows it; before #648's first check ends the verdict is
    `NotChecked`, so restored terminals of a session where the check is slower than the login
    shell's environment miss the variables (the risk stands).
  - `lsp.rust-analyzer.binary.path` starts the given program with no toolchain check
    (`lsp_store.rs:725`), after worktree trust, which `session.trust_all_worktrees: true` grants.
    No scenario has a stand-in language server yet.
  - `send_selection::send` (:350) is where the link route goes; the kind comes from
    `agent_in`; rich input (:373) and the waiting refusal (:380) stay first, inside `send_text`,
    so the link route repeats the two checks before it sends.
  - `tungstenite` 0.28.0 is in the lock (from `async-tungstenite` 0.33); `handshake` is its
    default feature and its four crates are locked; `derive_accept_key` is public;
    `from_partially_read(stream, bytes, Role::Server, config)` hands over what the header read
    took past the blank line. A read timeout returns `Io(WouldBlock)` and the next `read` resumes
    the frame.
  - `read_http_request` keeps no header map, so the IDE transport reads the upgrade with its own
    function over the same `Deadlined` reader, `read_bounded_line` and limits (made
    `pub(crate)`). `write_endpoint_file_in` writes in place, so the lock file is written to a
    temporary name with it and renamed.
  - The connection's terminal reuses #652's process walk (`report::stat_fields`, the chain read
    in `agent_reports`), so the workbench needs no `procfs-core`.
- **Amended at promotion:**
  - **D4: the rows ship with no tested version.** REQ-020's by-hand check needs a real Claude
    Code in Chad's session, and this batch runs none (R-D8), so the three rows carry an empty range
    (`before` equal to `from`, which `Range::words` reads as "no version yet") and only
    `marley.allow_untested_versions` turns the link on. The check, and the bounds it gives, are a
    question for Chad at the end of the batch; REQ-020 applies then.
  - REQ-014 and REQ-016 are shown through the allow map: removing an id turns its part off for the
    install and for every client. The per-client range test with a non-empty range is review only
    until the rows have bounds.
  - The scenario allows the three ids itself; `script/e2e.sh` writes only the switch off.
  - The unit of a server is the workspace's project (one per `Workspace`), followed by
    `observe_new::<Workspace>` and the project's release, not `MultiWorkspace`'s events.
- **The manifest, as amended:** add `crates/marley_workbench/src/agent_versions.rs` (the observer
  hook and the `wanted` clause), `crates/marley_agent/src/versions.rs` (`Range::words` for an
  empty range, the three rows), `crates/marley_workbench/src/agent_reports.rs` (its process chain
  shared with `claude_ide`), and `assets/settings/default.json`'s allow map (the three ids,
  `false`). Drop `crates/marley_workbench/Cargo.toml`'s `procfs-core`. The IDE transport lives
  in `crates/marley_mcp/src/ide_transport.rs` beside `transport.rs`, whose reader pieces become
  `pub(crate)`, so no `transport/` folder appears.
- **Phase 1 closeout:** spec status set to PASS; `## Reference (§20)`, `### Prior art` and
  `## UI proof` filled; the work runs autonomously under the batch's goal, so no review wait.

## Phase 2 — Code (2026-10-04)
- **Built.**
  - `marley_mcp::ide` (pure): the lock file's JSON and the stale-lock test; `Upgrade` and
    `admit` (a WebSocket upgrade, the server's own loopback host, a loopback `Origin` if any, the
    token in constant time; 400, 403, 401); the client's version from `User-Agent`; `Parts`;
    `Tool`; `step` over `initialize` (the client's protocol version echoed, its `clientInfo`
    version read), `ide_connected`, `tools/list` (the selection tools only with the selection
    part), `tools/call` (any other tool a tool error naming it), `prompts/list`, `ping`; the
    notifications and the five answers; `file_url` and `path_of_uri`.
  - `marley_mcp::ide_transport`: `spawn_ide(judge, handler)`, a listener on `127.0.0.1:0` and a
    thread per client, at most 8; the upgrade read through the HTTP transport's bounded reader
    (its pieces made `pub(crate)`), the 101 with `derive_accept_key` and `mcp`, then
    `tungstenite`'s `from_partially_read` with 1 MiB caps; each client's loop reads in 50 ms
    slices, writes the app's queue, the answered calls (30 s each) and a ping every 30 s, and
    drops a client that misses two pongs. Dropping `IdeServer` closes every client and wakes the
    listener to end it.
  - `marley_terminal::ide`: the two variable names and the port per project entity id.
  - `project` (Zed): the variables read by the project's entity id beside `MARLEY_PROJECT`'s
    folder, for a local project, and added after the ports and before the settings' `env`.
  - `marley_agent::versions`: the three rows with an empty range, which `Range::words` reads as
    "no version yet"; `INTEGRATIONS` of five.
  - `marley_workbench::claude_ide`: `ClaudeCodeIde`; the global of local workspaces and served
    projects; `reconcile` (the switch and the connection row's verdict, from the settings, #648's
    checks and the workspaces); `start` (the server, the port, the lock file written off the main
    thread after the stale sweep, through a temporary name), `stop`, `refolder`; the judge state
    the clients' threads read; the main-thread intake (versions logged per client, a client's
    terminal from its process chain, the selection sent once it named its process); the last
    file editor followed from `ActiveItemChanged`, its `SelectionsChanged` settled 100 ms; the
    five answers, `getDiagnostics` opening each file and reading its groups' primary entries;
    `mention` for send selection. `agent_versions` gained `observe`, `found`, and the IDE rows in
    `wanted`; `agent_reports` shares `process_chain` and `terminal_on_chain`.
  - `send_selection::send`: a Claude Code target that does not wait, without rich input open,
    whose link takes mentions gets `at_mentioned` and the focus; every other case types as #549
    does.
  - The switch (`settings_content`, `default.json` with the three ids, the Marley page's toggle
    and three Agent Versions items, `MarleySettings`), and `script/e2e.sh`'s copy with the switch
    off.
- **Deviations from the plan, and why.**
  - The rows ship with no tested version (D4, amended at promotion): the by-hand check waits on
    Chad.
  - The transport is `ide_transport.rs` beside `transport.rs`, not `transport/ide.rs`.
  - The server's threads judge a client's parts through a closure over a shared state the main
    thread refreshes (the allowed rows and the installed version), so `tools/list` answers right
    after `initialize` without a round trip to the app.
  - `getDiagnostics` without a file lists the files with errors or warnings: Zed's summaries count
    only those two.
  - A selection equal to the one sent last is not sent again (found in Test, below).
- **Review.**
  - Safety in Claude Code's folder: Marley removes only lock files naming `Marley` whose process
    is gone (`/proc`), never on a guess where `/proc` is missing; its own file is written 0600
    through a temporary name; every path comes from `CLAUDE_CONFIG_DIR` as Claude Code reads it,
    so a scenario's scratch folder keeps the user's untouched.
  - Re-entrancy: the workspace observer defers its reconcile, since the workspace is being built;
    the editor and project subscriptions run outside any update of what they read.
  - Pre-auth bounds: the upgrade goes through #524's limits; a refusal writes a status and closes.
- **Clippy rounds:** two first doc paragraphs, `Eq` derivable, a value parameter not consumed,
  `map_or_else`, a field named after its struct (`Upgrade.upgrade`), a lock guard held past its
  use, a `&mut App` not needed, a single-pattern `match`. **Docs gate:** a module doc linked a
  private constant.
- **Gate:** `just gate-diff` green after the Test phase's fix, 17 gates on the scope.

## Phase 3 — Test (2026-10-04)
- **Scenario:** `script/e2e/653-marley-as-claude-codes-ide.sh`, `compositor sway`, a terminal pane
  beside an editor pane (`pane: split and move right`). A Python stand-in `claude` links as
  Claude Code does, with a hand-written WebSocket client; a Python stand-in language server named
  in `lsp.rust-analyzer.binary.path` gives `src/main.rs` one error. Every check passes (25).
- **Shots, each read:**
  - `653-01-off`: `port: unset · auto-connect: unset`, `lock: none`, the folder holds `1.lock` and
    `2.lock` as set up. REQ-001.
  - `653-02-lock`: the port and `auto-connect: true`; `ideName Marley · transport ws`, a
    32-character token, `windows False`, the repository as the folder, `0o600` in `0o700`; the
    folder holds `2.lock` and the port's lock, `1.lock` gone; the 101 with `mcp`, `server:
    Marley`, the five tools. REQ-002, REQ-003, REQ-004, REQ-006.
  - `653-03-refused`: `badtoken: HTTP/1.1 401 Unauthorized`. REQ-005.
  - `653-05-selection`: lines 2 to 4 selected in the right pane; the stand-in got `src/auth.txt
    1:0-3:18` with the three lines (and the file's caret once, when it opened). REQ-007.
  - `653-06-open-file`: `main.rs` opened, `selection_changed src/main.rs 0:0-0:0 empty`; the
    stand-in server's error underlines `missing`. REQ-008.
  - `653-07-latest`: the terminal focused, `latest: src/main.rs 0:0-0:0 empty`. REQ-009.
  - `653-08-diagnostics`: `diag all: 1 file(s)`, `src/main.rs 2:4 Error stand-in E0425`;
    `diag src/auth.txt`: one file, none. REQ-010, REQ-011.
  - `653-09-mention`: `at_mentioned src/auth.txt 1-3`, the terminal focused, the Enter after it
    typed nothing. REQ-012.
  - `653-10a-picker`, `653-10-fallback`: two Claude Code targets; the offline one got
    `@src/auth.txt#L2-4` at its prompt and printed `typed: @src/auth.txt#L2-4`. REQ-013.
  - `653-11-old-client`: with `claude_ide_selection` out of the allow map, a new client is listed
    `getDiagnostics, getWorkspaceFolders`, gets no selection after a new one is made, and its
    `diag` is answered; the chip reads Untested Claude Code 2.1.288. REQ-014.
  - `653-12-off-again`: the switch off, the client printed `closed` and the folder holds `2.lock`
    alone. REQ-015.
  - `653-13-untested`: the switch on, the connection's id out of the allow map: `port: unset`,
    `lock: none`; the chip's tooltip reads "IDE links are off: Marley serves no IDE link, …
    Tested on no version yet. Turn on IDE Link on Untested Claude Code …", and the same for IDE
    selections. REQ-016.
- **Machine checks:** the stale lock removed and the other IDE's kept (REQ-004); one
  `at_mentioned` and one typed reference in the whole run; after `quit_marley`, with the link
  running, the folder holds `2.lock` alone (REQ-017).
- **Fixed in Test:** the first run showed the same empty selection sent about two dozen times:
  the workspace reports `ActiveItemChanged` far more often than its active item changes, and each
  report re-sent an unchanged selection. A broadcast now skips the selection sent last; the run
  checks it is sent once. The first run's checks also expected `3:16` for an 18-character line,
  and counted `port: unset` across runs; both were the scenario's.
- **Review only:** a project closed while Marley runs (REQ-017's other half), a folder added
  (REQ-018), the bounds and caps (REQ-019). Not reached: a real Claude Code (REQ-020, deferred to
  Chad's by-hand check; no real turn in this batch), two projects with a port each.

## Phase 4 — Complete (2026-10-04)
- **Docs:** `CHANGELOG.md` (Added); `docs/marley/guide.md` ("Claude Code's IDE link");
  `docs/marley/walkthrough.md` §5.12 (the mention); `docs/marley_architecture/marley_mcp.md` (the
  IDE server), `marley_workbench.md` (`claude_ide` and send selection's route), `marley_agent.md`
  (the three rows, the empty range's words), `terminal_blocks.md` (`marley_terminal::ide`); the
  plan's C0 row; the design note's B3 first slice; the six touchpoint rows read against what
  shipped.
- **Knowledge:** F-claude-653-an-active-item-event-resent-an-unchanged-selection-001,
  AD-claude-653-marley-is-claude-codes-ide-per-project-001,
  L-claude-653-a-stand-in-language-server-by-its-binary-path-001.
- **Brain:** consultation `a4c7036a0a7d40e5aeb2a0a7fb9c510a` closed with `brain decide`
  (`decisions/marley-serves-claude-codes-ide-link-per-local-project-its-unnamed-parts-gated-by-the-version-table`).
- **Open for Chad:** the by-hand check on a real Claude Code (REQ-020), which would give the three
  rows their first version.
- **Closed:** TICKET-653 in `tickets/closed/`; the pair archived to `completed/`.
