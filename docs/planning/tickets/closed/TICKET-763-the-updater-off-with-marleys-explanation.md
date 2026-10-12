# TICKET-763 — The updater off, with Marley's own explanation

- **Ticket:** LOCAL #763 (chore, the finish line)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [763-the-updater-off-with-marleys-explanation.spec.md](../../pipeline/completed/763-the-updater-off-with-marleys-explanation.spec.md)
- **Source ticket:** split from #445 (slice 5) on 2026-10-11, when Chad asked to "spec all" of the survey list. Plan: [finish-line shelf](../../design-notes/finish-line-2026-10-11.md).
- **Status:** closed

## Summary
Zed's auto-update stays off for good with Marley's own explanation (`ZED_UPDATE_EXPLANATION`) until Marley has an update source of its own.

## Acceptance
Check for Updates says Marley updates through its install script, and nothing reaches zed.dev for an update.
