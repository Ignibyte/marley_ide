# A command's colours at the prompt — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-626-a-commands-colours-at-the-prompt.md
- **Pipeline spec:** 626-a-commands-colours-at-the-prompt.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, second batch (#624 to #627): T3 and T6, the prompt.
- **Recall (§18.3):**
  - Changing alacritty's cell colours races readline's redraws; a paint-time override in `layout_grid` does not.
  - tree-sitter-bash and its highlights are already in the tree.
- **Discovery:** one Explore sweep for these slices (2026-09-30); the spec's Prior art cites what
  applies here, and the Plan phase re-verifies each seam at promotion.
