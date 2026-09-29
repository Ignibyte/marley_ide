---
pipeline_id: 57812f9e-60db-4246-a8e4-7fc7f2e11990
ticket: docs/planning/tickets/open/TICKET-589-remove-a-worktree-without-losing-work.md
status: Phase 3 — Complete PASS
title: Remove a worktree without losing work
type: feature
slice: prong 2, worktree agents, review and merge, slice 2 of 2 (after #511)
references: [TICKET-510, TICKET-511, TICKET-590, TICKET-591]
---

## Title
A worktree's row gets Remove: after a confirmation that names any work not committed, Marley
closes the worktree's workspace and its terminals, removes the worktree with Zed's own archive
code, which checks that Zed made it, and deletes its branch when git or a merged proof shows its
commits are in the base, so a finished worktree agent leaves nothing behind and unmerged work is
never lost.

## Scope
### In
- Remove on a linked worktree's row menu, after Review and Merge. In a repository the Rustal
  workflow merges (#511's `MergeOwner::Workflow`), Remove shows only when the branch has no commit
  its base lacks; otherwise the menu says the workflow merges it first.
- Always a prompt: "Remove <name>?", and when the worktree has changes git has not committed
  (untracked files included), "Remove <name> and its N uncommitted changes?" with Remove Anyway.
- The worktree's own workspace removed from the window (Zed asks to save its unsaved files; a
  refusal stops Remove), which drops its terminals and stops their processes.
- `agent_ui::thread_worktree_archive::build_root_plan` (built before the workspace goes) and
  `remove_root`: Zed's check that it made the worktree, the release from every open project,
  `git worktree remove --force`, and a rollback on failure.
- The branch: `git branch -d`; when git refuses, a merged proof against the recorded base,
  `origin/HEAD` and the main checkout's `HEAD` (the branch is an ancestor of the target; `git
  merge-tree --write-tree` of the two gives the target's own tree; or `git cherry` marks every
  commit `-`), then `git update-ref -d refs/heads/<b> <oid read first>` and the branch's config
  section (`base`, `marleySlot`) removed. Not proven: the branch stays and the toast says so.
- A toast naming what was removed and what was kept.

### Out (explicitly deferred)
- The teardown hook (`TaskHook::RemoveWorktree`): TICKET-591.
- A worktree that no open project holds: `build_root_plan` has nothing to plan, so Remove says to
  open it first or use `git worktree remove`.
- Orca's third proof (the branch's net `patch-id` against the target's last 200 commits), and a
  `fetch --prune` before the proof; `merge-tree` covers a squash merge whose lines the target has
  not changed since.
- Terminals of other workspaces whose working folder is inside the worktree.

## Reference (§20)
Upstream Zed: the sidebar's thread archive (`crates/sidebar/src/sidebar.rs`,
`archive_worktree_roots`) removes a Zed-made worktree through
`crates/agent_ui/src/thread_worktree_archive.rs` (`build_root_plan`, then the workspaces removed,
then `remove_root`). Marley calls the same two functions in the same order, from a row's menu, with
no thread and no saved state, and adds the branch step Zed has no need for (its worktrees are
detached). The branch step follows the behavior Orca's report describes
(`docs/orca_architecture/02-worktrees-and-review.md` §2.6 step 7), written from that description.

### Prior art
- The behavior maps: Orca's removal (§2.6): refuse dirty without Force Delete, kill the worktree's
  processes, remove, then `branch -d` or a merged proof and a compare-and-swap `update-ref -d`, the
  branch kept and listed when not proven.
- The code we ship: `thread_worktree_archive` (`build_root_plan`, `remove_root`,
  `verify_created_by_zed`, `rollback_root`), all `pub`, and `marley_workbench` already depends on
  `agent_ui`; `MultiWorkspace::remove` with `RemovalIntent::KeepProject`, as the sidebar removes an
  archived worktree's workspaces; the rail's `merge_worktree` (#511) for the prompt and task shape;
  `worktree_git`'s runner. `GitRepository::delete_branch` runs `branch -d` but turns git's refusal
  into an error string, so the adapter runs it to read the exit code.

## Locked-In Decisions
- D1 — Removal goes through Zed's `remove_root`, not Marley's own `git worktree remove`: Zed checks
  it made the worktree, releases it from every project and rolls back on failure.
- D2 — `remove_root` deletes the folder with `--force`, so Marley's prompt is the guard: it always
  asks, and names the uncommitted changes (`git status --porcelain`, untracked included) when
  there are any.
- D3 — A branch goes only when git (`branch -d`) or the proof shows its commits are in a target,
  and then by compare-and-swap on the oid read before the proof, so a branch that moved stays.
- D4 — The merged proof is written from the behavior Orca's report describes, not ported from
  Orca's file, so no notice travels with it; its patch-id leg waits.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user opens a linked worktree's row menu, the menu shall offer Remove, except in a repository the workflow merges while the branch has commits its base lacks, where it shall say the workflow merges it first. | Review |
| REQ-002 | WHEN the user chooses Remove, Marley shall ask first, naming the number of changes not committed when there are any, and shall change nothing on Cancel. | Review |
| REQ-003 | WHEN the user confirms, Marley shall remove the worktree's workspace (its terminals stopped, Zed asking about unsaved files, a refusal stopping Remove) and then remove the worktree through `remove_root`. | Review |
| REQ-004 | IF Zed did not make the worktree, or no open project holds it, THEN Marley shall remove nothing and say why. | Review |
| REQ-005 | WHEN the worktree is gone, Marley shall delete its branch when `git branch -d` succeeds or the proof shows it merged into the recorded base, `origin/HEAD` or the main checkout's `HEAD`, and shall keep it otherwise, the toast saying which. | Review |
| REQ-006 | WHEN Marley deletes a branch after the proof, it shall delete it only if it still points at the commit read before the proof, and shall remove its `branch.<b>` config. | Review |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `worktree_git` (`uncommitted`, `delete_branch` and the proof); the rail's menu
  entry and `remove_worktree`; a review of the diff; `script/gates.sh --diff` green (no tests, §7).
- **P3 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit, push, install.
