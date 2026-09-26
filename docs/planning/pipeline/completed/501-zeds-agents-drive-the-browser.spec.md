---
pipeline_id: 8ad0a606-3da2-4f92-9d51-2739eb2c5de8
ticket: docs/planning/tickets/open/TICKET-501-zeds-agents-drive-the-browser.md
status: Phase 4 — Complete PASS
title: "Zed's own agents get Marley's tools, the browser among them"
type: feature
slice: prong 3 with prong 2's C0 (after wave 2), from Chad's first look at the browser
references: [docs/planning/pipeline/completed/491-marley-mcp-server.spec.md, docs/planning/pipeline/completed/492-browser-tools-for-agents.spec.md]
---

## Title
Marley registers its MCP server as the context server `marley` among Zed's default settings,
so the Zed Agent lists Marley's tools and every external agent of the Agent Panel is handed
Marley's server when its session starts: any of them can pull up the browser and drive it.

## Scope
### In
- At start, once its server runs, Marley writes its bridge (the plugin's `marley-mcp-bridge`)
  into its data directory and adds `context_servers.marley` to Zed's default settings: a stdio
  server running the bridge, with `MARLEY_MCP_ENDPOINT` naming Marley's endpoint file. No bearer
  goes into any setting; the bridge reads it from the file, and follows Marley across restarts.
- The Zed Agent's default Write profile enables every context server, so it lists Marley's tools;
  `mcp_servers_for_project` hands the server to each external agent's new session.
- A user's own `context_servers.marley` (`{"enabled": false}`, say) wins, as settings do.

### Out (explicitly deferred)
- Remote projects (the server runs where Marley runs, so it is not handed to a remote agent).
- Profiles other than Write, whose tool lists stay the user's.

## Reference (§20)
Upstream Zed's context servers (`project::context_server_store`, `agent_servers::acp`'s
`mcp_servers_for_project`): a configured server's tools reach the Zed Agent through its
profile and reach an ACP agent through `session/new`'s `mcpServers`, as Zed documents for
user-added servers. Marley adds one as a default, the way its layout already patches two
defaults (`apply_defaults`). Warp: N/A.

### Prior art
- **Code we already ship.** `SettingsStore::update_default_settings`;
  `ContextServerSettingsContent::Stdio` and `ContextServerCommand`; the store's
  `maintain_servers`, which starts every enabled configured server; the ACP client's
  `mcp_servers_for_project`; the default profile `write` with `enable_all_context_servers`;
  the plugin's bridge (#491), which already serves Claude Code over stdio.
- **Published material.** MCP's stdio transport; ACP's `session/new` with `mcpServers`
  (agent-client-protocol 2.1, schema 1.7).
- Rejected: an HTTP context server with Marley's URL and bearer, which would put the bearer in
  the settings the Agent Panel shows and go stale at every start.

## UI proof
UI-AFFECTING. `script/e2e/501-zeds-agents-drive-the-browser.sh` (`compositor sway`, offline):
the scratch repository, and a stand-in external agent (a small ACP agent in Python, set as a
custom agent server in the run's settings) that logs the servers it is handed and, when
prompted "open <url>", calls `browser_navigate` through the `marley` server it was given.
Steps: the run log's proof that the store runs `marley` (Marley's log line and the bridge's
process); a new thread of the stand-in from the rail's + (`501-01-agents`: the stand-in in the
agents' list) and the prompt (`501-02-driven`: the page in a Browser tab, the agent's answer in
the thread); the run log (the stand-in's `mcpServers`, the tool's answer).

## Locked-In Decisions
- D1 — Stdio through the bridge, never HTTP with a bearer in settings.
- D2 — A default, not a write to the user's settings file: nothing of Marley's lands in it, and
  the user's own entry wins.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE Marley runs, Zed's context server store shall run the context server `marley` for a local project, where the Zed Agent's Write profile takes its tools from. | The run log (Marley's log line, the running bridge) |
| REQ-002 | WHEN an external agent starts a session in a local project, Marley shall hand it the `marley` server. | The run log |
| REQ-003 | WHEN that agent calls `browser_navigate` through it, a Browser tab shall show the page. | Shot `501-02-driven` |

## Phase Plan
- **P1 Plan** — mint, recall, the design.
- **P2 Code** — the bridge written at start, the default; fmt and clippy clean.
- **P3 Test** — the scenario, every shot; #492's scenario again; the gate.
- **P4 Complete** — docs, ledger, close, archive, commit.
