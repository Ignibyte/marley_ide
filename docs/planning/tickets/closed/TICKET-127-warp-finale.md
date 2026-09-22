# TICKET-127 — the Warp default arrangement + finale [M6 seq-8 FINALE]

- **Forge ticket:** #127 `fb0f6938-8b86-46fb-9480-6e734c2dbf1c` (feature, M6 seq-8; sprint #17)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `c58ed9fe-6b8c-45bc-9e39-8d1465c3ad86`
- **Pipeline doc:** ../../pipeline/active/warp-finale.spec.md
- **Status:** closed

## Summary
Land the Warp arrangement + chrome polish: make `default_grid()` public + tested (grid_layout.rs, cov/MSI
100); default the right dock closed so the right side is free (also fixes the #126 top-tab/dock-header
overlap); capture the representative Warp layout against chad's reference screenshots. **Closes M6 sprint
#17.** Deps seq-1..7 (all done).

## Acceptance
default_grid at cov/MSI 100; the boot arrangement (sidebar left · pane grid · top tabs · right free) matches
the reference in structure (live capture compare); FULL gate GREEN. Full EARS in the spec.
