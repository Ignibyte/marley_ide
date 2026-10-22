# TICKET-877 — Scenarios keep off the user's Claude Code sessions

- **Ticket:** LOCAL #877 (bug, the visual check's harness)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** [877-scenario-sessions.spec.md](../../pipeline/completed/877-scenario-sessions.spec.md)
- **Source ticket:** found running #645's scenario while planning #875 (2026-10-21): its rail
  click, measured before #873 turned the rail header's words into icons, landed on a Threads row,
  and the run's Marley started `claude --resume` of the user's real conversation in its terminal.
- **Size:** S
- **Status:** closed

## Summary

Every run points `CLAUDE_CONFIG_DIR` at its own folder, so a run's Threads view lists none of the
user's Claude Code sessions (Codex's home already is the run's, #725). The 13 scenarios that click
the rail header's old word positions (#644, #645, #656, #658, #659, #660, #662, #663, #672, #678
and others) are re-measured when the regression runs at the end (CONSTITUTION §7).

## Acceptance

A run's Threads view lists no Claude Code session of the user's.
