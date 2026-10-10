# Thread tabs after a restart — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-736-thread-tabs-after-a-restart.md
- **Pipeline spec:** 736-thread-tabs-after-a-restart.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-10)
- **Request:** Chad, 2026-10-10, the agents-anywhere plan
  (`docs/planning/design-notes/agents-anywhere-2026-10-10.md`), with the goal "lets make tickets
  and build it".
- **Classification:** feature.
- **Recall (§18.3):**
  - #494 (Browser tabs restored) and #576 (no `UNIQUE(item_id)` in an items table).
  - AD-claude-609: the Agent tab was kept out of `SerializableItem` until a store served its history; here Zed's thread store does.
  - `rusty/graph_store.rs:215` is the simplest template: one JSON `state` column.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again.
