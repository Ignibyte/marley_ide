# TICKET-732 — A feedback template for review notes

- **Ticket:** LOCAL #732 (feature; size small; the review batch after the finish)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [732-a-feedback-template-for-review-notes.spec.md](../../pipeline/queued/732-a-feedback-template-for-review-notes.spec.md)
- **Source ticket:** [plannotator-findings.md](../../intake/plannotator-findings.md), item 7; Chad, 2026-10-09: "take a look at https://github.com/backnotprop/plannotator and see what may be to add to the list"; Chad, 2026-10-11, on the [finish-line list](../../design-notes/finish-line-2026-10-11.md): "11.) yes"
- **Status:** open (deliberate: the review batch, after the finish)

## Summary
#522's notes reach the agent inside a message the user can set: `marley.review_notes_template` in
`settings.json`, with `{notes}` where the notes go. The default asks the agent to check each note
against the code, say whether it agrees and why, fix only what it agrees with, and change nothing
the notes do not name. Copy notes carries the same message.

## Acceptance
With no template set, a sent note arrives after the default's lines; with a template set, it arrives
inside the template and the default's lines are gone.
