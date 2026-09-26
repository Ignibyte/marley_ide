# Which files a worktree agent's branch would conflict on, before anyone merges — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-560-conflict-chip.md
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
- **Decisions:** D1 to D6 in the spec.

### Design
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
- **File manifest.** Marley crates: `crates/marley_workbench/src/worktree_git.rs` (new, or
  extended when #511 landed), `crates/marley_workbench/src/rail.rs`,
  `crates/marley_workbench/src/marley_workbench.rs` (the module), `crates/marley_rail/src/marley_rail.rs`.
  Test phase: `script/e2e/560-conflict-chip.sh`.
- **Ledger rows.** None: no path outside the Marley-owned set changes.
- **Knowledge at Complete (expected).** An AD for the local-base rule and the adapter spawn
  (D1, D3); a lesson on `merge-tree`'s exit 1 if the runner's output handling teaches one.

### E2E plan
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
- #511's `2 ahead of main` on the second line and this chip in the trailing slot share one row:
  whichever lands second checks the narrow rail (180 px) in its shots, as #531's risk says.
- A repository with many worktrees runs one summary per worktree per event; the cache on the
  tips keeps a quiet worktree at one `rev-parse`.
