# TICKET-738 — Marley and Rusty in tabs

- **Ticket:** LOCAL #738 (feature, the Marley layout: agents anywhere)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** Chad, 2026-10-10: Marley "is an all around agent", and "give them a fixed home".
  Plan: [agents-anywhere shelf](../../design-notes/agents-anywhere-2026-10-10.md).
- **Status:** open

## Summary
`marley: talk to marley` and `marley: talk to rusty` open that agent's latest conversation in a
tab of the shown group, or start one. Each runs in a private folder of its own under Marley's data
folder, since neither edits a file or runs a command, so neither needs a project. On Zed's own agent
the tab's thread takes the agent's profile. Their conversations head the Threads page.

## Acceptance
From Home, with no project open, the command starts a Marley conversation in a tab that answers;
run again, it brings that conversation forward.
