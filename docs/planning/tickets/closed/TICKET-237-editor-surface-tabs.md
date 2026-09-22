# TICKET-237 — Editor-surface tabs (files open as editor tabs, not rail rows)

- **Forge ticket:** #237 (2d01685e-ac8a-4b73-a2c7-d4233b279b88) (feature, M13 sprint #26)
- **Owner:** session c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** 57916c99-669d-42de-ae86-b20adebe9d0a
- **Pipeline doc:** ../../pipeline/completed/editor-surface-tabs.spec.md
- **Source ticket:** M13 sprint #26 (chad feedback #9, the anti-clutter model) — the LAST of #228-237.
- **Status:** closed

## Summary
Opening a file no longer adds a left-rail row. The read-only CodeView tab is recast to hold a multi-file
`EditorSurface` (a path-keyed set of file tabs + an active one), so N open files land in ONE editor
surface (ONE rail row) with N file tabs INSIDE it — the VSCode model, resolving chad's rail-clutter
concern. Read-only v1 (reuses `CodeViewState` + `code_view_body`). Editing (the `Buffer` core) and
terminal-split-to-file are deferred to their own tickets.

## Acceptance
Pure `EditorSurface` (open/dedupe/close/activate, cov/MSI 100); opening a file routes into the single
editor surface (not a new rail row); the editor tab renders a file-tab strip + the active file; N files →
1 rail row + N file tabs (driven). Full EARS in the spec. Editing + split-to-file + persistence deferred.
