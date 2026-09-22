# TICKET-031 — render: ANSI color (grid cell colors → themed output)

- **Forge ticket:** #31 `49691435-36ef-43f5-88b3-ce95f2d29535` (feature, M1.D — The Daily Driver, seq-4)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `b9177448-b5ee-4484-828f-06bc63509754`
- **Pipeline doc:** ../../pipeline/active/ansi-color.spec.md
- **Source ticket:** forge sprint #4 `a029f2bc-dabd-43f5-9f6d-13498f0916d5` (M1.D — The Daily Driver)
- **Status:** closed

## Summary
Output is monochrome (`term_to_rows` discards cell color). Preserve color end to end: a gpui-free
styled model in marley_terminal (`StyledRun` carrying alacritty `Color`/`Flags`, `coalesce_row`,
`term_to_styled_rows`, `Block.output` styled but `output_text()` kept so all callers stay green),
a theme `AnsiPalette` + `ansi_color_to_hsla` (Named table / Indexed 6×6×6 cube + grayscale ramp /
Spec passthrough) in marley_app, and a per-run colored render. The biggest visual upgrade.

## Acceptance
`coalesce_row` (marley_terminal) + `ansi_color_to_hsla` (marley_app) at cov 100/MSI 100 (run merge
+ trailing trim; named table + cube/ramp index math + Spec passthrough); `output_text()`
byte-identical (existing suites green); a colored-output visual baseline; FULL gate GREEN. Full
EARS in the pipeline spec.
