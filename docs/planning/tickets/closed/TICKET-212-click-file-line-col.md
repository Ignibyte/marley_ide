# TICKET-212 — click a file:line:col in terminal output → open the editor at that line (the wedge)

- **Forge ticket:** #212 `680aa96d-cb42-4a41-81a7-c5766e2fd73b` (feature, M18)
- **Owner:** autonomous /goal run (sprint #31 — M18 Terminal↔Editor Fusion)
- **AAR:** `e9aadbf3-f9e4-47f4-b0ab-1f4de35cdd4c`
- **Pipeline doc:** ../../pipeline/active/212-click-file-line-col.spec.md
- **Source ticket:** the M13 editor-as-peer / terminal-fusion set (the wedge foundation)
- **Status:** closed

## Summary
THE foundation of the M18 fusion wedge — where terminal-first beats a plain
editor+terminal. #196 already scans terminal block output for clickable file
paths and STRIPS a trailing `:line[:col]` to find the path (`strip_line_col` — its
own doc says "the line/col is #212's concern") but DISCARDS the location. This
ticket CAPTURES it: extend the pure parser to return `(path, line?, col?)`, carry
the location on `LinkTarget::File`, and place the editor caret at that line/col
when the ref is clicked — so a `crates/app.rs:2127:10` in a compiler/grep/test
line opens the file AT that line with the caret placed. Reuses #196's scanner,
#190 `resolve_under_root`, and the M15 editable editor's `OpenFile.caret` +
`Buffer::line_start`.

## Acceptance
Clicking a `path:line[:col]` reference in block output opens that file in the
editor with the caret at the referenced line (and col when present, both clamped);
a bare path (no location) opens at the top as before. Full EARS in the spec.
