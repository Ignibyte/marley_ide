# TICKET-717 — Second opinion and handoff

- **Ticket:** LOCAL #717 (feature; size medium; agents together, after the finish)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [717-second-opinion-and-handoff.spec.md](../../pipeline/completed/717-second-opinion-and-handoff.spec.md)
- **Source ticket:** [monocode-findings.md](../../intake/monocode-findings.md), item 4; Chad, 2026-10-09: "lets queue up the MonoCode findings for potential future use"; Chad, 2026-10-11: "12.) yes, with a star new thread with context would be cool"
- **Status:** closed

## Summary
"Ask another agent" on a finished turn, a turn's diff or a worktree branch: a second agent opens in
a split with a recap of what happened and the files changed. Handoff moves a conversation to another
agent the same way.

The first slice is the part Chad starred: **New Thread with Context**. On an open thread, it starts
a new thread, on the same agent or another, in a split beside the old one. The new thread's message
box already holds the old one's context: a budgeted selection of its messages after T3 Code's
handoff order, framed as history, and a file with the whole conversation for what did not fit. The
person adds what to do next and sends. Zed's own "new thread from summary" works only for its own
agent and asks a model for the summary; this works for every thread agent and calls no model.

Later slices: the second opinion on a turn, a diff or a worktree branch; the same for terminal
agents (Claude Code and Codex in Marley's terminals), which needs their transcripts read.

## Acceptance
New Thread with Context on a thread opens a new thread on the chosen agent beside it, its message
box holding the old thread's context within the budget, unsent; after a send, the new agent answers
from that context.
