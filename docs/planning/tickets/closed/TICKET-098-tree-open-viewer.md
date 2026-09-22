# TICKET-098 — open a file from the tree into the viewer

- **Forge ticket:** #98 `b3be0115-7d23-43ef-b115-51ebb7728260` (feature, M4 seq-2; sprint #15)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `aa68318f-2216-464a-8bec-efd5ecb96fff`
- **Pipeline doc:** ../../pipeline/active/tree-open-viewer.spec.md
- **Status:** closed

## Summary
The file-tree ⌘-click open into the #97 viewer + a shared `open_file_in_viewer` helper (the finder ⌘↵
refactors onto it) + a pure `viewer_open_path` dir-guard (cov/MSI 100). Deps #97 + #56 + #57.

## Acceptance
viewer_open_path at cov/MSI 100 (dir→None/file→Some); ⌘-click a tree file → the viewer opens (engine/
self-test); FULL gate GREEN. Full EARS in the spec.
