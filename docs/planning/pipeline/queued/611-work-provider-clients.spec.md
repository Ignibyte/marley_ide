---
pipeline_id: afded6ec-c204-4e90-a32a-9c91d1f71ee5
ticket: docs/planning/tickets/open/TICKET-611-work-provider-clients.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "The marley.work/v1 clients over MCP and HTTP"
type: feature
slice: prong 2, D20, wave 1; after #607
references: [docs/marley/fleet-contract.md, docs/planning/pipeline/queued/534-harness-sessions-in-the-rail.spec.md]
---

## Title
Marley reads a real workflow store: any server that answers `marley.work/v1` over MCP (stdio or
HTTP) or plain HTTP, with each source's state shown honestly.

## Scope
### In
- **Provider kinds** in `marley.fleet.providers`, beside `pseudo`:
  - `{ "kind": "mcp", "command": "...", "args": [...] }`: an MCP server over stdio, through
    Zed's `ContextServer::stdio`;
  - `{ "kind": "mcp", "url": "http://..." }`: an MCP server over HTTP, through
    `ContextServer::http`;
  - `{ "kind": "http", "url": "http://..." }`: the contract's plain HTTP endpoints.

  Each takes an optional `bearer_env`, the name of an environment variable holding its token;
  the token is sent as `Authorization: Bearer` and never logged or shown.
- **The calls:**
  - `work_handshake` first. `work_agents` then runs every `poll_s`, or `work_changes` from the
    cursor when the capabilities offer it, with `reset: true` rereading the list.
  - `work_agent` and `work_run` are called for what the Fleet panel and the Agent tab show.
  - Over MCP a call reads `structuredContent`, else the JSON in the first text content.
- **Source states**, each shown in the Fleet panel's header for its provider:
  - *not set up*: no provider;
  - *connecting*: the handshake is pending;
  - *ready*;
  - *unreachable*: the error's short line, then a retry with backoff;
  - *stale*: no answer for three polls;
  - *incompatible*: a `contract` other than `marley.work/v1`, named in the message.
- **Several providers:** their agents are listed together, each keyed by provider and id, and a
  provider's failure affects only its own agents.
- **The stub provider** for the scenario, `script/e2e/fleet-stub-provider.py`. It serves the
  contract's fixtures over HTTP and over MCP on stdio, and can be told to answer another contract
  or to stop answering.

### Out (explicitly deferred)
- Write calls (actions) and the write grant.
- rustal-brain's own implementation of the contract: a brain ticket, when the brain resumes.
- MCP resource subscriptions: polling and `work_changes` cover `v1`.

## Reference (§20)
Upstream Zed: the MCP client is `context_server` (`ContextServer::stdio` and `::http`,
`crates/context_server/src/context_server.rs:49-120`), calling a tool with `CallTool` and reading
`CallToolResponse::structured_content` (`crates/context_server/src/types.rs:40, 164, 717-727`), as
Zed's agent calls a context server's tools (`crates/agent/src/tools/context_server_registry.rs:362-395`).
The MCP specification (2025-06-18) defines `tools/call` and structured content.

### Prior art
- **Behavior maps:** #534's spec, which plans the same `ContextServer::stdio` client for the
  harness (`534-harness-sessions-in-the-rail.spec.md:21-29`), and #521's honest states.
- **Published material:** the MCP 2025-06-18 specification (`tools/call`, `structuredContent`,
  errors).
- **Code we already ship:**
  - `context_server` as above.
  - Zed's `http_client` as `system_one.rs` uses it: request builder, bearer header, timeout,
    backstop timer (`crates/marley_workbench/src/system_one.rs:858-945`).
  - `push.rs`'s URL checks (`crates/marley_workbench/src/push.rs:106-118`).
  - `marley_mcp`'s never-logged-bearer rule.
  - The three failure modes the plan asks to render honestly (`three-prong-plan.md`,
    prong 2's risks).

## UI proof
The scenario `script/e2e/611-work-provider-clients.sh` (sway) starts the stub provider and sets
two providers, one over HTTP and one over MCP stdio. Shots:
- `ready.png`: both providers' agents listed, each header ready;
- `unreachable.png`: the HTTP stub killed, its header unreachable and its agents offline, the MCP
  provider untouched;
- `incompatible.png`: the stub answering `marley.work/v9`, the header naming it;
- `stale.png`: the stub alive but silent, its header stale after three polls.

`log.txt` shows the bearer's variable name and never its value.

## Locked-In Decisions
- D1 — MCP goes through Zed's `context_server`, the client #534 also takes; HTTP through Zed's
  `http_client`.
- D2 — Tokens come from environment variables named in the settings, never from the settings
  file, and are never logged.
- D3 — One provider's failure touches only its own agents.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a provider over HTTP or over MCP answers the handshake with `marley.work/v1`, the Fleet panel shall list its agents and mark it ready. | Shot `ready.png` |
| REQ-002 | WHEN a provider stops answering, its header shall read unreachable with the reason, its agents offline, and Marley shall retry with backoff. | Shot `unreachable.png` |
| REQ-003 | WHEN a provider names another contract, its header shall read incompatible and name the contract. | Shot `incompatible.png` |
| REQ-004 | WHILE a provider has not answered for three polls, its header shall read stale. | Shot `stale.png` |
| REQ-005 | WHERE a provider names `bearer_env`, Marley shall send that variable's value as a bearer token and never write it to a log. | `log.txt`; review |
| REQ-006 | WHILE several providers are set, a failure of one shall leave the others' agents as they are. | Shot `unreachable.png` |

## Phase Plan
- **P1 Plan** — promote, recall, the design (the provider task, backoff, the change cursor).
- **P2 Code** — the three kinds, the states, the stub provider; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
