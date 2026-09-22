# TICKET-030 — terminal: PTY resize-to-pane (winsize follows the pane rect)

- **Forge ticket:** #30 `1b3f9d1d-c871-40b7-9ebb-679612780807` (feature, M1.D — The Daily Driver, seq-3)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `a50da8f0-5ae6-4828-b951-a46818017dfa`
- **Pipeline doc:** ../../pipeline/active/pty-resize-to-pane.spec.md
- **Source ticket:** forge sprint #4 `a029f2bc-dabd-43f5-9f6d-13498f0916d5` (M1.D — The Daily Driver)
- **Status:** closed

## Summary
Every pane's PTY is a fixed 80×24; output doesn't reflow to the pane/window. Add a pure
`plan_resize(rect, cell, current) -> Option<(cols, rows)>` (floor-divide per axis, clamp ≥1, guard
zero/neg/NaN, unchanged→None) and wire the shim to read the gpui cell metrics + `pane_rects` and
call `TerminalSession::resize` (R17) on changed panes; `PaneState` tracks the last `pty_size`.
Closes the #23/#26 80×24 deferral.

## Acceptance
`plan_resize` at cov 100/MSI 100 (floor-divide with a non-square fixture so the axis swap is
caught; clamp ≥1; zero/neg/NaN guard; equal→None / differ→Some); FULL gate GREEN [--diff]. Full
EARS in the pipeline spec.
