# TICKET-032 — terminal: scrollback viewport (scroll a pane's output; anchor-to-bottom)

- **Forge ticket:** #32 `ac48e97c-3937-4e91-9a8d-303013498b2b` (feature, M1.D — The Daily Driver, seq-5)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `d27fac81-be82-4699-a3bf-750630018f42`
- **Pipeline doc:** ../../pipeline/active/scrollback-viewport.spec.md
- **Source ticket:** forge sprint #4 `a029f2bc-dabd-43f5-9f6d-13498f0916d5` (M1.D — The Daily Driver)
- **Status:** closed

## Summary
No scrollback — output past the pane fold is clipped with no way to scroll up. Add a pure per-pane
`Viewport { top, following }` (visible-slice math + following-vs-held anchor: new output pins to the
bottom unless the user scrolled up) on `PaneState`; the shim counts content rows, derives capacity
from the cell metric, slices the rendered rows to the visible window, and wires wheel + PageUp/Down
+ shift-arrows. Builds on #30 (metric) + #31 (styled rows).

## Acceptance
`Viewport` at cov 100/MSI 100 (visible bottom-anchor + held window; scroll_up materialise + clamp;
scroll_down re-anchor threshold; held-window-holds-as-content-grows; capacity.max(1)); FULL gate
GREEN [--diff]. Full EARS in the pipeline spec.
