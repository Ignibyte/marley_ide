# TICKET-230 — Calibrate sidebar/files text to Warp size via type_scale (folds #223)

- **Forge ticket:** #230 (7707f0be-d9d6-43d1-a0c3-a765bc8bd79f) (chore, M13)
- **Owner:** c104f25c-5375-4d73-9dc5-5fcf30470018 (autonomous /work 228-237 run)
- **AAR:** 11e662ad-a41d-4597-90b7-e18fab7885bf
- **Pipeline doc:** ../../pipeline/active/chrome-type-scale.spec.md
- **Source ticket:** sprint #26 M13 "The Workspace Cockpit" (fd182395) — chad live feedback #2; folds #223
- **Status:** closed

## Summary
The files/sidebar chrome text is ~13px — too big vs Warp's ~11-12px (chad #2) — and ~20 hardcoded
`text_size(px(N))` literals scatter the chrome with no single source. Extend the pure `type_scale` Role
table with Warp-calibrated chrome bands, route the chrome literals through them, and calibrate the
sidebar/files text down. Folds the out-of-range #223 (the type_scale consolidation + the stale
`fallback_cell` 14.0). Pure seam (typography.rs) at cov/MSI 100 + a masked app.rs render shim.

## Acceptance
The sidebar/files text renders at the smaller Warp size via a `type_scale` Role; the routed chrome
literals no longer hardcode `px(N)`; the terminal roles stay ≥12px; `fallback_cell`'s stale 14.0 is
refreshed. Full EARS (REQ-001..005) in the pipeline spec.
