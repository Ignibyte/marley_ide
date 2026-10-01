# TICKET-628 — Blocks with native headers, PS1 hidden

- **Ticket:** LOCAL #628 (feature, prong 1 T5)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/queued/628-blocks-with-native-headers-and-ps1-hidden.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`)
- **Status:** open

## Summary
Stage two of the block terminal: a finished or running block shows a native one-row header (its command and pill) in place of the prompt and command rows the shell drew, the live prompt untouched. A display-row map between the grid and the screen does it without changing what the shell wrote, and the mouse and selection go through the map. Chad chose to build stage two (2026-09-30).

## Acceptance
WHEN the stage-two setting is on, each finished block shall show a one-row header with its command and pill in place of its prompt rows, and the live prompt shall stay as the shell draws it.
