---
pipeline_id: c4851110-c856-453e-93c5-d764400acbca
ticket: docs/planning/tickets/open/TICKET-894-rusty-dropped-and-a-missing-cli.md
status: Phase 4 — Complete PASS
title: Rusty dropped, with Try Now
type: feature
slice: The Marley look, W6 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
When Rusty is not connected, Home's Rusty card and the Memory and Secrets tabs show #857's card,
"Rusty is not connected", why, and Try Now, which ends Marley's wait before its next try at once.

## Scope
### In
- `rusty.rs`: the keeper's wait listens for a wake beside its timer; `try_now`;
  `render_dropped`.
- `rusty/home_tab.rs`, `memory_tab.rs`, `secrets_tab.rs`: the card in place of the line.

### Out (explicitly deferred)
- An agent CLI missing: Troubleshooting (#885) already says which is missing and where Marley
  looked, and every launcher offers only the CLIs it finds, so no start meets a missing one.

## Reference (§20)
N/A — Marley-specific: Rusty's connection (#643, #696); the States board.

### Prior art
- `rusty::keep` (the backoff from 1 s to 60 s), `unavailable`, `is_on`, `is_connected`;
  #857's `EmptyState`.
- The brain (consultation b1de08bb).

## UI proof
Scenario `script/e2e/894-rusty-try-now.sh` (sway): Rusty on, its stand-in told to fail, then to
answer.
- `894-01-dropped`: Home's Rusty card: the card, the reason, Try Now (REQ-001).
- `894-02-back`: the stand-in answering, Try Now clicked: Rusty connected well before its wait
  would end (REQ-002).
- `894-03-day`: Marley Day (REQ-001).

## Locked-In Decisions
- D1 — Try Now only ends the current wait; the backoff after a failed try is as before.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE Rusty is on and not connected, Home's Rusty card and the Memory and Secrets tabs shall show the card with why and Try Now, in Marley Night and Marley Day. | `894-01-dropped`, `894-03-day` |
| REQ-002 | WHEN the user clicks Try Now, Marley shall try to connect at once. | `894-02-back` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
