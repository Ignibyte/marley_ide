# TICKET-213 — a failing command → jump to the failing line

- **Forge ticket:** #213 `6c23355e-5688-404b-88e5-05fefa772aa9` (feature, M18)
- **Owner:** autonomous /goal run (sprint #31 — M18 Terminal↔Editor Fusion)
- **AAR:** `ea72f711-876f-44db-803f-98b07dd18c30`
- **Pipeline doc:** ../../pipeline/active/213-jump-to-failure.spec.md
- **Source ticket:** the M13 editor-as-peer / terminal-fusion set (the run→fix loop)
- **Status:** closed

## Summary
The run→fix half of the wedge. When a command block FAILS (non-zero exit) and its
output contains `file:line` refs, the block's right-click menu (#175) gains a
"Jump to Failure" row that opens the editor at the FIRST/primary failure location
— reusing #212's `scan_links`/`parse_line_col` + `open_file_at`. A pure
`first_failure_ref(output)` picks the primary ref (the first line-carrying file
ref); the menu row appears only when the block failed AND a ref exists.

## Acceptance
A failed block whose output has a `path:line` ref shows a "Jump to Failure" menu
row that opens the editor at that line; a succeeding block, or a failed block with
no ref, shows the unchanged 6-row menu. Full EARS in the pipeline spec.
