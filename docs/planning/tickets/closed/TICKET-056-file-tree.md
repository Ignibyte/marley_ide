# TICKET-056 — Files dock: real project file tree

- **Forge ticket:** #56 `1140cdd5-1102-4eb8-a109-c3aad7233c4f` (feature, M2.A seq-4)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `c8e2300f-ce2c-4ae1-95be-244b8947cdfd`
- **Pipeline doc:** ../../pipeline/active/file-tree.spec.md
- **Source ticket:** forge sprint #9 `fcb8d0fe-0630-4f4e-b465-4360fdb40d89` (M2.A — Project, Files & Search)
- **Status:** closed

## Summary
The Files dock shows the project's file tree. PURE (marley_project): `FileTree::from_files` (nest +
merge + dirs-before-files sort), `visible_rows` (collapse-aware pre-order), `toggle(visible_index)`.
SHIM (app.rs): RootView.file_tree built at startup (Project → list_files_in → from_files); the Left
dock renders the rows (indent + ▸/▾ disclosure); clicking a dir toggles it. cov/MSI 100 on the pure
model; the render/click is masked + self-test-verified. Deps #53 + #55.

## Acceptance
FileTree at cov/MSI 100 (nesting/merge; dirs-before-files; collapse-hiding; toggle dir-only+bounds);
the Left dock renders the tree + click collapses (self-test capture); FULL gate GREEN. Full EARS in the
pipeline spec.
