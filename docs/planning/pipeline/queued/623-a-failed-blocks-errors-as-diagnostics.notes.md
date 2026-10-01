# A failed block's errors as project diagnostics — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-623-a-failed-blocks-errors-as-diagnostics.md
- **Pipeline spec:** 623-a-failed-blocks-errors-as-diagnostics.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, first batch (#619 to #623): T2 and the rest of T4.
- **Recall (§18.3):**
  - `merge_lsp_diagnostics` is the one public way in; diagnostics are keyed by a language server id, and paths outside a worktree are dropped.
  - No Marley crate publishes diagnostics yet; the Diagnostics view reacts to the project's diagnostic events.
  - Plan D6 names this; #620's locator is its reader.
- **Discovery:** one Explore sweep for T2 and T4 (2026-09-30) over the block model, the
  terminal's links, the failure shapes, the task spawn path and the diagnostics store; the spec's
  Prior art cites what applies here, and the Plan phase re-verifies each seam at promotion.
