---
pipeline_id: 471897cc-1758-476e-9b84-e9ac6e337598
ticket: docs/planning/tickets/open/TICKET-841-forge-as-a-fleet-provider.md
status: Phase 4 — Complete PASS
title: Forge as a fleet provider
type: feature
slice: The Marley look, W4.1 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, docs/marley/fleet-contract.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
A `forge` kind of fleet provider: Marley calls Ignibyte Forge's `agent-list` over its MCP endpoint
and shows Forge's agents in the Fleet panel under their hosts, each with its harness, ticket,
folder and state, Forge's "needs attention" (running and silent past 3 minutes) as the row's
attention. The token comes from the system keyring, entered with `marley: set forge token`.

## Scope
### In
- `FleetProviderContent::Forge { name, url, projects, poll_seconds, bearer_env }` under
  `marley.fleet.providers`; none by default, so nothing reaches Forge until one is added.
- The adapter in `fleet_providers.rs`: the MCP-over-HTTP client it already makes, `agent-list`
  per project (or the token's own project), each `AgentSessionDto` mapped to an `AgentSummary`;
  a handshake made locally (the contract, `poll_s` from `poll_seconds`).
- The token: the keyring entry for the provider's URL (`cx.read_credentials`), else the
  `bearer_env` variable; `marley: set forge token` asks for it in a masked field and writes it.
- The connection's state in the Fleet panel's source header, as for every provider.

### Out (explicitly deferred)
- The rail's Forge group, Home's Forge card, the bottom row's Forge line, bulletins and knowledge
  search (W4.3, TICKET-843); the sprint board (W4.2, TICKET-842).
- Acting on Forge's agents (Attach, Give a ticket): Forge has no tool for either yet.

## Reference (§20)
N/A — Marley-specific: Forge is Ignibyte's own MCP server (`/srv/stacks/ignibyte-forge`:
`forge-mcp-bin/src/router.rs` serves stateless Streamable HTTP at `/mcp/forge` behind a
`forge_pk_` bearer; `tools/agents/agent_list.rs` and `AgentSessionDto` in `tools/agents/mod.rs`;
the 180 s rule in `forge-domain/src/models/agent_session.rs`). Marley's side is the fleet
contract (`docs/marley/fleet-contract.md`, `marley_sdk::work`).

### Prior art
- The code we ship: `fleet_providers.rs` (`connect` makes `ContextServer::http` with a bearer;
  `mcp_value` reads `structuredContent`; polling, backoff, `join_all`), `fleet.rs`'s
  `host_groups` (agents of an undescribed host get their own group), System One's keyring calls
  (`system_one.rs:501`, `:546`) and its masked key field (`system_one_calls.rs:124`).
- The brain (consultation 347107b2): nothing on this seam.

## UI proof
Scenario `script/e2e/841-forge-provider.sh` (sway): a stand-in Forge (Python, stateless JSON-RPC
over HTTP at `/mcp/forge`, refusing a missing or wrong bearer) serves four agents on three hosts;
the provider set with `bearer_env` (the scenario never writes the user's keyring).
- `841-01-fleet`: the Fleet panel: Forge's source, hosts forge, agent1, agent2 with their agents,
  each with harness, ticket and folder; agent1's silent agent marked as needing attention
  (REQ-001, REQ-002).
- `841-02-token`: `marley: set forge token`: the masked field, Escape before Enter (REQ-003).
- `841-03-day`: Marley Day, the Fleet panel (REQ-001).
- The run's log: the stand-in's request log shows only bearer-carrying calls (REQ-001), and a run
  with no forge provider makes none (REQ-004, by the absence of the log).

## Locked-In Decisions
- D1 — A provider kind under `marley.fleet.providers`, not a `marley.forge` section: the fleet
  already polls, backs off and shows a provider's state, and a missing entry is "off" (open
  decision 3 of the look plan: Marley adapts Forge's tools).
- D2 — Forge's derived `state` maps to the contract's: `working` → `Working`, `needs_attention`
  (running, silent past 180 s) → `Waiting`, whose row says how long it has been silent, `idle` →
  `Idle`; sessions whose `status` is `stopped` are left out.
- D3 — `projects` lists Forge project ids; empty asks for the token's own project.
- D4 — The token's keyring entry is keyed by the provider's URL, as System One keys its own.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a forge provider is set, the Fleet panel shall list Forge's running agents under their hosts with their harness, ticket and folder, in Marley Night and Marley Day. | `841-01-fleet`, `841-03-day` |
| REQ-002 | WHEN Forge marks an agent `needs_attention`, the Fleet panel shall show it as waiting, silent for its time. | `841-01-fleet` |
| REQ-003 | WHEN the user runs `marley: set forge token`, Marley shall ask for the token in a masked field and write it to the keyring under the provider's URL. | `841-02-token` (the write not run) |
| REQ-004 | WHILE no forge provider is set, Marley shall make no request to Forge. | the run's log |

## Phase Plan
- **P1 Plan**, **P2 Code** (settings kind, adapter, token action), **P3 Test**, **P4 Complete**.
