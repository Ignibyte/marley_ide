# TICKET-934 — Agent Activity lists Rusty's calls

- **Ticket:** LOCAL #934 (feature, the Marley look, Agent Activity)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [934-agent-activity-lists-rustys-calls.spec.md](../../pipeline/completed/934-agent-activity-lists-rustys-calls.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/F-tools.md, board Tools)
- **Boards:** Tools
- **Size:** M
- **Status:** closed

## Summary

Agent Activity lists only the calls agents make to Marley's own tools. The Tools board also lists
Rusty's: an `R` row for `brain_decide` that says the page was written. Marley never sees a call to
Rusty's server: Rusty's entry runs on Claude Code, whose own process talks to Rusty, and a Claude
Code in a terminal does the same. What Marley does see is what Claude Code reports of each call:
the Agent Panel thread's tool call (the adapter names the tool `mcp__rusty__<tool>`) and, in a
terminal, the plugin's hook events. This ticket lists those calls in Agent Activity beside
Marley's, with the agent's tile, what the call acted on, and its result or error.

## Acceptance

A call to one of Rusty's tools that changes something, made by Claude Code in the Agent Panel
(Rusty's entry or another thread) or in a Marley terminal, is a row in Agent Activity with its
result or error; shot in Marley Night and Marley Day.
