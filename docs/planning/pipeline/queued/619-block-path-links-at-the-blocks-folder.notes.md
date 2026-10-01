# A block's path links resolve against the block's own folder — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-619-block-path-links-at-the-blocks-folder.md
- **Pipeline spec:** 619-block-path-links-at-the-blocks-folder.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, first batch (#619 to #623): T2 and the rest of T4.
- **Recall (§18.3):**
  - `cwd_at_line` falls back to the current folder once the scrollback is full and for remote terminals; that fallback is where a block's own folder matters.
  - `process_hyperlink` runs without the terminal's lock, so the block lookup needs its inputs passed in.
  - Plan D6 and the fusion note name block-scoped links as the remaining half of the terminal-editor link.
- **Discovery:** one Explore sweep for T2 and T4 (2026-09-30) over the block model, the
  terminal's links, the failure shapes, the task spawn path and the diagnostics store; the spec's
  Prior art cites what applies here, and the Plan phase re-verifies each seam at promotion.
