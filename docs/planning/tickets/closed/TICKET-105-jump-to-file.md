# TICKET-105 — jump-to-file from a diff / agent output

- **Forge ticket:** #105 `41f545f7-e2b3-4f0f-b736-fc2089cf7ffc` (feature, M4 seq-9; sprint #15)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `d8f8b283-ea9f-4b3f-86a9-0fe14bb6bff1`
- **Pipeline doc:** ../../pipeline/active/jump-to-file.spec.md
- **Status:** closed

## Summary
`parse_file_ref(text) -> Option<FileRef{path,line}>` (recognize `path:line`; reject non-paths; cov/MSI 100)
+ the diff overlay's FileHeader rows click → open the file in the viewer at the line. Deps #98 + #101 + #103.

## Acceptance
parse_file_ref at cov/MSI 100 (path/line/None/trim); clicking a diff file header opens it (engine/live git);
FULL gate GREEN. Full EARS in the spec.
