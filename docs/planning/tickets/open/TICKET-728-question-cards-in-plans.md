# TICKET-728 — Question cards in plans

- **Ticket:** LOCAL #728 (feature; size medium, after #726 slice 1; the review batch after the finish)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [728-question-cards-in-plans.spec.md](../../pipeline/queued/728-question-cards-in-plans.spec.md)
- **Source ticket:** [plannotator-findings.md](../../intake/plannotator-findings.md), item 3; Chad, 2026-10-09: "take a look at https://github.com/backnotprop/plannotator and see what may be to add to the list"; Chad, 2026-10-11, on the [finish-line list](../../design-notes/finish-line-2026-10-11.md): "10.) yes"
- **Status:** open (deliberate: the review batch, after the finish)

## Summary
A plan's `:::question`, `:::question-multi` and `:::question-text` blocks (plannotator's published
syntax) are drawn in the plan tab as cards with choices, Other…, Accept recommended and Skip, and
a count of answered questions. While answers are unsent, Send Answers takes Approve's place; the
answers lead the feedback under "Answers to your questions". Agents learn the syntax from the
shared Claude Code plugin's skills, a rustal-harness request written out in the spec's notes.

## Acceptance
A plan's question shows as a card; the chosen answer leads the feedback.
