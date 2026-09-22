# TICKET-289 — a failed command's file:line refs become inline gutter markers in the open editor

- **Forge ticket:** #289 `084d5945-7a7f-45d2-a04f-4453e25d9235` (feature, M18)
- **Owner:** autonomous /goal run (sprint #31 — M18 Terminal↔Editor Fusion)
- **AAR:** `9bc05f30-652b-4532-bbcd-1d7b8daacb73`
- **Pipeline doc:** ../../pipeline/active/289-diagnostics-gutter.spec.md
- **Status:** closed

## Summary
The "errors show up IN the editor" half of the wedge. When a command block fails
and its output references lines in a file OPEN in the editor, those rows get an
inline gutter marker (a red bar in the line-number gutter) — the IDE-standard
inline-diagnostics behavior. A pure `diagnostics_for_file(output, open_path, root)`
returns the referenced 0-based rows (reusing #212's `scan_links` File{line} refs +
#190 `resolve_under_root`); the editor render captures the sorted set (mirroring
#272's find-match capture) and marks those rows. Unblocks #290 (nav) + #292
(run→fix loop).

## Acceptance
When a failed block references a line in the open editor file, that row shows a
gutter marker; a succeeding block, or refs to other files, show none. Full EARS in
the pipeline spec.
