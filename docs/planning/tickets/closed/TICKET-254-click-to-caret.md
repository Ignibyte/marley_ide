# TICKET-254 — Mouse click-to-place-caret in the editor

- **Forge ticket:** #254 (fb9e2682-6515-4acc-9514-bc88215eda20) (feature, M15)
- **Owner:** (autonomous /goal /work 249 to 258)
- **AAR:** 41875b20-6243-4151-b507-11650a559ece
- **Pipeline doc:** ../../pipeline/completed/click-to-caret.spec.md
- **Source ticket:** M15 — The Editable Editor (sprint #28, fa328492); the #242 decomposition (#248–258)
- **Status:** closed

## Summary
Today the editor caret starts at offset 0 and only moves by typing (#251) — a mouse click in the editor body does
nothing. #254 makes a left-click place the caret at the `char` nearest the click, by inverting #250's exact
char↔column map (`col_starts`). The pure core is `LineLayout::offset_of_col` (the column→offset inverse #250's
doc reserved) + a `(row, col) → CharOffset` composition; the app.rs shim converts the click pixel to `(row, col)`
via the #250 cell/scroll/gutter geometry and sets the active file's caret. Deps #250 (the map — the shared risk),
#249 (the caret/doc model), #251 (typing then inserts at the click).

## Acceptance
A left-click in the editor moves the caret to the char nearest the click (clamped past line-end / past the last
line, no panic); typing then inserts at the click point. `offset_of_col` round-trips `col_of_offset`; the pure
seam is cov/MSI 100; the behavior is driven-proven live. Full EARS criteria in the pipeline spec.
