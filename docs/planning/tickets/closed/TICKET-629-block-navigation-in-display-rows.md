# TICKET-629 — Block navigation in display rows

- **Ticket:** LOCAL #629 (feature, prong 1 T5)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/completed/629-block-navigation-in-display-rows.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`)
- **Status:** closed

## Summary
With headers in place of prompt rows (#628), the features that count grid lines must count display rows: block navigation, the pinned header (#529) which becomes the pinned header row, bookmark ticks, search highlights and the scrollbar.

## Acceptance
WHEN the user moves to a block with the stage-two headers on, its header row shall land on the top row, and the pinned header shall match it.
