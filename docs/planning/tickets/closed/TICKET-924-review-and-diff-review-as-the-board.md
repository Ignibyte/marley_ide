# TICKET-924 — Plan review and diff review as the board draws them

- **Ticket:** LOCAL #924 (feature, the Marley look, review)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [924-review-and-diff-review-as-the-board.spec.md](../../pipeline/completed/924-review-and-diff-review-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/C-threads.md, boards Review and DiffReview)
- **Boards:** Review, DiffReview
- **Size:** M
- **Status:** closed

## Summary

Plan review and the diff review work, but they are laid out and worded differently from the
boards. Plan review puts its send buttons, its hint and its title in the notes column, shows a
note's quote in italics with an X, and lists a turn's files as a muted count. The diff review's
file headers have no line counts, the guide's Send to rows are all muted with no Copy notes row,
and a noted line has no mark in the gutter. This ticket moves and rewords those parts to match
the boards, so the review screens read the same as the rest of the Marley look.

## Acceptance

Plan review shows its title as the main column's heading, Send Answers beside Send Feedback (N)
at the plan's foot with the comment hint, a "Your notes" column with mono quote chips and Remove,
turn cards with a state pill and per-file counts, and, for a file or a reply, Send Review to
Agent (N) at the column's foot. The diff review shows +/− in each file header, coloured Send to rows with a Copy notes row,
and a gutter mark on noted lines. Shot in Marley Night and Marley Day.
