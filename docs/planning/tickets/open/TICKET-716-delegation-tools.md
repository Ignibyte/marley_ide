# TICKET-716 — Delegation tools on Marley's MCP server

- **Ticket:** LOCAL #716 (feature; size medium)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** [monocode-findings.md](../../intake/monocode-findings.md), item 3; Chad, 2026-10-09: "lets queue up the MonoCode findings for potential future use"
- **Status:** open (deliberate: for future use)

## Summary
Tools for an agent to start another: a thread or a terminal agent, in a worktree or a split beside the caller; to draft a message for the user to review instead of sending it; and to hear back when the work it started ends. Through #704's agent-control modes and #703's log.

## Acceptance
An agent starts a second in a worktree, and its answer arrives when the second finishes; a drafted message waits for the user.
