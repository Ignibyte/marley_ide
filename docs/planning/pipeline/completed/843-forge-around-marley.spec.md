---
pipeline_id: dc8e110d-0a52-4f4f-b0bd-3cced4f8f54a
ticket: docs/planning/tickets/open/TICKET-843-forge-around-marley.md
status: Phase 4 — Complete PASS
title: Forge around Marley
type: feature
slice: The Marley look, W4.3 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
While a `forge` provider is set, Forge shows in three more places, as the ForgeAgents board draws
them: a FORGE section in the rail (its online agents, each with host, ticket and state, and Open
the sprint board), a FORGE card on Home (the sprint's progress, what is working, what needs you,
what failed QA, and Forge's bulletins), and a Forge item in the bottom row (online agents and the
newest QA failure), each opening the sprint board.

## Scope
### In
- `forge_watch.rs` (new): the `ForgeWatch` global, which reads Forge every 30 s while a `forge`
  provider is set (`agent-list`, `sprint-current`, the sprint's `ticket-list`,
  `bulletin-list`) and stops when none is; it writes only when the data changed.
- The rail: a FORGE section between Needs you and the projects, its rows two-line with a tile.
- Home: a FORGE card after Rusty's cards.
- The bottom row: a `ForgeItem` after the focus item, hidden while there is no Forge.
- Bulletins: on the Home card, critical first, each dismissable for the session.

### Out (explicitly deferred)
- Forge's knowledge search on the board (a later ticket with its own design).
- Attach and Give a ticket on the rail's rows (Forge has no tool for either).

## Reference (§20)
N/A — Marley-specific: Ignibyte Forge's tools (`agent-list`, `sprint-current`, `ticket-list`,
`bulletin-list` in `/srv/stacks/ignibyte-forge/crates/forge-mcp/src/tools/`); the ForgeAgents
board's "Forge in the rail", "On Home" and "In the bottom row".

### Prior art
- The code we ship: #841's `forge::summaries` and the fleet's Forge client; #842's `read_forge`;
  Rusty's keeper loop (`rusty.rs` `follow_setting`/`keep`), the rail's `render_inbox` and
  `render_inbox_card`, Home's `card`/`row`, the bottom row's `FocusRow` and its registration in
  `status_buttons.rs`.
- The brain (consultation 3c7ad8d4): nothing on this seam.

## UI proof
Scenario `script/e2e/843-forge-around-marley.sh` (sway): #842's stand-in Forge grown two
bulletins.
- `843-01-rail`: the rail's FORGE section, "4 online", four rows, Open the sprint board; the
  bottom row's "Forge · 4 online · #828 QA failed" (REQ-001, REQ-003).
- `843-02-home`: Home's FORGE card: "Sprint 14 · 3 of 11 done · 4 days left", "2 working",
  "1 needs you", "1 QA failed", and the two bulletins (REQ-002).
- `843-03-board`: the bottom row's Forge item clicked: the sprint board (REQ-003).
- `843-04-day`: Marley Day, the rail and the bottom row (REQ-001).

## Locked-In Decisions
- D1 — One `ForgeWatch` for every surface, apart from the Fleet panel's provider and the board,
  which read on their own schedules.
- D2 — "Needs you" counts Forge's `needs_attention` sessions; "QA failed" counts the sprint's
  `qa-failure` tickets, the bottom row naming the newest.
- D3 — A dismissed bulletin stays hidden until Marley restarts.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a `forge` provider is set, the rail shall show a FORGE section with Forge's online agents (name, host, ticket, state) and a way to the sprint board, in Marley Night and Marley Day. | `843-01-rail`, `843-04-day` |
| REQ-002 | WHILE a `forge` provider is set, Home shall show a FORGE card with the sprint's progress, the counts working, needing you and failing QA, and Forge's bulletins. | `843-02-home` |
| REQ-003 | WHILE a `forge` provider is set, the bottom row shall show Forge's online count and the newest QA failure, and a click shall open the sprint board. | `843-01-rail`, `843-03-board` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
