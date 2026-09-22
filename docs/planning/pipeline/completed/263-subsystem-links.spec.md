---
pipeline_id: bb8c66e7-0e07-4b4e-a7cc-86b4a8fa8724
ticket: forge#263 (4aca09ed-0ce3-4435-aa90-a7b2ba7cf271) · local docs/planning/tickets/open/TICKET-263-subsystem-links.md
aar_id: 60384728-fc0c-4dee-82c3-639549a14091
status: Phase 5 — Complete PASS
title: Fix broken Subsystem cross-links in the Warp reference docs
type: chore
milestone: M16
references: []
---

## Title
Pre-existing doc bug (found in the Round-3 re-review): every
`docs/warp_architecture/crates/*.md` `Subsystem` field links
`../architecture/<file>.md`, but that directory doesn't exist — the real path
is `../subsystems/`. Survey confirms: 78 files carry the broken prefix; all 7
linked target filenames exist under `../subsystems/`; nothing outside
`crates/` uses the broken prefix. Mechanical scoped replace + anchor
verification.

## Scope
### In
- `../architecture/` → `../subsystems/` across the 78
  `docs/warp_architecture/crates/*.md` files; verify every resulting link
  target exists on disk.

### Out (explicitly deferred)
- Any other link hygiene in the reference corpus (only the Subsystem-field
  prefix is broken).

## Reference (§20)
N/A — Marley-internal documentation hygiene on the reference-transcription
corpus itself (no app behavior involved).

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — sed-style scoped replace of the literal `../architecture/` prefix,
  crates/ dir only; then a link-existence check over every `](../` relative
  target in those files (the fix's own proof).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | After the fix, `grep -rn '\.\./architecture/' docs/warp_architecture/` shall return zero matches. | grep exit 1 |
| REQ-002 | Every relative `.md` link target in `docs/warp_architecture/crates/*.md` shall resolve to an existing file. | link-check script exit 0 (run in-transcript) |
| REQ-003 | The change shall touch only `docs/warp_architecture/crates/*.md` (no content edits beyond the path prefix). | `git diff --stat` review + word-diff spot check |

## Phase Plan
Compressed (docs-only, gate-is-test §7): P2 design = the survey above (done
at plan); P3 = the replace; P3.5 = self-review + link-check (the critic step
is the checker script — a mechanical text change needs no adversarial agent
panel, recorded as the inspect rationale); P4 = REQ greps + `gates.sh --fast`
(no `.rs` → static set; commit-receipt exemption per §15 no-`.rs` rule);
P5 = CHANGELOG + AAR + archive + close.
