---
pipeline_id: f7a0e4b8-ae40-42b6-b1a9-89a248676c4b
ticket: forge#102 (d0450d67-eb2a-49eb-b807-d47bd193fe3d) · local docs/planning/tickets/open/TICKET-102-git-diff-seam.md
aar_id: 14643a8e-1d40-422e-bf3f-10a58728c152
status: Phase 5 — Complete PASS
title: a read-only git-diff seam
type: feature
milestone: M4 — The Code Panel
references:
  - crates/marley_app/src/git_diff.rs (NEW PURE: FileDiff/Hunk/DiffLine, parse_diff)
  - crates/marley_app/src/lib.rs (mod git_diff)
  - crates/marley_app/src/app.rs (SHIM: git_working_diff spawn + the ⌘⇧D diff overlay)
---

## Title
A read-only git seam: parse `git diff` into a structured `Vec<FileDiff>`, and a ⌘⇧D that shows the working-
tree diff in a colored overlay. The foundation for the diff view (#103) and the agent-diff (#104).

## Scope
### In
- NEW pure `git_diff.rs`: `FileDiff`/`Hunk`/`DiffLine`/`DiffKind` + `parse_diff(output)`.
- SHIM: `git_working_diff` (spawn `git diff` in the project root, capture, parse) + ⌘⇧D toggles a diff
  overlay (colored +/-/context).

### Out
- The File↔Diff toggle inside the viewer (#103). The agent-scoped diff (#104). Any git WRITE. `--stat`.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `parse_diff` is the pure tested surface; the `git diff` spawn is a masked read-only adapter (app.rs, cov-excluded).
- D2 — ⌘⇧D toggles a separate diff overlay in v1 (#103 integrates it into the viewer).
- D3 — colors: Add→success, Remove→danger, Context→muted, hunk header→accent.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `parse_diff(output)` runs, it shall produce one FileDiff per `diff --git`, hunks per `@@`, and +/-/space lines classified. | unit |
| REQ-002 | WHEN the diff is empty or has pre-hunk lines, `parse_diff` shall yield [] / ignore them. | unit |
| REQ-003 (visual) | WHEN ⌘⇧D is pressed, the working diff shall show colored. | self-test (live git spawn + engine) |
| REQ-004 | gate GREEN, cov/MSI 100 on git_diff.rs; the shim masked. | gate |

## Phase Plan
- **P2** — git_diff.rs API + parse_diff algorithm; the spawn + overlay + ⌘⇧D; test plan.
- **P3** — implement (git_diff.rs + lib.rs + app.rs).
- **P3.5** — 1 critic: parse_diff MSI (file/hunk/line/path/flush); the read-only spawn; the overlay + routing.
- **P4** — parse_diff tests (cov/MSI 100) + a live `git diff` check + gate GREEN.
- **P5** — docs, AAR, archive, close #102.
