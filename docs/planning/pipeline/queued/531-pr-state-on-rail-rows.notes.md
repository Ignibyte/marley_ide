# Pull request state and diff counts on the rail's project rows — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-531-pr-state-on-rail-rows.md
- **Pipeline spec:** 531-pr-state-on-rail-rows.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-25, "yes" to item 7 of the Warp once-over: rows that show the
  branch's pull request, its status through the GitHub CLI, and diff stats, so that with #510's
  parallel worktree agents the rail shows whose work is ready. The brief for this draft: each
  project and worktree row shows its branch's PR state through `gh` and the added and removed
  counts against its base.
- **Classification / tier:** feature, M. Marley crates (`marley_workbench`, `marley_rail`) and two
  small additive Zed touches (`crates/git`, `crates/project`).
- **Split.** Project rows now; worktree rows when #510 draws them, from the same per-repository
  data (Out in the spec).
- **Recall (§18.3):**
  - #468 and `docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md`: the project header
    is a muted section label with Marley's chevron and +; rows are padded two-line cards.
  - L-claude-468-sample-the-capture-before-trusting-a-theme-token-001: several One Dark tokens
    share a value; read the capture's pixels before settling the chip's colors.
  - L-claude-487-a-headless-seat-has-no-devices-until-a-client-adds-them-001: read the pointer's
    targets from the run's first shot.
  - Ledger: nothing on `gh`, pull requests or merge-base counts. The gpui-era AD on Marley's first
    git write (#116) is history.
  - Brain: not consulted in this drafting pass, which was read-only; `/pipeline:plan` runs
    `brain_ask` at promotion.
- **Discovery (opened and checked):**
  - `crates/marley_workbench/src/rail.rs`: `Rail` (72) and its per-project subscriptions (the
    `project_subscriptions` field, kept by `sync_subscriptions` at 389); `build_snapshot` (1734)
    reads each group's last active workspace and fills `ProjectSnapshot`; `GroupEntry` (148);
    `render_project_row` (1016) draws the disclosure, the name in `row_label`, the attention dot
    and `render_project_menu`, in an `h_8` header.
  - `crates/marley_rail/src/marley_rail.rs`: `ProjectSnapshot` (30), `ProjectRow` (195),
    `rail_rows` (500); pure and gpui-free.
  - `crates/marley_workbench/src/agent_bar.rs`: `contents` finds a folder's branch from
    `project.repositories(cx)` and `branch_for` (138), the lookup the rows can follow.
  - `crates/project/src/git_store.rs`: `StatusEntry` with `diff_stat` (493 to 499); the status scan
    runs `diff_stat(HeadToWorktree | HeadToIndex | IndexToWorktree)` (12390 to 12400);
    `RepositorySnapshot::branch` (615), `remote_origin_url` (621), `remote_upstream_url` (622);
    `RepositoryEvent` (834: `StatusesChanged`, `HeadChanged`, `BranchListChanged`);
    `GitStoreEvent::RepositoryUpdated` (848); `Repository::default_branch` (9566), `diff_tree`
    (9591), `diff` (9661), each through `send_job` with a local and a remote arm.
  - `crates/git/src/repository.rs`: `trait GitRepository` (789) with default bodies at 793
    (`load_index_text`), 816 (`remote_url`), 831 (`head_sha`); `diff_stat` (1064); `DiffType`
    (1145) and `DiffStatType` (1152, `Clone, Copy`); `impl GitRepository for RealGitRepository`
    (1402); `diff` (2500, `MergeBase` at 2510 runs `diff --merge-base <base> --`); `diff_stat`
    (2527, `diff --numstat --no-renames` then `parse_numstat`); `default_branch` (3258: the
    `upstream/HEAD` and `origin/HEAD` symrefs, `init.defaultBranch`, `main`, `master`);
    `GitBinary` (3877, `pub(crate)`); `build_command` (3980).
  - `crates/git/src/status.rs`: `DiffStat` (571), `GitDiffStat` (577), `parse_numstat` (586, skips
    binary files' `-` lines).
  - `crates/git/src/hosting_provider.rs`: `GitHostingProviderRegistry::global` (169),
    `ParsedGitRemote` (235), `parse_git_remote_url` (240).
  - `crates/fs/src/fake_git_repo.rs:168`: `impl GitRepository for FakeGitRepository`, which takes
    the default body and needs no change.
  - `crates/ui/src/components/diff_stat.rs`: `DiffStat` draws `+ N` in the added color and `‒ M`
    in the deleted color, with thousands separators and an optional tooltip;
    `crates/git_ui/src/branch_diff.rs` uses it (810) with `calculate_changed_lines` (766).
  - `crates/icons/src/icons.rs:216`: `IconName::PullRequest`.
  - `crates/marley_workbench/Cargo.toml`: no `git` dependency yet.
  - Measured on this repository, 2026-09-25: `git diff --shortstat --merge-base origin/main` is
    1,872 files, 269,782 insertions and 36 deletions in 47 ms; the same diff as a patch is
    19,859,205 bytes in 82 ms. `upstream/HEAD` is absent here and `origin/HEAD` points at
    `origin/main`, so Zed's default branch for Marley's own repository is `origin/main`.
  - `gh` 2.101.0 on the PATH; `gh pr list --help` for `--head`, `--state`, `--limit`, `--json`,
    `--repo`.
  - Orca: `src/main/github/client/lookup/pr-branch-lookup.ts` (`getFallbackPRListForBranch`, the
    `gh pr list ... --head ... --state all --limit 1 --json` form) and
    `pull-request-lookup-data.ts:65` (the fields it asks for).
  - `git check-ref-format --branch 'feature/$(touch${IFS}pwned)'` passes: a branch name can carry
    shell syntax.
- **Decisions:** D1 to D6 in the spec.

### Design
- **Zed, `crates/git/src/repository.rs`.** In `trait GitRepository`, beside `diff_stat`:
  `fn diff_stat_merge_base(&self, base_ref: SharedString) -> BoxFuture<'static,
  Result<GitDiffStat>>` with a default body that fails ("no merge-base counts for this
  repository"), and a `// Marley:` comment. `RealGitRepository` overrides it the way its
  `diff_stat` runs: `git_binary_in_worktree()`, then `run(["diff", "--numstat", "--no-renames",
  "--merge-base", base_ref, "--"])`, then `parse_numstat`. A new row in
  `docs/marley/zed-touchpoints.md` first.
- **Zed, `crates/project/src/git_store.rs`.** `impl Repository`: `pub fn
  diff_stat_merge_base(&mut self, base_ref: SharedString) -> oneshot::Receiver<Result<
  GitDiffStat>>` through `send_job`, calling the backend for a local repository and failing for a
  remote one, with a `// Marley:` comment and its own row.
- **`crates/marley_rail`.** `ProjectSnapshot` and `ProjectRow` gain `branch: Option<BranchState>`,
  where `BranchState { changes: Option<Changes { added, removed, base }>, pull_request:
  Option<PullRequest { number, state, title, url }> }` and `PullRequestState` is `Open`, `Draft`,
  `Merged` or `Closed`. A pure `base_for(pr_base, remote_branches, default_branch)` picks the base
  (D2). `rail_rows` copies the field onto the row.
- **`crates/marley_workbench/src/github.rs`, new, the only module that starts `gh`.**
  `pull_request(owner, repo, branch) -> Result<Option<PullRequest>>`: refuses a branch that starts
  with `-`, runs `util::command::new_command("gh")` with the fixed arguments of the spec's lookup,
  the branch as one argument and `GH_PROMPT_DISABLED=1`, and parses the JSON array with serde
  (`isDraft` makes an open PR `Draft`). `gh` missing, a non-zero exit or bad JSON is an error the
  caller logs once. If #541 has landed, the call goes through `crate::process` (its output
  helper), so no spawn site is added and gate:22's pin stays; if not, `github.rs` makes the call
  itself and #541 moves it.
- **`crates/marley_workbench/src/rail.rs`.**
  - A cache keyed by (work directory, branch): the PR with when it was fetched, the counts with the
    base they used, and the lookups in flight. `build_snapshot` fills each group's `branch` from
    the active repository of the group's workspace and the cache.
  - `sync_subscriptions` also subscribes to each project's `GitStore`: `RepositoryUpdated` with
    `StatusesChanged` schedules the counts a second later (a newer event replaces the timer);
    `HeadChanged` and `BranchListChanged` start a PR lookup for the new key and new counts.
  - A timer on the background executor every two minutes starts lookups older than that for the
    keys on screen, while the window is active.
  - `render_project_row` draws, before the attention dot, `ui::DiffStat` with the tooltip
    "against <base>" and a chip (`IconName::PullRequest` and `#<n>`, one theme color per state,
    the tooltip "#42 open: <title>" and the URL). The name keeps `min_w_0` and `flex_1`, so it
    truncates first in a narrow rail.
- **`crates/marley_workbench/Cargo.toml`:** `git.workspace = true` (for `parse_git_remote_url`,
  the registry and `GitDiffStat`).
- **File manifest.** Zed: `crates/git/src/repository.rs`, `crates/project/src/git_store.rs`.
  Marley: `crates/marley_rail/src/marley_rail.rs`, `crates/marley_workbench/src/github.rs` (new),
  `rail.rs`, `marley_workbench.rs` (`pub mod github`), `Cargo.toml`;
  `script/e2e/531-pr-state-on-rail-rows.sh` at Test.
- **Ledger rows.** Two new rows in `docs/marley/zed-touchpoints.md` (the trait method with its
  default body and the real override; the `Repository` method), each with what to do at a merge:
  keep the method beside `diff_stat`, and drop both if upstream grows a merge-base variant of
  `diff_stat`. At Complete: an AD for the base rule and the `gh` lookup (D2 to D5).

### E2E plan
| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | setup: `repo` on `main` with `app.txt` (20 lines) and `notes.txt`; `feature/open` commits 12 lines added and 3 removed in `app.txt`; `origin` is `https://github.com/ignibyte-e2e/fixture.git`, never fetched; the stand-in `gh` (a bash script first on Marley's PATH) answers `--head feature/open` with `[{"number":42,"state":"OPEN","isDraft":false,"title":"Add the dashboard","url":"https://github.com/ignibyte-e2e/fixture/pull/42","baseRefName":"main"}]` and appends `printf '%q '` of its arguments to `gh.log`; open on `feature/open`, trust it | `531-01-open-pr`: `+ 12 ‒ 3` and the open chip `#42` on the project row |
| REQ-002, REQ-003 | type `git switch feature/merged` (5 added, 1 removed; the stand-in says MERGED #41); settle 4 | `531-02-merged` |
| REQ-002, REQ-003 | `git switch feature/draft` (2 added; #43, `isDraft` true) | `531-03-draft` |
| REQ-003, REQ-005 | `git switch feature/none` (1 added, 1 removed; the stand-in answers `[]`) | `531-04-no-pr`: the counts, no chip |
| REQ-004 | `echo more >> notes.txt` (a file no branch changes, so the edit follows every switch); settle 3 | `531-05-edited`: one more added line |
| REQ-006 | `git switch feature/draft`; the pointer on the chip, a shot; on the counts, a shot | `531-06-tooltips` (two captures: the PR's state, title and URL; "against main") |
| REQ-005, REQ-007 | `git switch -c 'feature/$(touch${IFS}pwned)'`; the stand-in exits 1 with "not logged in" for that head; settle 4 | `531-07-gh-fails`: counts and no chip, no error on screen; the run log prints `gh.log`'s last line (the name as one quoted argument), the result of looking for a `pwned` file under `$E2E_WORK` and in the directory Marley was started from (none may exist), and Marley.log's one line for the failure |

Not reachable by a scenario: the two-minute refresh (it would lengthen the run by minutes; the
branch switches drive every lookup here) and a real `gh` against github.com (no network in the
run, and Chad's account is not the scenario's).

### Risks
- Sibling tickets drafted the same night touch the project row (their queued specs, 2026-09-25):
  #542 puts agent counts after a collapsed project's name and reorders rows by attention, and
  #510 adds worktree rows under the project and writes `branch.agent/<name>.base`, which D2 reads
  first. The chip and the counts sit at the header's right, before the attention dot, and the
  name truncates first, so #542's counts and these fit on one header; whichever lands second
  checks the narrow rail (180 px) in its shots.
- A branch far from its base (this repository against `origin/main`) shows large counts. They are
  true, and `ui::DiffStat` formats them with separators; a shorter form can wait for Chad's eye.
- `default_branch` reads `init.defaultBranch` from Marley's own environment, so the scenario's
  base depends on the box's git config only when the branch has no PR and `main` is missing; the
  fixture has `main`, which `default_branch` finds either way.
- `gh` may be slow or rate-limited on a bad network; one lookup per key at a time and the
  two-minute spacing keep it bounded, and a failure never blocks the counts.
- The group's workspace may hold several repositories (a multi-root project). The row follows the
  workspace's active repository, as the agent bar's branch does.
- `git switch` in the scenario changes files Zed watches; the counts wait a second after the last
  status event so a burst of events costs one `git diff`.
