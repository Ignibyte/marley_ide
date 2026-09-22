# TICKET-228 — Focus-box border occluded by the drag-divider on the shared pane edge

- **Forge ticket:** #228 (2d14452f-4beb-4bab-b37a-36e7ddeb179c) (bug, M13)
- **Owner:** c104f25c-5375-4d73-9dc5-5fcf30470018 (autonomous /work 228-237 run)
- **AAR:** 02e8710c-4b71-4e08-a047-e8e76fcde61f
- **Pipeline doc:** ../../pipeline/active/focus-border-over-divider.spec.md
- **Source ticket:** sprint #26 M13 "The Workspace Cockpit" (fd182395) — chad live feedback #4
- **Status:** closed

## Summary
After splitting a pane and focusing one, the #191 4-side focus-accent border is drawn, but the #130
draggable split divider — painted AFTER the pane loop, `.occlude()`, 6px centered on the shared edge —
covers the border's bar on that shared edge, so the focus box looks broken/missing on the divider side.
Root cause is z-order (later children paint on top). Fix: draw the focus border AFTER the divider loop so
it is the topmost pane-chrome. A render-order shim; the pure `focus_border_rects` is unchanged.

## Acceptance
A focused pane adjacent to a divider shows the 2px accent border on all four edges (including the
divider-side edge); the divider drag still works; a non-adjacent pane's border is unchanged. Full EARS
(REQ-001..003) in the pipeline spec.
