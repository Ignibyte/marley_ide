# TICKET-103 — a diff view (diff_rows projection)

- **Forge ticket:** #103 `b1eaffcb-0290-4c13-8c20-9869d0a34bc1` (feature, M4 seq-7; sprint #15)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `56535934-d25f-4c8a-aeca-7820268bec61`
- **Pipeline doc:** ../../pipeline/active/diff-view.spec.md
- **Status:** closed

## Summary
Formalize the diff render as pure `diff_rows(files) -> Vec<DiffRow>` (FileHeader/HunkHeader/Add/Remove/
Context rows) + refactor #102's ⌘⇧D overlay onto it (role→color). cov/MSI 100. Deps #102 + #35.

## Acceptance
diff_rows at cov/MSI 100 (order/roles/signs/empty); the ⌘⇧D overlay renders via diff_rows; FULL gate
GREEN. Full EARS in the spec.
