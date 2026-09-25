---
pipeline_id: 93d9699b-70bc-4566-9f2a-4be75c75da1e
ticket: docs/planning/tickets/closed/TICKET-491-marley-mcp-in-the-app.md
status: Phase 4 — Complete PASS
title: "C0: Marley's MCP server runs in the app"
type: feature
slice: prong 2 C0 (pulled forward for prong 3, three-prong-plan.md D17)
references: [docs/marley/three-prong-plan.md, docs/marley_architecture/orchestration-shell.md, crates/marley_mcp, docs/planning/pipeline/completed/482-claude-code-notifications-chip.spec.md]
---

## Title
Marley starts its MCP server, answers tool calls from the app, and gives Claude Code a way to
reach it; the terminal's blocks are the first tools.

## Scope
### In
- **The server in the app.** Marley starts `marley_mcp`'s server once per process, at startup:
  127.0.0.1 on a port the OS chooses, a per-boot bearer, the existing guards (the 1 MiB body
  cap, origin, bearer, session). It writes the endpoint to `mcp-endpoint.json` in Marley's
  data directory, mode 0600, in `marley_mcp`'s shape (an MCP client's server entry: `type`,
  `url`, and an `Authorization` header with the bearer), and removes the file when Marley
  quits. A failure to start is logged and shown once as a notification; Marley runs on
  without it.
- **Tools answered by the app.** `marley_mcp`'s pure core gains a deferred call: a tool whose
  answer needs the app hands the connection's thread a call for the main thread, which
  answers; the thread waits up to 30 seconds, without the server's lock, and returns the
  result, or a tool error that names the tool.
- **Tool names on the wire** are `family_verb` (`terminal_blocks`): Claude Code and the
  Anthropic API take tool names without dots.
- **The terminal family** (read tools over every terminal in Marley's windows, the center
  panes' and the terminal panel's):
  - `terminal_list`: each terminal's id, title, project, working directory, and the command
    running in it, if any;
  - `terminal_blocks`: a terminal's blocks, oldest first: the command and whether the shell's
    own hook reported it (#474), the exit status, the working directory, when it started and
    how long it took, whether it still runs, and whether its output is still in the
    scrollback;
  - `terminal_read`: one block's output as text, at most 2,000 lines, the end kept when there
    are more.
  The terminal stamps each block's start and end as the shell's hooks arrive. The ported
  `fleet` and `session` families stay in the crate and are not listed until prong 2's C1
  feeds them.
- **The bridge.** The Marley Claude Code plugin (#482) declares a stdio MCP server, `marley`,
  run by `bin/marley-mcp-bridge` (Python 3, standard library only). It reads the endpoint
  from `$MARLEY_MCP_ENDPOINT`, else `${XDG_DATA_HOME:-~/.local/share}/marley/mcp-endpoint.json`,
  sends the bearer only to a loopback URL, and passes each JSON-RPC message to the server with
  its session. While no Marley answers, it answers `initialize` itself and lists no tools;
  when Marley appears or goes away it sends `notifications/tools/list_changed`. The plugin's
  version goes to 1.1.0.

### Out (explicitly deferred)
- The browser tools (#492); writing tools of any family; grants beyond what `marley_mcp`
  already enforces.
- A block's git branch: Marley's shell integration does not send it, so no block has one.
- Offering an update to an older installed plugin from the agent bar's chip, which checks
  only whether the plugin is installed (no copy of 1.0.0 is installed on the dev box).
- A `.mcp.json` written into a project, and any change to Claude Code's user configuration:
  the plugin is the one channel.
- Rusty's agent host and the harness as clients (prong 2 C2 and on); they can read the
  endpoint file as any client can.

## Reference (§20)
N/A — Marley-specific: Marley exposing its own state to agents over MCP (the orchestration
shell §4 and §9). Zed exposes nothing of its own over MCP; its `context_server` crate is a
client. The protocol followed is MCP's Streamable HTTP transport (and stdio for the bridge),
as `marley_mcp` already implements it.

### Prior art
- **Published material.** MCP's Streamable HTTP and stdio transports and
  `notifications/tools/list_changed`. Claude Code's documentation (read through the
  claude-code-guide agent, 2026-09-24): a plugin may declare MCP servers in a `.mcp.json` at
  its root; `${CLAUDE_PLUGIN_ROOT}` and `${VAR:-default}` expand in `command`, `args`, `env`,
  `url` and `headers`; Claude Code asks the user before each MCP tool call unless an allow
  rule (`mcp__marley__…`) says otherwise; tool names reach the model as
  `mcp__<server>__<tool>`. The docs are silent on how an unset variable in a server's URL is
  shown, which is why the bridge, not a bare HTTP entry, carries the endpoint: a Claude Code
  session outside Marley never shows a failed server.
- **Code we already ship.** `marley_mcp` (#370 to #379 in the old repository): the pure core
  (`handle_message`, the registry, grants), the `std::net` transport (loopback listener,
  thread per connection, SSE stream, session registry with its TTL), the 0600 discovery
  writer and its server-entry JSON (`discovery_json`), the per-boot bearer (`mint_secret`).
  Nothing starts it today; the gpui-era host (`mcp_host.rs`) was not ported. The terminal's
  anchored blocks (`Terminal::blocks`, `block_output`, #464 and #474) are the data the tools
  read; they hold no times and no eviction flag. The Marley Claude Code plugin and its local
  marketplace (`claude_plugin.rs`, #482). Zed's `main.rs` starts the process's services that
  its tests must not (the crash handler); `initialize_workspace` runs in Zed's tests too.

## UI proof
UI-AFFECTING (a new client-visible surface, shown through a terminal).
`script/e2e/491-marley-mcp.sh`, under `compositor sway` so nothing depends on Chad's desktop
(keys only). The scenario's terminal runs `echo hi`, `false` and `seq 3`, then a stand-in
client that pipes JSON-RPC through the plugin's bridge (`initialize`, `tools/list`,
`terminal_list`, `terminal_blocks`, `terminal_read`) and prints each answer.
`terminal_env MARLEY_MCP_ENDPOINT` points the bridge at the e2e profile's endpoint file. The
run log carries the file's mode and its URL's host (never its bearer). A second client,
started from the harness while Marley runs, waits through Marley's quit (Ctrl+Q) for
`notifications/tools/list_changed` and lists the tools again; then the harness shows the file
is gone and runs the bridge once more. Shots: `491-01-tools`, `491-02-blocks`,
`491-03-read`.

## Locked-In Decisions
- D1 — One server per Marley process, started with the app, not on demand: an agent can list
  what Marley offers before anything opens.
- D2 — The discovery file, not an environment variable, is the endpoint's home; the bridge
  reads it, so every Claude Code session on the machine reaches the Marley that runs, and
  `MARLEY_MCP_ENDPOINT` points at another profile's (an e2e run's) file.
- D3 — The Claude Code plugin carries a stdio bridge, not a bare HTTP entry: outside Marley,
  Claude Code sees a working server with no tools instead of a failure.
- D4 — Wire names are `family_verb`.
- D5 — Terminal tools read Marley's own terminals, all of them. The old rule that agents see
  only manager-created panes (TICKET-043) gives way to Chad's 2026-09-24 direction, "first
  class access to what the user sees"; Claude Code's approval of each call is the check.
- D6 — Deferred calls time out at 30 seconds with an error that names the tool.
- D7 — The server starts from `zed`'s `main`, after `initialize_workspace`: Zed's tests run
  `initialize_workspace`, and a server started there would write its endpoint over a running
  Marley's.
- D8 — The discovery file keeps `marley_mcp`'s server-entry shape, which any MCP client can
  take as it is.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley starts, it shall serve MCP on 127.0.0.1 with a per-boot bearer and write its endpoint to `mcp-endpoint.json` in its data directory with mode 0600. | The run log: the file's mode and its URL's host |
| REQ-002 | WHEN a client lists tools, the list shall hold `terminal_list`, `terminal_blocks` and `terminal_read`, and no fleet or session tool. | Shot `491-01-tools` |
| REQ-003 | WHEN a client calls `terminal_blocks` for a terminal, the answer shall list its blocks in order, with their commands, exit statuses, working directories and durations, the running one marked. | Shot `491-02-blocks`: `echo hi` 0, `false` 1, `seq 3` 0, the client's own command running |
| REQ-004 | WHEN a client calls `terminal_read` for a block, the answer shall be that block's output. | Shot `491-03-read`: `1 2 3` |
| REQ-005 | WHEN a tool call needs the app and the app does not answer within 30 seconds, the client shall get a tool error that names the tool. | Review |
| REQ-006 | WHEN Marley quits, the endpoint file shall be removed and a connected bridge shall send `notifications/tools/list_changed`; WHEN no Marley answers, the bridge shall answer `initialize` and list no tools, without an error. | The run log after Marley quits |
| REQ-007 | The plugin shall declare the bridge as the `marley` MCP server, at version 1.1.0. | Review; every shot's client runs the plugin's bridge |

## Phase Plan
- **P1 Plan** — this spec, and the design and E2E plan in the notes.
- **P2 Code** — the start in `zed`'s `main`, the stop on quit, deferred calls in `marley_mcp`,
  the terminal family, the wire names, the block stamps, the plugin's bridge and `.mcp.json`;
  fmt and clippy clean; a review of the diff.
- **P3 Test** — write and run `491-marley-mcp.sh`, read every shot; `script/gates.sh --diff`
  green.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_mcp.md` (or the crate's note),
  the plan's C0 row, ledger capture, close, archive, commit.
