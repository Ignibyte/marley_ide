# TICKET-034 — monospace terminal font + accurate cell metric

- **Forge ticket:** #34 `2d48faf5-6191-4a84-8066-0e5af1b70b67` (feature, M1.E — The Warp Look, seq-1)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `55039a8f-140d-4ba1-b0f3-ffa6bdfe86cc`
- **Pipeline doc:** ../../pipeline/active/terminal-font.spec.md
- **Source ticket:** forge sprint #5 `cf1ba6de-3af6-4158-813e-b300e596c831` (M1.E — The Warp Look)
- **Intake:** docs/planning/intake/terminal-monospace-font-and-cell-metric.md
- **Status:** closed

## Summary
Terminal text renders in the ambient proportional font (columns don't align; the #30 cell metric is
read from the wrong font). Set an explicit monospace `TERMINAL_FONT="Menlo"` on the terminal spans +
read the cell metric from it, and add a pure `fallback_cell(font_size)` (0.6·size advance / 1.2·size
height) replacing the hardcoded px(8.0) fallback. Mostly shim; `fallback_cell` is the tested surface.

## Acceptance
`fallback_cell` at cov 100/MSI 100 (the ratios + the >0 clamp); the mono font applied to terminal
content (masked visual — aligned columns, chad-verified); FULL gate GREEN. Closes the intake. Full
EARS in the pipeline spec.
