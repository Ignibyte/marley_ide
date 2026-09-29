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

### Changed at promotion (2026-09-29; each item overrides the design below)
- **Checklist** (no task tool): pre-flight ✓ (no other active pipeline; #522's release install
  compiling, so no crate edits until it ends); recall ✓; the brain ✓ (nothing on this seam);
  promoted ✓; the seams re-verified by an Explore agent at 7e92fc9846 ✓.
- #510 and #560 shipped: worktree rows exist with their drift, and Marley runs its own `git`
  (`worktree_git`) for repositories Zed trusts. So the counts need no Zed touch: `numstat(folder,
  base)` in `worktree_git`, `git diff --numstat --merge-base --end-of-options <base> --`.
- The project row today shows no git at all and `ProjectSnapshot`/`ProjectRow` carry no branch:
  both gain `git: Option<ProjectGit>` (counts and base, the PR).
- The reads copy #560's pattern: a run per project folder, a second's debounce, the trust check
  twice, `default_branch(true)` for a base with its remote, a PR's `baseRefName` first
  (`origin/<base>`). The rail's `GitStore` subscription skips `StatusesChanged` for its full
  refresh; a separate arm marks the project's counts stale so edits show without a rail refresh.
- `gh` is installed here through mise (2.101.0), so a Marley started from the desktop may not find
  it: no chip, one log line.
- Worktree rows keep their drift chip; the PR chip on them waits.

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

### For the quality pass
- No tests (§7, since 2026-09-29): the drafted scenario (a stand-in `gh`, the PR states, a branch
  switch, an edit, `gh` failing) waits for the quality pass.

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

## Folded in from the Orca second pass (2026-09-26)
Finding 4 of `docs/planning/design-notes/orca-second-pass-2026-09-25.md`: a PR URL printed in a
terminal ties the PR to its row. When an agent's `gh pr create` prints the new PR's URL, the row
shows the PR at once instead of at the next two-minute lookup (D5).

- **The hint.** A GitHub PR URL (`https://github.com/<owner>/<repo>/pull/<n>`) printed in a
  terminal of the project, found by the scan #503 adds for local URLs (`links.rs` over
  `last_n_non_empty_lines`, where escapes are gone and soft wraps joined; if #503 has not landed,
  a scan of the same shape in `rail.rs`'s terminal subscriptions, on `terminal::Event::Wakeup`,
  at most twice a second). Orca scans the byte stream instead and carries 512 bytes across reads
  so a URL split over two writes still matches, strips SGR sequences and trailing `),.;]}`,
  ignores a URL over 2,048 bytes and matches each URL once per terminal
  (`src/shared/terminal-github-pr-link-detector.ts`, MIT, read).
- **A hint, not a fact.** The hit starts one lookup for the terminal's repository and branch, the
  same `gh pr list --head <branch>` as D3, and the chip changes only when that lookup returns the
  printed number, because terminal output can carry any PR URL (docs, an agent's log). A PR
  already on the row is never replaced by a different printed number (Orca's rule,
  `src/renderer/src/store/slices/worktrees/session/worktree-unread-activity.ts`).
- **Bounded.** At most one hint-started lookup per repository and branch a minute, on top of D5's
  two-minute timer; a `gh` answer whose stderr says `API rate limit exceeded` (and not
  `secondary rate limit`) pauses every lookup for that repository for one two-minute period,
  Orca's breaker reduced to one bucket (`src/main/git/gh-rate-limit-breaker.ts`).
- **Acceptance to add at promotion.** REQ-008: WHEN a terminal of the project prints a GitHub PR
  URL for the project's repository, the row shall show that PR within five seconds when `gh`
  confirms it for the branch, and shall not change when the printed number differs from `gh`'s
  answer. Shots `531-08-printed-pr` (the stand-in `gh` answers #42 for the branch; the terminal
  echoes `.../pull/42`; the chip appears before any timer would fire) and `531-09-printed-other`
  (`.../pull/7` echoed; the chip still reads #42; the run log's `gh.log` shows one lookup for the
  hint). Worktree rows (#510) take the same hint from their own terminals.

## Phase 2 — Code
- **Checklist** (no task tool): `worktree_git::changed_lines` ✓; `github.rs` ✓; the rail's group
  entries, `follow_project_git`, `project_git_run`, `read_project_git`, the `StatusesChanged` arm
  and the drawing ✓; the review ✓; the gate ✓.
- **Built as the changes at promotion say**, with the data in the rail's `GroupEntry` rather than
  `marley_rail`'s `ProjectRow` (whose nine test literals would all have changed), and a read's end
  calling `cx.notify()`.
- **Clippy** asked for `github` to be a `pub mod` with `pub(crate)` items (the two visibility lints
  conflict inside a private module), `try_global` for the hosting registry (`default_global` takes
  `&mut App`), an `Option<Instant>` for the edit mark (the rail's fourth bool), `git_source` out of
  `build_snapshot`, `read_project_git` out of `project_git_run` (both past 100 lines), and a
  borrowed pull request for the chip.
- **The review**, against each criterion:
  - REQ-001, REQ-004: the counts are `numstat --merge-base` of the working tree, so uncommitted
    tracked edits count; a save marks `git_edited` and schedules only this read.
  - REQ-002, REQ-006: the chip's color by state, and the tooltips (the counts' names the base).
  - REQ-003: a branch or `HEAD` change is a move, which asks `gh` again and recounts.
  - REQ-005: `gh` missing or refusing is logged once per folder and leaves no chip.
  - REQ-007: `gh` gets `--head=<branch>` as one argument through `new_command`, no shell.
  - **Found and fixed: a read loop.** `asked_at` was set only when `gh` ran, so a project with no
    GitHub remote, or on no branch, was due again at each read's own refresh, a read a second
    while the window was active. A read due to ask now counts as asked. F-block below.
- **The gate:** `just gate-diff` green: 16 passed, 0 failed, `GATE GREEN [diff]`, the receipt
  written.

---
## Phase 3 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented:** `CHANGELOG.md`; `docs/marley_architecture/marley_workbench.md` ("A project's
  changed lines and pull request"); `docs/marley/workbench-shell.md`; `docs/marley/guide.md` (the
  project header). No Zed path changed.
- **Knowledge:** F-claude-531-a-project-with-nothing-to-ask-was-read-again-each-second-001,
  AD-claude-531-a-project-rows-counts-and-pull-request-come-from-marleys-git-and-gh-001.
- **Brain:** consultation ec57d538c37a434fbcf93561728db0df closed with a decision (follow-up
  2026-10-29).
- **Closed:** TICKET-531 moved to `tickets/closed/`; its BACKLOG row went at promotion.
- **No tests** (§7): the drafted scenario waits for the quality pass.
