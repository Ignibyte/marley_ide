---
pipeline_id: f63d69be-728b-438a-9621-cd09e4b405e0
ticket: docs/planning/tickets/open/TICKET-881-plan-review-turns-and-diff-review.md
status: Phase 4 — Complete PASS
title: Plan review's question cards and answers as the Review board draws them
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
In plan review, the recommended choice carries a "Recommended" chip rather than "(recommended)"
in its text, a card's Accept recommended and Skip sit at its foot as buttons, and Approve, Send
Answers, Send Feedback and Send Notes are #870's primary and plain buttons.

## Scope
### In
- `plan_review.rs`: `choice_label`, `render_card`, `card_buttons`, `answer_buttons`,
  `send_notes_button`.

### Out (explicitly deferred)
- The diff review (Changes since main, viewed ticks, notes and findings, the guide, Send to) and
  this terminal's turns in the notes column: TICKET-888.
- Other… as a button: the card's Other field already takes a free answer.

## Reference (§20)
N/A — Marley-specific: Marley's plan review (#726 to #729); the Review board.

### Prior art
- `render_card`, `card_buttons`, `answer_buttons`, `send_notes_button`; #870's
  `prompts::render_button`.
- The brain (consultation 1df0c46e).

## UI proof
Scenario `script/e2e/881-plan-review-cards.sh` (sway): #728's stand-in `claude` asks with a
plan holding four questions, the first with a recommended choice.
- `881-01-cards`: the cards with the Recommended chip and their buttons at the foot; Approve and
  Send Feedback in the column (REQ-001).
- `881-02-day`: Marley Day (REQ-001).

## Locked-In Decisions
- D1 — A disabled action is drawn dimmed and takes no click.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | Plan review shall mark the recommended choice with a Recommended chip, put a card's Accept recommended and Skip at its foot, and draw its answers as primary and plain buttons, in Marley Night and Marley Day. | `881-01-cards`, `881-02-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
