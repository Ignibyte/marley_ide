# TICKET-620 — Jump to a failed block's first failure

- **Ticket:** LOCAL #620 (feature, prong 1 T2)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/completed/620-jump-to-a-failed-blocks-first-failure.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`)
- **Status:** closed

## Summary
A failed build's first `path:line:col` is somewhere in its output. Jump to First Failure finds it in the block's rows, scrolls it into view and opens the file at that line, resolved against the block's folder: from the block's menu, its failed-block chip, or a key on the last failed block.

## Acceptance
WHEN the user chooses Jump to First Failure on a failed block, Marley shall scroll its first failure into view and open that file at its line.
