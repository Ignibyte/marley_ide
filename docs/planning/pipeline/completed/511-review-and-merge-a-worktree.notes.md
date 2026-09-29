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

### Promotion (2026-09-29)
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (cargo busy with #586's release install,
  so no cargo ran and no crate or scenario was edited); recall ✓; promote ✓ (the pair to
  `active/`, the backlog row removed, the ticket in progress); the seams re-verified (Explore, over
  `235046907a`) ✓; spec and design updated ✓.
- **Recall, added:** #560's `worktree_git.rs` (`git(dir, args)` with `MARLEY_GIT`, Zed's four
  flags and `GIT_TERMINAL_PROMPT=0`; `recorded_base`; `summary`) and its drift cache (runs per main
  checkout, a second after the tips move, only while the rail shows and for a trusted repository);
  L-claude-560-merge-tree-reports-conflicts-with-exit-one-001; #587's in-window prompt and
  notifications. The brain (consultation d55397c303c748c8b499e9641aa8e669): nothing on this seam.
- **What the code says now** (the Explore report, 2026-09-29):
  - `deploy_branch_diff_with_base_ref` (`crates/git_ui/src/branch_diff.rs:195-203`,
    `pub(crate)`): `(workspace, project, intended_repo, base_ref, Option<DiffBufferList>, window,
    cx)`; it reuses an open diff of the same base and repository, titles it "Changes since
    {base_ref}" (478-480), and compares `merge-base(base, HEAD)..HEAD` of `intended_repo`, which
    must be the worktree's own repository. No `pub` way to a diff against a given base exists.
    `git_ui` is a dependency of `marley_workbench` already (#509).
  - `render_worktree_row` (`rail.rs:2585-2616`) is `row_card` with the drift chip and a left
    click (`open_row` → `open_worktree`, 1963-1988, which for a worktree not open calls
    `worktree_service::handle_switch_worktree`, returning nothing). Menus: the terminal row's
    `right_click_menu` with a weak `Rail` (2725-2739); `ContextMenu::label` for a line that is not
    a choice. gpui's `on_click` is the left button only.
  - `WorktreeEntry.repository` is the main checkout's repository, or the first open member's; the
    worktree's own comes from `member_git(member)` once its workspace is open.
  - `DriftState { base, tips, drift }` loses whether the base was recorded; nothing runs a drift
    read when `marley.merge` or `workflow.toml` changes.
  - Nothing in Marley or Zed runs `git merge`; `Project::dirty_buffers` (`project.rs:6327-6337`)
    is the unsaved-buffer check (`Item::is_dirty` counts a belled terminal).
  - The model for the confirmation: `browser::clear_project_browser_data` (`browser.rs:8133-8176`,
    `window.prompt` with `PromptLevel::Warning`, then a toast); Linux prompts draw in the window,
    so a scenario answers with Return or Escape.
  - The Rustal workflow: `workflow.toml` (`rustal-workflow/src/config.rs:14`) with `[project]
    slug = "…"`; `rw` has no merge verb. `toml = "0.8"` is a workspace dependency, not yet
    `marley_workbench`'s.
  - The harness right-clicks with `click <x> <y> right` (`e2e.sh:476-484`), as 581 does.
- **The design as promoted** (overriding the drafted design below it):
  - *`crates/marley_workbench/src/worktree_git.rs`:* `summary` counts `--left-right --count
    <branch_tip>...<base_tip>` (ahead, behind); `Drift` gains `ahead`. New:
    `merge_owner(main) -> MergeOwner { Marley, Workflow }` (`config --get marley.merge`, else the
    root's `workflow.toml` parsed for a `[project]` table, read off the main thread by the caller's
    background task); `merge_checks(main, worktree, branch, base) -> Result<u32, Refusal>` (the
    checks in D3's order, `symbolic-ref --short HEAD`, `status --porcelain
    --untracked-files=no` in each checkout, `rev-list --count base..branch`); `merge(main, branch)
    -> Result<String, MergeFailure>` (`merge --no-ff --no-edit`; on a failure the conflicted paths
    from `diff --name-only --diff-filter=U` and `merge --abort` whenever `MERGE_HEAD` is there;
    success, `rev-parse --short HEAD`).
  - *`crates/marley_rail/src/marley_rail.rs`:* `DriftSnapshot.ahead`; `words()` none while
    `ahead` is 0; `ahead_words()`, `2 ahead of main`.
  - *`crates/marley_workbench/src/rail.rs`:* `DriftState` gains `recorded: bool` and
    `owner: MergeOwner`; `read_drifts` fills them; `render_worktree_row` joins the ahead words to
    the second line and gains a `right_click_menu` (Review; then a label or Merge), built from the
    kept state through a weak `Rail`; `review_worktree` (open it first when it is not open, a
    `pending_review` taken at the refresh that finds its workspace, for 30 s, then
    `deploy_branch_diff_with_base_ref` in that workspace with its own repository);
    `merge_worktree` (the unsaved-buffer check on the main checkout's open workspace, the checks
    in the background, the prompt, the merge, a toast or `detach_and_prompt_err`).
  - *Zed touch:* `crates/git_ui/src/branch_diff.rs:195`, `pub(crate)` to `pub` with a
    `// Marley:` comment; its ledger row first.
  - *`crates/marley_workbench/Cargo.toml`:* `toml`.
- **File manifest** (as promoted): `crates/git_ui/src/branch_diff.rs` (Zed, one word);
  `crates/marley_workbench/src/worktree_git.rs`, `crates/marley_workbench/src/rail.rs`,
  `crates/marley_workbench/Cargo.toml`, `crates/marley_rail/src/marley_rail.rs` (Marley). Test:
  `script/e2e/511-review-and-merge-a-worktree.sh`, `script/e2e/golden`.
- **E2E plan** (as promoted): the drafted plan below, with the worktrees made as 560's scenario
  makes them, `MARLEY_GIT` naming a logging wrapper so the run log shows no `push`, the
  confirmation answered with Return, and `511-04-merged` also showing the merged row with no
  drift chip (REQ-011). 560's golden check on the chip's git (`config`, `merge-base`, `rev-list`,
  `merge-tree`) still holds: the count stays `rev-list`, and the owner is `config` and a file.

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

## Phase 2 — Code
- **Checklist** (no task tool): the ledger row for `crates/git_ui/src/branch_diff.rs` ✓, then its
  one-word change ✓; `worktree_git.rs` ✓; `marley_rail` ✓; `rail.rs` ✓; `Cargo.toml` ✓; fmt ✓;
  clippy ✓ (`just clippy marley_rail marley_workbench git_ui`, all targets: six findings on the first run, `single_match_else`, `semicolon_if_nothing_returned` and four `needless_pass_by_ref_mut` on `review_worktree` and `merge_worktree`, fixed at the source; clean on the second); the review of the diff ✓.
- **What was built:**
  - *Zed touch:* `BranchDiff::deploy_branch_diff_with_base_ref` is `pub`, with a `// Marley:`
    comment; its row in `docs/marley/zed-touchpoints.md` came first.
  - *`worktree_git.rs`:* `summary` counts both ways in one `rev-list --left-right --count
    <branch_tip>...<base_tip>`, and `Drift` carries `ahead`. `MergeOwner { Marley, Workflow }` and
    `merge_owner(main, fs)`: `git config --get marley.merge` when it is `workflow` or `marley`
    (another value is logged and ignored), else the root's `workflow.toml` read through Zed's `Fs`
    and parsed with `toml` for a `[project]` table. `ready_to_merge(main, worktree, branch, fs)`:
    the owner, the recorded base (none: refused, Marley did not start the branch), a base that is a
    commit refused, then `merge_checks` in D3's order (`symbolic-ref --quiet --short HEAD` equal to
    the base; `status --porcelain --untracked-files=no` empty in the main checkout, then in the
    worktree, naming the files; `rev-list --count refs/heads/<base>..refs/heads/<branch>` above
    zero), giving the base and the count. `merge(main, branch)`: `merge --no-ff --no-edit
    refs/heads/<branch>`, the merge commit from `rev-parse --short HEAD`; a failure reads the
    unmerged files (`diff --name-only --diff-filter=U`), runs `merge --abort` whenever `MERGE_HEAD`
    is there, and says which, the conflict's files or git's own words. `is_commit` moved here from
    `rail.rs`, which uses it too.
  - *`marley_rail`:* `DriftSnapshot.ahead`; `words()` none while `ahead` is 0 (REQ-011);
    `ahead_words()`, `2 ahead of main`, with the base named as the tooltip names it.
  - *`rail.rs`:* `DriftState` keeps `recorded` and `owner`; `read_drifts` reads the owner once
    per run and refreshes both on a kept state too. The row's second line joins the branch and the
    ahead words with ` · `. The row sits in a `right_click_menu`: Review, then `merge_line`'s
    Merge entry or label, built when the menu opens from the kept state through a weak `Rail` (no
    git): "Merge N commits into <base>…"; "Nothing to merge into <base>"; "No base recorded";
    "N commits ahead of <base>: the Rustal workflow merges here"; and three the spec did not name,
    "On no branch: nothing to merge", "Merge waits until Zed trusts the repository" and "Reading
    the branch…" before the first run lands. `review_worktree` reads the recorded base in the
    background, else `Repository::default_branch(true)` as `git: diff branch` does, then
    `start_review` keeps a `PendingReview` for `REVIEW_WAIT` (30 s), opening the worktree as its
    row's click does when it is not open; `take_review`, at the end of each refresh, finds the
    worktree's workspace and its own repository (`member_git`), and in a deferred update shows the
    workspace and deploys the diff. `merge_worktree`: `merge_target` (the entry, its branch and
    main checkout, Zed's trust, and `unsaved_in`: the window's dirty buffers inside the main
    checkout and outside its listed worktrees), `ready_to_merge` in the background, Zed's prompt
    (Merge, Cancel; the count, the branch, the base, the main checkout's path, "Nothing is
    pushed"), both checks again with the count and base compared to what the prompt said, then the
    merge; success is a toast in the displayed workspace, and every refusal or failure is a
    `detach_and_prompt_err` "Could not merge <name>".
  - *`Cargo.toml`:* `toml` (the lock gains the edge).
- **Deviations from the plan:**
  - The spec's `Result<u32, Refusal>` and `Result<String, MergeFailure>` are `anyhow::Result`s
    whose messages are the refusal: the UI shows only the text, and the rail's other git flows
    (#560) do the same.
  - Merge also refuses in a repository Zed does not trust (a merge runs the repository's hooks),
    and when the branch or its base moved while the prompt was up, so what merges is what the
    prompt said.
  - The workflow check is read with the drift runs, which start when a tip moves; a marker
    changed with no commit is found at the next run, and Merge reads the owner again when chosen,
    as the promoted spec says. The scenario commits after each marker change.
- **The review of the diff:** against each criterion:
  - REQ-001 to REQ-004, REQ-010: the menu's lines come from `merge_line` over the kept state; Merge
    shows only with a recorded base that is a branch, Zed's trust and `MergeOwner::Marley`; Review
    takes the recorded base, else `default_branch(true)`.
  - REQ-005, REQ-006, REQ-009: the only git that writes is `merge --no-ff --no-edit` and
    `merge --abort`; no code path runs `push` or `fetch`; the toast names `rev-parse --short
    HEAD`; a merged branch's `ahead` is 0, so `ahead_words` and `words` give none (REQ-011).
  - REQ-007, REQ-008: the checks run before the prompt and again after it, in D3's order, each
    naming what failed.
  - **One real bug, fixed:** `merge_checks` read `git status --porcelain` through `text()`, which
    trims the whole output, so the first line lost its leading space (` M README` became
    `M README`, checked on the box) and the refusal would have named `EADME`. The output is now
    read untrimmed; an F-block at Complete.
  - gpui re-entrancy: the menu reads the Rail when it opens, from the menu's mouse handler, outside
    any update; its entries update the Rail from the menu's own update, as the terminal row's do;
    `take_review` runs last in `refresh` and deploys in `defer_in`, so no workspace is updated
    inside the rebuild.
  - §20: nothing from Warp; Zed's `deploy_branch_diff_with_base_ref` is called, not copied, and no
    Zed function body moved into a Marley crate. Upstream: one word and a comment, with its row.
  - IO: every git and file read runs in a background task (`blocking_io_on_foreground` holds).

## Phase 3 — Test
- **Checklist** (no task tool): the scenario ✓; three runs, every shot read ✓; 510 and 560 alone
  ✓; 511 in `script/e2e/golden` and the golden set ✓ (all 52 passed); the gate ✓ (`GATE GREEN [diff]` on the second run).
- **The scenario:** `script/e2e/511-review-and-merge-a-worktree.sh`, `compositor sway` (it
  right-clicks rows). Setup: a HOME of its own and a scratch identity; `main` with README and
  notes.txt, pushed to a bare `origin`; `ok` and `clash` made as #510 makes them (`worktree add
  --no-track -b agent/<n> … main`, `branch.agent/<n>.base main`), ok with two commits (notes.txt's
  first line, a new ok.txt), clash with a commit to README's first line, then a commit on main to
  the same line; `manual` made by hand with a commit and no base; `MARLEY_GIT` naming a wrapper
  that logs every argument; `git ls-remote origin main` kept. Steps: the trust prompt, a rebuild
  for the first drift run, then each criterion below. A row's menu opens with `click 248 <row>
  right`, from its right edge, so the menu leaves the row's lines in view; Home then Return is
  Review, End then Return is Merge (the menu's last entry; a label is not selectable); Return
  answers a prompt with its first button. Every refusal is checked with `rev-parse main`, the
  main checkout's `status --porcelain --untracked-files=no` and `MERGE_HEAD`, before and after.
- **Found in Test, fixed at the source** (the scenario then run twice more):
  - Review of a worktree not open left its diff behind a terminal: the Marley layout's
    first-terminal seed gives a fresh folder workspace a terminal, which was added after the diff
    and took the front (`511-02-review` and `511-06-no-base-review` of the first run showed the
    "Changes since main" tab behind "ok — bash"). A Review that opens the worktree now marks its
    folder with `worktree_agents::skip_seed` first, the mark #510's agent worktrees use, and
    `take_review` drops a mark the seed did not take (`drop_seed_skip`). An F-block at Complete.
  - The merge commit's title read `Merge branch 'refs/heads/agent/ok'`: the merge names the full
    ref so a tag of the same name is never merged, and git titles it by what it was given. It now
    passes `-m "Merge branch '<branch>' into <base>"`; `merge` takes the base.
  - The confirmation's title, "Merge 2 commits of agent/ok into main?", wrapped its `?` onto a
    line of its own; it is "Merge 2 commits into main?", and the detail names the branch.
  - The scenario gained what the E2E plan left to review: REQ-008's two refusals (the main
    checkout on another branch; a change in the worktree), a shot of the clean status after the
    aborted merge, the owner read again with the key unset (`511-07-unset`), and a check of the
    merge's title.
- **The shots** (the third run; every one read):
  - `511-01-menu` (REQ-001, REQ-002): the rail's rows: clash `agent/clash · 1 ahead…` with
    `1 conflict`, manual `manual · 1 ahead of main` with no chip, ok `agent/ok · 2 ahead of main`
    with `1 behind`; ok's menu to the right of its row: Review, Merge 2 commits into main….
  - `511-02-review` (REQ-003): ok's workspace (`repo · ok / agent/ok`) with one tab, Changes since
    main, Base: main, notes.txt's first line and the new ok.txt; none of main's README change.
  - `511-03-refused` (REQ-007): "Could not merge ok" — "the main checkout has changes not
    committed: README."; the terminal's `git status -sb` shows ` M README`; the check passed.
  - `511-03-off-base` (REQ-008): "the main checkout is on other, not on main; check out main there
    first."; the title bar shows `other`.
  - `511-03-worktree-dirty` (REQ-008): "the worktree has changes not committed: ok.txt."
  - `511-04-asked` (REQ-005): "Merge 2 commits into main?", "Marley merges agent/ok into main with
    a merge commit in <the main checkout>. Nothing is pushed.", Merge (default) and Cancel.
  - `511-04-merged` (REQ-005, REQ-006, REQ-011): `git log --graph` in the main terminal with
    `(HEAD -> main) Merge branch 'agent/ok' into main` over ok's two commits and main's change,
    `origin/main` still on Start; the toast "Merged 2 commits of agent/ok into main: <sha7>.
    Nothing was pushed."; ok's row reads `agent/ok` with no count and no chip. The run log: origin's
    `ls-remote` equal before and after; main's tip's parents are the old main and agent/ok.
  - `511-05-conflict` (REQ-009): "Could not merge clash" — "the merge stopped on conflicts in
    README; it was aborted, and the main checkout is as it was."; the state check passed.
  - `511-05-clean` (REQ-009): `## main` and main still on ok's merge.
  - `511-06-no-base` (REQ-004): manual's menu: Review, No base recorded (a label; no Merge).
  - `511-06-no-base-review` (REQ-004): manual's workspace with Changes since main alone, Base:
    main, manual.txt.
  - `511-07-workflow` (REQ-010): after `git config marley.merge workflow` and a commit, clash's
    menu: Review, "1 commit ahead of main: the Rustal workflow merges here"; no Merge.
  - `511-07-unset` (REQ-010): the key unset and a commit: Merge 1 commit into main… again.
  - `511-08-workflow-toml` (REQ-010): `workflow.toml` with `[project]` and a commit: the workflow
    line again; the file in the project panel.
  - The run log's git: config, merge-base, rev-list, merge-tree, symbolic-ref, status, merge,
    diff and rev-parse; no push and no fetch; two merges, clash's aborted.
- **Focus:** the scenario ran in its own headless sway ("sway: stopped, with the run's Marley,
  pointer and keyboard"); no key or click reached the user's session.
- **510 and 560 alone:** both pass; 560's `560-02-drift` still shows `2 behind` and `1 conflict`,
  now beside `1 ahead of main` on each row.
- **The golden set:** `just regress`, 52 scenarios with 511 added, against the debug build before the lock's update: all 52 passed, 511 in 149 s (the runs under `~/.local/state/marley/regress/20260929-031714`).
- **The gate:** `script/gates.sh --diff`, log kept. Run 1 was red on gate:7 alone: `cargo-audit` reported
  RUSTSEC-2026-0314, -0315 and -0316 against wasmtime 48.0.1 (Zed's extension host), published
  after #586's green. Fixed at the source with the wasmtime family's patch releases: one `cargo
  update` of its 21 crates, 48.0.1 to 48.0.3 with cranelift 0.135.1 to 0.135.3, lockfile only
  (cap-std and cap-fs-ext drop out); the `Cargo.lock` row in `docs/marley/zed-touchpoints.md`
  says so. Run 2: `GATE GREEN [diff]`, 16 passed. The gate's clippy scope builds no wasmtime, so
  `just build` compiled it, and scenario 511 ran again on that build: its 11 checks passed.
- **Pre-existing, not in scope:** none left; the wasmtime advisories came from the database, not from #511, and were fixed at the source as above because the gate blocks the commit.

## Phase 4 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented:** `CHANGELOG.md` (Added: review and merge from a worktree's row; Security:
  wasmtime 48.0.3); `docs/marley_architecture/marley_workbench.md` ("Review and merge from a
  worktree's row": the adapter's owner, checks and merge, the kept state, the menu, Review with the
  seed's mark, Merge's flow); `docs/marley_architecture/marley_rail.md` (`DriftSnapshot::ahead`,
  `ahead_words`, `words` none at 0); `docs/marley/workbench-shell.md` (W4's record gains #511);
  `docs/marley/tutorial-outline.md` (511 shipped, 589 added); `docs/marley/guide.md` ("Worktree
  agents": the count, Review, Merge's checks and prompt, the workflow's marker, the chip's none).
  The touchpoint rows: `crates/git_ui/src/branch_diff.rs` describes what shipped;
  `Cargo.lock`'s row names the wasmtime patch bump. The three-prong plan lists no worktree slice
  (#510 and #560 recorded theirs in the shell plan too).
- **Knowledge:** F-claude-511-the-merge-checks-trimmed-porcelains-leading-space-001,
  F-claude-511-a-reviewed-worktree-opened-with-its-terminal-over-the-diff-001,
  PR-claude-column-significant-git-output-is-read-untrimmed-001,
  L-claude-511-a-fresh-folder-workspace-gets-its-first-terminal-after-its-openers-item-001,
  L-claude-511-git-titles-a-merge-by-the-name-it-was-given-001,
  AD-claude-511-marley-merges-a-worktree-branch-locally-and-the-workflow-owns-it-where-it-runs-001.
- **Brain:** consultation d55397c303c748c8b499e9641aa8e669 closed with
  `decisions/marley-merges-a-worktrees-branch-locally-and-leaves-it-to-the-rustal-workflow-where-it-runs`
  (follow-up 2026-10-29).
- **Filed:** TICKET-589, slice 2 (Remove a worktree without losing work), from "The split", its
  row in the queue after #585.
- **Closed:** TICKET-511 moved to `tickets/closed/`; its BACKLOG row went at promotion.
