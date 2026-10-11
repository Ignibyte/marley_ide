# TICKET-733 — Guided review

- **Ticket:** LOCAL #733 (feature; size medium, after #730; the review batch after the finish)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [733-guided-review.spec.md](../../pipeline/queued/733-guided-review.spec.md)
- **Source ticket:** [plannotator-findings.md](../../intake/plannotator-findings.md), item 8; Chad, 2026-10-09: "take a look at https://github.com/backnotprop/plannotator and see what may be to add to the list"; Chad, 2026-10-11, on the [finish-line list](../../design-notes/finish-line-2026-10-11.md): "11.) yes"
- **Status:** open (deliberate: the review batch, after the finish)

## Summary
`review_guide_open`, in #730's `review` tool family: an agent sends a change split into ordered
sections, each with a summary and its files. Marley opens a Guide tab in the project: the sections
in order with their summaries, file chips and a Reviewed box, and "K of N reviewed". A ticked
section folds; a chip shows its file in the project diff, or opens the file when the diff does not
hold it. Ticks live with the tab.

## Acceptance
A submitted guide shows its sections in order, each tickable, and a file chip shows its file in the
project diff.
