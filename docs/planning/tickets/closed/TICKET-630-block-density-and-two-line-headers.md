# TICKET-630 — Block density: two-line headers and gaps

- **Ticket:** LOCAL #630 (feature, prong 1 T5)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/completed/630-block-density-and-two-line-headers.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`)
- **Status:** closed

## Summary
Re-scoped at promotion (2026-10-01): the folder and branch in the header, in its prompt's rows, and the headers on by default; gaps and density split into #631 (Deliberate).

The last slice of stage two: a header of two lines (the folder and branch, then the command and pill), a gap between blocks, and a density setting, compact or comfortable. These add rows the grid does not have, so the view scrolls past the grid in pixels as Zed's inline assist does. With it the headers go on by default.

## Acceptance
WHILE compact density is on, block headers and gaps shall have the sizes the spec sets, and the live prompt shall still sit on the last row.
