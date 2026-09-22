---
pipeline_id: 7a89459e-366b-4885-a064-047ca1910052
ticket: forge#130 (236fab76-d501-466c-82ce-de143ca87151) · local docs/planning/tickets/open/TICKET-130-pane-resize.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: pane drag-resize (M6)
type: feature
milestone: M6 — The Warp Layout
references:
  - crates/marley_app/src/layout.rs (PURE: resize_split, PaneGroup::resize_boundary)
  - crates/marley_app/src/workspace.rs (PURE: Workspace::resize_boundary)
  - crates/marley_app/src/app.rs (SHIM: draggable divider handles; drag state)
---

## Title
Drag a pane divider to resize — the fixed 50/50 splits become adjustable, clamped so no pane collapses.

## Scope
### In
- PURE `resize_split(ratios, boundary, delta, min)` + `PaneGroup::resize_boundary` + `Workspace::resize_boundary`.
- SHIM: a draggable divider handle at each top-level pane boundary; a drag-state field.

### Out
- Nested-split resize (top-level only for now). Persisting the ratios across launch.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `resize_split` moves size across one boundary (children b and b+1), clamped to `[min, total-min]`;
  the other ratios and the sum (=1) are unchanged; an out-of-range boundary or no-room (`total < 2·min`)
  returns the ratios unchanged.
- D2 — `resize_boundary` applies only to a top-level `Split` (a `Leaf`/nested is a no-op for now).
- D3 — the drag gesture (synthetic mouse) is ENV-BLOCKED → the shim divider is code-reviewed; the pure
  `resize_split` carries the cov/MSI-100 proof; the capture shows the handles.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `resize_split([0.5,0.5],0,+0.1,0.1)` runs, it shall return `[0.6,0.4]`. | unit |
| REQ-002 | WHEN a delta would push a side below `min`, `resize_split` shall clamp so both sides ≥ min (sum=1). | unit |
| REQ-003 | WHEN a 3-way split resizes boundary 1, only children 1 and 2 shall change (child 0 unchanged). | unit |
| REQ-004 | WHEN the boundary is out of range or `total < 2·min`, `resize_split` shall return the ratios unchanged. | unit |
| REQ-005 | gate GREEN, cov/MSI 100 on resize_split + resize_boundary; the shim masked. | gate |

## Phase Plan
- **P2** — resize_split/resize_boundary; the divider shim; test plan.
- **P3** — implement (layout.rs + workspace.rs + app.rs).
- **P3.5** — 1 self-review: delta signs, clamp bounds, boundary range.
- **P4** — resize_split/resize_boundary tests (cov/MSI 100) + a capture (divider handles) + gate GREEN.
- **P5** — docs, AAR, archive, close #130.
