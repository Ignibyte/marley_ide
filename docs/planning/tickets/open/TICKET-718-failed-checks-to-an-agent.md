# TICKET-718 — Failed checks to an agent, then a GitHub inbox

- **Ticket:** LOCAL #718 (feature; size medium, then large)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** [monocode-findings.md](../../intake/monocode-findings.md), item 5; Chad, 2026-10-09: "lets queue up the MonoCode findings for potential future use"
- **Status:** open (deliberate: for future use)

## Summary
#531 already polls `gh` for the PR chip. Show a PR's checks there, and "Send failures to agent" with the failed logs. Later, an inbox of issues and PRs whose "Start work" opens a worktree agent on the issue.

## Acceptance
A PR with a failed check offers to send it to an agent, which gets the failing log.
