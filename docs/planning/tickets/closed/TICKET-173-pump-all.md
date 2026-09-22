# TICKET-173 — M11: pump EVERY grid

- **Forge ticket:** #173 `48330513-d9a8-43d5-acc2-b6ca9faff3da` (feature; sprint #22)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `d87d0f69-4d6c-4a8e-b263-195b6c803a83`
- **Pipeline doc:** ../../pipeline/active/pump-all.spec.md
- **Status:** closed

## Summary
Background tabs stay live: grids()/grids_mut()/locate_pane (pure) + the pump drains all grids, closes dead
panes in their owning grid, and refreshes agent statuses everywhere. Deps #151, #157, #167.
