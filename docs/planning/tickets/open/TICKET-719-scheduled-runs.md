# TICKET-719 — Scheduled runs

- **Ticket:** LOCAL #719 (feature; size medium to large)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** [monocode-findings.md](../../intake/monocode-findings.md), item 6; Chad, 2026-10-09: "lets queue up the MonoCode findings for potential future use"
- **Status:** open (deliberate: for future use)

## Summary
Recurring prompts for Rusty or the Marley agent in a background session (hourly, daily, weekdays), each run kept, reporting in Needs you only when it found something. Per Chad's 2026-10-02 call that agent hosting is the harness's, the scheduler may belong in rustal-harness, with Marley showing the runs.

## Acceptance
A daily run reports in Needs you only on a finding, and its history lists every run.
