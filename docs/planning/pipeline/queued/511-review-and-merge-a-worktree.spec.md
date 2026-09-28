---
pipeline_id: c5a7d13c-5321-421c-9308-a648c5c1db8e
ticket: docs/planning/tickets/open/TICKET-511-review-and-merge-a-worktree.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Review and merge a worktree agent's branch, leaving the merge to the Rustal workflow where it runs"
type: feature
slice: prong 2, worktree agents, review and merge, slice 1 of 2 (removal is slice 2); after #510
references: [docs/planning/pipeline/completed/510-worktree-agents.spec.md, docs/orca_architecture/02-worktrees-and-review.md]
---

## Title
A worktree's row in the rail gets Review, Zed's branch diff of the worktree against the base its
branch recorded, and Merge, a merge commit of the branch in the main checkout that never pushes;
where the Rustal workflow manages the project, the row shows the branch's state and leaves the
merge to the workflow.

## Scope
### In
- **The row's menu** (right-click on a worktree's row, #510): Review; then one line about merging,
  which is "Merge N commits into <base>…", "Nothing to merge into <base>", "No base recorded" or,
  in a project the workflow manages, "N commits ahead of <base>: the Rustal workflow merges here".
- **The row's second line** adds the count, `agent/ok · 2 ahead of main`, while the branch has
  commits its base lacks.
- **The branch's state** (the base from `git config branch.<b>.base`, the commits ahead with
  `git rev-list --count <base>..<branch>`, the workflow check) is read by a background git task in
  the main checkout when the repository reports a head, branch or worktree change, and kept per
  worktree; the rail's rebuild reads the kept state and runs no git (D7).
- **Review** shows the worktree's workspace (opening it as #510's row click does when it is not
  open) and opens Zed's `BranchDiff` there against the recorded base, "Changes since <base>"
  (D1). With no base recorded, against the repository's default branch
  (`Repository::default_branch`), as Zed's own `git: diff branch` (`git::DiffBranch`) does.
- **Merge** (D2 to D4): offered only for a branch that records its base, in a project the workflow
  does not manage. It checks, in order, and refuses with the failed check named: the main checkout
  is on the base; the main checkout has no change not committed (tracked files on disk, and no
  unsaved buffer in its open workspace); the worktree has no change not committed; the branch has
  a commit to merge. Then a confirmation names the count, the base, the main checkout's path and
  "nothing is pushed". Then `git merge --no-ff --no-edit <branch>` in the main checkout. A conflict
  is aborted (`git merge --abort`) and its files named. Success names the merge commit.
- **The workflow's projects** (D5): `git config marley.merge` decides when it is set (`workflow`
  or `marley`); unset, a `workflow.toml` with a `[project]` table at the main checkout's root (the
  file `rw init` writes) means the workflow merges. Such a project gets Review and the state
  line, never Merge.

### Out (explicitly deferred)
- **Slice 2, a ticket of its own (numbered when filed): Remove.** Releasing the worktree from
  every open project and removing it (Zed's `thread_worktree_archive::build_root_plan` and
  `remove_root`), then deleting its branch only when git proves it merged, squash merges
  included (a Rust port of Orca's `src/shared/git-branch-cleanup.ts`: `merge-tree --write-tree`,
  `cherry`, a patch-id match, then `update-ref -d` with the expected commit), and a teardown hook
  that runs before removal and blocks it when it fails. The notes carry its outline.
- Handing a conflict to the worktree's agent as a prompt: it needs #522's sender to an idle
  agent; until then the conflict's files are named and the main checkout is left clean.
- Squash, rebase or fast-forward merges, and "update from base" in the worktree.
- Pull requests and anything `gh` (report 02 §3 item 12), and any push.
- A merge started by the workflow from Marley: Marley only stays out of its way here.
- Showing how far the base moved since the branch was made (Orca's drift).

## Reference (§20)
Upstream Zed for Review: `BranchDiff`, the "Changes since {branch}" item over
`DiffBase::Merge { base_ref }`, kept as it is and opened with the recorded base instead of the
default branch. Zed has no local merge; the merge follows `git-merge(1)` and the checklist of Orca
report 02 §3 item 3 (Orca itself merges only through `gh pr merge`, §2.11). The workflow check
follows the Rustal workflow's own files (`/srv/stacks/rustal-workflow`). Warp: N/A, a git and
workspace behavior outside Warp's terminal; the once-over ruled worktree review out as planned
here.

### Prior art
- **Behavior maps and reports.** Orca report 02: §2.8 (a branch diff is
  `merge-base(base, HEAD)..HEAD`, `src/main/git/source-control/branch-compare.ts`), §2.11 (no
  local merge anywhere; `gh pr merge` without `--delete-branch`,
  `src/main/github/client/merge/merge-pr.ts:84`), §3 item 3 (require the main checkout clean and
  on the base, `--no-ff`, abort a conflict), §2.6 and §3 item 2 (removal, slice 2). The Rustal
  workflow: `rw init` writes `workflow.toml` (the name is `config::FILE`,
  `rustal-workflow/src/config.rs:14`) with a `[project]` table and a `slug`, writes
  `.rw/hooks/pre-commit` and sets `core.hooksPath` (`src/commands/init.rs:70-100`); it has no merge
  verb today (`src/commands/land.rs` exports and accepts a landed commit). rustal-brain is enrolled
  that way on this box; Marley's own repository and the harness are not.
- **Published material.** `git-merge(1)` (`--no-ff`, `--no-edit`, `--abort`), `git-config(1)`
  (`branch.<name>.*` keys, slashes allowed in the subsection), `git-rev-list(1)` (`--count`).
- **Code we already ship.** `crates/git_ui/src/branch_diff.rs`: `deploy_branch_diff_with_base_ref`
  (195, `pub(crate)`) opens a `BranchDiff` for a repository and a base, `deploy_branch_diff` (80)
  for the default branch, and the toolbar's Review Diff sends the diff to an Agent Panel thread
  (`ReviewBranchDiff`, `crates/zed_actions/src/lib.rs:623`). `DiffBase::Merge { base_ref }`
  (`crates/project/src/git_store/diff_buffer_list.rs:27-32`). Zed's git layer has no merge;
  its status runs with `--no-optional-locks` (`crates/git/src/repository.rs:3990`), so Zed's
  background status does not hold the index lock a merge takes. Marley spawns non-PTY programs with
  `util::command::new_command` (`crates/marley_browser/src/service.rs:133`). #510's worktree rows
  and its `branch.<b>.base`.

## UI proof
UI-AFFECTING. `script/e2e/511-review-and-merge-a-worktree.sh` (`compositor sway`: it right-clicks
rows). Setup: a scratch repository on `main` with a commit, pushed to a bare `origin`; worktrees
made the way #510 makes them (`git worktree add --no-track -b agent/<n> ...` and
`git config branch.agent/<n>.base main`): `ok` with two commits, and `clash` with a commit to a line
that a later commit on `main` also changes; a worktree `manual` with no base; the scenario's HOME.
Shots: `511-01-menu` (the `ok` row's menu: Review, Merge 2 commits into main; the row reads
`2 ahead of main`), `511-02-review` (Changes since main in `ok`'s workspace, the two commits'
changes), `511-03-refused` (the main checkout made dirty: the refusal names it; `git status`
unchanged), `511-04-merged` (after the confirmation: the success names the merge commit;
`git log --graph --oneline` shows it; the row without a count; `git ls-remote origin main` in the
run log equal to setup's), `511-05-conflict` (Merge on `clash`: the refusal names the conflicting
file, `git status` clean), `511-06-no-base` (the `manual` row's menu: Review and "No base
recorded"), `511-07-workflow` (after `git config marley.merge workflow`: `clash`'s menu shows its
state and the workflow line, no Merge), `511-08-workflow-toml` (the key unset and a `workflow.toml`
written: the same).

## Locked-In Decisions
- D1 — Review is Zed's `BranchDiff` in the worktree's own workspace, against the recorded base:
  `deploy_branch_diff_with_base_ref` becomes `pub` (a one-word Zed touch), called with the
  worktree's repository and `branch.<b>.base`. With no base recorded, the same call takes the
  repository's default branch (`Repository::default_branch`), as `git: diff branch` does. The
  diff's review comments work there, and #522 sends them to the worktree's agent.
- D2 — Merge makes a merge commit and never pushes (Chad, 2026-09-25):
  `git merge --no-ff --no-edit <branch>` in the main checkout, run with `util::command` and
  `GIT_TERMINAL_PROMPT=0`; the repository's hooks run as they would for Chad. No code path in this
  ticket runs `git push`.
- D3 — Every check fails closed and changes nothing: the main checkout on the base, the main
  checkout clean on disk (`git status --porcelain --untracked-files=no`) with no unsaved buffer in
  its open workspace, the worktree clean (its work not committed would be left out of the merge),
  at least one commit to merge, and Chad's confirmation.
- D4 — A conflict aborts: the conflicting paths are read (`git diff --name-only
  --diff-filter=U`), then `git merge --abort` runs, and the refusal names them. The main checkout
  is as it was.
- D5 — The workflow owns the merge where it runs: `git config marley.merge` (`workflow` or
  `marley`) decides when set, else a `workflow.toml` with a `[project]` table at the main
  checkout's root means the workflow's. Both live in the clone and read the same from every
  worktree; the workflow can set the key with one command, and an `rw` project needs nothing.
  Rejected: a Marley setting per project, which lives outside the clone, so the same repository
  opened on another profile would merge.
- D6 — Merge is offered only for a branch that records its base, which Marley writes when it
  makes a worktree (#510). A worktree the harness or a person made without it gets Review and no
  Merge, so Marley never merges a branch it did not start.
- D7 — The state comes from git in the background, on the repository's `HeadChanged`,
  `BranchListChanged` and `GitWorktreeListChanged`, and is kept per worktree for the rows and the
  menu; the menu, built synchronously, reads the kept state.
- D8 — Slice 2 is removal (squash-aware branch cleanup, the teardown hook). Merging leaves the
  worktree and its branch in place until then.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user right-clicks a worktree's row, the menu shall offer Review. | Shot `511-01-menu` |
| REQ-002 | WHERE a worktree's branch records its base in a project the workflow does not manage, WHILE the branch has commits its base lacks, the menu shall offer Merge with their count, and the row shall show the count. | Shot `511-01-menu` |
| REQ-003 | WHEN the user chooses Review, the system shall show the worktree's workspace with Zed's branch diff against the recorded base, titled Changes since <base>. | Shot `511-02-review` |
| REQ-004 | WHERE a worktree's branch records no base, Review shall compare with the repository's default branch, and the menu shall offer no Merge. | Shot `511-06-no-base` |
| REQ-005 | WHEN the user confirms Merge, the system shall make a merge commit of the branch on the base in the main checkout, and shall push nothing. | Shot `511-04-merged`; the run log's `git ls-remote origin main` |
| REQ-006 | WHEN a merge succeeds, the system shall name the merge commit, and the row shall stop counting commits ahead. | Shot `511-04-merged` |
| REQ-007 | IF the main checkout has changes not committed, THEN the system shall refuse to merge, say so, and change nothing. | Shot `511-03-refused` |
| REQ-008 | IF the main checkout is not on the base, or the worktree has changes not committed, THEN the system shall refuse to merge and name the check that failed. | Review of the checks' order and messages |
| REQ-009 | IF the merge stops on a conflict, THEN the system shall abort it, name the conflicting files, and leave the main checkout as it was. | Shot `511-05-conflict` |
| REQ-010 | WHERE `git config marley.merge` is `workflow`, or is unset while the main checkout's root holds a `workflow.toml` with a `[project]` table, the menu shall show the branch's state and that the Rustal workflow merges the project, and shall offer no Merge. | Shots `511-07-workflow`, `511-08-workflow-toml` |

## Phase Plan
- **P1 Plan** — this spec; the design and the E2E plan in the notes. #510 ships first.
- **P2 Code** — the touchpoint row, then `deploy_branch_diff_with_base_ref` made `pub`; the git
  adapter (state, checks, merge, abort) and the workflow check in `marley_workbench`; the row's
  state in `marley_rail`; the menu; fmt and clippy clean; a review of the diff (§18.1).
- **P3 Test** — write and run `script/e2e/511-review-and-merge-a-worktree.sh`, read every shot;
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_workbench.md` and `marley_rail.md`,
  the plan's prong 2, the touchpoint row checked, ledger capture, close, archive, commit; file
  slice 2's ticket.
