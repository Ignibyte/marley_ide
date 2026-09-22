# TICKET-024 — docks for real: 3-region render + toggle-dock actions

- **Forge ticket:** #24 `0764b4e3-affc-45f5-ba73-8f2ae6c7e43f` (feature, M1.C — The Wired Cockpit, seq-3)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `e928eb68-9ccf-4e72-81af-b764a01a3ddd`
- **Pipeline doc:** ../../pipeline/active/docks-for-real.spec.md
- **Source ticket:** forge sprint #3 `b295976b-4f32-4edc-be3c-aecd80b02785` (M1.C — The Wired Cockpit)
- **Status:** closed

## Summary
The docks are the last render-inert cockpit state: `dock()`/`toggle_dock()` ship unit-tested but
nothing renders them and no chord calls them. Make R4–R6 real: left dock | center panes | right
dock, a Closed dock at zero width with the center reflowing (the #23 `pane_rects` bounds is the
inset seam), toggle chords (cmd-b / cmd-shift-b proposed) + palette commands.

## Acceptance
`region_widths` pure at cov 100/MSI 100 across all four dock-state combinations; the keymap maps
the two new chords; the render sizes all three regions from the ONE width computation; FULL gate
GREEN [--diff]. Full EARS in the pipeline spec.
