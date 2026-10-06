# TICKET-660 — Record a decision's follow-up from the Decisions tab

- **Ticket:** LOCAL #660 (feature, Rusty in Marley R7b)
- **Owner:** claude-opus-5-5, 2026-10-05 (Chad's answer at the end of the #643 to #659 batch)
- **Pipeline doc:** ../../pipeline/completed/660-rusty-decision-follow-up.spec.md
- **Source ticket:** `docs/marley/rusty-in-marley.md` (the slices table's R7b); #659's Out (the
  follow-up form split from the Decisions tab); Chad, 2026-10-05: write it and queue it first
- **Status:** closed

## Summary
The Decisions tab (#659) only reads. Chad's brain holds about 15 overdue follow-ups, and only an
agent's `brain_follow_up` or `rusty-cli brain follow-up` records one. This ticket adds a Follow Up
action to each decision row: choose kept, revised or superseded, write the outcome, and give a new
follow-up day when revised or the successor decision when superseded. Marley sends Rusty's
`brain_follow_up { slug, outcome, status, successor, follow_up_by }`, shows a refusal in Rusty's
words, and reads `brain_due` again. The rows also show what Rusty already serves and #659 left
out: the day of the last follow-up (`followed_up`) and the decision that replaced a superseded
one (`superseded_by`), which opens that page.

## Acceptance
From a decision row, a follow-up recorded in Marley reaches Rusty as one `brain_follow_up` call,
and the tab then shows the decision's new status, its new or cleared follow-up day and its last
follow-up's day; a refusal shows Rusty's words and changes nothing; with Rusty off or not
connected the action is not offered. The full EARS criteria live in the pipeline spec.
