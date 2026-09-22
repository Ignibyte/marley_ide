# TICKET-277 — Wide-glyph (CJK/emoji) column width in the editor

- **Forge:** #277 `d656abeb-06e0-4038-90bc-7dd023feae68` (sprint #30, M17)
- **Type:** bug
- **Status:** closed
- **Pipeline:** `docs/planning/pipeline/active/277-wide-glyph-columns.spec.md` (86a7842f-882d-4205-b1e8-008b165b228b)

## Summary
#267 made CJK/IME input work; every wide glyph still counts ONE display
column in the #250 `line_layout`, so the caret bar, click mapping,
selection tint, mark bands, and syntax remap drift left on wide lines
(compounding per glyph). Wire UAX#11 widths (unicode-width 0.2.2, the
lock's version) into `line_layout` (wide=2, zero-width=0, `\t`
positional, control→1) and make `cols_to_bytes` column-aware (it is
display-CHAR-INDEX based today — the plan-recon premise correction; the
other downstream fns are genuinely col_starts-driven and follow free).
Display string stays unpadded (columns are a mapping). Terminal out of
scope (alacritty owns its widths); ZWJ emoji sequences + grapheme
clustering are the recorded follow-up.

## Acceptance
See the spec's EARS table (REQ-001..006): width table wiring, wide
round-trips + click tie rules, the cols_to_bytes column fix, width-2
selection bands, tab-stop composition over accumulated width, and
ASCII-line byte-identical behavior (full suite green).
