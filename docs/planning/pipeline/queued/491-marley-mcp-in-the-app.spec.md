---
pipeline_id: 93d9699b-70bc-4566-9f2a-4be75c75da1e
ticket: docs/planning/tickets/open/TICKET-491-marley-mcp-in-the-app.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
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
- **The server in the app.** `marley_workbench::init` starts `marley_mcp`'s server once per
  process: 127.0.0.1 on a port the OS chooses, a per-boot bearer, the existing guards (origin,
  bearer, session). It writes `mcp-endpoint.json` (`{url, bearer}`, mode 0600) into Marley's
  data directory and removes it when Marley quits. A failure to start is logged and shown
  once as a notification; Marley runs on without it.
- **Tools answered by the app.** `marley_mcp`'s pure core gains a deferred call: a tool whose
  answer needs the app hands the connection's thread a request for the main thread, which
  answers (asynchronously where it must); the thread waits up to 30 seconds and returns the
  result, or a tool error that says what timed out.
- **Tool names on the wire** are `family_verb` (`terminal_blocks`): Claude Code and the
  Anthropic API take tool names without dots.
- **The terminal family** (read tools; Marley's own terminals in the window that owns the
  server):
  - `terminal_list`: each terminal's id, title, working directory, and the command running
    in it, if any;
  - `terminal_blocks`: a terminal's blocks: command (and whether the shell's frame vouched
    for it, #474), exit status, working directory, branch, start and duration, whether it
    still runs, and whether its output was evicted;
  - `terminal_read`: one block's output as text, at most 2,000 lines, the end kept when more.
  The ported `fleet` and `session` families stay in the crate and are not listed until prong
  2's C1 feeds them.
- **The bridge.** The Marley Claude Code plugin (#482) declares a stdio MCP server, `marley`,
  run by `bin/marley-mcp-bridge` (Python 3, standard library only). It reads the endpoint
  from `$MARLEY_MCP_ENDPOINT`, else `${XDG_DATA_HOME:-~/.local/share}/marley/mcp-endpoint.json`,
  and passes each JSON-RPC message to the server (the bearer, the session header). While no
  Marley answers, it answers `initialize` itself and lists no tools; when Marley appears or
  goes away it sends `notifications/tools/list_changed`. The plugin's version goes up so the
  agent bar's chip offers the update.

### Out (explicitly deferred)
- The browser tools (#492); writing tools of any family; grants beyond what `marley_mcp`
  already enforces.
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
  writer, the per-boot bearer (`mint_secret`). The terminal's anchored blocks
  (`Terminal::marley_anchored`, #464 and #474) are the data the tools read. The Marley Claude
  Code plugin and its local marketplace (`claude_plugin.rs`, #482).

## UI proof
UI-AFFECTING (a new client-visible surface, shown through a terminal). `script/e2e/491-marley-mcp.sh`
(the Hyprland backend is enough: keys only). The scenario's terminal runs a few commands
(`echo hi`, `false`, `seq 3`), then a stand-in client that pipes JSON-RPC through the plugin's
bridge (`initialize`, `tools/list`, `terminal_list`, `terminal_blocks`, `terminal_read`) and
prints each answer. `terminal_env MARLEY_MCP_ENDPOINT` points the bridge at the e2e profile's
endpoint file. After Marley quits, the harness runs the bridge once more and prints its tool
list. Shots: `491-01-tools`, `491-02-blocks`, `491-03-read`; the run log carries the endpoint
file's mode and the bridge's answer with Marley closed.

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

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley starts, it shall serve MCP on 127.0.0.1 with a per-boot bearer and write the endpoint to `mcp-endpoint.json` in its data directory with mode 0600. | The run log: the file's mode and its URL's host |
| REQ-002 | WHEN a client lists tools, the list shall hold `terminal_list`, `terminal_blocks` and `terminal_read`. | Shot `491-01-tools` |
| REQ-003 | WHEN a client calls `terminal_blocks` for a terminal, the answer shall list its blocks with their commands, exit statuses and working directories, in order. | Shot `491-02-blocks`: `echo hi` 0, `false` 1, `seq 3` 0 |
| REQ-004 | WHEN a client calls `terminal_read` for a block, the answer shall be that block's output. | Shot `491-03-read`: `1 2 3` |
| REQ-005 | WHEN a tool call needs the app and the app does not answer within 30 seconds, the client shall get a tool error that names the tool. | Review |
| REQ-006 | WHEN Marley quits, the endpoint file shall be removed; WHEN no Marley answers, the bridge shall answer `initialize` and list no tools, without an error. | The run log after Marley quits |
| REQ-007 | The plugin shall declare the bridge as the `marley` MCP server, and its new version shall be offered by the agent bar's chip. | Review; shot `491-01-tools` shows the bridge in use |

## Phase Plan
- **P1 Plan** — this spec, and the design and E2E plan in the notes.
- **P2 Code** — the server's start and stop in the workbench, deferred calls in `marley_mcp`,
  the terminal family, the wire names, the plugin's bridge and `.mcp.json`; fmt and clippy
  clean; a review of the diff.
- **P3 Test** — write and run `491-marley-mcp.sh`, read every shot; `script/gates.sh --diff`
  green.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_mcp.md` (or the crate's note),
  the plan's C0 row, ledger capture, close, archive, commit.
