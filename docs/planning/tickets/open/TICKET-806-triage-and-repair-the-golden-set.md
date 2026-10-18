# TICKET-806 — Triage and repair the golden set

- **Ticket:** LOCAL #806 (chore, the regression suite)
- **Owner:** b6416b37-a6bf-4172-91e4-8d05bc38cda5
- **Pipeline doc:** ../../pipeline/parked/806-triage-and-repair-the-golden-set.spec.md
- **Source ticket:** Chad, 2026-10-17: "yep lets work through see if they are actual bugs or regression spec issues"
- **Status:** open (parked)

## Summary
The golden set (`script/e2e/golden`, #517) had 29 of 53 scenarios failing on 2026-10-17, the
same 29 before and after #797's upstream merge. Each failure is read from its run's log and shots
and sorted: a scenario whose steps no longer match Marley's UI is repaired; a failure that is
Marley's is filed as a bug ticket, with the evidence.

## Acceptance
Every failing scenario has a verdict with evidence; the stale ones pass again; each real bug has
its own ticket.
