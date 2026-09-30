# Delete and unarchive threads from the rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-616-delete-and-unarchive-threads-from-the-rail.md
- **Pipeline spec:** 616-delete-and-unarchive-threads-from-the-rail.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones".
- **Batch:** wave 2 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - AD-claude-605: the rail archives as Zed's history does; delete and unarchive were left to Zed's archive view.
  - A live `ConversationView` re-saves its metadata on its events, so a thread open in a panel must be closed before its delete, or it comes back.
  - Opening a thread unarchives it (`load_agent_thread`), which the rail's `open_thread` already calls.
  - L-claude-605-zed-sends-acp-request-ids-as-strings-001, for the fixture agent.
- **Discovery:** one Explore sweep for the wave (2026-09-30) over the rail, the ports scan, the
  thread store, terminal views and tooltips; the spec's Prior art cites what applies here, and the
  Plan phase re-verifies each seam at promotion.
