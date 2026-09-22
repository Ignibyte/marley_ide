# TICKET-130 — pane split + drag-resize + close affordances [M6]

- **Forge ticket:** #130 `236fab76-d501-466c-82ce-de143ca87151` (feature, M6; sprint #17)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `220cc4ce-03e8-48aa-be1c-2d4483ea85de`
- **Pipeline doc:** ../../pipeline/active/pane-resize.spec.md
- **Status:** closed

## Summary
Pure `resize_split(ratios, boundary, delta, min)` (layout.rs, cov/MSI 100) + `resize_boundary` on
PaneGroup/Workspace + a draggable divider handle (app.rs shim) so the fixed 50/50 splits become adjustable,
clamped so no pane collapses. Deps #120 + M2 layout.

## Acceptance
resize_split + resize_boundary at cov/MSI 100; divider handles render (live capture; the drag gesture is
env-blocked / code-reviewed); FULL gate GREEN. Full EARS in the spec.
