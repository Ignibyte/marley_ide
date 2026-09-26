# TICKET-510 — Worktree agents: an agent on its own branch and worktree

- **Ticket:** LOCAL #510 (feature, prong 2)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/510-worktree-agents.spec.md
- **Source ticket:** Chad, 2026-09-25: "lets do 1 through 8" (item 5 (first half) of the list after the browser waves)
- **Status:** open

## Summary
Two agents in one checkout trample each other's files. The project's + starts an agent in a new git worktree on its own branch; the worktree shows as its own project in the rail, and its dev servers get their own ports, so several agents work in parallel.

## Acceptance
The + menu's worktree entry creates a worktree and branch, opens it as a project in the rail and starts the agent there.
