# TICKET-356 — An unfocused editable split pane renders from its live Buffer, not stale cv.lines

- **Forge ticket:** #356 (1c248d03-43d4-4424-89f1-aec2074f5f6c) (bug, M15 → ships M22)
- **Owner:** 466e35ad-09f6-4b81-89e7-b7fd16c1e45d
- **AAR:** 2bc1e904-1aa2-4449-bc65-c20afbea8531
- **Pipeline doc:** ../../pipeline/active/356-unfocused-pane-stale-render.spec.md
- **Source ticket:** M22 IDE wrap-up train (#306, #295, #356, #357, #355)
- **Status:** closed

## Summary
An UNFOCUSED editable split pane passed `None` to `code_view_body` → the lossy `cv.lines` path (never re-synced
from the Buffer), so edits made while focused "vanished" on click-away until re-focus. Fix: an editable pane
renders from its own live Buffer whether focused or not (via the shared `editor_draw_for`), with only the FOCUSED
surface drawing carets + writing the single geom/IME slot. A pure `code_view::pane_body(has_buffer, focused)`
decision drives it; the #246 read-only file pane (no Buffer) keeps `cv.lines`.

## Acceptance
An unfocused editable pane renders its live Buffer text (not stale cv.lines); only the focused surface owns the
geom/IME slot + carets. Full EARS in the pipeline spec.
