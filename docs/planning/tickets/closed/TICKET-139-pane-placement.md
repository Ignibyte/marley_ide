# TICKET-139 — deterministic file/pane placement (open to the right) [M8 seq-3]

- **Forge ticket:** #139 `9736b3a8-268f-4b84-9f20-e9e1c99b306e` (feature, M8; sprint #19)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `034dbc10-3be5-4675-a3cc-afb38aa84fa0`
- **Pipeline doc:** ../../pipeline/active/pane-placement.spec.md
- **Status:** closed

## Summary
Non-terminal panes (files/code/git) open at the RIGHTMOST position instead of splitting the focused pane.
Pure `rightmost_pane` (DFS-last leaf) + `open_pane_rightmost` (shared `insert_split_at`); the openers route
their OpenNew through it. Deps M6 #120/#124/#128.

## Acceptance
rightmost_pane + open_pane_rightmost at cov/MSI 100; a file/git pane opens right of the terminals regardless
of focus (live capture); FULL gate GREEN. Full EARS in the spec.
