---
pipeline_id: 6746739e-cbac-4eb9-9de6-19b4cac9c4fd
ticket: docs/planning/tickets/open/TICKET-858-motion-and-focus.md
status: Phase 4 — Complete PASS
title: The needs-you pulse
type: feature
slice: The Marley look, W6 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
When a rail row starts needing the user, its state dot pulses twice, growing from 10 to 12 px with
a soft ring, and then stays still, as the States board draws it. With reduced motion on, the dot is
still from the start.

## Scope
### In
- `rail.rs` `row_card`: the needs-you dot as an animated element.

### Out (explicitly deferred)
- The rail's shrink in 180 ms and one focus ring everywhere: TICKET-895.

## Reference (§20)
N/A — Marley-specific: the rail's state dots (#827); the States board's pulse.

### Prior art
- `row_card`'s dot (`RowState::dot`), gpui's `with_animation`, which shows one still frame under
  `App::reduce_motion`.
- The brain (consultation 27a1d4aa).

## UI proof
Scenario `script/e2e/858-needs-you-pulse.sh` (sway): a stand-in `claude` that asks.
- `858-01-pulse`: mid-pulse: the dot larger with its ring (REQ-001).
- `858-02-still`: after the two pulses: the dot at rest (REQ-001).
- `858-03-reduced`: with `reduce_motion` on, a new ask: the dot still at once (REQ-002).

## Locked-In Decisions
- D1 — Two pulses in 1.2 s, once per time the row starts needing the user; nothing repeats after.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a rail row starts needing the user, its dot shall pulse twice and then stay still. | `858-01-pulse`, `858-02-still` |
| REQ-002 | WHILE reduced motion is on, the dot shall not pulse. | `858-03-reduced` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
