---
pipeline_id: dd0911c3-898f-469d-a6e5-d9bc64b26209
ticket: docs/planning/tickets/open/TICKET-874-rusty-decisions-and-follow-up.md
status: Phase 4 — Complete PASS
title: Rusty's Decisions and Follow Up as the boards draw them
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
Rusty's Decisions tab (#659) and its Follow Up form (#660) read as the MoreTabs board draws them:
the decisions due in a card of their own, each row its title over its dates with its status chip
coloured by status (decided and kept green, revised yellow, superseded muted), and the form's
Cancel quiet and Record primary.

## Scope
### In
- `decisions_tab.rs`: the Due section as a card (the warning border while one is overdue), a row
  two lines (title, then its dates), the chip's colour by status, Follow Up as an outlined button.
- `follow_up.rs`: Cancel and Record as #870's quiet and primary buttons.

### Out (explicitly deferred)
- The form's fields and their behaviour (#660's), the tab's menus and reads.

## Reference (§20)
N/A — Marley-specific: the Decisions tab and Follow Up form are Marley's (#659, #660); the
MoreTabs board's "Decisions, with a follow-up".

### Prior art
- `decisions_tab.rs` (`render_row`, `render_end_slot`), `follow_up.rs`; #870's
  `prompts::render_button`.
- The brain (consultation ea57bb9d): nothing on this seam.

## UI proof
Scenario `script/e2e/874-decisions-tab.sh` (sway): the stand-in Rusty with six decision pages on
2026-10-03 (two due, one overdue; decided, kept, superseded).
- `874-01-decisions`: the Due card, the rows with their dates under them and chips coloured by
  status (REQ-001).
- `874-02-follow-up`: Follow Up: the form with Cancel and Record (REQ-002).
- `874-03-day`: Marley Day (REQ-001).

## Locked-In Decisions
- D1 — Chip colours: decided and kept `Success`, revised `Warning`, superseded `Muted`, any
  other status `Default`.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Decisions tab shall show the decisions due in a card, and each decision as its title over its dates with its status chip coloured by status, in Marley Night and Marley Day. | `874-01-decisions`, `874-03-day` |
| REQ-002 | The Follow Up form shall offer Cancel as a quiet button and Record as the primary one. | `874-02-follow-up` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
