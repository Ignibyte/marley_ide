# TICKET-252 — Editor save (⌘S) + dirty ● indicator

- **Forge ticket:** #252 (6fba8467-f1bf-4a9c-ad79-88938c0b6cba) (feature, M15 sprint #28)
- **Owner:** session c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** c3576454-eafb-4236-b84d-2cc8599209d1
- **Pipeline doc:** ../../pipeline/completed/editor-save.spec.md
- **Source:** the M15 "Editable Editor" train (#252, after #251 made the editor editable).
- **Status:** closed

## Summary
⌘S writes the active editor file's `buffer.text()` to its path on disk (a new app-shim `save_active` — `fs::write`
+ `active_mark_saved`, keeping `editor_surface` pure; `mark_saved` runs ONLY on a successful write, a save error
→ a status flash not a crash) + a dirty ● in the #237 file-tab strip on any file with `buffer.version() !=
saved_version` (a new pure `EditorSurface::file_dirty_flags`). ⌘S is bound to a "save" keymap action, guarded to
an active editor tab.

## Acceptance
Typing shows a ● on that file's tab (per-file); ⌘S (editor active) writes the buffer to disk; a successful save
clears the ● (saved_version == version); a failed write leaves the ● + doesn't crash; ⌘S with a terminal tab
active is a no-op. Pure units cov/MSI 100 (file_dirty_flags + the mark-on-Ok logic + the ⌘S binding); driven
proof on a THROWAWAY temp file. Full EARS in the pipeline spec.
