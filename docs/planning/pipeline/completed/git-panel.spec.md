---
pipeline_id: a20cadbb-d4f8-44a5-b218-39f037b7955b
ticket: forge#115 (a3897c5f-fecf-48fb-a199-8f1327434739) · local docs/planning/tickets/open/TICKET-115-git-panel.md
aar_id: 18d512ca-f7d2-4770-b47e-54d4af83e9b0
status: Phase 5 — Complete PASS
title: the git changes panel (source control)
type: feature
milestone: M5 — The Warp Workspace
references:
  - crates/marley_app/src/git_diff.rs (PURE: ChangeStatus, ChangeRow, parse_status, change_summary)
  - crates/marley_app/src/app.rs (SHIM: ⌘⇧C git panel + git_status spawn)
---

## Title
A source-control panel (⌘⇧C): the working tree's uncommitted changes as a per-file list (status + staged),
or the "No open changes" empty state — the Warp git pane.

## Scope
### In
- PURE `parse_status(porcelain)` → `Vec<ChangeRow>` + `change_summary(rows)`.
- SHIM: ⌘⇧C toggles a right-side git panel; a masked `git status --porcelain` spawn feeds it; a row opens the diff.

### Out
- Staging/unstaging + commit (that's #116). Per-file diffs (the row opens the whole working diff for now).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `parse_status` reads the porcelain XY columns: staged = the index column non-blank; `??` = untracked.
- D2 — ⌘⇧C toggles the panel; its handler runs BEFORE the copy handler (which ignores shift) and returns.
- D3 — the right region shows the git panel when open, else the viewer (#114); it takes precedence.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `parse_status(porcelain)` runs, each line shall yield a ChangeRow (status + staged from XY; `??`→Untracked). | unit |
| REQ-002 | WHEN `change_summary(rows)` runs, it shall be "no changes" / "N changed". | unit |
| REQ-003 (visual) | WHEN ⌘⇧C is pressed, a git panel shall show the changes (or "No open changes"). | live capture |
| REQ-004 | gate GREEN, cov/MSI 100 on parse_status/change_summary; the shim masked. | gate |

## Phase Plan
- **P2** — ChangeStatus/ChangeRow + parse_status/change_summary; the ⌘⇧C panel + git_status spawn; test plan.
- **P3** — implement (git_diff.rs + app.rs).
- **P3.5** — 1 critic: parse_status MSI (XY/staged/status/untracked); the ⌘⇧C-before-copy order; right-region precedence.
- **P4** — parse_status/change_summary tests (cov/MSI 100) + a live capture + gate GREEN.
- **P5** — docs, AAR, archive, close #115.
