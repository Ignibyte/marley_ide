# TICKET-483 — E2E visualization tests replace unit tests

- **Ticket:** LOCAL #483 (chore, the workflow: CONSTITUTION §0, §3, §7)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/483-e2e-visualization-tests.spec.md
- **Source ticket:** Chad, 2026-09-23
- **Status:** closed

## Summary
Chad, 2026-09-23: "We are removing unit tests from the workflow entirely with instead doing e2e
visualization tests only. Please update the workflow accordingly. No more unit tests". He chose
to keep the tests already in the tree, still compiled by clippy but run by no gate, and scripted
live captures as the e2e test: per ticket, a script launches the real debug Marley on hidden
workspace 9 with a seeded profile and fakes on its PATH, sends keys to that window only, and
shoots each step; the Test phase reads every shot.

## Acceptance
The gate runs no test suite, coverage or miri; `script/e2e.sh` runs a scenario against the real
Marley without moving the user's focus; the Test phase's hook wants an e2e run; the constitution
and the phase commands say so. The EARS criteria are in the spec.
