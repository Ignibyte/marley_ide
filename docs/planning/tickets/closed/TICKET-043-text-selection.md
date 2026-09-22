# TICKET-043 — terminal text selection model

- **Forge ticket:** #43 `9a6582b5-76c3-4bde-9c2a-258ce5469d1c` (feature, M1.G — Block Workflows & Selection, seq-1)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `98d605b0-1892-4622-9d1b-69280eba076d`
- **Pipeline doc:** ../../pipeline/active/text-selection.spec.md
- **Source ticket:** forge sprint #7 `eb842cb6-f4ab-4b27-89e0-78634476f304` (M1.G — Block Workflows & Selection)
- **Status:** closed

## Summary
No text selection exists. Add a pure `text_selection` module — `GridPos{row,col}` (char cols),
`Selection{anchor,head}` + `normalized`, and `selected_text(rows, sel)` (char-safe, clamped, `\n`-
joined row-span extraction). `PaneState` gains `selection: Option<Selection>`; the mouse-drag →
GridPos map + the highlight render are the app.rs shim. The FOUNDATION for copy (#44). Deps #32/#34.

## Acceptance
`selected_text` + `normalized` + `row_slice` at cov 100/MSI 100 (single/multi/adjacent rows; backward
== forward; clamp; multi-byte char boundaries); the drag-highlight (masked visual — chad-verified);
FULL gate GREEN. Full EARS in the pipeline spec.
