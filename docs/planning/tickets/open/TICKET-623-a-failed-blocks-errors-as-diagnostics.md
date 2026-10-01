# TICKET-623 — A failed block's errors as project diagnostics

- **Ticket:** LOCAL #623 (feature, prong 1 T4)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/queued/623-a-failed-blocks-errors-as-diagnostics.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`)
- **Status:** open

## Summary
A failed build's `path:line:col` errors belong in Zed's project diagnostics beside the language servers'. Marley reads every failure in a failed block (with #620's locator) and publishes them under a source of its own, so they show in the Diagnostics view and the status bar's count, and clears them when the same command next succeeds.

## Acceptance
WHEN a block fails with located errors, Marley shall list them in the project's diagnostics, and clear them when the same command next succeeds.
