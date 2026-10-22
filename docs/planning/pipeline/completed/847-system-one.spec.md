---
pipeline_id: d85994d3-6436-46ba-8073-088279448705
ticket: docs/planning/tickets/open/TICKET-847-system-one.md
status: Phase 4 — Complete PASS
title: System One as its board draws it
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
Each System One use picks its mode from Off / Shadow / Suggest / Act buttons, as the SystemOne
board draws them; the System One calls tab opens with a Provider card and a Today card (the spend
of the budget, a bar under it), and closes on what leaves the machine.

## Scope
### In
- `settings_ui.rs` (Zed, #844's hunk): a Marley row's enum of up to four variants draws as buttons
  (was three), which takes in the System One uses and the Agent Control modes.
- `system_one_calls.rs`: the header as two stat cards; the note at the bottom.

### Out (explicitly deferred)
- "Where its answers show" (a gallery of the rail, block and inbox marks the board draws): those
  marks show in their own places already.
- The calls table's column header: the rows keep #565's lines.

## Reference (§20)
N/A — Marley-specific: System One (#565 onward), its calls tab, the SystemOne board; Zed's
`settings_ui` dropdown hunk is #844's.

### Prior art
- #844's `marley_row` and segmented buttons; `system_one_calls.rs` `render_header`.
- The brain (consultation 6053843c): nothing on this seam.

## UI proof
Scenario `script/e2e/847-system-one.sh` (sway): System One on with the `rules` provider (no key,
nothing leaves the machine), three uses set.
- `847-01-uses`: Settings searched for "kind": Stop Kind and Stall Kind as Off / Shadow / Suggest
  / Act buttons (REQ-001).
- `847-02-calls`: the calls tab: the Provider and Today cards, the note (REQ-002).
- `847-03-day`: Marley Day, the uses (REQ-001).

## Locked-In Decisions
- D1 — Buttons for a Marley enum of two to four variants (#844's D2 widened); five or more keep the
  dropdown.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Settings window shall offer each System One use's mode as Off / Shadow / Suggest / Act buttons, in Marley Night and Marley Day. | `847-01-uses`, `847-03-day` |
| REQ-002 | The System One calls tab shall open with a Provider card and a Today card showing the spend of the budget with a bar, and close on what leaves the machine. | `847-02-calls` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
