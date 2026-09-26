---
pipeline_id: 0e66e227-e2d2-46aa-bb41-5c4b0a7eb791
ticket: docs/planning/tickets/open/TICKET-560-conflict-chip.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Which files a worktree agent's branch would conflict on, before anyone merges"
type: feature
slice: prong 2, worktree agents, on #510's rows and beside #511; the Orca second pass, finding 2
references: [docs/planning/design-notes/orca-second-pass-2026-09-25.md, docs/orca_architecture/02-worktrees-and-review.md, docs/planning/pipeline/queued/510-worktree-agents.spec.md, docs/planning/pipeline/queued/511-review-and-merge-a-worktree.spec.md]
---

## Title
Each worktree row in the rail (#510) shows how far its branch has fallen behind its base and,
when a merge would stop, how many files it would stop on, with the files in a tooltip. Marley
learns it from three read-only git calls against the local base branch, run off the main thread
whenever the branch tip or the base tip moves, so the collision between two agents on one
repository shows before anyone merges and before #511's Merge is asked to.

## Scope
### In
- **The summary** (`crates/marley_workbench/src/worktree_git.rs`, the git adapter #511 designed;
  made in that shape when #511 has not landed): `conflict_summary(main, branch, base)`, four git
  calls in the main checkout: `rev-parse --verify refs/heads/<branch> refs/heads/<base>` (the two
  tips; a pair already summarized ends here), `merge-base <branch> <base>`, `rev-list --count
  <branch>..<base>` (the commits behind) and `merge-tree --write-tree --name-only -z --no-messages
  --merge-base <merge base> <branch> <base>`, whose exit 0 means clean and exit 1 means conflicts,
  with the merged tree's id and then the conflicting paths on stdout, NUL-separated. Any other
  exit is an error the summary reports. Run with `util::command::new_command("git")`,
  `current_dir` the main checkout, `GIT_TERMINAL_PROMPT=0` and the flags Zed's own git runs with
  (`-c core.fsmonitor=false --no-optional-locks --no-pager`), on the background executor.
- **The base**: `branch.<b>.base` (#510 writes it), else the repository's default branch's local
  name (`Repository::default_branch(false)`), else no base and no chip.
- **When it runs**: on the repository's `RepositoryUpdated` events (`HeadChanged`,
  `BranchListChanged`, `GitWorktreeListChanged`; the subscription #510 adds for its rows), one
  second after the last event, one run in flight per repository; the result is kept per worktree
  with the pair of tips it was made for, so a refresh with nothing new costs one `rev-parse`.
- **The chip** (`marley_rail`: `WorktreeSnapshot::drift`; `rail.rs`: the worktree row's trailing
  slot): `N behind`, muted, while the branch is behind and merges cleanly; `N conflicts` (`1
  conflict`) with the warning icon in the warning color while it would not; a tooltip with the
  behind count, the base and its short commit, and the conflicting files (twenty at most, then
  `and N more`). Up to date, no chip.
- **Trust**: the summary runs only for a repository Zed trusts (`Repository::is_trusted`).
- **Old git**: when git refuses `--write-tree` (before 2.38), the chip shows the behind count
  alone and Marley logs one line for the repository.
- `script/e2e/560-conflict-chip.sh`.

### Out (explicitly deferred)
- Conflicts between two sibling worktree branches, the same call over each pair of agents: a
  later slice, once the number of agents on one repository makes an O(n²) sweep worth its cost.
- A fetch of the base's remote tip (Orca's throttled `git fetch origin <base>`): Marley's worktree
  flow merges locally (#511 D2), so the local base is the merge that will happen, and the rail
  runs no network call.
- The main checkout's own row: it is normally on the base, and #531 owns its chip.
- Handing a conflict to the worktree's agent as a prompt (#522's sender) and "update from base"
  in the worktree (#511's Out).
- Orca's "the hosting provider's verdict is stale" message: Marley shows no provider verdict on
  worktree rows (#531 shows a PR's state on project rows only).
- Remote (SSH) projects, whose git runs on another machine.

## Reference (§20)
Orca's conflict summary (the second pass, finding 2; report 02 §2.5, the worktree card's checks,
and §2.12, base drift as a behind count): when a pull request is marked conflicting, the Checks
panel lists the conflicting files and the commits behind, from `git merge-base`, `git rev-list
--count` and `git merge-tree --write-tree --name-only -z --no-messages --merge-base`, cached on
the pair of commit ids, with no GitHub API call (`src/main/github/conflict-summary.ts`,
`conflict-summary-cache.ts`, `src/renderer/src/components/right-sidebar/checks-panel/conflict-summary.tsx`).
Marley keeps the three calls, the cache on the tips and the wording ("N commits behind", the
files), drops the fetch and the pull request, and runs it for every worktree row, not only a
conflicting PR. Upstream Zed: nothing computes a merge's outcome before it runs; its conflict
view opens after a merge stopped (report 02, "a conflict view"), and its git layer runs
`merge-base` only inside `diff_tree`. Warp: N/A, a git and workspace behavior outside the
terminal.

### Prior art
- **Behavior maps and reports.** The second pass, finding 2 (the three calls; exit 1 with the
  tree id first; the cache on the pair of ids; git before 2.38 has no `--write-tree`; `--write-tree`
  writes objects gc prunes; "keep the git calls off the UI thread and debounce them, since an
  agent commits often"). Report 02 §2.5 (the card's checks and provenance) and §2.12 (drift
  events `current`, `drift`, `base_changed` with a behind count). Orca's files, read at
  `1c2cf120e3`: `conflict-summary.ts` (the fetch at 160 with a 10 s timeout, failures swallowed;
  `rev-parse --verify` at 170; `merge-base` at 193; `rev-list --count` at 204; `merge-tree` at
  217 with the legacy two-commit form at 228 when `--merge-base` is refused; exit 1's stdout
  recovered at 256; the NUL split that drops the tree id at 297); `conflict-summary-cache.ts`
  (one fetch per base a minute, the summary keyed on runtime, path, base, head tip and base tip,
  a success kept for good, a failure for 60 s, requests in flight shared);
  `src/shared/git-merge-tree-capability.ts` (the version check is a match on git's error text:
  "unknown option", "unrecognized option", the old usage line; "fail closed"); the panel's wording
  (`{n} commits behind (base commit: <sha7>)`, `Conflicting files`, and three commands offered
  when GitHub's verdict is stale). `src/shared/git-branch-cleanup.ts:115` runs the same
  `merge-tree --write-tree` to prove a branch merged, #511's slice 2.
- **Published material.** `git-merge-tree(1)`: `--write-tree` (since 2.38) performs a real merge
  "without touching the index or the working tree", writes the merged tree's objects, exits 0
  clean and 1 with conflicts; `--name-only`, `-z`, `--no-messages`, `--merge-base <tree-ish>`.
  `git-rev-list(1)` `--count`; `git-merge-base(1)`. git 2.55.0 on the dev box has every option
  (`git merge-tree -h`, read 2026-09-26).
- **The code we already ship.** Zed's `trait GitRepository` (`crates/git/src/repository.rs`)
  has no `merge-tree`, `rev-list` or `merge-base` method: `merge-base` runs only inside
  `diff_tree` (2027, when a deleted file reappears), and `--merge-base` is a diff flag
  (1930-1952, 2511); the ahead and behind it knows are against the upstream
  (`UpstreamTrackingStatus`, 506, from `%(upstream:track)` in `branches()`, 2141), which #510's
  `--no-track` branches never have. `GitBinary` is `pub(crate)` (3877), `run_raw` treats a
  non-zero exit as an error (3968), so `merge-tree`'s exit 1 needs its stdout read the way
  `diff_tree` reads its own, and `build_command` (3980) adds `core.fsmonitor=false`,
  `--no-optional-locks` and `--no-pager`, which the adapter passes itself. `Repository::send_job`
  (`crates/project/src/git_store.rs:6828`) runs one job at a time per repository, so a summary
  in that queue would hold up status refreshes: the adapter runs beside it. `Repository::is_trusted`
  (6438); `RepositoryEvent` (834) and `GitStoreEvent::RepositoryUpdated` (851);
  `RepositorySnapshot::linked_worktrees` (6292) and `main_worktree_abs_path` (6261);
  `default_branch` (9566). Marley: `util::command::new_command` (`crates/util/src/command.rs:16`,
  the pattern at `crates/marley_browser/src/service.rs:133`); `ui::Chip` with `icon`,
  `icon_color`, `label_color` and `tooltip` (`crates/ui/src/components/chip.rs:14-92`, drawn once in
  `browser.rs:4138`); `IconName::Warning` (`crates/icons/src/icons.rs:294`); the project row's
  trailing slot (`rail.rs:1113-1131`) that the worktree row copies, and `Tooltip::text` at 1180;
  `agent_bar.rs:97-112` reads the git store the way the rail will. Does a crate we build own
  the seam? No: Zed's git layer owns running git for trusted and untrusted repositories, and
  offers no way in for these three commands without a Zed touch; the adapter #511 designed is
  the owner on Marley's side.

## UI proof
UI-AFFECTING (a chip on the worktree rows and its tooltip). `script/e2e/560-conflict-chip.sh`
(`compositor sway`: it hovers the chip). Setup, as #511's: a scratch repository on `main` with
`README` and `notes.txt`; two worktrees made the way #510 makes them (`git worktree add
--no-track -b agent/<n> <path> main`, then `git config branch.agent/<n>.base main`): `ok` with a
commit to `notes.txt`, `clash` with a commit to README's first line; a wrapper `git` first on
Marley's PATH that logs each invocation's arguments and execs the real git, except that while
`$E2E_WORK/no-write-tree` exists it answers `merge-tree` with `error: unknown option 'write-tree'`
and exit 129; the scenario's HOME. Shots: `560-01-clean` (both worktree rows, no chip);
`560-02-drift` (two commits on `main` typed in the main checkout's terminal, one of them to
README's first line: `ok` reads `2 behind`, `clash` reads `1 conflict` in the warning color);
`560-03-tooltip` (the pointer on `clash`'s chip: `2 commits behind main (main at <sha7>)` and
`README`); `560-04-merged` (`git -C <clash> merge --no-edit -X theirs main` and `git -C <ok>
merge --no-edit main` typed in the same terminal: no chip on either row); `560-05-unsupported`
(the flag file made, one more commit on `main`: both rows read `1 behind`, and Marley.log holds
the one line). The run log prints `git status --porcelain --untracked-files=no` and
`git for-each-ref refs/heads` for the main checkout and both worktrees before and after each
step, and the wrapper's log, in which no `fetch` appears.

## Locked-In Decisions
- D1: Local git only, no fetch. The base is the local branch #510 records, the one #511 merges
  into, so the chip answers the question #511's Merge will ask. Orca fetches origin's tip
  because its merges happen on GitHub; Marley's rail makes no network call.
- D2: Orca's three read-only calls, cached on the pair of tips. `merge-tree --write-tree` moves
  no ref and touches no index or working tree; it writes the merged tree's objects, which nothing
  references and gc prunes. Rejected: the legacy `merge-tree` form, whose output is a diff to
  parse; before 2.38 the chip shows the behind count alone, with one log line.
- D3: Marley runs git itself, in the adapter #511 designed, off the main thread. Zed's git layer
  has no seam for these commands, its `GitBinary` is crate-private, and its `run_raw` turns
  `merge-tree`'s exit 1 into an error, so a Zed touch would need its own output reading; a Marley
  spawn with Zed's flags keeps the Zed diff at zero. It runs only for a repository Zed trusts,
  as Zed's own guards do for untrusted ones, and its arguments are refs git itself reported,
  never text from a terminal. When #541 has landed, the call goes through its process adapter.
- D4: The summary runs on the repository's events, one second after the last, one run in flight
  per repository, and the rail's rebuild reads the kept result and runs no git (#510 D10, #511
  D7). An agent commits often; the cache on the tips makes a refresh with nothing new one
  `rev-parse`.
- D5: One chip, two states: `N behind` muted, `N conflicts` in the warning color; the tooltip
  carries the rest. Rejected: the counts on the row's second line, where #511's `2 ahead of
  main` sits, in a rail 180 px wide.
- D6: Worktree rows only. A worktree with no recorded base takes the repository's default
  branch, as #511's Review does; the main checkout's row keeps #531's chip.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a worktree's branch is behind its base and merges cleanly, the worktree's row shall show a chip reading `N behind`. | Shot `560-02-drift` (`ok`) |
| REQ-002 | WHEN a commit on the base changes a line the worktree's branch also changed, the row's chip shall read the number of conflicting files in the warning color within five seconds. | Shot `560-02-drift` (`clash`) |
| REQ-003 | WHEN the pointer rests on the chip, the system shall show the behind count, the base with its short commit, and the conflicting files. | Shot `560-03-tooltip` |
| REQ-004 | WHEN the worktree merges its base, the chip shall go within five seconds. | Shot `560-04-merged` |
| REQ-005 | WHILE a worktree's branch is up to date with its base, its row shall show no chip. | Shots `560-01-clean`, `560-04-merged` |
| REQ-006 | WHEN the summary runs, it shall move no ref, change no index or working tree of the main checkout or a worktree, and run no fetch. | The run log: `git status --porcelain` and `git for-each-ref` before and after each step; the wrapper's log |
| REQ-007 | WHERE git refuses `merge-tree --write-tree`, the row shall show the behind count and no conflict count, and Marley shall log one line for the repository. | Shot `560-05-unsupported`; Marley.log in the run log |
| REQ-008 | WHERE Zed does not trust the repository, the system shall run no git for the chip. | Review of the diff (the scenario's repository is trusted) |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes. #510 ships first. On
  promotion, check whether #511 landed: its `worktree_git.rs` is extended, else made in its shape.
- **P2 Code:** `conflict_summary` in the adapter; `Drift` in `marley_rail`; the cache, the
  subscription and the chip in `rail.rs`; fmt and clippy clean; a review of the diff against each
  REQ and the spawn's arguments.
- **P3 Test:** write and run the scenario and read every shot; rerun #510's scenario, whose rows
  gain a chip, and #511's if it landed; `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_workbench.md` and `marley_rail.md`;
  the plan's prong 2; the ledger capture; close the ticket, archive, commit.
