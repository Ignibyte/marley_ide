# A prompt editor at the shell's prompt, on a key — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-624-a-prompt-editor-at-the-shells-prompt.md
- **Pipeline spec:** 624-a-prompt-editor-at-the-shells-prompt.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, second batch (#624 to #627): T3 and T6, the prompt.
- **Recall (§18.3):**
  - #481's editor opens only while an agent runs; its spec defers the shell's prompt editor to T3.
  - #484 reads the typed line from the grid (`typed_text`), giving up when text follows the cursor or the view is scrolled back.
  - Plan D4: keys route to an editor docked at the bottom, with the proven raw ladder for everything else (T3c).
- **Discovery:** one Explore sweep for these slices (2026-09-30); the spec's Prior art cites what
  applies here, and the Plan phase re-verifies each seam at promotion.
