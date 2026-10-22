---
pipeline_id: 549a6dd9-cdf8-4854-83b6-4e75d52da3a9
ticket: docs/planning/tickets/open/TICKET-846-mcp-servers-and-agent-access.md
status: Phase 4 — Complete PASS
title: MCP Servers as a card per agent
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The MCP Servers page draws a card per agent, as the Tools board does: Claude Code's card holds its
version and place, its integrations and its servers with Add Server; Codex's the same; Zed's card
its context servers, Marley's own among them.

## Scope
### In
- `mcp_servers.rs`: `render_agents`' rows folded into each CLI's card with its servers; a card
  for Zed.

### Out (explicitly deferred)
- Agent Activity, the server form and Outside Clients: they keep their tabs and forms (the
  consent and pairing questions are #871's cards).

## Reference (§20)
N/A — Marley-specific: the MCP Servers page (#512 era), the Tools board.

### Prior art
- `mcp_servers.rs` `render_agents`, `render_agent`, `render_read`, the `heading` helper.
- The brain (consultation 45618e5c): nothing on this seam.

## UI proof
Scenario `script/e2e/846-tools.sh` (sway): a scratch repository, the run's own Claude Code
config (#877).
- `846-01-servers`: Claude Code's card (version, integrations, its servers or why none, Add
  Server), Codex's card, Zed's card with marley (REQ-001).
- `846-03-day`: Marley Day (REQ-001).

## Locked-In Decisions
- D1 — The cards are bordered, rounded blocks on the panel background, one per agent, in the
  order Claude Code, Codex, Zed.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The MCP Servers page shall draw a card per agent, each with what Marley knows of it and its servers, in Marley Night and Marley Day. | `846-01-servers`, `846-03-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
