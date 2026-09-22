# TICKET-253 — Editor undo/redo — edit-history + ⌘Z / ⌘⇧Z

- **Forge ticket:** #253 (83b62a5a-f30e-48cf-b135-0c2430474eca) (feature, M15 sprint #28)
- **Owner:** session c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** df83094b-ffd1-42e1-88ef-1c85c04ac691
- **Pipeline doc:** ../../pipeline/completed/editor-undo.spec.md
- **Source:** the M15 "Editable Editor" train (#253, after save #252); chad put undo in v1.
- **Status:** open

## Summary
Add a pure in-memory edit-history to `marley_editor::Buffer` (an `UndoHistory` of invertible
`EditRecord{at,removed,inserted}`; `edit()` records + coalesces contiguous single-char inserts; `undo()`/`redo()`
apply the inverse via a non-recording `apply_raw` + return the caret site; a new edit clears redo) + ⌘Z (undo) /
⌘⇧Z (redo) keymap bindings + guarded `dispatch_action` arms routing to the active editor buffer with a caret
sync. So ⌘Z removes a typed run/word (not one char) and restores the caret.

## Acceptance
edit() records an invertible step (not on the inverse re-apply); undo() reverts the last coalesced edit + returns
the caret site (None when empty); a typed run coalesces to one undo step; redo() re-applies + a new edit clears
redo; ⌘Z/⌘⇧Z with an editor tab active undo/redo + sync the caret, terminal unaffected. Pure units cov/MSI 100 on
the history; driven proof (type→⌘Z→gone→⌘⇧Z→back). Full EARS in the pipeline spec.
