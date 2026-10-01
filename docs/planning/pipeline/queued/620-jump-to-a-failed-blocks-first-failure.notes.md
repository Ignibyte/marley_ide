# Jump to a failed block's first failure — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-620-jump-to-a-failed-blocks-first-failure.md
- **Pipeline spec:** 620-jump-to-a-failed-blocks-first-failure.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, first batch (#619 to #623): T2 and the rest of T4.
- **Recall (§18.3):**
  - #572's `running_errors` knows failure shapes but keeps only text, not where a line sits; `absolute_lines_text` joins wrapped rows.
  - `blocks::reveal` scrolls to an absolute line and is the jump's base.
  - #433's `file:line:col` extractor belonged to the gpui-era app; nothing in this tree extracts positions.
- **Discovery:** one Explore sweep for T2 and T4 (2026-09-30) over the block model, the
  terminal's links, the failure shapes, the task spawn path and the diagnostics store; the spec's
  Prior art cites what applies here, and the Plan phase re-verifies each seam at promotion.
