# TICKET-591 — A teardown hook before a worktree is removed

- **Ticket:** LOCAL #591 (feature, prong 2, worktree agents, review and merge; after #589)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet
- **Source ticket:** TICKET-589, split at its planning (2026-09-29); #511's notes, "The split"; Orca report 02 §2.6 step 2 (`scripts.archive`, `src/main/hooks.ts`).
- **Status:** open

## Summary
A worktree's dev database, containers or ports outlive it when Remove (#589) takes it away. A `TaskHook::RemoveWorktree` beside Zed's `CreateWorktree` (`crates/task/src/task_template.rs`, one variant, a Zed touch with its ledger row) marks tasks Marley runs in the worktree before Remove goes on, and waits for them with a deadline (Orca's is two minutes). A task that fails or runs past the deadline stops the removal unless the user confirms; the task's terminal stays open so its output can be read.

## Acceptance
Remove runs the worktree's `remove_worktree` tasks first and waits up to two minutes; a failure or a timeout stops the removal unless the user confirms, and a worktree with no such task is removed as before.
