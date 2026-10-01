# Blocks with native headers, PS1 hidden — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-628-blocks-with-native-headers-and-ps1-hidden.md
- **Pipeline spec:** 628-blocks-with-native-headers-and-ps1-hidden.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, third batch (#628 to #630, and #466): T5 stage two, and fish.
- **Recall (§18.3):**
  - The hooks send DCS frames, not OSC 133: a block's `prompt_line..output_start` holds PS1 and the command, with nothing marking where PS1 ends.
  - `display_offset` counts grid lines; a header that replaces rows one for one keeps the scroll arithmetic.
  - Zed's `block_below_cursor` with its pixel `scroll_top` is the precedent for rows outside the grid.
- **Discovery:** one Explore sweep for these slices (2026-09-30); the spec's Prior art cites what
  applies here, and the Plan phase re-verifies each seam at promotion.
