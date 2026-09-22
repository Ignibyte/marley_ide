# TICKET-101 — scroll + jump-to-line in the viewer

- **Forge ticket:** #101 `1be96239-3b41-441e-9838-bb8992a876f1` (feature, M4 seq-5; sprint #15)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `93c9d996-fe0d-4dce-a66d-635cd1c96fde`
- **Pipeline doc:** ../../pipeline/active/viewer-scroll.spec.md
- **Status:** closed

## Summary
Scroll the viewer + jump_to a line. PURE `visible_range(scroll,height,total)` + `jump_to(line,height,total)`
(cov/MSI 100); the shim adds ↑/↓/⌘↑/⌘↓/wheel scroll + renders the visible window. Deps #97 + #32 + #48.

## Acceptance
visible_range + jump_to at cov/MSI 100 (clamps/arith); scrolling tracks the window (engine/self-test);
FULL gate GREEN. Full EARS in the spec.
