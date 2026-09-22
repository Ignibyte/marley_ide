# TICKET-057 — fuzzy file-open overlay (cmd-P)

- **Forge ticket:** #57 `2d7d3df4-0ff3-4fe7-8e44-07a5c568496e` (feature, M2.A seq-5)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `5edda1b8-fc27-4af7-a856-ea28ec7c5f91`
- **Pipeline doc:** ../../pipeline/active/file-finder.spec.md
- **Source ticket:** forge sprint #9 `fcb8d0fe-0630-4f4e-b465-4360fdb40d89` (M2.A — Project, Files & Search)
- **Status:** closed

## Summary
cmd-P fuzzy file-open. PURE `FinderState` (new marley_app/finder.rs, mirroring PaletteState) that ranks
the project files via `marley_search_core::fuzzy_rank`; SHIM (app.rs) = a cmd-P overlay listing ranked
matches with a selection highlight, Enter writes the chosen path to the PTY. cov/MSI 100 on the model;
the overlay/keys are the masked + self-test-verified shim. Deps #54 + #56.

## Acceptance
FinderState at cov/MSI 100 (query-reset; move clamps; results ranks; chosen selected/empty); cmd-P
overlay lists ranked matches (self-test capture); no cmd-P/cmd-shift-p conflict; FULL gate GREEN. Full
EARS in the pipeline spec.
