# TICKET-138 — unified macOS title bar (icons + search in the traffic-light row) [M8 seq-2]

- **Forge ticket:** #138 `6b735046-2bd1-4e0a-bc2b-09a73a34689f` (feature, M8; sprint #19)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `f1af3314-c227-4502-adc2-f20d3e6465d3`
- **Pipeline doc:** ../../pipeline/active/unified-titlebar.spec.md
- **Status:** closed

## Summary
Make the window titlebar transparent (`appears_transparent` + `traffic_light_position`) so the icons + search
share the macOS traffic-light row (one bar, no separate row); drop the "Marley" title; inset the left icons
past the lights via pure `topbar_icon_x`. Deps #132 + #137.

## Acceptance
topbar_icon_x at cov/MSI 100; a unified bar with the lights unobscured (live capture); FULL gate GREEN. Full
EARS in the spec.
