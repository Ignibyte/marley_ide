# TICKET-131 — pane focus navigation + the Warp focus accent [M6]

- **Forge ticket:** #131 `b0ed44a3-66fd-4470-867d-43416ab2538e` (feature, M6; sprint #17)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `48c84b14-4691-4508-8cae-90be84caad52`
- **Pipeline doc:** ../../pipeline/active/focus-accent.spec.md
- **Status:** closed

## Summary
`Workspace::focus_neighbor(dir)` (reuses the tested tree `neighbor`) + 4 ⌘⌥-arrow keymap bindings +
dispatch; the focused pane shows a Warp-style left+top accent edge instead of a full cyan border. cov/MSI
100 on focus_neighbor + the keymap. Deps #120 + M1.B.

## Acceptance
focus_neighbor + keymap at cov/MSI 100; the focus accent edge (live capture); FULL gate GREEN. Full EARS in
the spec.
