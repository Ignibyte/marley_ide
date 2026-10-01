# TICKET-635 — The golden regression run green

- **Ticket:** LOCAL #635 (chore, wave 5: the test pass)
- **Owner:** claude-opus-5-5, 2026-10-01
- **Pipeline doc:** ../../pipeline/queued/635-the-golden-regression-green.spec.md
- **Source ticket:** wave 5 of `design-notes/remaining-work-2026-09-30.md`
- **Status:** open

## Summary
The golden set (`script/e2e/golden`, #517) has not run since 2026-09-29: each ticket ran its own
scenario only. This ticket runs `just regress` over the set, triages each failure as a stale
scenario (the UI a ticket changed on purpose: the scenario follows it, citing the ticket) or a
regression (the code is fixed), and runs it again green.

## Acceptance
`just regress` passes over the golden set, each failure of the first run accounted for in the notes.
