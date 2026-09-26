---
pipeline_id: 626b2c82-d9d7-46d6-a662-d46662100d9e
ticket: docs/planning/tickets/open/TICKET-531-pr-state-on-rail-rows.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Pull request state and diff counts on the rail's project rows"
type: feature
slice: workbench shell (the rail's rows, after #468), Warp once-over item 7; worktree rows with #510
references: [docs/planning/pipeline/completed/468-rail-rows-after-warps-tab-list.spec.md, docs/planning/design-notes/warp-once-over-2026-09-25.md, docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md]
---

## Title
Each project row in the rail shows how much its branch changed against the branch's base, as
Zed's own added and removed counts, and the branch's pull request on GitHub as a chip with its
number and state. Chad can see from the rail which project's work is ready and which PR it is in.

## Scope
### In
- **The counts.** Per project row: the lines added and removed between the branch's merge base
  with its base and the working tree, uncommitted edits to tracked files included, drawn with
  Zed's `ui::DiffStat`. git counts them (`git diff --numstat --merge-base <base>`) through a new
  method on Zed's git layer, so untrusted repositories keep Zed's protections. They follow each
  status change of the repository, a second after it settles.
- **The base.** The pull request's base branch when there is a PR (`origin/<base>` when that
  remote-tracking branch exists, else the local branch); otherwise Zed's default branch
  (`Repository::default_branch`: `upstream/HEAD`, `origin/HEAD`, `init.defaultBranch`, `main`,
  `master`). A project on its default branch counts against that branch's remote copy, which
  gives its unpushed and uncommitted work. No base, no counts.
- **The chip.** When the branch has a pull request on GitHub: Zed's pull request icon and `#<n>`,
  colored by state (open, draft, merged, closed). A tooltip gives the state, the title and the
  URL; the counts' tooltip names the base.
- **The lookup.** `gh pr list --repo <owner>/<repo> --head <branch> --state all --limit 1 --json
  number,state,isDraft,title,url,baseRefName`, where owner and repo come from `origin`'s URL
  through Zed's `parse_git_remote_url` (GitHub remotes only). It runs when a project and branch
  are first seen, when the branch changes, and every two minutes while Marley's window is active,
  one lookup at a time per repository and branch. It runs from one new adapter module, with the
  branch as its own argument and never through a shell.
- **Failures.** No `gh`, `gh` not logged in, no network, or a remote that is not GitHub: no chip,
  the counts as usual, one log line per repository and kind of failure, nothing in the rail.

### Out (explicitly deferred)
- Worktree rows: #510 draws them nested under their project and passes each worktree's repository
  to the same code, whose data is kept per repository and branch; its `branch.<b>.base` record
  becomes the first choice of base.
- CI checks on the chip (`statusCheckRollup`), review state, and a click that opens the PR.
- Hosts other than github.com (GitHub Enterprise), GitLab and the rest.
- Remote projects: the new git method runs only for a local repository.
- Untracked files in the counts (git's numstat leaves them out, as Zed's branch diff does).

## Reference (§20)
Warp, from its published docs only (docs.warp.dev/terminal/windows/vertical-tabs/, item 7 of the
once-over note): the vertical tab list's expanded rows show the branch, a diff stats badge (lines
added and removed) and a PR badge with the pull request's status, which needs the GitHub CLI, and
a hover card gives the full details. Marley keeps the counts, the PR badge through `gh` and the
details on hover, on the rows #468 laid out after Warp's
(`docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md`). Upstream Zed: the added and
removed counts as its branch diff toolbar draws them (`ui::DiffStat` in
`crates/git_ui/src/branch_diff.rs`), and its merge-base diff (`git diff --merge-base`).

### Prior art
- **Behavior maps.** `docs/orca_architecture/02-worktrees-and-review.md` §2.5 (Orca's worktree
  card shows a PR or MR chip with checks) and §2.12 (GitHub through the `gh` CLI, with a
  rate-limit guard per bucket), and §3 item 12 (PR status through `gh`, "later"); the Warp
  once-over note, item 7. Orca's lookup of a branch's PR (MIT, read):
  `src/main/github/client/lookup/pr-branch-lookup.ts` runs `gh pr list --repo <owner>/<repo>
  --head <branch> --state all --limit 1 --json ...`, where an empty list means no PR.
- **Published material.** `gh pr list --help` and `gh pr view --help` (gh 2.101.0 on this box):
  `-H, --head` (a branch name; the `<owner>:<branch>` form is not supported), `-s, --state`
  (`open`, `closed`, `merged`, `all`), `-L, --limit`, `--json` with `number`, `state`, `isDraft`,
  `title`, `url` and `baseRefName`, and `-R, --repo [HOST/]OWNER/REPO`. git's
  `diff --merge-base` and `--numstat`.
- **Code we already ship.**
  - Zed computes per-file counts in every status scan (`StatusEntry::diff_stat`,
    `crates/project/src/git_store.rs` 493 to 499, from `git diff --numstat HEAD` at 12390 to
    12400), but against `HEAD` only, which misses a branch's commits.
  - `GitRepository::diff_stat` (`crates/git/src/repository.rs:1064`) takes `DiffStatType`
    (`HeadToIndex`, `HeadToWorktree`, `IndexToWorktree`, 1152), a `Copy` enum a base ref cannot
    join without changing upstream's derive. `Repository::diff` with `DiffType::MergeBase` runs
    `git diff --merge-base <base>` (repository.rs 2510) but returns the whole patch, about 20 MB
    for this repository's branch against `origin/main` (measured 2026-09-25), too much to read on
    each change.
  - Running `git` from a Marley crate would drop Zed's guards: `GitBinary` is `pub(crate)`
    (repository.rs 3877) and its `build_command` (3980) adds `core.fsmonitor=false`,
    `--no-optional-locks`, `--no-pager`, and for an untrusted repository no hooks, no
    `credential.helper`, `core.sshCommand=ssh` and no `diff.external`. So the counts come through
    a new trait method with a default body, the pattern the trait already uses
    (`load_index_text`, `remote_url`, `head_sha`).
  - Reused as they are: `git::status::parse_numstat` (`crates/git/src/status.rs:586`),
    `Repository::default_branch` (git_store.rs 9566), `RepositorySnapshot::branch` and
    `remote_origin_url` (615, 621), `GitStoreEvent::RepositoryUpdated` with
    `RepositoryEvent::StatusesChanged`, `HeadChanged` and `BranchListChanged` (834, 848),
    `git::parse_git_remote_url` and `ParsedGitRemote` (`crates/git/src/hosting_provider.rs` 235,
    240), `ui::DiffStat` (`crates/ui/src/components/diff_stat.rs`), `IconName::PullRequest`
    (`crates/icons/src/icons.rs:216`), `util::command::new_command`.

## UI proof
UI-AFFECTING. `script/e2e/531-pr-state-on-rail-rows.sh` (`compositor sway`, for the hover): a
scratch repository with `origin` set to a GitHub URL that is never fetched, `main` and four
branches with known changes (a fifth is made during the run), and a stand-in `gh` first on
Marley's PATH that answers from a
table by `--head` and logs every call's arguments. Steps: open on `feature/open`, whose PR is #42
and open (`531-01-open-pr`); `git switch feature/merged` typed in the terminal (`531-02-merged`);
`git switch feature/draft` (`531-03-draft`); `git switch feature/none`, which has no PR
(`531-04-no-pr`); a line appended to a tracked file (`531-05-edited`); the pointer on the chip of
`feature/draft` after switching back, then on the counts (`531-06-tooltips`); last, a branch named
`feature/$(touch${IFS}pwned)`, for which the stand-in fails as a `gh` that is not logged in does
(`531-07-gh-fails`). The run log shows that the stand-in got that name as one argument and that
no `pwned` file exists.

## Locked-In Decisions
- D1: The counts run through Zed's git layer: a new `GitRepository` method with a default body
  (`crates/git/src/repository.rs`) and its `Repository` twin (`crates/project/src/git_store.rs`),
  two small additive Zed touches, because Marley cannot reach `GitBinary` and a `git` of its own
  would lose Zed's guards for untrusted repositories.
- D2: The base is the PR's base when there is one, else Zed's default branch. #510's
  `branch.<b>.base` goes first when it exists.
- D3: `gh pr list --head ... --state all --limit 1` (Orca's lookup), so "no PR" is an empty list
  and exit 0, and a closed or merged PR still shows.
- D4: `gh` is asked from one module, `crates/marley_workbench/src/github.rs`, with fixed
  arguments, the branch as a single argument, `GH_PROMPT_DISABLED=1`, and no shell. A git branch
  name may hold `$`, `;` and parentheses, so a shell would be an injection. When #541 has landed,
  the call goes through its process adapter (`process.rs`) and adds no spawn site; otherwise
  `github.rs` makes the one `new_command("gh")` call, which #541 then moves.
- D5: Two minutes between lookups per repository and branch while the window is active, plus one
  at each branch change: a few projects cost a few `gh` calls a minute, far under GitHub's limits.
- D6: The data is per repository and branch, not per rail group, so #510's worktree rows reuse it.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a project's branch has a merge base with its base, the project row shall show the lines added and removed since that merge base, uncommitted edits to tracked files included. | Shot `531-01-open-pr` (the counts the setup committed) |
| REQ-002 | WHEN the branch has a pull request on GitHub, the row shall show a chip with the PR's number and its state: open, draft, merged or closed. | Shots `531-01-open-pr`, `531-02-merged`, `531-03-draft` |
| REQ-003 | WHEN the project's branch changes, the row shall show the new branch's pull request and counts. | Shots `531-02-merged` to `531-04-no-pr` |
| REQ-004 | WHEN a tracked file changes, the row's counts shall follow within a few seconds. | Shot `531-05-edited` |
| REQ-005 | WHERE a branch has no pull request, or `gh` is missing or fails, the row shall show its counts and no chip, and the rail shall show no error. | Shots `531-04-no-pr` (no PR) and `531-07-gh-fails` (the stand-in exits 1); the log's one line for the failure |
| REQ-006 | WHEN the pointer rests on the chip, the rail shall show the PR's state, title and URL; on the counts, the base they are counted against. | Shot `531-06-tooltips` |
| REQ-007 | WHEN Marley asks `gh` about a branch, it shall pass the branch name as one argument and start no shell. | The run log: the stand-in's argument list for `feature/$(touch${IFS}pwned)`, and no `pwned` file |

## Phase Plan
- **P1 Plan:** promote the pair, recall, consult the brain, confirm the design in the notes.
- **P2 Code:** the two Zed touchpoint rows first, then the trait method and its `Repository`
  twin; `github.rs`; the rows' data in `marley_rail`; the cache, the subscriptions and the
  drawing in `rail.rs`; fmt and clippy clean; a review of the diff.
- **P3 Test:** write and run the scenario, read every shot; #500's scenario again for the project
  row and its + (#468 predates the e2e runner and has no scenario); `script/gates.sh --diff`
  green.
- **P4 Complete:** CHANGELOG; the rows in `docs/marley/workbench-shell.md` (D3, the row model);
  the two touchpoint rows checked against what shipped; the ledger; close, archive, commit.
