# C0: Marley's MCP server runs in the app — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-491-marley-mcp-in-the-app.md
- **Pipeline spec:** 491-marley-mcp-in-the-app.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-24)
- **Request:** wave 1 of prong 3; the agents' way in (plan D17). Prong 2's C0, pulled forward.
- **Classification:** feature; `marley_mcp` (deferred calls, the terminal family, wire names),
  `marley_workbench` (start, stop, answering calls; the plugin's files). No Zed path expected:
  the scenario points the bridge at its profile with `terminal_env`, so Zed's terminal needs
  no new variable.
- **Recall (§18.3):**
  - The never-logged-bearer rule (#370 to #375 lineage): the bearer never reaches a log, a
    tool result or a file other than the 0600 endpoint file.
  - AD-claude-482-…: the plugin is installed from the agent bar's chip with `claude plugin`
    from Marley's local marketplace; a new version must reach installed copies.
  - orchestration-shell.md §10: read tools loose, write tools granted; observation scopes
    (TICKET-043), superseded here for Marley's own tools (D5).
- **Discovery:** `marley_mcp`'s public surface (`transport::spawn`, `ServerData`, `Effect`,
  `Handled`, the const registry); Claude Code's plugin MCP rules from the docs agent.
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.

## Phase 1 — Plan (promoted 2026-09-25)
- **Pre-flight:** #490 committed (cb995b8c9f); no other active pipeline; cargo idle; the README
  marker present.
- **Recall (§18.3):**
  - `BF-mcp-pre-auth-body-alloc-001`: the transport caps a body at 1 MiB before it allocates;
    the guard order (cap, origin, bearer, session, dispatch) stays.
  - The prevention rule on registries: `tools/list` is derived from the registry, never a
    parallel table; the terminal family goes into `REGISTRY`.
  - The prevention rule on secrets: a bearer goes only to a destination checked in code (the
    bridge refuses a URL that is not loopback), and no log or `Debug` shows it.
  - `AD-claude-mcp-first-class-control-plane-001`: MCP as Marley's control plane, observation
    scopes per TICKET-043, which D5 supersedes for Marley's own tools. Brain consultation
    `988fc93f406b433db7559c4d54369a1b`: nothing on this seam.
- **Seams re-verified (an Explore map, 2026-09-25):**
  - `marley_mcp` has no dependents and no `[workspace.dependencies]` entry. `transport::spawn
    (shared, effects: std::sync::mpsc::Sender<Effect>) -> io::Result<ServerHandle>` binds
    `127.0.0.1:0`, mints the bearer from `/dev/urandom` and runs an accept thread for the
    life of the process (no stop); one thread and one request per connection;
    `handle_message` is synchronous and runs under the `ServerData` lock, which
    `serve_connection` drops before it writes the answers (`Outgoing::Response` as a one-shot
    SSE body). `tool_name` is the one place names are joined, with a dot.
    `discovery_json(url, bearer)` gives the server-entry shape;
    `write_discovery_file_in(dir, json)` (0600, fchmod before the write) and
    `remove_discovery_file_in(dir)` take the directory.
  - Blocks: `Terminal::blocks()`, `block_output(&block) -> Option<String>` (`None` once the
    output's first line was evicted); `AnchoredBlock` holds the command, `command_verified`,
    the state, the exit code, the prompt (`pwd`; `git_branch` is never sent by `marley.bash`
    or `marley.zsh`) and line anchors, and no times. `apply` has eleven test call sites and
    `AnchoredBlock` five struct literals, so times go beside the blocks, not in them.
  - Terminals: `cx.windows()`, `downcast::<MultiWorkspace>()`, `workspaces()`; each
    workspace's `items_of_type::<TerminalView>(cx)` and its `TerminalPanel`'s panes; ids are
    the view's `entity_id().as_u64()`, as the rail's.
  - The plugin: `claude_plugin.rs`'s `FILES` (four files, the bool marks the executable one);
    version 1.0.0 in `plugin.json` and `marketplace.json`; the chip checks presence only;
    `marley@marley` is not installed on the dev box.
  - `zed`'s `main.rs` calls `initialize_workspace(app_state.clone(), cx)` at line 877; gpui's
    test detection (`as_test`) exists only under `test-support`.

### Design
- **`marley_mcp`:**
  - `registry.rs`: `Family::Terminal`; `tool_name` joins with `_`; three rows (`Tier::Read`,
    no grant class) with input and output schemas; `Family::is_served`, true for `Terminal`
    only, and `tools_list` lists served families. `lookup` still finds every row (a call to
    `fleet_snapshot` returns the empty fleet; `session_surface_to_human` stays denied without
    a grant).
  - The crate root: `Outgoing::Deferred(PendingCall { id, tool, arguments })`; `AppCall
    { tool, arguments, answer }` with `AppCall::answer(result)`; `ToolAnswer { structured,
    text }`; `pub type AppCaller = Arc<dyn Fn(AppCall) + Send + Sync>`.
  - `dispatch.rs`: `tools/call` of a served terminal tool gives `Outgoing::Deferred`;
    `deferred_response(pending, outcome)` builds the answer (`tool_result`, or `tool_error`
    naming the tool for a failure, a 30-second timeout, or an app that takes no calls);
    `initialize` advertises `tools.listChanged`, which the bridge's notifications need.
  - `transport.rs`: `spawn(shared, effects, caller)`; for `Outgoing::Deferred` the connection
    thread, holding no lock, sends an `AppCall` through the caller and waits on its answer
    channel (`recv_timeout`, 30 seconds).
  - `tools.rs`: `tool_text_result(text, structured)`, for an answer whose text is not its JSON
    (a block's output).
- **`marley_terminal`:** `BlockTimes { started, finished }` kept beside the blocks in
  `AnchoredBlocks`; `stamp(now)` gives each new block its start and each block finished since
  its end; `times(index)`.
- **`terminal` (Zed crate, additive):** `apply_shell_hook` stamps the blocks after a hook
  applies (`SystemTime::now()`); `block_output_kept(&block)`, whether a block's first output
  line is still held.
- **`marley_workbench`:** `mcp.rs` (new): `start(cx)` spawns the server, writes the endpoint
  file in `paths::data_dir()`, removes it in `on_app_quit`, and answers calls in a foreground
  task (`terminal_list`, `terminal_blocks` with an optional `last`, default 50, `terminal_read`
  capped at 2,000 lines and 256 KiB); a failure is logged and shown once, as a notification in
  the first workspace. `Cargo.toml` gains `marley_mcp`. The plugin gains `.mcp.json`
  (`${CLAUDE_PLUGIN_ROOT}/bin/marley-mcp-bridge`) and the bridge, version 1.1.0, both in
  `FILES`.
- **The bridge (`marley-mcp-bridge`, Python 3):** reads stdin line by line; loads the endpoint
  file per attempt and refuses a URL whose host is not 127.0.0.1, `localhost` or `::1`; keeps
  the client's `initialize` to replay when Marley appears; POSTs each message with the bearer,
  `Mcp-Session-Id` and `MCP-Protocol-Version`, reads the JSON or the SSE `data:` lines back; a
  404 re-initializes once; with no Marley, answers `initialize` (tools with `listChanged`),
  `tools/list` (none), `tools/call` (an error result: Marley is not running) and `ping`. A
  thread checks every two seconds whether Marley answers and sends
  `notifications/tools/list_changed` when that changes. Nothing it writes names the bearer.
- **`zed` (Zed crate, additive):** `main.rs` calls `marley_workbench::mcp::start(cx)` after
  `initialize_workspace`.
- **Manifest:** `crates/marley_mcp/src/{marley_mcp.rs, registry.rs, dispatch.rs,
  transport.rs, tools.rs}` (Marley); `crates/marley_terminal/src/anchored.rs` (Marley);
  `crates/terminal/src/terminal.rs` (Zed: ledger row updated); `crates/marley_workbench/
  {Cargo.toml, src/mcp.rs, src/marley_workbench.rs, src/claude_plugin.rs, claude_plugin/
  .claude-plugin/marketplace.json, claude_plugin/marley/.claude-plugin/plugin.json,
  claude_plugin/marley/.mcp.json, claude_plugin/marley/bin/marley-mcp-bridge}` (Marley);
  `crates/zed/src/main.rs` (Zed: a new ledger row); `Cargo.toml` (Zed: the row updated);
  `script/e2e/491-marley-mcp.sh`. The old tests that pin dotted names or call `spawn` get the
  new names and the new argument, so they keep compiling.

### E2E plan
| REQ | Step (`compositor sway`) | Shot or log |
|---|---|---|
| REQ-001 | the harness prints the endpoint file's mode and its URL's host (the client reads `url` only) | run log |
| REQ-002, 007 | `mcp tools` in the terminal: the client runs the plugin's bridge and lists the names | `491-01-tools` |
| REQ-003 | `echo hi`, `false`, `seq 3`, then `mcp blocks`: index, command, exit, directory, duration; `mcp blocks` itself running | `491-02-blocks` |
| REQ-004 | `mcp read "seq 3"` | `491-03-read` |
| REQ-006 | a watching client started from the harness (three tools), Ctrl+Q, then its `list_changed` and its second list (none); the file's absence; `mcp tools` once more with no Marley (no tools, no error) | run log |
| REQ-005 | review: no scenario makes the app stall | — |

### Risks
- A block's output goes to the agent unredacted: the user's approval of each call in Claude
  Code is the check (D5).
- A deferred call holds its connection's thread for up to 30 seconds; the server takes only
  loopback clients that hold the bearer.
- The bridge needs `python3`, which Omarchy ships.
- A Marley that crashed leaves its endpoint file; the bridge treats a refused connection as no
  Marley, and the next start overwrites the file.

### Checklist (no TaskCreate in this harness)
pick ✓, pre-flight ✓, recall ✓, promote ✓, prior art ✓, spec ✓, design ✓.

## Phase 1 — closeout
Phase 1 PASS (autonomous).

## Phase 2 — Code (2026-09-25)
- **Ledger first:** the `Cargo.toml` row (`marley_mcp` in `[workspace.dependencies]`), the
  `crates/terminal/src/terminal.rs` row (the stamp and `block_output_kept`), a new
  `crates/zed/src/main.rs` row, and the `.gitignore` row (below).
- **Built:**
  - `marley_mcp`: `Family::Terminal` and `Family::is_served`; `tool_name` joins with `_`; the three
    terminal rows with their schemas (one function per tool); `tools_list` lists served
    families; `Outgoing::Deferred(PendingCall)`, `AppCall` (with `answer`), `ToolAnswer`,
    `AppOutcome`, `AppCaller`, `APP_CALL_TIMEOUT_SECONDS`; `dispatch::deferred_response`;
    `initialize` advertises `tools.listChanged`; `tools::tool_answer_result`; `transport::spawn`
    takes the caller, and the POST loop hands a deferred call to the app with the lock released
    (`ask_app`, `recv_timeout` 30 s). The old tests take the new names and `spawn`'s caller
    (`Arc::new(drop)`), and their two irrefutable `Outgoing` patterns became `let … else`.
  - `marley_terminal`: `BlockTimes`, kept in `AnchoredBlocks` beside the blocks; `stamp(now)`
    and `times(index)`.
  - `terminal` (Zed): `apply_shell_hook` stamps after a hook applies; `block_output_kept`.
  - `marley_workbench::mcp`: `start` (spawn, the endpoint file written in the background, its
    removal in `on_app_quit`, a failure shown once as a toast in the first workspace, the call
    loop); `terminal_list`, `terminal_blocks` (the newest 50 by default, at most 500; a running
    block's duration is how long it has run), `terminal_read` (the last 2,000 lines, at most
    256 KiB).
  - `zed`'s `main.rs`: `marley_workbench::mcp::start(cx)` after `initialize_workspace`.
  - The plugin: `marley/.mcp.json` (`${CLAUDE_PLUGIN_ROOT}/bin/marley-mcp-bridge`), the bridge
    (Python 3, standard library; `py_compile` clean), both in `FILES` (six entries, the bridge
    executable); `plugin.json` and `marketplace.json` at 1.1.0 with a description that names
    the tools.
- **Deviations from the design, and why:**
  - `SESSION_CAP` goes from 8 to 32, and the bridge closes its session (`DELETE`) when its
    input ends: each Claude Code session's bridge holds a session, and one that ends without
    closing keeps it for the 30-minute idle TTL, so eight would refuse a desk of agents. The
    bound (reject new, never evict) stands.
  - `.gitignore` ignores every `.mcp.json` (a local MCP config carries bearers); the plugin's
    names only its bridge, so an exception after the rule keeps it tracked, with the row
    updated.
  - `resources/*` stay as they were: the fleet resource reads the empty fleet, which is true,
    and hiding it would touch more of the crate than C0 needs.
- **Review of the diff:**
  - The lock: `serve_connection` drops the `ServerData` guard before the loop over outgoing
    messages, so a deferred call's wait holds up no other connection.
  - Re-entrancy: the call loop runs `answer` in a top-level `cx.update`, which only reads
    entities.
  - Security: the bearer reaches only the 0600 file and the `Authorization` header; the bridge
    sends it only to a loopback `http` URL and logs the host and port, never the header; the
    failure messages carry IO errors, not the file's content.
  - Blocking IO: the endpoint file is written and removed on the background executor.
  - Tests: the in-tree tests keep compiling (`--all-targets`); none is added (§7).
- **Checks:** `cargo check` on the touched crates; `cargo fmt`; `cargo clippy -p marley_mcp -p
  marley_terminal -p marley_workbench --all-targets -- -D warnings` clean (fixed at the source:
  a 108-line schema function split per tool, three first doc paragraphs shortened, one
  needless qualification); `zed` builds with `just build`.
- **Checklist (no TaskCreate in this harness):** ledger ✓, marley_mcp ✓, marley_terminal ✓,
  terminal ✓, marley_workbench::mcp ✓, main.rs ✓, plugin ✓, review ✓.

## Phase 2 — closeout
Phase 2 PASS (autonomous).

## Phase 3 — Test (2026-09-25)
- **Scenario:** `script/e2e/491-marley-mcp.sh` (`compositor sway`). `setup` gives the terminal a
  HOME whose `.bashrc` defines `mcp`, a stand-in client (`mcp-client.py`, written by the
  scenario) that runs the plugin's bridge from the tree, as Claude Code runs it, and speaks
  JSON-RPC over its stdio; `terminal_env` points `MARLEY_MCP_ENDPOINT` at the run profile's
  endpoint file. The terminal runs `echo hi`, `false`, `seq 3` and `sleep 1` (so one block has a
  duration worth reading), then `mcp tools`, `mcp blocks`, `mcp read 'seq 3'`. The harness then
  prints the endpoint file's mode and host, starts a watching client, quits Marley through the
  palette (`zed: quit`: Ctrl+Q in a terminal is sent to the shell), and runs the client once
  more with no Marley.
- **One run, every shot read:**
  - `491-01-tools` (REQ-002, REQ-007): "via the plugin's marley/bin/marley-mcp-bridge", "marley
    lists 3 tools": `terminal_list`, `terminal_blocks`, `terminal_read`; no fleet or session
    tool.
  - `491-02-blocks` (REQ-003): the client's own terminal (found by its running `mcp blocks`),
    project `repo`, six blocks in order: `echo hi` exit 0, `false` exit 1, `seq 3` exit 0,
    `sleep 1` exit 0 in 957 ms, `mcp tools` exit 0 in 29 ms, and `mcp blocks` running (65 ms so
    far), each in `repo` and reported by the shell's own hook.
  - `491-03-read` (REQ-004): "output of #2 'seq 3', truncated False": `1`, `2`, `3`.
- **The run log (REQ-001, REQ-006):** "endpoint file: mode 600, type http, host 127.0.0.1, path
  /mcp"; the watcher: "3 tools while Marley runs", "notifications/tools/list_changed came", "0
  tools after the quit"; "endpoint file after the quit: removed"; with no Marley, the bridge
  answered `initialize` and "marley lists 0 tools", with no error. The bridges' log names only
  the address ("connected to Marley at 127.0.0.1:33939" four times, "Marley quit", "answering
  alone: no endpoint file").
- **REQ-005 (review):** no scenario makes the app stall. `ask_app` waits
  `APP_CALL_TIMEOUT_SECONDS` (30) on its answer channel with the server's lock released, and
  `deferred_response` turns a timeout into "<tool>: Marley did not answer within 30 seconds"
  and a dropped call into "<tool>: Marley is not taking tool calls".
- **Seen in the run:** `sleep 1` measured 957 ms. A block's times are stamped when the main
  thread applies each hook, so a busy main thread moves a stamp by tens of milliseconds (here
  the command's start came in late); that is what `BlockTimes` says it records.
- **Focus report:** a headless sway; "hyprland: 0 Marley windows before the run, 0 after; the
  run added no rule and did not reload it".
- **Gate:** `just gate-diff`, touched `terminal`, `marley_terminal`, `marley_mcp`,
  `marley_workbench` and `zed`: gate:1 rustfmt, gate:2 clippy (every target), gate:7
  cargo-audit, gate:8 cargo-deny, gate:9 cargo-shear, gate:10 gitleaks, gate:11 shellcheck,
  gate:12 no-suppressions, gate:13 source-bans, gate:14 docs, gate:16 zed-ledger, gate:17
  manifests, gate:18 typos, gate:20 semgrep, gate:21 dylint, and the receipt: 16 passed, 0
  failed, `GATE GREEN [diff]`; the receipt matches the tree.
- **Verdict:** every criterion has its shot or its line in the run log, read; the gate is green.
- **Checklist (no TaskCreate in this harness):** REQ-001 ✓, REQ-002 ✓, REQ-003 ✓, REQ-004 ✓,
  REQ-005 (review) ✓, REQ-006 ✓, REQ-007 ✓, gate ✓.

## Phase 3 — closeout
Phase 3 PASS (autonomous).

## Phase 4 — Complete (2026-09-25)
- **Documented:** `CHANGELOG.md` (Marley's tools for agents, over MCP); `marley_mcp.md` (new: the
  core, the transport, the app); `marley_workbench.md` (Marley's MCP server, and the plugin's
  bridge); `terminal_blocks.md` (block times); `orchestration-shell.md` (the `marley_mcp` row's
  C0); `three-prong-plan.md` (C0 shipped, in prong 2's slices and prong 3's table). The Zed-path
  rows (`Cargo.toml`, `.gitignore`, `crates/terminal/src/terminal.rs`, `crates/zed/src/main.rs`)
  describe what shipped.
- **Knowledge appended:** `F-claude-491-the-root-gitignore-hid-the-plugins-mcp-json-001`,
  `L-claude-491-ctrl-q-in-a-terminal-goes-to-the-shell-001`,
  `L-claude-491-hook-stamps-are-when-the-main-thread-applies-them-001`,
  `L-claude-491-a-session-per-agent-needs-room-and-a-close-001`,
  `AD-claude-491-marleys-mcp-server-runs-in-the-app-behind-a-stdio-bridge-001`.
- **Brain:** consultation `988fc93f406b433db7559c4d54369a1b` closed by
  `decisions/marleys-mcp-server-runs-in-the-app-reached-through-a-stdio-bridge-in-the-claude-code-plugin`,
  follow-up by 2026-10-24.
- **Checklist (no TaskCreate in this harness):** document ✓, capture knowledge ✓, close the
  ticket ✓, archive ✓, commit (below).
