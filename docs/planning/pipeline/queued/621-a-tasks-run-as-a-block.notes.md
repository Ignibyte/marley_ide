# A task's run as a block — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-621-a-tasks-run-as-a-block.md
- **Pipeline spec:** 621-a-tasks-run-as-a-block.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, first batch (#619 to #623): T2 and the rest of T4.
- **Recall (§18.3):**
  - `terminal.rs` loads shell hooks only for non-task, local, PTY terminals: a task never makes a block today.
  - AD-claude-441: the Marley layout routes tasks to the center through `TerminalPanel::spawn_task`, keeping Zed's rerun rules.
  - `register_task_finished` is where the exit code arrives and the summary line is appended.
- **Discovery:** one Explore sweep for T2 and T4 (2026-09-30) over the block model, the
  terminal's links, the failure shapes, the task spawn path and the diagnostics store; the spec's
  Prior art cites what applies here, and the Plan phase re-verifies each seam at promotion.
