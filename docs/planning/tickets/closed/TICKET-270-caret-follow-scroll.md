# TICKET-270 — Editor caret-follow scroll (keep the caret visible on motion)

- **Forge ticket:** #270 `8334665d-8f60-4a52-b9b2-97237087d43f` (bug, M18)
- **Owner:** autonomous /goal run (sprint #31 — M18 Terminal↔Editor Fusion)
- **AAR:** `8e65777f-8e89-4949-9ddd-6bf71963fab5`
- **Pipeline doc:** ../../pipeline/active/270-caret-follow-scroll.spec.md
- **Source ticket:** #265 inspect A1 (a pre-existing editor limitation) — the #290 nav prerequisite
- **Status:** closed

## Summary
The editor's uniform_list scrolls only via the wheel; no caret motion adjusts it,
so a far motion (⌘↓ to document end, a wrapping ⌘D select-next, Down past the
viewport) moves the caret off-screen — the press looks like a no-op though state
is correct. #273 already built `scroll_editor_to_row(row)` (the shared,
non-strict gpui scroll mechanism whose doc names "#270 caret-follow" as a
consumer) but the motion sites never call it. Fix: a `follow_editor_caret()` shim
that scrolls the caret's row into view, called after each caret-moving editor
action. The nav prerequisite for #290 (jumping to an off-screen diagnostic).

## Acceptance
After a caret/selection-moving editor action that lands off-screen, the editor
scrolls so the caret row is visible; a motion within the viewport is a no-op
(non-strict scroll). Full EARS in the pipeline spec.
