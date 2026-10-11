# TICKET-715 — A usage meter, and resume at the reset

- **Ticket:** LOCAL #715 (feature; size medium)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [715-usage-meter-and-resume-at-reset.spec.md](../../pipeline/completed/715-usage-meter-and-resume-at-reset.spec.md)
- **Source ticket:** [monocode-findings.md](../../intake/monocode-findings.md), item 2; Chad, 2026-10-09: "lets queue up the MonoCode findings for potential future use". [T3 Code survey](../../../t3code_architecture/README.md), items 2 and 12 (report 01, items 1 and 6); Chad, 2026-10-11, on usage limits as a state with Continue at Reset and a meter: "2.) yes".
- **Status:** closed

## Summary
A limited agent stops looking failed. A Codex on its own App Server that hits its usage limit
reads `limited · resets in 1 h 35 m` on its row and in Needs you, with a Continue at Reset button
that sends "Continue where you left off." through the App Server when the window reopens. A
setting, off by default, arms it for every limit. Claude Code and Codex rows carry a usage meter:
the account's most-used window on a line of the row, every window in its tooltip, as #640 draws a
harness session's. Codex's windows come from the updates its App Server already sends; Claude
Code's from the shared plugin's reports (#709).

This ticket is the first of two slices. The second, which needs a number, gives Claude Code the
same limited state and Continue at Reset once it is checked on the box whether Claude Code's TUI
ends a turn at its usage limit or holds it.

## Acceptance
Each Claude Code and Codex row shows its account's most-used window and its reset. A Codex at its
limit reads limited with the reset on its row and in Needs you, and continues at the reset when
asked to, or by itself when the setting is on.
