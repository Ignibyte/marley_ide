# Which files a worktree agent's branch would conflict on, before anyone merges — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-560-conflict-chip.md
- **Pipeline spec:** 560-conflict-chip.spec.md

## Phase 1 — Plan
- **Request:** the Orca second pass of 2026-09-25, finding 2 ("Which files a branch would
  conflict on, before anyone merges"), ranked second of its four findings; Chad decided on
  2026-09-26 that every remaining Orca and Warp finding gets built. The finding's shape: a chip
  on each worktree agent's rail row, "12 behind main, conflicts in 3 files", the files in its
  tooltip, computed against the local base whenever the branch head or the base tip moves; it
  needs no pull request, so it serves a local merge and a workflow-managed one alike.
- **Classification / tier:** feature, S. Marley crates only (`marley_workbench`, `marley_rail`);
  no Zed touch (D3). After #510 (the rows and `branch.<b>.base`); beside #511, whose adapter
  module it shares.
- **Recall (§18.3):**
  - PR-claude-a-path-hook-resolves-the-files-own-repo-001 (recalled by #510): linked worktrees
    are one repository; key the cache on the main checkout's common git directory, never on a
    checkout's own path.
  - L-claude-458-a-projects-folder-events-come-before-its-group-is-rekeyed-001: the rail reads
    the groups in a deferred refresh; the summary's result lands through the same deferred path.
  - BF-changelog-hook-worktree-vs-staged-001 and its rule: "clean" is what git reports for the
    paths that matter; REQ-006 checks with `git status --porcelain --untracked-files=no` and
    `for-each-ref`, before and after.
  - L-claude-498-a-new-toolbar-button-moves-older-scenarios-clicks-001: a chip in the row's
    trailing slot moves nothing above or below it; #510's clicks on the row's name are unaffected.
  - The ledgers hold nothing on `merge-tree`, `merge_tree` or a pre-merge conflict check (grepped
    2026-09-26); #511's spec is the only place `merge-tree` appears, for its slice 2.
  - Brain: not consulted in this drafting pass; `/pipeline:plan` runs `brain_ask` at promotion.
- **Discovery (opened and checked, 2026-09-26):**
  - `crates/git/src/repository.rs`: `trait GitRepository` (789) with default-body methods
    (`load_index_text` 793, `remote_url` 816, `head_sha` 831); `diff_tree` (impl 1921) runs
    `merge-base <base> HEAD` at 2027 only for `MergeBaseWithWorktree` when a deleted file came
    back; `diff` (2500) with `DiffType::MergeBase` at 2511; `branches()` (2141) and
    `UpstreamTrackingStatus` (506); `fetch` (trait 1038, impl 2903, `--all`, `--unshallow` or a
    remote name); `GitBinary` (3877, `pub(crate)`), `run` (3950), `run_raw` (3962, non-zero exit
    is an error at 3968), `build_command` (3980, the flags). No `merge-tree`, `rev-list` or
    `cherry` anywhere in Zed's crates (grepped `crates/git`, `crates/project`, `crates/git_ui`,
    `crates/git_ui_core`, `crates/agent_ui`).
  - `crates/project/src/git_store.rs`: `RepositoryEvent` (834: `StatusesChanged`, `HeadChanged`,
    `BranchListChanged`, `StashEntriesChanged`, `GitWorktreeListChanged`, `PendingOpsChanged`,
    `GraphEvent`); `GitStoreEvent::RepositoryUpdated(RepositoryId, RepositoryEvent, bool)` (851,
    emitted at 2570); `RepositorySnapshot::linked_worktrees` (6292), `main_worktree_abs_path`
    (6261), `is_linked_worktree` (6288); `Repository::is_trusted` (6438); `send_job` (6828, one
    job at a time per repository, 10114); `fetch` (8665, refreshes the branch list on success);
    `default_branch` (9566).
  - `crates/marley_workbench/src/rail.rs`: `GroupEntry` (155, the key and the workspace);
    `render_project_row` (1067; the frame at 1089, the disclosure at 1094, `row_label` at 1107,
    the trailing slot at 1113-1131 with the attention dot at 1117 and the `+` at 1124); the row
    menus (1135-1161); `Tooltip::text` at 1180; `build_snapshot` (1805, `ProjectSnapshot` built at
    1873). No chip or badge on any row today; the crate's one `Chip` is in `browser.rs:4138`.
  - `crates/marley_rail/src/marley_rail.rs`: `ProjectSnapshot` (30-42), `ProjectRow` (198-212),
    `rail_rows` (516). #510 adds `WorktreeSnapshot` and `Row::Worktree`; #511 adds
    `WorktreeSnapshot::ahead`.
  - `crates/marley_workbench/src/agent_bar.rs:97-112`: the git store read (`git_store()`,
    `repositories()`, `.branch`, `.work_directory_abs_path`). `marley_workbench/Cargo.toml` has
    `project` and no `git`.
  - `crates/ui/src/components/chip.rs`: `Chip::new` (29), `label_color` (45), `icon` (57),
    `icon_color` (63), `tooltip` (92). `crates/icons/src/icons.rs`: `Warning` (294), `GitBranch`
    (166).
  - `crates/util/src/command.rs:16`: `new_command`. `crates/marley_browser/src/service.rs:133`:
    the spawn pattern.
  - `crates/marley_workbench/src/`: no `worktree_git.rs` yet (#511 is queued).
  - The box: `git version 2.55.0`; `git merge-tree -h` lists `--write-tree`, `--name-only`, `-z`,
    `--[no-]messages`, `--[no-]merge-base <tree-ish>`.
  - Orca (MIT, read at `1c2cf120e3`): `src/main/github/conflict-summary.ts` (160, 170-172, 193,
    204, 217-236, 256-261, 297-305), `conflict-summary-cache.ts` (11, 15-16, 49-62, 96, 102-114),
    `src/shared/git-merge-tree-capability.ts` (11-27, 269-271), the panel's `conflict-summary.tsx`
    (15-21, 34-49, 80-104), `src/shared/git-branch-cleanup.ts:115`.
- **Decisions:** D1 to D9 in the spec.

### Promotion (2026-09-28)
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (gate, e2e, hooks, README marker; cargo busy
  with #510's release install, so no cargo ran and no crate or scenario was edited; git 2.55.0
  has `merge-tree --write-tree`, `--name-only`, `-z`, `--no-messages` and `--merge-base`, read
  from `git merge-tree -h`); recall ✓; promote ✓ (the pair to `active/`, the backlog row
  removed, the ticket in-progress); the seams re-verified (Explore, over `e487a0a978`) ✓; spec and
  design updated ✓.
- **Recall, added:** #510's rows as shipped (`WorktreeSnapshot`, `render_worktree_row`,
  `group_worktrees`, `member_git`, the `GitStore` subscription); L-claude-510-zeds-worktree-create
  (a detached main checkout's base is its full commit); F-claude-521 and
  PR-claude-run-a-views-poll-while-it-shows-001 (a background read runs only while its rail
  shows). The brain (consultation 639e8832688d4ec481e8af3c9a8e381f): nothing on this seam.
- **What the code says now** (the Explore report, 2026-09-28):
  - `render_worktree_row` (`rail.rs:2415-2443`) is `row_card` and a click, with no end element
    and no menu; `row_card`'s caller adds the end, as the terminal row adds its chips
    (`2537-2539`). `permission_chip` (`4266-4281`) is the pill to copy.
  - `WorktreeSnapshot` (`marley_rail.rs:53-68`) derives `Eq`; `branch` is display text, a
    branch or a short commit. `MemberGit` and `worktree_rows` keep no raw ref, no commit and no
    repository; `Snapshot.worktrees` is rebuilt at each refresh, so a cache lives on `Rail`, as
    `risk` does (`follow_risk`, `664-733`: gone keys dropped, unchanged ones skipped, a task's
    result kept only while current, then a refresh).
  - `quiet_timers` (`1238-1243`) is the debounce's model: a timer task replaced by the next.
  - Zed's snapshot holds both tips: `linked_worktrees[..].sha` (each worktree's `HEAD`) and
    `branch_list[..].most_recent_commit.sha` (every local branch's tip), from the same
    `compute_snapshot` that emits the three events; an agent's commit in a worktree rescans the
    main checkout's repository too (the scanner watches `<common dir>/refs/**`).
  - `build_command` (`repository.rs:3983`) adds four flags, `log.showSignature=false` among them;
    no Marley spawn sets them or `GIT_TERMINAL_PROMPT`. `run_raw` (`3965`) errors on a non-zero
    exit; `diff_tree` (`1972-1976`) reads its own output.
  - `Repository::is_trusted` (`git_store.rs:6438`), false while loading and for remote ones;
    `default_branch(false)` (`9566`) through `send_job`, which can name a remote's branch with no
    local ref.
  - #511 designed `branch_state`, `merge_checks` and `merge` for `worktree_git.rs` and a
    `worktree_states` map filled by one task per project, on the same events; no `run_git`.

### Design
- **Changed at promotion** (each item overrides the drafted design after it):
  - **The adapter** `worktree_git.rs`: `pub(crate) struct Drift { base: String, base_commit:
    String, behind: u32, conflicts: Option<Vec<String>> }`; `recorded_base(main, branch) ->
    Result<Option<String>>` (`config --get`, exit 1 is none); `summary(main, branch_tip,
    base_tip, base) -> Result<Drift>` (`merge-base`, `rev-list --count`, `merge-tree --write-tree
    --name-only -z --no-messages --merge-base`), each through one `git(main, args)` that builds
    `new_command(MARLEY_GIT or "git")` with Zed's four flags, `current_dir(main)` and
    `GIT_TERMINAL_PROMPT=0`, and returns the output unjudged. `merge-tree`'s exit 0 is clean,
    1 conflicts (the files after the tree id, NUL-separated), 129 with Orca's refusal text is
    unsupported, anything else an error.
  - **The cache** on `Rail`: `drift: HashMap<String (the worktree's path), DriftState { base:
    Option<Option<String>> (unread, none, or the base), tips: Option<(String, String)>, drift:
    Option<Drift> }>`, and `drift_runs: HashMap<PathBuf (the main checkout), Task<()>>`, one per
    repository, a new one replacing the last after a one-second timer. `follow_drift` after the
    rebuild, only while the rail shows: gone worktrees dropped; for each listed worktree, the tips
    from the snapshot's facts and the kept base; a changed pair, or a base not read, schedules its
    repository's run. The run reads the bases it lacks, the base's tip from the snapshot's
    branches (or the recorded commit), runs the summaries off the main thread, keeps each result
    with its tips, and refreshes. A repository Zed does not trust runs nothing.
  - **The facts** the rows need: `worktree_rows` keeps, per worktree, its raw branch (without
    `refs/heads/`) and commit, and the group's main checkout path and repository (weak) with its
    local branches' tips.
  - **The model**: `WorktreeSnapshot.drift: Option<DriftSnapshot { behind, conflicts:
    Option<Vec<String>>, base, base_commit }>`, with `words()` and `tooltip()`, copied onto
    `WorktreeRow`.
  - **The chip**: `drift_chip(id, drift, cx)` beside `permission_chip`, joined to the worktree
    row's card.
- **Approach.**
  - *The adapter (`crates/marley_workbench/src/worktree_git.rs`).* `pub struct Drift { base:
    String, base_commit: String, behind: u32, conflicts: Option<Vec<String>> }` (`conflicts` is
    `None` when `--write-tree` is refused) and `async fn conflict_summary(main: &Path, branch:
    &str, base: &str) -> Result<Drift>`. One `git` runner, `run_git(main, args)`, builds
    `util::command::new_command("git")` with `-c core.fsmonitor=false --no-optional-locks
    --no-pager`, `current_dir(main)`, `env("GIT_TERMINAL_PROMPT", "0")`, and returns the
    `Output` (status, stdout, stderr) without judging the exit, so `merge-tree`'s 1 is read as
    "conflicts". The refs are passed as `refs/heads/<name>`, so a name can never read as an
    option. A refused `--write-tree` (exit 129 and `unknown option` or `unrecognized option` in
    stderr, Orca's match) gives `conflicts: None`.
  - *The cache (`rail.rs`).* `drift: HashMap<PathBuf, DriftEntry { tips: (String, String), drift:
    Option<Drift> }>` keyed on the worktree's path; `drift_runs: HashMap<PathBuf (the main
    checkout), Task<()>>`, one per repository. On a `RepositoryUpdated` event of the kinds D4
    names, a one-second timer on the background executor replaces any pending timer; when it
    fires, one task per repository reads each worktree's branch and base (`branch.<b>.base`
    through `git config --get` in the same runner, else `default_branch(false)`), runs
    `rev-parse --verify` for the pair, skips a pair the cache already holds, runs the summary
    for the rest, and updates the entries on the main thread; a change during a run schedules one
    more. The rail's `refresh` rebuilds the snapshot from the cache alone.
  - *The model (`marley_rail`).* `WorktreeSnapshot::drift: Option<DriftSummary { behind, conflicts:
    Option<usize>, base, base_commit, files: Vec<String> }>`, copied onto the worktree row; pure.
  - *The chip (`rail.rs`).* In the worktree row's trailing slot, before the row's menu: a
    `ui::Chip` reading `N behind` (muted label) or `N conflicts` (`IconName::Warning`,
    `Color::Warning` for the icon and the label), with `tooltip` building a column: the first
    line `N commits behind <base> (<base> at <sha7>)`, then the files, twenty at most and
    `and N more`. No drift, no chip.
  - *Trust.* The task skips a repository whose `Repository::is_trusted` is false.
- **File manifest** (as promoted): the draft's below, with `worktree_git.rs` new (#511 has not
  landed), and `script/e2e/golden` (Test). No Zed path, no ledger row.
- **File manifest** (the draft's). Marley crates: `crates/marley_workbench/src/worktree_git.rs` (new, or
  extended when #511 landed), `crates/marley_workbench/src/rail.rs`,
  `crates/marley_workbench/src/marley_workbench.rs` (the module), `crates/marley_rail/src/marley_rail.rs`.
  Test phase: `script/e2e/560-conflict-chip.sh`.
- **Ledger rows.** None: no path outside the Marley-owned set changes.
- **Knowledge at Complete (expected).** An AD for the local-base rule and the adapter spawn
  (D1, D3); a lesson on `merge-tree`'s exit 1 if the runner's output handling teaches one.

### E2E plan
As promoted, over the draft below: the wrapper `git` is named by `MARLEY_GIT`, not put first on
Marley's PATH, and serves only the chip's summaries, so Zed's own git calls are the real git's;
REQ-009 is the gate and the golden set.

The draft's plan:

Setup makes the repository and the worktrees, the wrapper `git` (a bash script first on Marley's
PATH: `printf '%q ' "$@" >> git.log`, then the `merge-tree` refusal while the flag exists, else
`exec /usr/bin/git "$@"`), records `git rev-parse main` and the two branch tips, and prints the
`status` and `for-each-ref` lines the run log compares. Coordinates of the chip are measured on
the first run (the worktree rows sit under the project header, after #510's layout).

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-005 | the rail after Marley opens the repository; the project expanded | `560-01-clean`: `ok` and `clash` rows, no chip |
| REQ-001, REQ-002 | in the main checkout's terminal: `printf 'changed on main\n' > README; git commit -qam 'main 1'; echo more >> notes.txt; git commit -qam 'main 2'`; settle 4 | `560-02-drift`: `ok` reads `2 behind`; `clash` reads `1 conflict` |
| REQ-003 | `pointer_to` the `clash` chip; settle for the tooltip | `560-03-tooltip`: `2 commits behind main (main at <sha7>)`, `README` |
| REQ-004 | `git -C <clash> merge --no-edit -X theirs main; git -C <ok> merge --no-edit main`; settle 4 | `560-04-merged`: no chip on either row |
| REQ-007 | `touch $E2E_WORK/no-write-tree`; `echo again >> notes.txt; git commit -qam 'main 3'`; settle 4 | `560-05-unsupported`: both rows `1 behind`; the run log greps Marley.log for the line |
| REQ-006 | after each step, the run log's `git status --porcelain --untracked-files=no` for the three checkouts and `git for-each-ref refs/heads`, compared with the values before; `grep -c fetch git.log` is 0 | the run log |
| REQ-008 | review: the task's `is_trusted` check | review |

Not reachable by a scenario: a repository Zed does not trust (the runner trusts the scratch
repository at open, and an untrusted one shows no rail rows to hover); the review reads the
check. A real git older than 2.38 (the box has 2.55): the wrapper's refusal stands in, with
Orca's error text.

### Risks
- `merge-tree --write-tree` writes the merged tree's objects on every run with a new pair of
  tips; on a repository the size of Marley's own that is a few hundred objects a run, unreferenced,
  pruned by gc. Test logs `git count-objects -v` before and after the run.
- The cost of `merge-tree` on a large repository is unmeasured here (this drafting pass ran no
  git that writes); Test logs each summary's duration, and a summary over two seconds moves the
  debounce to five.
- A tip that moves while a summary runs: the result is keyed on the tips it read, and the next
  event runs again; the chip can lag one event, never lie about the tips it names.
- The wrapper `git` in the scenario also serves Zed's own git calls (Zed finds `git` on the
  PATH), so the wrapper must exec the real git for everything but the flagged `merge-tree`.
  Changed at promotion: `MARLEY_GIT` names it for the chip alone.
- #511's `2 ahead of main` on the second line and this chip in the trailing slot share one row:
  whichever lands second checks the narrow rail (180 px) in its shots, as #531's risk says.
- A repository with many worktrees runs one summary per worktree per event; the cache on the
  tips keeps a quiet worktree at one `rev-parse`.

## Phase 2 — Code
- **Checklist** (no task tool): the README marker present ✓; no Zed path and no ledger row ✓;
  `worktree_git.rs` ✓; `marley_rail` ✓; `rail.rs` ✓; `marley_workbench.rs` ✓; `just clippy
  marley_rail marley_workbench` ✓, after #510's release install had ended; the review ✓.
- **What was built.**
  - `crates/marley_workbench/src/worktree_git.rs` (new, in the shape #511 designed): `Drift`;
    `recorded_base(main, branch)`, `git config --get branch.<b>.base`, whose exit 1 is no record;
    `summary(main, branch_tip, base_tip, base)`, `merge-base`, `rev-list --count` and `merge-tree
    --write-tree --name-only -z --no-messages --merge-base`, whose exits the adapter reads itself:
    0 clean, 1 the files after the tree id, 129 with git's refusal words unsupported, anything
    else an error. Every call goes through `git(main, args)`: `MARLEY_GIT` or `git`, Zed's four
    flags, the main checkout as its folder, `GIT_TERMINAL_PROMPT=0`, no shell.
  - `crates/marley_rail/src/marley_rail.rs`: `DriftSnapshot { behind, conflicts, base,
    base_commit }` with `conflicted`, `words` (`2 behind`, `1 conflict`, `3 conflicts`, none up to
    date) and `tooltip` (`N commits behind main (main at <sha7>)`, or the short commit alone for a
    commit base; then `A merge would stop on:` and twenty files at most with `and N more`, `It
    merges cleanly.`, or `This git cannot tell whether it merges cleanly.`), on
    `WorktreeSnapshot::drift` and `WorktreeRow::drift`.
  - `crates/marley_workbench/src/rail.rs`: `member_git` keeps the member's branch as git names it,
    its `HEAD` and its repository; `worktree_rows` keeps each worktree's branch and `HEAD`
    (`WorktreeFacts`) and derives the shown branch from them; `group_worktrees` puts them on
    `WorktreeEntry` with the main checkout and the group's repository, the main checkout's when it
    is open. The Rail keeps `drift` (per worktree: the base the last run read, the tips it read
    for, the drift), `drift_runs` (one per main checkout) and `drift_unsupported`. `refresh` calls
    `note_drift` before its compare and `follow_drift` last. `follow_drift` drops the worktrees no
    longer listed and, while the rail shows, schedules a run for a repository Zed trusts that has
    a worktree unread or whose tips moved, unless its run is in flight: the worktree's `HEAD` from
    the rows, its base's tip from `branch_list` (`drift_tips`, a local branch only), or the base
    itself when it is a full commit, compared without allocating (`same_tips`). `drift_run` waits
    a second, reads the rows, the kept states, the branches and Zed's `default_branch(false)` on
    the main thread, checks the trust again, runs `read_drifts` on the background executor (the
    base, then a summary unless the base and the tips are the ones kept), keeps the results, logs
    one warning per repository whose git refuses `--write-tree`, and refreshes. `drift_chip` joins
    the worktree row's card as `permission_chip` joins a terminal row's: a bordered pill, `N
    behind` muted, `N conflicts` in the warning color after `IconName::GitMergeConflict`, the
    tooltip from the snapshot, `marley-rail-drift-<name>` for scenarios.
  - `crates/marley_workbench/src/marley_workbench.rs`: `pub mod worktree_git;`.
- **Deviations from the design.**
  - The base is read at each run of its repository, one `git config --get` per worktree, not once
    and again when the worktree list changes: #510 writes the base after its worktree is listed,
    so a base read once could stay the default branch for good. A run follows only a moved tip or
    an unread worktree, so the summaries stay the cost.
  - A drift read for tips that have since moved stays on the row until the next run lands: the
    chip lags a commit by about a second rather than blinking at each one.
- **The review**, against each REQ:
  - REQ-001, REQ-002, REQ-005: `words` gives `N behind` for a clean merge behind its base, the
    count of files with any conflict, and nothing up to date; the chip's color follows
    `conflicted`. The five seconds: a one-second timer and three git calls per moved worktree.
  - REQ-003: `tooltip`, from the kept drift; the base's short commit is the one the drift was
    read for.
  - REQ-004: a merged worktree is 0 behind with no conflict, so its row has no chip.
  - REQ-006: the calls are `config --get`, `merge-base`, `rev-list --count` and `merge-tree
    --write-tree`, which moves no ref and touches no index or working tree, and writes the merged
    tree's objects, which nothing references; no fetch anywhere.
  - REQ-007: a refused `--write-tree` gives `conflicts: None`, so the chip reads `N behind`, and
    `drift_unsupported` keeps the warning to one per repository.
  - REQ-008: `follow_drift` schedules nothing for a repository whose `is_trusted` is false, and a
    run checks again before its first git call. Zed flips a repository's trust in its backend
    without an event, so the first run after the user grants trust waits for the rail's next
    rebuild (a terminal's output, a git event); the scenario types a command after the trust.
  - Re-entrancy: `follow_drift` reads repositories inside the rail's own update; the run reads and
    updates the repository from its task, inside no other entity's update.
  - The arguments: commits from Zed's snapshot, or a recorded base only when it is a full hex
    commit; the config key is one argument, and a branch name git accepts never starts with `-`.
  - Provenance: Orca's calls, its refusal words and its wording (MIT), reimplemented from the
    report and the published git manuals; no Zed function body copied; nothing of Warp.
- **What the review changed**: the tips compared at each rebuild were two new strings per
  worktree, at every terminal chunk; `drift_tips` now borrows and `same_tips` compares.
- **Checked by hand before Test** (a scratch repository in the scratchpad, git 2.55.0): exit 1 with
  `<tree id>\0README\0` for a conflict, exit 0 and the tree id alone for a clean merge (a change to
  the first line of `notes.txt` against an appended sixth line), `config --get` exit 1 for an
  unset key; both checkouts' statuses clean after.
- **Clippy**: the first run found three lints in `rail.rs`: `clone_on_ref_ptr` on the branch list's `Arc`, `redundant_closure_for_method_calls` on `log_err`, and `needless_pass_by_ref_mut` on `drift_run`'s context (and so `follow_drift`'s); all three fixed at the source. The second run, `cargo clippy -p marley_rail -p marley_workbench --all-targets -- -D warnings`, is clean, and `cargo fmt --check` on both crates is clean.

## Phase 3 — Test
- **Checklist** (no task tool): the scenario per the E2E plan as promoted ✓; 560 in the golden set
  ✓; `just build` ✓; the scenario run and every shot read ✓; the golden set ✓; `gates.sh --diff` ✓.
- **The scenario**, `script/e2e/560-conflict-chip.sh` (compositor sway, no Chromium): a scratch
  repository on `main` with `README` and `notes.txt`, under a git config of the scenario's own
  (also the terminal's `~/.gitconfig`); worktrees `ok` and `clash` made as #510 makes them,
  `worktree add --no-track -b agent/<name>` from `main` with `branch.agent/<name>.base main`, `ok`
  changing the first line of `notes.txt` and `clash` the first line of `README`; the chip's git a
  wrapper named by `MARLEY_GIT`, which logs each call and answers `merge-tree` with git's refusal
  and exit 129 while `$E2E_WORK/no-write-tree` exists. The trust press, then a command in the main
  checkout's terminal, since Zed flips a repository's trust without an event and the first run
  waits for a rebuild; `main 1` (README's first line) and `main 2` (a sixth line of `notes.txt`)
  typed in that terminal; the pointer on `clash`'s chip; both worktrees merging `main` (`clash`
  with `-X theirs`); the flag file and `main 3`. The run log prints `main`'s short commit after
  the drift and after the refusal, and checks the wrapper's log and Marley.log.
- **The runs.** Run 1 passed every check with every shot as planned, but its shots came six
  seconds after each change, over REQ-002's and REQ-004's five. Run 2 took them four seconds
  after, and typed the merges with relative paths; it passed. Run 3 added `main`'s short commit to
  the run log, to hold against the tooltips, and passed.
- **The shots** (run 3, read one by one; the rails cropped together):
  - `560-01-clean` (REQ-005): under `repo`, its terminal's row, then `clash` (`agent/clash`) and
    `ok` (`agent/ok`), muted since neither is open, and no chip on either; the terminal's graph
    shows both branches one commit on `Start`.
  - `560-02-drift` (REQ-001, REQ-002): four seconds after the two commits on `main`, `clash` reads
    `1 conflict` in the warning color after the merge-conflict icon, and `ok` reads `2 behind`,
    muted, both in bordered pills at the rows' ends.
  - `560-03-tooltip` (REQ-003): the pointer on `clash`'s chip: `2 commits behind main (main at
    3a7597e)`, `A merge would stop on:`, `README`; the run log: `main is at 3a7597e`.
  - `560-04-merged` (REQ-004, REQ-005): four seconds after the merges (`Auto-merging README`,
    `Auto-merging notes.txt` in the terminal), no chip on either row.
  - `560-05-unsupported` (REQ-007): with `merge-tree` refused and `main 3` made, both rows read `1
    behind`; the pointer on `ok`'s chip: `1 commit behind main (main at ded9933)`, `This git cannot
    tell whether it merges cleanly.`; the run log: `main is at ded9933`.
- **The checks** (every run): the chip's git ran by the first shot; the refs, the worktree list,
  the three checkouts' statuses and the four index files' hashes the same before Marley opened the
  repository and after its first run (REQ-006); the wrapper's calls 8 `config`, 8 `merge-base`, 8
  `rev-list` and 8 `merge-tree`, four runs over two worktrees and nothing else, no `fetch`
  (REQ-006); one line in Marley.log, `WARN [marley_workbench::rail] git in <repo> refuses
  merge-tree --write-tree, which came in git 2.38: ...` (REQ-007).
- **Focus**: the scenario ran in its own headless sway; the runner reported `hyprland: 0 Marley
  windows before the run, 0 after`, no rule added.
- **Not reachable by a scenario**: a repository Zed does not trust (REQ-008): the runner trusts the
  scratch repository at open; the review reads the two checks. A git older than 2.38: the
  wrapper's refusal, with git's own words, stands in. **Observed, not in scope**: the first run
  after a trust grant waits for the rail's next rebuild (Zed's `set_trusted` sends no event), which
  a user's next keystroke or git change brings.
- **The gate**: `script/gates.sh --diff`: `GATE GREEN [diff]`, 15 gates PASS (rustfmt, clippy on every target, cargo-audit, cargo-deny, cargo-shear, gitleaks, shellcheck, no-suppressions, source-bans, docs, zed-ledger, manifests, typos, semgrep, dylint); the full log is scratchpad `560/gate.log`.
- **The golden set** with 560 added: 48 of 49 (`~/.local/state/marley/regress/20260928-185249`), 560 among them (52 s). `508-approvals-inbox` failed after all five of its checks had passed: the headless sway exited between its eighth shot and the next pointer move (`seat-pointer: the compositor went away`, then a broken pipe on Marley's Wayland connection), with no core dump, no OOM kill and no GPU fault in the kernel's log at that moment, and 508's repository has no worktree, so no drift read ran in it. The runner's cleanup then stopped on `SEAT_POINTER_PID: unbound variable`. Run alone right after, 508 passed all five checks through `508-07-cleared`. Pre-existing, not in scope: the compositor's unexplained exit, and the cleanup reading the pointer helper's PID after bash had unset it (TICKET-588).
- **Verdict**: PASS.

## Phase 4 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented**: `CHANGELOG.md` (Added: a worktree's drift on its row);
  `docs/marley/workbench-shell.md` (W4's record gains #560's chip); `docs/marley/tutorial-outline.md`
  (a row for the chip, shipped); `docs/marley/guide.md` ("Worktree agents": the chip's three
  states, its tooltip, the base, when Marley reads it and what it never does);
  `docs/marley_architecture/marley_workbench.md` ("A worktree's drift": the adapter, the rows'
  facts, the cache and the runs, the chip); `docs/marley_architecture/marley_rail.md`
  (`WorktreeSnapshot::drift`). No path outside the Marley-owned set changed, so no touchpoint row.
- **Knowledge**: L-claude-560-zed-turns-a-repositorys-trust-on-without-an-event-001,
  L-claude-560-merge-tree-reports-conflicts-with-exit-one-001,
  AD-claude-560-a-worktrees-drift-is-read-from-local-git-on-the-rails-schedule-001. No F-block:
  Code and Test found no product bug (the review's one change was an allocation at each rebuild,
  and the golden run's one failure was the harness's, filed as TICKET-588).
- **Brain**: consultation 639e8832688d4ec481e8af3c9a8e381f closed with
  `decisions/marley-reads-a-worktrees-drift-from-local-git-on-the-rails-schedule` (follow-up
  2026-10-28).
- **In the same commit**, Chad's answers of 2026-09-28: TICKET-417 and TICKET-271 closed;
  TICKET-446's holder, Ignibyte, and its row moved to the queue; TICKET-586 (a project's `file://`
  pages in its Browser tab) and TICKET-587 (Claude Code's trust question in a new worktree) filed;
  the Delta note's answer. TICKET-588 filed from the golden run.
- **Closed**: TICKET-560 moved to `tickets/closed/`; its BACKLOG row went at promotion.
