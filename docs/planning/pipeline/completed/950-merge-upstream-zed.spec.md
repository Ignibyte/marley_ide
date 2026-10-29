---
pipeline_id: c3252f74-5ab9-4b81-8644-fc61aa10af5c
ticket: docs/planning/tickets/closed/TICKET-950-merge-upstream-zed.md
status: Phase 4 — Complete PASS
title: Merge upstream Zed through 2026-10-29
type: chore
slice: upstream sync
references:
  - docs/marley/zed-touchpoints.md
  - docs/planning/pipeline/completed/797-merge-upstream-zed.spec.md
  - docs/planning/pipeline/completed/695-merge-upstream-zed.spec.md
---

## Title
Merge upstream Zed's main of 2026-10-29 (`0373366f43`) into Marley, so the fork carries what
v1.26.2 and v1.27.0-pre ship and the work since, and move Marley's turn checkpoints off the git
API upstream removed.

## Scope
### In
- `git merge upstream/main` at `0373366f43` on a branch from `marley/workbench-shell`, each of the
  ten conflicts resolved by keeping upstream's change and re-applying Marley's hunk as its
  touchpoints row describes.
- Turn checkpoints: upstream's `eee7501970` (#65989) removes `Repository::checkpoint`,
  `compare_checkpoints` and `GitRepositoryCheckpoint`. `marley_workbench`'s `turns.rs` and
  `turn_undo.rs` move to `Repository::create_archive_checkpoint` (kept upstream for worktree
  archives) and compare trees in `turn_git.rs` (D4).
- Build breaks in the Marley crates from upstream's API changes, fixed at the source:
  `ScrollWheelEvent` renamed `ScrollEvent` (#65805), `Modifiers::secondary` takes `cx` (#65970),
  `remote_connection_options` returns a reference, `PathStyle` reworked (#65782, #65966), ACP SDK
  3.6.0 (#66245), Taffy 0.15 (#65950).
- `Cargo.lock`: Marley's lockfile kept and cargo left to add upstream's new dependencies
  (PR-claude-695-an-upstream-merge-keeps-marleys-lockfile-001).
- The merge-time steps in `docs/marley/zed-touchpoints.md` ("At an upstream merge"), the upstream
  base moved in that doc and in `UPSTREAM_BASE_FALLBACK`.

### Out (explicitly deferred)
- Adopting upstream's new features in Marley (ACP v1 subagents behind `acp-beta`, the subagent
  cards).
- Bringing back Zed's own "Restore Checkpoint" button: upstream removed it, and Marley's turn
  undo covers terminal agents.
- The golden set's drift (29 of 53 failing before #797); its repair belongs to the testing phase
  at the end (§7).

## Reference (§20)
Upstream Zed itself: this is the merge. The turn checkpoints keep #509's and #714's behavior; the
reference for them is those tickets' own specs.

### Prior art
- `docs/marley/zed-touchpoints.md`: every change outside Marley's paths, with what to keep at a
  merge.
- #695 and #797's notes (`docs/planning/pipeline/completed/`): the lockfile trap, the merge-aware
  `upstream_base` and gates 12 and 13, the lint nightly gap, semgrep's time limit on a loaded box,
  the golden set compared like with like.
- The code we ship, for the checkpoint seam: upstream kept `Repository::create_archive_checkpoint`
  (`crates/project/src/git_store.rs`, `crates/git/src/repository.rs`), which commits the full
  working state (staged, unstaged and untracked) through a temporary index in the repository's
  job queue. It owns the seam Marley's turns need, so Marley takes it instead of carrying the
  removed `checkpoint` as a Zed hunk or writing its own in `turn_git.rs`. Tree comparison is one
  `git diff-tree --quiet`, the same call the removed `compare_checkpoints` made, and
  `turn_git.rs` already runs `git` for the turn commits.
- `vendor/README.md`: the carried alacritty; upstream did not move Zed's alacritty rev.

## UI proof
The visual check is the turn scenarios on the merged release build, since the turn checkpoints
are the one behavior this ticket rewrites: `script/e2e/509-per-turn-diffs.sh`,
`script/e2e/714-undo-a-turn.sh` and `script/e2e/753-codex-turn-checkpoints.sh`, each run on the
installed pre-merge build too, so only a pass-before, fail-after counts against the merge. Plus
`just shot 950-merged`, the merged build starting in the Marley layout.

## Locked-In Decisions
- **D1:** merge `main` at `0373366f43`, not a release tag: the fork's base is main, and the tags
  sit on release branches cut from it.
- **D2:** a merge commit, not a rebase, so the pushed commits keep their hashes.
- **D3:** Marley's lockfile is the base; upstream's is never taken whole.
- **D4:** turn checkpoints use upstream's `create_archive_checkpoint`, its full-state commit
  (`unstaged_sha`) as the checkpoint; no Zed hunk brings the removed API back.
- **D5:** upstream is right in every conflict (Chad, 2026-10-29: "lets fix the conflicts in terms
  of assuming zed is right and working around it"). Each conflicted file starts from upstream's
  side. A Marley hunk goes back only as an addition that leaves upstream's lines as they are;
  where it would change them, the behavior moves around upstream's code (a Marley crate, a
  caller, a gate tool) instead.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN merged, the tree shall carry upstream through `0373366f43` with every touchpoints row's hunk present | review of each conflict against its row; `Marley:` markers counted per file before and after; gate:16 green |
| REQ-002 | WHEN built, the gate shall be green | `script/gates.sh --diff` exit code |
| REQ-003 | WHEN a terminal's Claude Code turn changes the tree, the rail shall list the turn with its files, as before the merge | `509-per-turn-diffs.sh` on the merged release build, against the pre-merge build |
| REQ-004 | WHEN the user undoes a turn, Marley shall take out that turn's changes and keep later ones, as before the merge | `714-undo-a-turn.sh` on the merged release build, against the pre-merge build |
| REQ-005 | WHEN a Codex turn ends, Marley shall record its checkpoint as before the merge | `753-codex-turn-checkpoints.sh` on the merged release build, against the pre-merge build |
| REQ-006 | WHEN started, the merged Marley shall open in the Marley layout with the rail and a center terminal | `just shot 950-merged` |

## Phase Plan
- **P1 Plan:** this spec, from the trial merge (10 conflicted files) and the checkpoint removal.
- **P2 Code:** merge, resolve, move the turn checkpoints, build, fix, the review, and the gate.
- **P3 Test:** the three turn scenarios on a release build against the pre-merge build, and the
  shot.
- **P4 Complete:** the CHANGELOG, the touchpoints rows and base that moved, the ledger, commit,
  push and install.
