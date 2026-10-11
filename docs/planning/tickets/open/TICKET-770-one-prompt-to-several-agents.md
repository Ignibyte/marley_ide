# TICKET-770 — One prompt to several agents

- **Ticket:** LOCAL #770 (feature, the finish line)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** not started; the design is drafted in [716's spec, Out](../../pipeline/queued/716-delegation-tools.spec.md)
- **Source ticket:** split from #716 (agents that start agents) on 2026-10-11, when Chad asked to "spec all" of the survey list. Plan: [finish-line shelf](../../design-notes/finish-line-2026-10-11.md).
- **Status:** open

## Summary
Agent checkboxes in the New Agent in Worktree prompt (#510): one prompt starts each chosen agent in a worktree of its own, each with its own row (T3 Code item 9).

## Acceptance
One prompt with two agents ticked starts two agents, each in its own worktree with its own row.
