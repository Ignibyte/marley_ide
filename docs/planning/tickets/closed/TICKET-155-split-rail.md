# TICKET-155 — M9 seq-6: right-click → Split + nested pane rows in the rail

- **Forge ticket:** #155 `60e0f233-3ec5-4e2d-bbdb-6189da9f6dc7` (feature, M9; sprint #20)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `d826f887-0acb-4b7c-be87-d3ca8b8632d9`
- **Pipeline doc:** ../../pipeline/active/split-rail.spec.md
- **Status:** closed

## Summary
Splitting is opt-in per terminal tab: right-click a terminal → tile the tab's PaneGrid (reuse M6
split_focused), and the split panes show NESTED under the tab in the rail. Pure: RailLevel::Pane +
RailRow.pane + rail_rows nested pane rows (cov/MSI 100). Shim: MouseButton::Right → split; the rail Pane render
arm (click → focus). Deps #150-153.

## Acceptance
rail_rows nesting at cov/MSI 100; a driven capture — right-click → 2 tiled panes + 2 nested pane rows; FULL
gate GREEN. Full EARS in the spec.
