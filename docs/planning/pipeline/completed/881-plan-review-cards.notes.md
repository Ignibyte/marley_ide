# Plan review's question cards and answers as the Review board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-881-plan-review-turns-and-diff-review.md
- **Pipeline spec:** 881-plan-review-cards.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #879; split while planning: plan review's cards and
  buttons here, the diff review and the turns list TICKET-888.
- **Classification / tier:** feature, S. Marley crate only.
- **Recall (§18.3):** #728's scenario and its stand-in `claude` (a plan with four questions).
- **Discovery:** a probe run of #728: the cards have their state, Accept recommended and Skip in
  their header, "(recommended)" in the choice's text; Approve and Send Feedback are Zed buttons.
- **Decisions:** D1 in the spec.

### Design
- `plan_review.rs`: `choice_label` without "(recommended)"; each choice an `h_flex` of its box
  and, when recommended, a "Recommended" chip; `card_buttons` at the card's foot as
  `render_button`s; `answer_buttons` and `send_notes_button` as `render_button`s, a disabled one
  dimmed with no click (D1).
- **File manifest:** `plan_review.rs`; the scenario.

## Phase 2 — Code
- **Built:** `plan_review.rs`: `choice_label` without "(recommended)"; each choice an `h_flex` of
  its box and, when recommended, `recommended_chip`; `card_buttons` at the card's foot (Accept
  recommended Primary, Skip Quiet); `answer_buttons` (Send Answers, Approve Primary; Send Feedback
  Plain) and `send_notes_button` as `action_button`, #870's `render_button` dimmed and without a
  click when disabled (D1).
- **Review:** clippy asked for the chip in its own function (`render_card` at 104 lines) and
  `Styled::cursor_pointer` for a closure; the actions call what they called.
- **Gate:** green on the first run, 17 of 17.


## Phase 3 — Test
- **Scenario:** `script/e2e/881-plan-review-cards.sh` (sway), #728's setup.
- `881-01-cards`: Question 1's "In memory — fastest, lost on restart" with the Recommended chip;
  Accept recommended (primary) and Skip under each card's Other field; the column's Approve
  (primary) and Send Feedback dimmed (no feedback yet). REQ-001 met.
- `881-02-day`: Marley Day, the same. REQ-001 met.
- The first run's resting pointer raised the Agent Panel tooltip in the corner; the pointer moved.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation 1df0c46e.
- Ticket closed; pipeline archived; TICKET-888 holds the diff review and the turns.
