# TICKET-250 — Faithful editor renderer — draw from the Buffer with a visible caret (exact offset↔column)

- **Forge ticket:** #250 (da40fb6e-a4b2-4100-b567-2811776095d7) (feature, M15 sprint #28)
- **Owner:** session c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** 786bca71-dafc-48e1-b418-e54f0c30fe7c
- **Pipeline doc:** ../../pipeline/completed/faithful-renderer.spec.md
- **Source:** the M15 "Editable Editor" train; THE make-or-break ticket (#251-258 build on the offset↔column seam).
- **Status:** closed

## Summary
Render the active editor file FROM the `marley_editor::Buffer` (not the lossy, tab-expanded, 200-col-truncated
`CodeViewState.lines`), un-truncated, with a VISIBLE caret at the active file's caret offset — so on-screen
columns line up EXACTLY with character offsets. Expose the offset↔column mapping as a tested PURE seam in
`code_view.rs` (reused by this ticket's caret, #254 mouse-click→offset, #255 selection); `expand_tabs` is
reimplemented on it (one source of truth). New plumbing: `Buffer::line_text(row)` (marley_editor) +
`EditorSurface::active_buffer()` (immutable). The caret is an overlaid block at `column × measured monospace cell
width` (reusing the terminal's `em_advance`). The #246 split pane (bufferless), #243 persistence, and the file-tab
strip are unaffected. Deferred: caret movement/input (#251), mouse (#254), selection (#255), h-scroll + wide-char
width, split-pane-from-buffer (#258).

## Acceptance
A pure offset↔column fn maps char offset→display column (tab→tab-stop, char=1 col v1) and back, exact for
tabs/empty/ascii (round-trip); the editor renders line content from the Buffer un-truncated so columns == offsets;
tabs expand to tab-stops consistent with the map; a visible caret sits at `column × cell-width` when the editor
tab is active; the split pane + persistence + tab strip are unaffected + the workspace compiles. Pure units cov/MSI
100 on the map + `line_text`; driven capture (mac unlocked) proves faithful render + caret vs the Warp observed
reference. Full EARS in the pipeline spec.
