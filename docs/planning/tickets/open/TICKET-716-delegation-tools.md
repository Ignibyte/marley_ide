# TICKET-716 — Delegation tools on Marley's MCP server

- **Ticket:** LOCAL #716 (feature; size large; agents together, after the finish)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [716-delegation-tools.spec.md](../../pipeline/queued/716-delegation-tools.spec.md)
- **Source ticket:** [monocode-findings.md](../../intake/monocode-findings.md), item 3; Chad, 2026-10-09: "lets queue up the MonoCode findings for potential future use"; Chad, 2026-10-11: "13.) yes as long as its tracking and not lost"
- **Status:** open (deliberate: the agents-together batch, after the finish; waits on a rustal-harness request)

## Summary
An agent in a Marley terminal starts another agent: Claude Code or Codex, as a terminal or a
thread, in a new worktree or in its own folder in a split beside it. Every agent started this way
is a rustal-harness seat. The harness hosts and supervises it and delivers its first prompt once,
and its record says who started it. Marley serves the tools (`agent_start`, `agent_status`), asks
the person first, opens the view, draws the rows, and types a one-line pointer into the caller when
the work ends. The caller gets the pointer even if it was busy, Marley restarted, or both. A caller
whose terminal is gone gets a Needs you entry instead. Nothing is lost: the harness keeps the
record until the pointer is in.

The harness side is one request, the next after MREQ-009: labels on a seat set at `seat add` and
later, and a delivery state for a turn that ended. This ticket waits on it.

Proposed split (numbers not minted): **a drafted message for the user to review** (a tool that
leaves text in a card with Send, Edit and Discard) and **one prompt to several agents** (#510's
worktree prompt with agent checkboxes, T3 item 9). Both are Marley's alone and can go before this
one.

## Acceptance
An agent calls `agent_start`; after the person allows it, a seat starts with the prompt, its view
opens beside the caller, and its row names the caller. When the seat's turn ends, the caller gets
a one-line pointer once it is idle, after a Marley restart too, and `agent_status` returns the
result.
