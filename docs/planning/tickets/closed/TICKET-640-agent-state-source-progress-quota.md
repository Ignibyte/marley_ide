# TICKET-640 — A harness session's state source, progress and quota on its rail row

- **Ticket:** LOCAL #640 (feature, prong 2 C1, the harness's read side)
- **Owner:** claude-opus-5-5, 2026-10-02
- **Pipeline doc:** ../../pipeline/completed/640-agent-state-source-progress-quota.spec.md
- **Source ticket:** rustal-harness `docs/planning/MARLEY_REQUESTS.md` MREQ-005 to MREQ-007 and its
  D164; `design-notes/herdr-and-hermes-2026-10-02.md` (Part 1, item 3; the harness's D164 section)
- **Status:** closed

## Summary
rustal-harness will send three facts about each session as labels on the `marley_fleet` envelope
Marley already follows (#534): where its state came from (`state.source`: `protocol`, `reported`
or `detected`), how far it is (`progress.percent`, `progress.activity`), and its account's quota
per window (`quota.KIND.percent_used`, `quota.KIND.resets_at_ms`), tokens only. Marley reads them
and shows them on the session's row in the rail's Harness section: a state read off a screen, or
from a source Marley does not know, is drawn weaker, as a stale row is, and never puts a question
in the approvals inbox; progress and the most-used quota window each get a line; a tooltip names
the source and lists every window and the account. The Fleet panel is a follow-up: no harness
session reaches it today, and `marley.work/v1` would take the same values as optional fields.

## Acceptance
A detected or unknown source is drawn weaker and its question stays out of the inbox; a declared
or absent source draws as #534 does; progress and the most-used quota window show on the row, bad
values left out; the tooltip lists the source, every window and the account; changes show within
five seconds. Thirteen EARS criteria in the spec.
