# A task block's Rerun and its pill — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-622-a-task-blocks-rerun-and-pill.md
- **Pipeline spec:** 622-a-task-blocks-rerun-and-pill.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, first batch (#619 to #623): T2 and the rest of T4.
- **Recall (§18.3):**
  - #474's Rerun types the block's command at the prompt, which a task terminal has none of.
  - AD-claude-441: tasks run through `TerminalPanel::spawn_task` so Zed's rerun and reuse rules hold.
- **Discovery:** one Explore sweep for T2 and T4 (2026-09-30) over the block model, the
  terminal's links, the failure shapes, the task spawn path and the diagnostics store; the spec's
  Prior art cites what applies here, and the Plan phase re-verifies each seam at promotion.
