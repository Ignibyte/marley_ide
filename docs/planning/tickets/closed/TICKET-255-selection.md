# TICKET-255 — Editor text selection — shift+arrows + mouse-drag

- **Forge ticket:** #255 (fc52ec37-67ce-46a1-a976-57dd7ab90069) (feature, M15)
- **Owner:** (autonomous /goal /work 249 to 258)
- **AAR:** dac6c569-dc59-4832-a831-dc658eb44b85
- **Pipeline doc:** ../../pipeline/completed/selection.spec.md
- **Source ticket:** M15 — The Editable Editor (sprint #28, fa328492); the #242 decomposition (#248–258)
- **Status:** closed

## Summary
The editor has a single caret (#251 types, #254 clicks). #255 makes it a live selection: shift+arrows extend a
range from the caret (anchor fixed, head moves), a mouse-drag selects (down sets the anchor, move extends the
head), the #250 renderer highlights the span, and typing/backspace over a selection replaces it. Reuses
`marley_editor::Selection` + #254's click→offset map. The pure seam (the shift-extend/collapse fn +
`row_selection_cols`) is cov/MSI 100; the drag + shift wiring + the highlight render are the shim. Deps #249/#250/
#251/#254.

## Acceptance
Shift+arrow extends / an unshifted arrow collapses; a mouse-drag selects a range; the selection is highlighted;
typing/backspace replaces a non-empty selection and collapses to a caret; the model is pure. Driven-proven live.
Full EARS criteria in the pipeline spec. Deferred: multi-cursor, block-select, double/triple-click, shift+word/
Home/End (#257), clipboard copy/paste (#256).
