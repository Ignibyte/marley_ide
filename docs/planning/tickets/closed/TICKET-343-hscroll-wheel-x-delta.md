# TICKET-343 — The editor h-scroll wheel scales its x delta by line height, not cell width

- **Forge ticket:** #343 `79ef81d4-fff4-4d43-a0bb-2cd7360a83e8` (bug, editor/hscroll/cosmetic, M22)
- **Owner:** session `99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c`
- **AAR:** `ce6bbd26-bdac-433f-8c78-ee953f9bac21`
- **Pipeline doc:** ../../pipeline/active/343-hscroll-wheel-x-delta.spec.md
- **Source:** the polish/debt goal `/work 341,342,343,344,346,347,349,358,359,334` (a #336 follow-up)
- **Status:** closed

## Summary

#336's editor wheel handler (app.rs:4937) computes the horizontal delta as
`event.delta.pixel_delta(px(cell_h)).x` — gpui's `pixel_delta` applies ONE scalar to both axes, so a
`ScrollDelta::Lines` (mouse) x-delta gets scaled by the LINE HEIGHT (~19px) when it should use the cell WIDTH
(~8px), moving ~1.5-2.4x too fast. Narrow: trackpads send `ScrollDelta::Pixels` (exact px, bypasses the scalar),
so only a real mouse's horizontal tilt-wheel is affected. Fix: match on `ScrollDelta` — `Pixels` passthrough,
`Lines` scaled by `cell_w` — via a pure `h_scroll::wheel_x_px` (cov/MSI 100); the app.rs handler is a skip'd
shim. The sign convention (dx negated, positive-right) is unchanged.

## Acceptance

A `Lines` wheel x-delta scales by cell_w (`wheel_x_px(3.0, false, 8.0) == 24.0`), a `Pixels` delta passes through
unchanged (cell_w ignored), the fn is total over hostile input, and the shim still feeds the clamp the negated
delta (sign unchanged). Full EARS (REQ-LINES-CELL-WIDTH, REQ-PIXELS-PASSTHROUGH, REQ-TOTALITY,
REQ-SIGN-UNCHANGED) in the pipeline spec. Verified by pure unit tests — no live drive (a mouse tilt-wheel can't
be synthesized headlessly; the pure fn + the diff-reviewed shim carry it).
