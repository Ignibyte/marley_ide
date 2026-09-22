# TICKET-141 — search UX polish (keyboard nav + activation) [M8 seq-5]

- **Forge ticket:** #141 `dfc6cda0-499e-4304-96ae-c28f2118ff85` (feature, M8; sprint #19)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `376bd6f3-43bb-4e7f-affc-4f956a930084`
- **Pipeline doc:** ../../pipeline/active/search-nav.spec.md
- **Status:** closed

## Summary
Make the top search navigable: pure `move_selection` (clamped index) + a `top_search_selected` state + ↑/↓/↵
routing + a highlighted selected row + `activate_search_hit` (File→viewer, Session→focus-by-label; Action a
follow-up). Fixes the "funky" (typing worked but you couldn't select/open with the keyboard). Deps M5 #117.

## Acceptance
move_selection at cov/MSI 100; a driven `down` moves the highlight (live capture); ↵ opens the selected hit
(code-reviewed + driven); FULL gate GREEN. Full EARS in the spec.
