# TICKET-511 — Review and merge a worktree agent's branch

- **Ticket:** LOCAL #511 (feature, prong 2)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/511-review-and-merge-a-worktree.spec.md
- **Source ticket:** Chad, 2026-09-25: "lets do 1 through 8" (item 5 (second half) of the list after the browser waves)
- **Status:** open

## Summary
Once a worktree agent is done, its branch has to be reviewed, merged and cleaned up by hand. A worktree project's row offers Review, the branch's diff against its base, and Merge, which merges the branch into the base and removes the worktree and its project from the rail.

## Acceptance
Review shows the branch's diff against its base; Merge merges it and removes the worktree, and the row goes.
