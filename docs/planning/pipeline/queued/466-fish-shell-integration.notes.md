# Shell integration for fish — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-466-fish-shell-integration.md
- **Pipeline spec:** 466-fish-shell-integration.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`); Chad, 2026-09-30, on installing fish: "Yes, install fish".
- **Batch:** wave 3, third batch (#628 to #630, and #466).
- **Recall (§18.3):**
  - #463 and #465 left fish, OSC 133 and remote bootstraps for later; the scripts send DCS frames.
  - fish 4.9.2 installed on 2026-09-30 (`omarchy pkg add fish`), recorded in the ops handbook.
  - No gate runs the PTY tests since #483; #475 is deferred, so the scenario is the proof.
- **Discovery:** one Explore sweep (2026-09-30) over the scripts, `install_in`, `for_program` and
  the tests; the Plan phase re-verifies each seam at promotion.
