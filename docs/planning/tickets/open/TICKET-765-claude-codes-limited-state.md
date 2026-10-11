# TICKET-765 — Claude Code's limited state and Continue at Reset

- **Ticket:** LOCAL #765 (feature, the finish line)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** not started; the design is drafted in [715's spec, Out](../../pipeline/queued/715-usage-meter-and-resume-at-reset.spec.md)
- **Source ticket:** split from #715 (the usage meter and resume at reset, slice 2) on 2026-10-11, when Chad asked to "spec all" of the survey list. Plan: [finish-line shelf](../../design-notes/finish-line-2026-10-11.md).
- **Status:** open

## Summary
#715 gives Codex a limited state and Continue at Reset; this gives Claude Code the same. It starts with a check on the box: whether Claude Code's TUI ends a turn at its usage limit (a `StopFailure` with `rate_limit`, then the shared plugin's report with the windows) or holds it. If it ends the turn, the reset comes from the seat's window and the continuation is typed into the idle terminal.

## Acceptance
A Claude Code seat at its limit shows `limited · resets in …` and Continue at Reset types the continuation once the window reopens.
