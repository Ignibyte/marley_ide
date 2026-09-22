# TICKET-220 — Warp visual parity: cursor & selection styling

- **Forge ticket:** #220 (6172b7fc-2430-425b-af71-3b0b7cea493d) (feature, M12.2, warp-parity, cursor)
- **Owner:** c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** 79dfdb78-0b12-4a7f-af7c-653ba2c48791
- **Pipeline doc:** ../../pipeline/active/warp-cursor-styling.spec.md
- **Source ticket:** Warp-parity thread (#215); M12.2 sprint #25
- **Status:** closed

## Summary
Match Warp's terminal cursor focus behavior. The ticket's "2px bar" premise is stale — #218 already made
the prompt caret a solid block. The genuine delta: Marley's block cursor is always solid accent, even on
an unfocused pane (so in a split, both panes' cursors look identical); Warp draws a solid filled block on
the focused pane and a HOLLOW outline on unfocused panes. Make the block cursor focus-aware (solid focused
/ hollow unfocused) via the existing `is_focused` flag. Also evaluate the drag-selection tint (accent@0.3)
vs Warp's neutral selection — calibrate via a pure helper only if warranted. Clean-room — tokens only.

## Acceptance
Focused pane cursor solid-filled; unfocused pane cursor a hollow accent outline; the selection tint reads
Warp-like (calibrated or kept with rationale); caret position + find tint unchanged. Full EARS
(REQ-001..005) in the pipeline spec.
