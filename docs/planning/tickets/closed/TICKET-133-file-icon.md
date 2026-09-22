# TICKET-133 — file-explorer icon in the top bar (open the file tree) [M7 seq-2]

- **Forge ticket:** #133 `ee402b99-849d-4993-8a90-a0e6ac168a86` (feature, M7; sprint #18)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `71e24b56-db2c-4107-b51e-0cab2d38748f`
- **Pipeline doc:** ../../pipeline/active/file-icon.spec.md
- **Status:** closed

## Summary
A clickable 📁 file-explorer icon in the top bar (reuse `pane_icon(FileTree)`) → `open_files_pane` (#128).
Makes the file tree discoverable. Shim-only. Deps #132 + M6 #128.

## Acceptance
The icon renders in the top bar (live capture); click opens a FileTree pane (code-reviewed; open path tested
via #128); FULL gate GREEN. Full EARS in the spec.
