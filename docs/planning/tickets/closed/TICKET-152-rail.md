# TICKET-152 — M9 seq-3: the rail (Workspace→Project→Tab tree, click-to-switch)

- **Forge ticket:** #152 `43343e48-3d19-4a04-b921-4bdd0c565325` (feature, M9; sprint #20) · absorbs #145
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `c8b44b83-4e8b-4f8b-921b-f85b6944d676`
- **Pipeline doc:** ../../pipeline/active/rail.spec.md
- **Status:** closed

## Summary
Replace the flat left-dock session list with the Workspace→Project→Tab hierarchy tree: pure `rail_rows`
projection (cov/MSI 100) + a masked render with the active tab highlighted and click-to-switch
(`switch_project` + `switch_tab`). Deps #150/#151.

## Acceptance
rail_rows at cov/MSI 100; a driven capture shows the tree + click switches the full-screen tab; FULL gate
GREEN. Full EARS in the spec.
