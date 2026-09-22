# TICKET-112 — "Search tabs" sidebar filter [M5 seq-6]

- **Forge ticket:** #112 `f883c3c8-09c2-42c4-a9af-8dbe417e9f34` (feature, M5 seq-6; sprint #16)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `cb33804b-e8f9-405d-b730-450607d10916`
- **Pipeline doc:** ../../pipeline/active/search-tabs.spec.md
- **Status:** closed

## Summary
`filter_sessions(sessions, query)` (fuzzy on title/subtitle via marley_search_core, order preserved,
empty→all; cov/MSI 100) + a "Search tabs" input atop the sidebar that filters before grouping. Deps
seq-3/4/5 + #57.

## Acceptance
filter_sessions at cov/MSI 100 (empty/match/no-match); the search field shows atop the sidebar (live
capture); FULL gate GREEN. Full EARS in the spec.
