# TICKET-730 — Agent findings as diff notes

- **Ticket:** LOCAL #730 (feature; size medium)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** [plannotator-findings.md](../../intake/plannotator-findings.md), item 5; Chad, 2026-10-09: "take a look at https://github.com/backnotprop/plannotator and see what may be to add to the list"
- **Status:** open (deliberate: for future use)

## Summary
An MCP tool (such as `review_note_add`) puts a reviewing agent's or a linter's finding into the diff as a note tagged with its source; the user keeps or drops each, then sends the review. Any skill can be the review's prompt. Pairs with #717, so one agent reviews another's diff.

## Acceptance
An agent's note lands in the diff with its source; a kept note reaches the author agent.
