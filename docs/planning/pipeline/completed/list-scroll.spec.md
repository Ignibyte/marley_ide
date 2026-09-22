---
pipeline_id: eba15a93-9e0c-4ad3-ad90-e7403fdbcc14
ticket: forge#169 (ab767d07-a74a-4c27-8bb2-b38456d4ed52) · local docs/planning/tickets/open/TICKET-169-list-scroll.md
aar_id: ec887cf5-7d59-40cf-8a74-920b28cabc36
status: Phase 5 — Complete PASS
title: M10 — scroll the rail + Files panel on overflow
type: feature
milestone: M10 — Warp polish + shell hardening
references:
  - crates/marley_app/src/app.rs (SHIM: the two wheel handlers + row-skip offsets; scroll_code reused)
---

## Title
The left lists scroll — the rail (many tabs) and the Files tree (a deep repo) wheel past their folds instead
of clipping. Proven needed by #168's own capture.

## Scope
### In — SHIM only (the clamp is the reused, already-tested `scroll_code`)
- `rail_scroll` / `files_scroll` row offsets (+ f32 remainders, the #165 precision); wheel handlers on the
  rail container + the Files panel; the render loops skip the first N rows (render-time re-clamp so a shrunk
  list never blanks); `sync_active_project` resets `files_scroll`.

### Out
- Scrollbars/indicators; pixel-smooth (row-granular is the codebase's list idiom); keyboard paging.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 (visual) | WHEN the rail overflows (12+ tabs via ⌘T), wheeling over it shall bring later rows into view; wheeling far up shall pin back to the top. | driven capture |
| REQ-002 (visual) | WHEN the Files tree overflows, wheeling over the panel shall reveal rows past the fold (e.g. `crates/`). | driven capture |
| REQ-003 | The offsets shall clamp via `scroll_code` (no new pure surface — its cov/MSI-100 tests stand) and re-clamp at render so a shrunk list never renders blank. | code + gate |
| REQ-004 | gate GREEN. | gate |

## Phase Plan
P2 folded. P3 implement. P3.5 self-review (offset lifecycle: shrink/e-walk/reset; wheel-region disjointness
vs #165's code-tab wheel). P4 driven + gate. P5 docs.
