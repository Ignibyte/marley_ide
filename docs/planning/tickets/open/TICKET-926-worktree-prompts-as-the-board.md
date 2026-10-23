# TICKET-926 — The worktree merge and remove prompts as the board words them

- **Ticket:** LOCAL #926 (feature, the Marley look, worktrees)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [926-worktree-prompts-as-the-board.spec.md](../../pipeline/queued/926-worktree-prompts-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/C-threads.md, board Worktrees)
- **Boards:** Worktrees
- **Size:** S
- **Status:** open

## Summary

Merging a worktree's branch runs its checks first, but the prompt only asks the question and
gives one line; it does not show what was checked. Removing a worktree says its branch "goes too
once its commits are in its base; otherwise it stays", which leaves the user to work out which
case this is. The Worktrees board lists the checks with a tick each, and the Remove prompt says
plainly whether this branch stays. This ticket makes both prompts say that.

## Acceptance

The Merge prompt lists its four checks, each ticked; the Remove prompt says how many changes go
with the folder and whether the branch stays or goes, and why. Shot in Marley Night and Marley
Day.
