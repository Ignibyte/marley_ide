---
pipeline_id: 62319c16-52b1-4a97-be7b-6fda1a5088a8
ticket: docs/planning/tickets/open/TICKET-876-rusty-memory-skills-secrets-capture-server.md
status: Phase 4 — Complete PASS
title: Rusty's Memory and Skills chips as the RustyTabs board draws them
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
A memory's importance leads its line as a chip in its colour (High in the accent, Normal plain, Low
muted), and every skill and script carries its status as a chip, active green and pending yellow.

## Scope
### In
- `memory_tab.rs`: the importance chip before the category chip; the meta line without it.
- `skills_tab.rs`: `status_chip` on each skill and script row.

### Out (explicitly deferred)
- Secrets, Capture, Import and Rusty's Server: a probe run (`876-03-secrets`, `876-04-capture`)
  found them already in the board's shape (the names with their masked values and Delete; the
  capture box with its line and where it goes).

## Reference (§20)
N/A — Marley-specific: Rusty's tabs (#664, #665); the RustyTabs board.

### Prior art
- `memory_tab.rs` `render_row`, `skills_tab.rs`'s rows; `marley_rusty::memories::Importance`.
- The brain (consultation 534d2fcc): nothing on this seam.

## UI proof
Scenario `script/e2e/876-rusty-tabs.sh` (sway): #664's memories (one high, two normal, one low),
#665's skills (two active, one pending) and script.
- `876-01-memory`: each memory's importance chip in its colour, then its category (REQ-001).
- `876-02-skills`: active green, pending yellow, the script's chip (REQ-002).
- `876-05-day`: Marley Day, Memory (REQ-001).

## Locked-In Decisions
- D1 — Importance colours: High `Accent`, Normal `Default`, Low `Muted`; statuses: active
  `Success`, pending `Warning`.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Memory tab shall lead each memory's line with its importance as a chip in its colour, in Marley Night and Marley Day. | `876-01-memory`, `876-05-day` |
| REQ-002 | The Skills tab shall show each skill's and script's status as a chip, active green and pending yellow. | `876-02-skills` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
