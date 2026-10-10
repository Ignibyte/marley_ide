# TICKET-726 — Plan review in a center tab

- **Ticket:** LOCAL #726 (feature; size large, in slices)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** [plannotator-findings.md](../../intake/plannotator-findings.md), item 1; Chad, 2026-10-09: "take a look at https://github.com/backnotprop/plannotator and see what may be to add to the list"
- **Status:** open (deliberate: for future use)

## Summary
Catch Claude Code's `ExitPlanMode` (a `PermissionRequest` hook in the shared plugin, which needs a request to rustal-harness) and Codex's plan from the App Server (#650), and show the plan as rendered Markdown in a center tab: select text to comment, delete or label it, then Approve or Send Feedback. The plan also waits in Needs you.

## Acceptance
A plan from Claude Code opens in the tab; a comment and Send Feedback reach the agent, and Approve lets it go on.
