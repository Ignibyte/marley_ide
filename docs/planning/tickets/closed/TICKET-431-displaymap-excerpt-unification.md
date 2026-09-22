# TICKET-431 — DisplayMap excerpt unification (the B-c chain's prize)

- **Ticket:** LOCAL #431 (feature, M33)
- **Tags:** editor, display-map, multibuffer, architecture
- **Created:** 2026-08-15
- **Provenance:** the B-c chain's recorded "later prize" (roadmap + the
  #425/#427 notes); shelf: ../../design-notes/m33-tail-and-wedge-shelf.md
- **Pipeline doc:** ../../pipeline/completed/431-displaymap-excerpt-unification.spec.md
- **Status:** closed (2026-08-15 — shipped byte-identical; GATE GREEN [diff] 15/15)

## Summary
Fold the multibuffer's slot rim into the ONE `DisplayMap` facade so
fold∘wrap∘excerpt is a single typed projection stack: the mb's materialized
`slots: Vec<Row>` becomes a facade LAYER (like #426's wrap over #425's
fold), the editor and the multibuffer stop owning parallel row math, and
future features (wrap inside excerpts, folds inside excerpts) compose for
free instead of being cross-products. Byte-identical by proof — the #425
recipe: facade property-equals the direct model on every surface, the full
suite unchanged, live-drive verified.

## Acceptance
Headline: no behavior change anywhere (search mb, problems mb, editor tab,
splits) while the raw slot/row conversions collapse into the facade; the
mb render/movers/caret/jump consume typed rows through it. Full EARS in the
queued spec.
