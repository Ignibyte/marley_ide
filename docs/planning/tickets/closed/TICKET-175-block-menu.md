# TICKET-175 — M11: the block context menu

- **Forge ticket:** #175 `ad4f71a6-e985-4313-bb09-464811148316` (feature; sprint #22)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `e21aca98-eb76-453b-b3a8-8576129d75c8`
- **Pipeline doc:** ../../pipeline/active/block-menu.spec.md
- **Status:** closed

## Summary
Right-click inside a block → a unified menu (Copy command/Copy output/Rerun + split) targeting that block;
block_at_row (pure) + MenuKind item tables (pure) + shared action helpers. Deps #166, #96, R50.
