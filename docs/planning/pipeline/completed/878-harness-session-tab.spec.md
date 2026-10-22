---
pipeline_id: 1211a970-b192-41c7-93c0-9fcdeb0e69b7
ticket: docs/planning/tickets/open/TICKET-878-harness-sessions-and-runs.md
status: Phase 4 — Complete PASS
title: A harness session's tab as its board draws it
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
A harness session's tab reads as the MoreTabs board draws it: a header with the session's title,
its harness and its state, and with writes on its Views as chips; the session's question as a card
with an answer button each, the first primary; the lines; and a Send row at the bottom.

## Scope
### In
- `harness.rs` `HarnessView`: `render_header`, `render_question`, `render_send` in place
  of `render_controls`; the lines' area.

### Out (explicitly deferred)
- The rail's Harness section (connected line, title, line and quota), a scheduled task's runs and
  the New Harness Seat form: TICKET-886.

## Reference (§20)
N/A — Marley-specific: Marley's harness tabs (#534, #689, #690); the MoreTabs board.

### Prior art
- `HarnessView` and `render_controls`; the rail's state words (`rail.rs`); #870's
  `prompts::render_button`.
- The brain (consultation 7acd569f).

## UI proof
Scenario `script/e2e/878-harness-session-tab.sh` (sway): a scratch harness (`rh serve` on a
short root, never built here) with `worker` and `asker`, writes on.
- `878-01-asker`: the asker's tab: header with "waiting", the question card with main and
  release, the Send row (REQ-001).
- `878-02-views`: Views asked for: the views as chips in the header (REQ-001).
- `878-03-day`: Marley Day (REQ-001).

## Locked-In Decisions
- D1 — The state words and colours are the rail's, so a session reads the same in both.
- D2 — With writes off the tab is the header and the lines, as before.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | A harness session's tab shall show a header with the session's title and state (and with writes on its Views as chips), its question as a card with an answer button each, its lines, and a Send row at the bottom, in Marley Night and Marley Day. | `878-01-asker`, `878-02-views`, `878-03-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
