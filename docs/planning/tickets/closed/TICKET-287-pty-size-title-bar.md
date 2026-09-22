# TICKET-287 — pty_size ignores the pane title bar (grid ~1 row too tall — render clip + mouse-row skew)

- **Forge ticket:** #287 `defe74a1-3684-4e49-83ca-7df32bc5d332` (bug, M17)
- **Owner:** autonomous /goal run (sprint #30)
- **AAR:** `4497d877-6c70-4a36-b822-ce994a5d5c3c`
- **Pipeline doc:** ../../pipeline/active/287-pty-size-title-bar.spec.md
- **Source ticket:** #280 inspect F3 (pre-existing since M5 #108)
- **Status:** closed

## Summary
The pane-resize loop (`app.rs:7258`) feeds `plan_resize` the FULL pane rect, but
the render carves `PANE_TITLE_H` (24px) off the content div
(`.top(r.y + PANE_TITLE_H).h((r.h - PANE_TITLE_H).max(0))`). So the PTY grid is
`floor(r.h / cell_h)` rows instead of `floor((r.h - 24) / cell_h)` — ≈1 row too
many. Two consequences: the alt-screen TUI's true TOP row is clipped off-screen
(the grid is bottom-packed under `overflow_hidden`), and #280's `pane_mouse_cell`
(which correctly trusts `pty_size`) inherits the same ≈1-row skew for clicks near
the top of a tall pane. ONE fix heals both: size the PTY to the CARVED content
rect via a pure `inset_top(rect, inset)` (mirroring `inset_right`), matching the
render exactly.

## Acceptance
The PTY row count is `floor((pane.h - PANE_TITLE_H) / cell_h)` (the content
height), so the TUI's top row is visible and `pane_mouse_cell` maps a top-row
click to row 0/1. Full EARS in the pipeline spec.
