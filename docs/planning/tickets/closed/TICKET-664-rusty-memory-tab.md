# TICKET-664 — The Memory tab

- **Ticket:** LOCAL #664 (feature, Rusty in Marley R8)
- **Owner:** claude-opus-5-5, 2026-10-06 (Chad: "we need to look into then adding the last things missing", after the R1 to R7b batch)
- **Pipeline doc:** ../../pipeline/completed/664-rusty-memory-tab.spec.md
- **Source ticket:** `docs/marley/rusty-in-marley.md` (R-D3, the slices table); the read-only survey of
  Rusty at `13249a8` and Marley on 2026-10-06
- **Status:** closed

## Summary
Rusty stores memories (`list_memories`, `store_memory`, `update_memory`, `delete_memory`; content, category, importance, source, times). Marley shows none. This ticket adds a Memory tab: an add field (category, importance), the memories newest first with a category filter, a row opens an editor (content, category, importance) with Save and Delete (asked first), reads again on Rusty's announcement while shown. Rusty documents importance as `low/medium/high` in `store_memory` and `low/normal/high` in `update_memory`; the tab follows what Rusty stores and the mismatch goes to Rusty.

## Acceptance
A memory added in the tab is listed and found by its category; an edit and a delete reach Rusty and show at once; with Rusty off the tab says so.
