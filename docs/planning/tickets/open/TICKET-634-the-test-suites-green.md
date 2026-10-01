# TICKET-634 — The test suites run and green

- **Ticket:** LOCAL #634 (chore, wave 5: the test pass)
- **Owner:** claude-opus-5-5, 2026-10-01
- **Pipeline doc:** ../../pipeline/queued/634-the-test-suites-green.spec.md
- **Source ticket:** wave 5 of `design-notes/remaining-work-2026-09-30.md`; the workflow's "unit tests at the end" (2026-09-29)
- **Status:** open

## Summary
No gate has run the tests in the tree since 2026-09-23 (#483); every ticket since was proven by a
visual check. This ticket runs the suites of the Marley crates and the Marley tests inside the Zed
crates Marley changed, and brings them green: a test that fails because a ticket changed the
behavior on purpose follows the shipped behavior, citing the ticket; a failure that is a real bug
gets its code fixed. No test is added.

## Acceptance
`cargo nextest run` over the Marley crates and the Zed crates Marley's tests live in passes, each
failure of the first run accounted for in the notes as stale (with its ticket) or a bug (with its
fix).
