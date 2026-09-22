# TICKET-249 — Editor doc model — Buffer + caret + saved_version behind the active file

- **Forge ticket:** #249 (bcec6818-fec6-40b3-adf1-1fb8eacc13f8) (feature, M15 sprint #28)
- **Owner:** session c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** 64751716-e58b-4c44-aaf0-6c2756d55f34
- **Pipeline doc:** ../../pipeline/completed/editor-doc-model.spec.md
- **Source:** the M15 "Editable Editor" train; the FOUNDATION ticket (#250-258 build on it). Decomposed from #242.
- **Status:** closed

## Summary
Back each open editor file with a real editable `marley_editor::Buffer` + a caret + a `saved_version`, via an
`OpenFile { view: CodeViewState, buffer, caret, saved_version }` per file in `EditorSurface` (option B — keep
`CodeViewState` derivable for the read-only split pane). `active_file()` still returns the `CodeViewState` so
render + #243 persistence + the file-tab strip are unchanged; new accessors expose the active file's buffer /
caret / dirty. Resolve the `Buffer`-derives-nothing cascade (contained at `EditorSurface`; add derives to Buffer
or drop them on the surface — whichever is minimal). The model is invisible until #250 renders from it.

## Acceptance
Each open file carries a Buffer (seeded from its text) + caret + saved_version; dirty ⇔ version ≠ saved_version;
opening a path dedupes; the read/split/persist paths are unaffected + the workspace compiles. Pure units cov/MSI
100 on the surface's OpenFile management. Full EARS in the pipeline spec.
