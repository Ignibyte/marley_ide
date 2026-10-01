---
pipeline_id: 6cd0f3b4-cd9a-4f0e-9c6d-726ef3f1b72b
ticket: docs/planning/tickets/open/TICKET-633-rusty-tools-for-zeds-agents.md
status: Phase 4 — Complete PASS
title: "Rusty's tools for Zed's agents, when Rusty is installed"
type: feature
slice: prong 2 C2 (its context-server half), plan D11
references: [docs/marley/three-prong-plan.md]
---

## Title
When `rusty-mcp` is on the search path, Marley offers Rusty's MCP server to Zed's agents as a
context server named `rusty`, beside its own `marley` server, so an agent in Marley reaches the
brain loop and Rusty's tools; a user's own `rusty` entry, or `marley.rusty_tools` off, wins.

## Scope
### In
- **Detection:** `rusty-mcp` on the search path Marley's agents use, at start and when the
  settings change.
- **The server:** Zed's `ContextServerSettingsContent::Stdio`, command `rusty-mcp` with no
  arguments (Rusty's own `.mcp.json`), added to Zed's default settings the way #501 adds `marley`
  (`SettingsStore::update_default_settings`), so a user's own `context_servers.rusty` replaces it.
- **The switch:** `marley.rusty_tools: Option<bool>`, on by default (it only acts where Rusty is
  installed), with a toggle on the Marley page.
- Zed's own confirmation before each tool call stays as it is: nothing is allowed ahead.
- **Scenarios stay off the user's Rusty:** Zed starts every enabled context server when a project
  opens, and the e2e harness copies the user's own Marley settings, so `script/e2e.sh` sets
  `marley.rusty_tools` false in each run's copy; a scenario that wants it sets it back.
- `script/e2e/633-rusty-tools-for-zeds-agents.sh`.

### Out (explicitly deferred)
- Rusty's agent sessions in the rail (`intake/rusty-sessions-in-the-rail.md`: rusty-mcp serves no
  session tools, and Rusty's constitution keeps its back end MCP only).
- Narrowing Rusty's tools to a read set: Rusty has no grant model; Zed's per-call confirmation is
  the gate.
- The brain-loop hooks for Zed's own agent (Rusty's hooks bind Claude Code).
- Rusty over HTTP (`127.0.0.1:4174/mcp`): stdio works wherever `rusty-mcp` is installed.

## Reference (§20)
Upstream Zed: the agent's context servers (`context_server`, `project`'s `context_server_store`,
the Agent settings' server list) are kept as they are; Marley adds a default entry, as #501 does.

### Prior art
- **Behavior maps:** plan D11 ("Rusty stays Rusty … points agents at `rusty-mcp`") and C2.
- **Published material:** Rusty's README and `.mcp.json` (`"rusty": {"type": "stdio", "command":
  "rusty-mcp"}`); the MCP specification's stdio transport.
- **Code we already ship:** `marley_workbench::mcp::offer_to_zeds_agents` (#501: the `marley`
  server added to Zed's defaults, a user's own entry winning); `settings_content::project`'s
  `ContextServerSettingsContent::{Stdio, Http}`; `agents::launcher`'s search path.

## UI proof
`script/e2e/633-rusty-tools-for-zeds-agents.sh` (`compositor sway`): a stand-in `rusty-mcp` (a
small stdio MCP server the scenario writes, answering `initialize` and `tools/list` with
`brain_ask`, `brain_decide`, `brain_no_decision` and `brain_follow_up`) first on the `PATH` Marley
starts with, and `marley.rusty_tools` set back on; never the user's Rusty. Shots, on the Settings
window's MCP Servers page (Zed's `mcp_servers_page`, which lists each server and whether it runs):
- `633-01-listed`: `rusty` listed and running, beside `marley`;
- `633-02-off`: `marley.rusty_tools` off: no `rusty`;
- `633-03-own`: a user's own `context_servers.rusty` (another command): that one listed.

## Locked-In Decisions
- D1 — Offered only where `rusty-mcp` is installed, so no user without Rusty sees a failing server.
- D2 — Stdio, as Rusty's own `.mcp.json` names it; no token exists to carry.
- D3 — The user's own entry and the switch win over Marley's default.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE `rusty-mcp` is on the search path and `marley.rusty_tools` is on, the system shall offer a `rusty` context server to Zed's agents. | Shot `633-01-listed` |
| REQ-002 | WHERE `rusty-mcp` is not on the search path, or the setting is off, the system shall offer none. | Shot `633-02-off`; review |
| REQ-003 | WHERE the user's settings define `context_servers.rusty`, the system shall leave it as the user wrote it. | Shot `633-03-own` |

## Phase Plan
- **P1 Plan** — promote, confirm how a scenario puts a program on Marley's own search path.
- **P2 Code** — detection, the default entry, the switch; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
