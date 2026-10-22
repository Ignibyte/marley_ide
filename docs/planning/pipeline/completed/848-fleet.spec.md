---
pipeline_id: bf2dddcf-220c-4c12-9f0b-d48f61f46834
ticket: docs/planning/tickets/open/TICKET-848-fleet-and-harness.md
status: Phase 4 — Complete PASS
title: The Fleet panel as its board draws it
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The Fleet panel reads as the Fleet board draws it: under its title a line "N agents on M hosts ·
the providers", each host a card (its name, its resources at the right), each agent a row with a
lettered tile for its runtime (CC, CX), its name over its work, and its state.

## Scope
### In
- `fleet.rs`: the summary line, the host groups as cards, the agent's runtime tile.

### Out (explicitly deferred)
- Harness sessions in the rail, a session's tab, a scheduled task's runs and the New Harness Seat
  form (TICKET-878); the agent snapshot pane keeps #608's look.

## Reference (§20)
N/A — Marley-specific: the Fleet panel (#607, #608, #841); the Fleet board.

### Prior art
- `fleet.rs` `render_source`, `render_agent`, `host_groups`, `host_line`; #843's rail tiles.
- The brain (consultation be835a67): nothing on this seam.

## UI proof
Scenario `script/e2e/848-fleet.sh` (sway): the pseudo provider (three agents on two hosts).
- `848-01-fleet`: the summary line, two host cards, the agents' tiles and states (REQ-001).
- `848-02-day`: Marley Day (REQ-001).

## Locked-In Decisions
- D1 — Tiles: `claude-code`/`claude` CC, `codex` CX, `gemini` GE, anything else its first two
  letters upper-cased.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Fleet panel shall show how many agents on how many hosts, each host as a card with its resources, and each agent with its runtime's tile, name, work and state, in Marley Night and Marley Day. | `848-01-fleet`, `848-02-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
