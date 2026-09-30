# Threads and ports under a closed project — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-617-rows-under-a-closed-project.md
- **Pipeline spec:** 617-rows-under-a-closed-project.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones".
- **Batch:** wave 2 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - AD-claude-606 deferred a closed project's threads and ports "since they need its workspace"; the store queries need only its path list and host.
  - `group_threads` needs a workspace for `ThreadEntry.workspace`, the agent's icon and name, and live statuses; each has a way round for a closed group.
  - `project_folders` takes roots from the group's workspaces only, so a closed group gets none today.
  - F-claude-606-closed-headers-would-have-shared-their-element-ids-001: closed rows take their ids from `closed_id`.
- **Discovery:** one Explore sweep for the wave (2026-09-30) over the rail, the ports scan, the
  thread store, terminal views and tooltips; the spec's Prior art cites what applies here, and the
  Plan phase re-verifies each seam at promotion.
