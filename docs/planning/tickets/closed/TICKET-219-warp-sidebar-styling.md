# TICKET-219 — Warp visual parity: session sidebar & tab-row styling

- **Forge ticket:** #219 (6de7c89d-96b2-4f5d-9cc7-c875c1dad801) (feature, M12.2, warp-parity, sidebar)
- **Owner:** c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** 1d75b4a5-6514-4ab5-bce9-cc38ac424ddd
- **Pipeline doc:** ../../pipeline/active/warp-sidebar-styling.spec.md
- **Source ticket:** Warp-parity thread (#215); M12.2 sprint #25
- **Status:** closed

## Summary
Match Warp's left session list. Genuine delta: the active Tab/Pane row highlights with `bg(surface)`,
but the rail's dock is also `surface` (#194) → the selection is invisible (reads only via brighter text).
Warp shows a distinct rounded lighter box + a hover highlight. Make Marley's active-row highlight visible
+ Warp-like (a distinct rounded fill), add a row hover highlight, and pin the fix with a pure
`rail_active_highlight` helper guarded to stay perceptibly distinct from `surface`. Defer per-row icons /
2-line subtitles (larger separate decisions). Clean-room — tokens only, no Warp assets.

## Acceptance
Active Tab/Pane row shows a visible rounded highlight distinct from the dock bg; rows highlight on hover;
the highlight helper is guarded distinct-from-surface; text/labels/×/glyph unchanged. Full EARS
(REQ-001..005) in the pipeline spec.
