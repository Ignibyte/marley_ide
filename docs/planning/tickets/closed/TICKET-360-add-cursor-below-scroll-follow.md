# TICKET-360 — add-cursor-below (⌘⌥↓) scroll-follow tracks the primary, not the new bottom cursor

- **Forge ticket:** #360 `c243e4d4-59d2-4d19-8623-faa8c6103a37` (bug, editor/multi-cursor/336-followup/342-followup/scroll-follow)
- **Owner:** session `99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c`
- **AAR:** `98f090dd-d3a3-4e3f-b913-fc9095f0372c`
- **Pipeline doc:** ../../pipeline/active/360-add-cursor-below-scroll-follow.spec.md
- **Source ticket:** the follow-up goal `/work 360,361,362,363,364` (a #342 inspect follow-up)
- **Status:** closed

## Summary
The add-cursor arm (`add-cursor-above`|`add-cursor-below`, app.rs:7358) mutates the selection set via
`add_cursor_vertical` then calls the bare `self.follow_editor_caret()`, which follows `active_caret()` = the
PRIMARY = member 0. ⌘⌥↑ grows from `set.primary()`, so the new caret sorts to the TOP and BECOMES member 0 →
followed (correct by luck). ⌘⌥↓ grows from `set.last()`, so the new caret sorts to the BOTTOM while the primary
stays member 0 at the top, stationary → the follow tracks the stationary top primary, NOT the new bottom caret →
sustained ⌘⌥↓ marches a column of carets past the bottom viewport edge and the view never follows them (the
"cursors they cannot see" failure `added_member`'s doc + #342 cite). The fix reuses #342's exact seam: gate the
follow on `added_member(&before, &after)` (which names the actually-new cursor regardless of where it sorts) and
route through the column-aware `scroll_editor_to(row, Some(head))` — mirroring the `add-next-occurrence` arm.

## Acceptance
Sustained ⌘⌥↓ past the bottom edge keeps the newest caret's row on screen (the view follows it); ⌘⌥↑ still
follows correctly; a single press with the new cursor already visible does not move the view. Headless-proven via
`editor_scroll_y`. App-shim wiring only (no new pure seam).
