# TICKET-179 — M12: fix the pane_grid_pos bottom-anchor skew

- **Forge ticket:** #179 `61a841ee-2a48-4bf5-8c30-143e7386954c` (bug; sprint #23)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `db594748-d203-4ac1-86e3-299498838ffa`
- **Pipeline doc:** ../../pipeline/active/hittest-skew.spec.md
- **Status:** closed

## Summary
A pure bottom_anchored_row(y_from_bottom, cell_h, start, end) replaces the top-anchored row_at/row_hit;
pane_grid_pos + the #175 hit-test map from the pane bottom so selection + the block menu land on the right
row. Deps R47, #175.
