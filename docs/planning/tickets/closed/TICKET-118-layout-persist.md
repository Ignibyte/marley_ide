# TICKET-118 — layout persistence + Warp-chrome FINALE [M5 seq-12]

- **Forge ticket:** #118 `2bfc66bc-19f0-49b3-9d6c-4bd781ab7c27` (feature, M5 seq-12 FINALE; sprint #16)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `c4710e1d-af95-4cc8-b543-3bb5976a5b57`
- **Pipeline doc:** ../../pipeline/active/layout-persist.spec.md
- **Status:** closed

## Summary
`serialize_layout`/`restore_layout` + a `workspace.layout` setting + `persist_layout` (settings.rs, cov/MSI
100); boot restores the git-panel state, the ⌘⇧C toggle persists it. **Closes M5 — The Warp Workspace.**
Deps seq-1..11 + marley_settings.

## Acceptance
serialize/restore + the settings round-trip at cov/MSI 100; FULL gate GREEN. Full EARS in the spec.
On close: **close M5 sprint #16**.
