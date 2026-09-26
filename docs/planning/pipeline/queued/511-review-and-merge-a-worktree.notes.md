# Review and merge a worktree agent's branch, leaving the merge to the Rustal workflow where it runs — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-511-review-and-merge-a-worktree.md
- **Pipeline spec:** 511-review-and-merge-a-worktree.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-25: "lets do 1 through 8", item 5 of the list after the browser
  waves, second half: review a worktree agent's branch, merge it, clean up. On merging he said,
  the same day: "the rustal workflow will probably be where this lives. whatever default you want
  to set is fine but it should not impede the harness / workflow." The lead's brief turns that
  into: Review is the branch's diff against its base (Zed's `BranchDiff`); Merge defaults to a
  merge commit and never pushes; a project the Rustal workflow manages shows the state and leaves
  the merge to the workflow, and Marley needs a way to know such a project (a setting per project
  or a marker the workflow writes); removal with squash-aware branch cleanup (Orca's
  `src/shared/git-branch-cleanup.ts`, Zed's `thread_worktree_archive.rs`); a teardown hook.
- **Classification / tier:** feature, prong 2. The whole of it (review, merge, removal with the
  cleanup port, the teardown hook) is a week of work, so it splits: this slice is Review and Merge
  with the workflow check (M), slice 2 is Remove (M). One one-word Zed touch. Depends on #510's
  worktree rows and its `branch.<b>.base`.
- **Recall (§18.3):**
  - #510 (queued, `510-worktree-agents.spec.md`): the rows, `agent/<name>`, `--no-track` and
    `branch.<b>.base`. This slice reads what #510 writes, and nothing else.
  - L-claude-458-a-projects-folder-events-come-before-its-group-is-rekeyed-001: read the groups in
    a deferred refresh; the state refresh here is keyed on repository events and runs after it.
  - BF-changelog-hook-worktree-vs-staged-001 and its prevention rule: "clean" means what git
    reports for the paths that matter. For the merge, tracked changes on disk block it, untracked
    files do not (git itself refuses a merge that would overwrite one), hence
    `--untracked-files=no` in D3.
  - No ledger entry covers a local merge or `index.lock`; the ledgers were grepped for `git
    merge`, `merge-base`, `index.lock` and `no-optional-locks`.
  - Brain: no consultation run by this drafting agent; the Planner's `brain_ask` at promotion is
    owed.
- **Discovery (checked in the tree and on the box):**
  - `crates/git_ui/src/branch_diff.rs`: `BranchDiff` (48) over one `DiffMultibuffer` with
    `DiffBase::Merge`; `register` (72); `deploy_branch_diff` (80, the default branch through
    `repo.default_branch(true)`); `deploy_branch_diff_with_base_ref` (195, `pub(crate)`), which
    reuses an open diff of the same base and repository or builds one with `new_with_branch_base`;
    `editor` (435); the toolbar's Review Diff and Send Review buttons (880 to 905).
  - `crates/git_ui/src/project_diff.rs:67`: the `git: diff branch` action, `DeployBranchDiff`
    (`git::DiffBranch`), is `pub(crate)`, so Marley calls the function in D1, not the action.
  - `crates/project/src/git_store/diff_buffer_list.rs:27-32`: `DiffBase::{Head, Index, Staged,
    Merge { base_ref }}`.
  - `crates/project/src/git_store.rs`: `default_branch` (9566); `RepositoryEvent` (833) with
    `HeadChanged`, `BranchListChanged`, `GitWorktreeListChanged`; `GitStoreEvent::RepositoryUpdated`
    (851); `RepositorySnapshot::main_worktree_abs_path` (6261) for the main checkout's path.
  - `crates/git/src/repository.rs`: no merge command anywhere in the git layer (grepped); status
    with `--no-optional-locks` (3990).
  - `crates/zed_actions/src/lib.rs:623`, `ReviewBranchDiff`, and 643, `ResolveConflictsWithAgent`
    (Agent Panel paths Zed already has; neither is this slice's).
  - `crates/agent_ui/src/thread_worktree_archive.rs`: `build_root_plan` (112, only linked
    worktrees Zed recorded, inside `git.worktree_directory`), `remove_root` (216,
    `remove_worktree(path, true)`, which is `--force`), `persist_worktree_state` (499), for slice 2.
  - `crates/marley_browser/src/service.rs:133`, `util::command::new_command("systemd-run")`, the
    pattern for Marley's git spawns.
  - `crates/marley_workbench/src/rail.rs`: the row menus are `right_click_menu` with
    `ContextMenu::build` (the project header's at 1084 to 1110).
  - `/srv/stacks/rustal-workflow`: `src/config.rs:14` (`FILE = "workflow.toml"`),
    `src/commands/init.rs:70-100` (writes `workflow.toml` with `[project] slug`, the pre-commit shim,
    `core.hooksPath = .rw/hooks`), `src/commands/land.rs` (no merge). On this box
    `/srv/stacks/rustal-brain` has `workflow.toml` and `core.hooksPath=.rw/hooks`; no other repository
    under `/srv/stacks` has a `workflow.toml` but rustal-workflow's own.
  - `/srv/stacks/rustal-harness/docs/DECISIONS.md` (D107): "Rustal workflow, the Plan → Code →
    Test → Complete process", one of the ecosystem's five parts; "Agents run the workflow", and
    the harness keeps them running. Its enforcement today is `rw` in rustal-workflow. Marley's own
    repository follows the same four phases (CONSTITUTION §3) through its own hooks and
    `script/gates.sh`, with no marker file, so it keeps Marley's Merge unless Chad sets
    `git config marley.merge workflow` there.
  - Orca (MIT, read): `src/main/github/client/merge/merge-pr.ts:84` (no `--delete-branch`),
    `src/main/git/source-control/branch-compare.ts`, `src/shared/git-branch-cleanup.ts` (282 lines:
    candidate targets `branch.<b>.base`, `origin/HEAD`, `HEAD` at 48 to 64; the proofs at 109 to
    282), `src/main/git/worktree-branch-removal.ts` (190 lines).
- **Decisions:** D1 to D8 in the spec.

### The split
- **Slice 1, this ticket:** Review, Merge, the workflow check, the row's state.
- **Slice 2, a ticket of its own (numbered when filed), "remove a worktree without losing work":**
  - Remove on the row's menu: refuse a worktree with work not committed unless Chad confirms,
    stop its terminals (the rail knows them), release it from every open project and remove it
    with Zed's `build_root_plan` and `remove_root`, which also check that Zed made it.
  - Then the branch: `git branch -d`; when git refuses, Orca's proof in Rust against
    `branch.<b>.base`, `origin/HEAD` and `HEAD` (`merge-tree --write-tree` equals the target's
    tree; or `git cherry` marks every commit `-`; or the branch's net `patch-id --stable` equals a
    commit on the target within 200, whose merge also changes nothing), then
    `git update-ref -d refs/heads/<b> <expected-oid>`; not proven, the branch stays and a toast says
    so. The port keeps Orca's copyright and permission notice and names the Orca file.
  - The teardown hook: a `TaskHook::RemoveWorktree` beside Zed's `CreateWorktree`
    (`crates/task/src/task_template.rs:95`, a Zed touch of one variant) run before removal with a
    deadline (Orca: two minutes, `src/main/hooks.ts`), a failure or timeout blocking the removal
    unless Chad confirms.
  - In a workflow project, Remove is offered once the branch is proven merged, so the workflow's
    tree is never taken from under a run.

### Design
- **Approach.**
  - *Zed touch, `crates/git_ui/src/branch_diff.rs:195`:* `pub(crate) fn
    deploy_branch_diff_with_base_ref` becomes `pub`, with a `// Marley:` comment. Its arguments
    are all public types (`Workspace`, `Project`, `Repository`, `SharedString`, an optional
    `DiffBufferList`).
  - *`marley_workbench/src/worktree_git.rs` (new module, the git adapter):* async functions over
    `util::command::new_command("git")` with `current_dir` set and `GIT_TERMINAL_PROMPT=0`:
    `branch_state(main, branch)` reads `git config --get branch.<b>.base`, `git rev-list --count
    <base>..<b>`, `git config --get marley.merge` and whether `workflow.toml` parses with a
    `[project]` table (the `toml` crate, a workspace dependency); `merge_checks(main, worktree,
    branch, base)` runs D3's checks in order; `merge(main, branch)` runs the merge, and on a failure
    with conflicts reads `git diff --name-only --diff-filter=U` and runs `git merge --abort`.
    Every failure is a typed reason the UI shows as text.
  - *`marley_rail`:* `WorktreeSnapshot` gains `ahead: Option<u32>`, drawn on the second line
    (`agent/ok · 2 ahead of main`); nothing else in the pure model changes.
  - *`marley_workbench/src/rail.rs`:* a `worktree_states: HashMap<PathBuf, BranchState>` filled by
    a background task per project on `RepositoryUpdated` events (one run in flight per project, a
    change during a run schedules one more); the worktree row's `right_click_menu` built from the
    kept state; Review activates or opens the worktree's workspace (as #510's click) and calls
    `BranchDiff::deploy_branch_diff_with_base_ref` in it; Merge refuses unsaved buffers first
    (`Item::is_dirty` over the main checkout workspace's items, when it is open), then runs the
    checks, then `window.prompt` with Merge and Cancel, then the merge, reporting through a toast
    (success, the merge commit's short id) or `detach_and_prompt_err` (a refusal).
  - *`marley_workbench/Cargo.toml`:* `git_ui` and `toml`.
- **File manifest.** Zed crate: `crates/git_ui/src/branch_diff.rs`. Marley crates:
  `crates/marley_rail/src/marley_rail.rs`, `crates/marley_workbench/src/rail.rs`,
  `crates/marley_workbench/src/worktree_git.rs` (new), `crates/marley_workbench/src/marley_workbench.rs`
  (the module), `crates/marley_workbench/Cargo.toml`. Test phase:
  `script/e2e/511-review-and-merge-a-worktree.sh`.
- **Ledger row (`docs/marley/zed-touchpoints.md`, before the edit):**
  `crates/git_ui/src/branch_diff.rs`: `deploy_branch_diff_with_base_ref` is `pub` (#511). Why:
  Review opens a branch diff of a worktree against the base its branch recorded, and the only way
  in with a base is this function; the action compares with the default branch. On merge: keep it
  `pub`; if upstream renames it or adds a public way to open a diff against a base, move Marley
  there and drop the row.

### E2E plan
| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | setup: `main` with a commit pushed to a bare `origin`; worktree `ok` as #510 makes one, two commits in it; steps: right-click the `ok` row | `511-01-menu` |
| REQ-002 | the same menu: Merge 2 commits into main, and the row's `2 ahead of main` | `511-01-menu` |
| REQ-003 | steps: Review | `511-02-review` |
| REQ-004 | setup: `git worktree add -q $E2E_WORK/manual -b manual`; steps: right-click `manual`, then Review | `511-06-no-base` |
| REQ-005 | steps: Merge on `ok`, confirm; in the main terminal `git log --graph --oneline -4`; the run log prints `git ls-remote origin main` before and after | `511-04-merged`; the run log |
| REQ-006 | the same shot: the success names the merge commit, and `ok`'s row no longer counts commits ahead | `511-04-merged` |
| REQ-007 | steps: from the main checkout's terminal `echo x >> README`, then right-click `ok`, Merge; then `git checkout README` | `511-03-refused` (the refusal; `git status -s` unchanged) |
| REQ-008 | none: the other two checks (the main checkout off the base, the worktree dirty) are read in the review of the diff; P3 may add them to the run log with the adapter's messages | review |
| REQ-009 | setup: worktree `clash` with a commit to README's first line, then a commit on `main` to the same line; steps: Merge on `clash`, confirm; `git status -s` in the main terminal | `511-05-conflict` |
| REQ-010 | steps: `git config marley.merge workflow` in the main terminal, right-click `clash`; then `git config --unset marley.merge`, `printf '[project]\nslug = "repo"\n' > workflow.toml`, right-click `clash` again | `511-07-workflow`, `511-08-workflow-toml` |

Every refusal is checked for "changed nothing" with `git status -sb` and `git rev-parse main` in the
main terminal before and after, in the shots or the run log.

### Risks
- A merge while Zed writes the index (staging from the git panel) fails on `index.lock`; the
  refusal shows git's message and nothing else happens. Zed's background status does not take the
  lock (`--no-optional-locks`).
- A repository hook (pre-merge-commit, commit-msg) can stop the merge; it is reported with its
  output and the merge aborted, as a conflict is.
- The kept state can be stale for a moment after a change outside Marley's view (a commit from a
  terminal): the repository events refresh it, and Merge runs its checks again before it asks.
- A `workflow.toml` belonging to some other tool would count as the workflow's if it has a
  `[project]` table. `git config marley.merge marley` overrides it for that repository.
- With the merge done, the worktree and its branch stay until slice 2; the row stops counting
  commits ahead, and the branch shows in `git branch` as merged.
