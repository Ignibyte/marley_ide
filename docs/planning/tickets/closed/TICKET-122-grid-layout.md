# TICKET-122 — persist + boot the pane grid [M6 seq-3]

- **Forge ticket:** #122 `43b3a888-195a-4af0-bb61-b97b180678a1` (feature, M6 seq-3; sprint #17)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `f4f5806d-af25-44c5-9396-0475125e7d7b`
- **Pipeline doc:** ../../pipeline/active/grid-layout.spec.md
- **Status:** closed

## Summary
`serialize_grid`/`restore_grid` over a flat row of pane kinds + a `workspace.grid` setting (grid_layout.rs +
settings.rs, cov/MSI 100); boot rebuilds the workspace from the saved grid + persists on layout change,
retiring the #121 temp default. Lets us boot into `[terminal|files|code|git]`. Deps #120 + M5 #118.

## Acceptance
Grid + settings round-trip at cov/MSI 100; boot into a 4-pane row (live capture); FULL gate GREEN. Full EARS
in the spec.
