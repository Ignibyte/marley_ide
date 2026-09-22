# TICKET-191 — Full 4-side accent border on the focused pane

- **Forge ticket:** #191 (578fad03-7e09-467a-8f56-2c13d43907a7) (feature, M12.1)
- **Owner:** claude (session c104f25c)
- **AAR:** 5aa2c0f2-07eb-4d87-9db9-892bbd667157
- **Pipeline doc:** ../../pipeline/active/focus-border-full.spec.md
- **Source ticket:** M12.1 sprint #24 — chad live-app feedback #2
- **Status:** closed

## Summary
The focused pane shows only a left+top 2px accent edge (#131). chad wants the "blue box" to frame all four
sides. Add a pure `focus_border_rects(content, thickness)` (4 edge rects, right/bottom inset so they stay inside
the pane — works with #189's gutter) and draw 4 thin accent bars in the render (not a frame div, so the pane
interior stays clickable).

## Acceptance
The focused pane shows a full 4-side accent border; an unfocused pane shows none. Full EARS in the pipeline spec.
