# TICKET-589 — Remove a worktree without losing work

- **Ticket:** LOCAL #589 (feature, prong 2, worktree agents, review and merge; slice 2 of 2, after #511)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet (the outline is in `docs/planning/pipeline/completed/511-review-and-merge-a-worktree.notes.md`, "The split")
- **Source ticket:** TICKET-511, split at its planning: Chad, 2026-09-25, "lets do 1 through 8" (item 5, second half: review, merge, clean up). Orca report 02 §2.6 and §3 item 2.
- **Status:** open

## Summary
Once a worktree agent's branch is merged (#511), its worktree and branch stay until they are removed by hand. A worktree's row gets Remove. It refuses a worktree with work not committed unless the user confirms, stops the worktree's terminals, releases it from every open project, and removes it with Zed's `thread_worktree_archive::build_root_plan` and `remove_root`, which also check that Zed made it. Then the branch goes: `git branch -d`, and when git refuses, a Rust port of Orca's `src/shared/git-branch-cleanup.ts` proves the branch merged against `branch.<b>.base`, `origin/HEAD` and `HEAD` (`merge-tree --write-tree` equal to the target's tree, `git cherry` marking every commit `-`, or the branch's net `patch-id --stable` matching a commit on the target within 200), then `git update-ref -d refs/heads/<b> <expected-oid>`. A branch not proven merged stays, and a toast says so. The port keeps Orca's copyright and permission notice and names the file. A teardown hook, `TaskHook::RemoveWorktree` beside Zed's `CreateWorktree` (`crates/task/src/task_template.rs`, one variant), runs before the removal with a deadline (Orca's is two minutes); a failure or a timeout blocks the removal unless the user confirms. In a project the Rustal workflow merges, Remove is offered once the branch is proven merged, so a run's tree is never taken from under it.

## Acceptance
Remove takes a worktree away only with its work committed or the user's confirmation, stops its terminals first, deletes its branch only when git or the port proves it merged (squash merges included), and runs the teardown hook first, whose failure blocks the removal unless confirmed.
