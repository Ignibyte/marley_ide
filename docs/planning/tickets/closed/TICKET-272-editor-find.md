# TICKET-272 — Editor find & replace (⌘F) + ⌘A select-all

- **Forge ticket:** #272 ef9424f3-45d6-4d67-8d12-c3488cb136d6 (feature, M17)
- **Owner:** ede913c3-d048-4f39-ad2b-b21cef1efc8e
- **AAR:** d9e23b8d-57ea-4f12-8114-49362f47835c
- **Pipeline doc:** ../../pipeline/completed/272-editor-find.spec.md
- **Source ticket:** sprint #30 (forge)
- **Status:** closed

## Summary
An Editor-context ⌘F find/replace bar (the editor has NO find today):
live "i of N", wrapping Enter/⇧Enter via find.rs, match tints through a
generalized N-range styled_slices channel (current match brighter),
Replace One/All via back-to-front Buffer::edit, Esc closes; the current
match centers via #273's scroll_editor_to_row. ⌘A select-all rides
along. The bar arm obeys the #267 overlay rules (stop_propagation +
text_input_blocked).

## Acceptance
find_all exact + non-overlapping; cycle wraps + centers; tints coexist
with syntax; replaces undo-recorded with valid offsets; the bar owns
typing with no IME leak; terminal ⌘F/⌘A untouched. Full EARS in the
spec.
