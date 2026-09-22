# TICKET-039 — typography + focus/hover affordances pass (M1.E finale)

- **Forge ticket:** #39 `85e6b003-36f3-4dbf-8f4f-d8885d84b5f6` (feature, M1.E — The Warp Look, seq-6 FINALE)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `af2f8cc1-6a56-4ecb-aee0-8bedd4533c40`
- **Pipeline doc:** ../../pipeline/active/typography-polish.spec.md
- **Source ticket:** forge sprint #5 `cf1ba6de-3af6-4158-813e-b300e596c831` (M1.E — The Warp Look)
- **Status:** closed

## Summary
The M1.E finale — a cohesion pass. Add a `muted` caption color to ThemeColors (dark+light) and a pure
gpui-free `typography` module (`type_scale(Role) -> TextStyle` + `weight_value(TextWeight) -> f32`)
formalizing the ad-hoc weights/sizes; apply them in the render (command/output/caption), refine the
focus border, add a Block/palette hover affordance. Pure surfaces cov/MSI 100; the render tuning is
shim. Deps #34–#38 done. Phase 5 also closes the M1.E sprint.

## Acceptance
`muted` + `type_scale` + `weight_value` at cov 100/MSI 100 (the values/arms + distinctness); the
render application (masked full-cockpit visual — chad's final "does it look like Warp" sign-off);
FULL gate GREEN. Full EARS in the pipeline spec.
