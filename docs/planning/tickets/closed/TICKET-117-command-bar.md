# TICKET-117 — the global top command bar [M5 seq-11]

- **Forge ticket:** #117 `afc03377-f5ab-4391-94f4-61904d2f7ed9` (feature, M5 seq-11; sprint #16)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `5df08bc9-289d-4070-abef-99ee9a7da0c1`
- **Pipeline doc:** ../../pipeline/active/command-bar.spec.md
- **Status:** closed

## Summary
`search_everything(query, sessions, files, actions, cap)` → `Vec<SearchHit>` (fuzzy across 3 sources,
kind-tagged, capped; new command_bar.rs, cov/MSI 100) + a top "Search sessions, agents, files…" bar with a
mixed-results dropdown (file hits open the viewer). Deps #19 + #57 + #109.

## Acceptance
search_everything at cov/MSI 100 (empty/per-source/kind/cap); the top bar renders (live capture); FULL gate
GREEN. Full EARS in the spec.
