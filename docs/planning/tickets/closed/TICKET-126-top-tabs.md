# TICKET-126 — retire the right rail → top tabs [M6 seq-7]

- **Forge ticket:** #126 `b10f40e4-dbb6-4a7e-bb9a-74b56995b008` (feature, M6 seq-7; sprint #17)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `e2e346d3-707e-4bd2-968a-f759b1696ab6`
- **Pipeline doc:** ../../pipeline/active/top-tabs.spec.md
- **Status:** closed

## Summary
`top_tabs(active)` (right_dock.rs, cov/MSI 100) + a top tab strip (Details/Agents/Forge) replacing the
right-dock rail's tab strip; a tab click selects the section + opens the (content-only) right dock, which
defaults closed. Chad's "tabs at the top" — clears the last duplication. Deps M2 #90/#95 + #120.

## Acceptance
top_tabs at cov/MSI 100; the tabs render at the top (live capture); FULL gate GREEN. Full EARS in the spec.
