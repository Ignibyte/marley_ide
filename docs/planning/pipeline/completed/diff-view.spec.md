---
pipeline_id: fd66734e-26d3-4de0-bce4-0be673057a99
ticket: forge#103 (b1eaffcb-0290-4c13-8c20-9869d0a34bc1) · local docs/planning/tickets/open/TICKET-103-diff-view.md
aar_id: 56535934-d25f-4c8a-aeca-7820268bec61
status: Phase 5 — Complete PASS
title: a diff view (diff_rows projection)
type: feature
milestone: M4 — The Code Panel
references:
  - crates/marley_app/src/git_diff.rs (PURE: DiffRole, DiffRow, diff_rows)
  - crates/marley_app/src/app.rs (SHIM: the ⌘⇧D overlay renders diff_rows)
---

## Title
Formalize the diff render as a pure `diff_rows` projection (file/hunk/line rows with color roles) and
refactor #102's ⌘⇧D overlay onto it — the +/- styling is now a tested projection.

## Scope
### In
- PURE `diff_rows(files) -> Vec<DiffRow>` + `DiffRow`/`DiffRole`.
- SHIM: the #102 ⌘⇧D overlay renders `diff_rows` (role → theme color), replacing its inline nested loop.

### Out
- A per-file File↔Diff toggle in the code viewer (folded into the ⌘⇧D whole-diff overlay). Word-level diff.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `diff_rows` flattens the FileDiff tree to rows (FileHeader / HunkHeader / Add / Remove / Context);
  add/remove rows carry a +/- sign; empty → one "no working-tree changes" row.
- D2 — role → color: FileHeader→foreground, HunkHeader→accent, Add→success, Remove→danger, Context→muted.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `diff_rows(files)` runs, it shall emit FileHeader → HunkHeader → signed line rows in order, roles per kind. | unit |
| REQ-002 | WHEN files is empty, `diff_rows` shall emit one "no working-tree changes" row. | unit |
| REQ-003 (visual) | WHEN ⌘⇧D shows, the diff shall render colored via diff_rows. | self-test (engine + #102 live git) |
| REQ-004 | gate GREEN, cov/MSI 100 on diff_rows; the shim masked. | gate |

## Phase Plan
- **P2** — diff_rows + DiffRow/DiffRole; the overlay refactor; test plan.
- **P3** — implement.
- **P3.5** — 1 critic: diff_rows MSI (empty/order/role+sign); the overlay refactor output-equivalent.
- **P4** — diff_rows tests (cov/MSI 100) + gate GREEN.
- **P5** — docs, AAR, archive, close #103.
