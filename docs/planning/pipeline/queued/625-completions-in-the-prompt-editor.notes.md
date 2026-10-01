# Completions in the prompt editor — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-625-completions-in-the-prompt-editor.md
- **Pipeline spec:** 625-completions-in-the-prompt-editor.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, second batch (#624 to #627): T3 and T6, the prompt.
- **Recall (§18.3):**
  - Zed's `CompletionsMenu` belongs to an `Editor`; over the bare grid it would need a menu of Marley's own.
  - #484 already reads history (the terminal's commands, then the file) and matches prefixes (`suggest.rs`).
- **Discovery:** one Explore sweep for these slices (2026-09-30); the spec's Prior art cites what
  applies here, and the Plan phase re-verifies each seam at promotion.
