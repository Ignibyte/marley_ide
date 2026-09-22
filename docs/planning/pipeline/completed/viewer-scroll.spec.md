---
pipeline_id: c0d0de6b-95c5-45ac-b8ca-2cb0d2502004
ticket: forge#101 (1be96239-3b41-441e-9838-bb8992a876f1) · local docs/planning/tickets/open/TICKET-101-viewer-scroll.md
aar_id: 93c9d996-fe0d-4dce-a66d-635cd1c96fde
status: Phase 5 — Complete PASS
title: scroll + jump-to-line in the viewer
type: feature
milestone: M4 — The Code Panel
references:
  - crates/marley_app/src/code_view.rs (PURE: visible_range, jump_to)
  - crates/marley_app/src/app.rs (SHIM: the viewer scroll keys + the windowed render)
---

## Title
Scroll the code viewer (keys + wheel) and jump to a line (feeds #105's open-at-line) — the viewer renders
only its visible window instead of the first 40 lines.

## Scope
### In
- PURE `visible_range(scroll, height, total)` (clamped window) + `jump_to(line, height, total)` (clamped scroll).
- SHIM: ↑/↓ + ⌘↑/⌘↓ (top/bottom) + wheel scroll `cv.scroll`; the overlay renders the `visible_range` slice.

### Out
- A cursor / current-line highlight. Horizontal scroll. A scrollbar. PgUp/PgDn (uncertain key name).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `visible_range` clamps start to `total-height` (no scroll past the end); `jump_to` puts the line at the top, clamped.
- D2 — the escape close (from #97) stays; handle it BEFORE the `as_mut` borrow.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `visible_range(scroll, height, total)` runs, it shall be the clamped `[start, end)` window. | unit |
| REQ-002 | WHEN `jump_to(line, height, total)` runs, it shall be the clamped scroll showing `line` at the top. | unit |
| REQ-003 (visual) | WHEN ↑/↓/⌘↑/⌘↓/wheel are used, the viewer window shall scroll. | self-test (engine; synthetic keys env-blocked) |
| REQ-004 | gate GREEN, cov/MSI 100 on visible_range/jump_to; the shim masked. | gate |

## Phase Plan
- **P2** — visible_range + jump_to; the scroll-key handler + windowed render; test plan.
- **P3** — implement.
- **P3.5** — 1 critic: visible_range/jump_to MSI (clamps/arith/saturating); the handler bounds; the window.
- **P4** — the range/jump tests (cov/MSI 100) + gate GREEN.
- **P5** — docs, AAR, archive, close #101.
