# TICKET-054 — marley_search_core: the fuzzy ranking engine

- **Forge ticket:** #54 `d57071a4-c0bd-407e-96c0-3c7688eb25e1` (feature, M2.A seq-2)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `a9a135e6-5a33-4394-b5e7-4c77ee0e94a7`
- **Pipeline doc:** ../../pipeline/active/search-core.spec.md
- **Source ticket:** forge sprint #9 `fcb8d0fe-0630-4f4e-b465-4360fdb40d89` (M2.A — Project, Files & Search)
- **Status:** closed

## Summary
A new `marley_search_core` crate wrapping the `nucleo` subsequence primitive: `fuzzy_score(text, query)
-> Option<u32>` + `fuzzy_rank(candidates, query) -> Vec<Scored>` (empty→all-in-order; else matches by
score desc, index tie-break). Extracted from the palette's `field_score`; the palette is refactored
onto it (one matcher) and `nucleo` moves from marley_app to the new crate. cov/MSI 100; the palette's
existing tests are the behavior-preservation guard. First consumer: #57 fuzzy file-open.

## Acceptance
fuzzy_score + fuzzy_rank at cov/MSI 100; palette tests still green; machete clean (nucleo removed from
marley_app); FULL gate GREEN. Full EARS in the pipeline spec.
