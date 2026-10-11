# TICKET-771 — Codex 0.162 in the tested range

- **Ticket:** LOCAL #771 (chore, the finish line)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [771's spec](../../pipeline/completed/771-codex-0162-in-the-tested-range.spec.md)
- **Source ticket:** split from #750, #752 and #753's findings on 2026-10-11, when Chad asked to "spec all" of the survey list. Plan: [finish-line shelf](../../design-notes/finish-line-2026-10-11.md).
- **Status:** closed

## Summary
Marley reads Codex's state only through its App Server (#650), which it allows only on tested Codex versions (#648's table: 0.155.1 to 0.158.0). The box runs Codex 0.162.0, so #750, #752 and #753 reach nothing until the App Server's protocol is checked at 0.162 and the range widened. The check diffs the App Server's generated schema between the tested version and 0.162.

## Acceptance
With Codex 0.162 and `marley.codex_app_server` on, Marley shows a Codex session's state as it does on 0.158.
