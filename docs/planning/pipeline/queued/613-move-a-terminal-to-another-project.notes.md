# Move a terminal to another project in the rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-613-move-a-terminal-to-another-project.md
- **Pipeline spec:** 613-move-a-terminal-to-another-project.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones".
- **Batch:** wave 2 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - L-claude-602-nested-drop-targets-need-a-drag-type-each-001: a drop goes to the deepest `on_drop` of its type, so the drop on another project sits on its header frame.
  - L-claude-493 and F-claude-493: a move between panes is a remove then an add; `pane_for` lags until the events drain.
  - AD-claude-575: `added_to_workspace` moves the terminal database row and Marley's id, except for task terminals.
  - The view's `workspace` and `project` are read by `close_guard.rs`, `agent_events.rs`, `system_one.rs`, `push.rs` and Zed's link opening, so they must follow the move.
- **Discovery:** one Explore sweep for the wave (2026-09-30) over the rail, the ports scan, the
  thread store, terminal views and tooltips; the spec's Prior art cites what applies here, and the
  Plan phase re-verifies each seam at promotion.
