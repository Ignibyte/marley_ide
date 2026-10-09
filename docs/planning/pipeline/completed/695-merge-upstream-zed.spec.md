---
pipeline_id: 030514e9-b780-4045-a822-0a142cd1bdd6
ticket: docs/planning/tickets/open/TICKET-695-merge-upstream-zed.md
status: Phase 4 — Complete PASS
title: Merge upstream Zed
type: chore
slice: upstream sync
references:
  - docs/marley/zed-touchpoints.md
---

## Title
Merge upstream Zed's main of 2026-10-08 into Marley.

## Scope
### In
- `git merge upstream/main` on `marley/upstream-2026-10-08`, with conflicts resolved by
  keeping upstream's change and re-applying each Marley hunk as its touchpoints row describes.
- Build breaks in Marley crates from upstream API changes, fixed at the source.
- `Cargo.lock` taken from upstream, then re-resolved by cargo for Marley's crates.

### Out
- Adopting new upstream features in Marley.

## Reference (§20)
Upstream Zed itself: this is the merge.

### Prior art
`docs/marley/zed-touchpoints.md` (every change outside Marley's paths, read at each merge) and
`vendor/README.md` (the carried alacritty).

## UI proof
The e2e golden regression set (`script/e2e/golden`), which covers each part of Marley, plus
`just shot`. A merge touches everything, so this is the one time the regression set is the visual
check.

## Locked-In Decisions
- **D1:** merge `main`, as the fork's base was main (the v1.22 bump of 2026-09-16), not a
  stable tag.
- **D2:** a merge commit, not a rebase. Marley's 305 commits keep their hashes on the public
  origin.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN merged, the tree shall carry upstream through `9ab0715969` with every touchpoints row's hunk present | review of each conflict against its row |
| REQ-002 | WHEN built, the gate shall be green | `just gate` |
| REQ-003 | WHEN run, the golden e2e set shall pass | the regression run's log |

## Phase Plan
- **P1 Plan:** this spec, from the trial merge (8 conflicted files).
- **P2 Code:** resolve, build, fix, and the gate.
- **P3 Test:** the golden set.
- **P4 Complete:** the CHANGELOG, the touchpoints rows that moved, the ledger, commit, push and
  install.
