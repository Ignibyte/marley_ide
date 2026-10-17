---
pipeline_id: 4b5b0d36-188c-440b-a185-8c4584dcb3bf
ticket: docs/planning/tickets/closed/TICKET-797-merge-upstream-zed.md
status: Phase 4 — Complete PASS
title: Merge upstream Zed through 2026-10-17
type: chore
slice: upstream sync
references:
  - docs/marley/zed-touchpoints.md
  - docs/planning/pipeline/completed/695-merge-upstream-zed.spec.md
---

## Title
Merge upstream Zed's main of 2026-10-17 (`56934ec056`) into Marley, so the fork carries what
v1.24.3 and v1.25.0-pre ship and the work since.

## Scope
### In
- `git merge upstream/main` at `56934ec056` on `marley/workbench-shell`, conflicts resolved by
  keeping upstream's change and re-applying each Marley hunk as its touchpoints row describes.
- Build breaks in the Marley crates from upstream's API changes, fixed at the source. Most of the
  110 commits move the agent protocol to ACP v2 (ACP SDK 3.3, v2 sessions, tool ids and
  permission routing), which `marley_workbench`'s thread, permission and session code reads.
- `Cargo.lock`: Marley's lockfile kept and cargo left to add upstream's new dependencies
  (PR-claude-695-an-upstream-merge-keeps-marleys-lockfile-001).
- The merge-time steps in `docs/marley/zed-touchpoints.md` ("At an upstream merge").

### Out (explicitly deferred)
- Adopting upstream's new features in Marley.
- The pre-existing golden-set failures #695 recorded (508, 510, 516, 568, 569, 585, 587).

## Reference (§20)
Upstream Zed itself: this is the merge.

### Prior art
- `docs/marley/zed-touchpoints.md`: every change outside Marley's paths, with what to keep at a
  merge.
- #695's notes (`docs/planning/pipeline/completed/695-merge-upstream-zed.notes.md`): the
  lockfile trap, the hook and gate fixes made for merges (`upstream_base` reads `MERGE_HEAD`;
  gates 12 and 13 judge only lines new against both sides), the font-size and menu traps in the
  scenarios.
- `vendor/README.md`: the carried alacritty, re-synced only if upstream moves Zed's rev.

## UI proof
The e2e golden regression set (`script/e2e/golden`) on a release build, plus `just shot`. A
merge touches everything, so, as in #695, the regression set is the visual check.

## Locked-In Decisions
- **D1:** merge `main` at `56934ec056`, not a release tag: the fork's base is main, and the tags
  sit on release branches cut from it.
- **D2:** a merge commit, not a rebase, so the pushed commits keep their hashes.
- **D3:** Marley's lockfile is the base; upstream's is never taken whole.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN merged, the tree shall carry upstream through `56934ec056` with every touchpoints row's hunk present | review of each conflict against its row; gate:16 green |
| REQ-002 | WHEN built, the gate shall be green | `script/gates.sh --diff` exit code |
| REQ-003 | WHEN run on a release build, the golden e2e set shall pass apart from the seven failures #695 recorded | the regression run's log |
| REQ-004 | WHEN started, the merged Marley shall open in the Marley layout with the rail and a center terminal | `just shot 797-merged` |

## Phase Plan
- **P1 Plan:** this spec, from the trial merge (2 conflicted files).
- **P2 Code:** merge, resolve, build, fix, and the gate.
- **P3 Test:** the golden set on a release build, and the shot.
- **P4 Complete:** the CHANGELOG, the touchpoints rows that moved, the ledger, commit, push and
  install.
