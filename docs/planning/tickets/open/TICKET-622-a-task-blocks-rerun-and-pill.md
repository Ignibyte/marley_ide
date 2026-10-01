# TICKET-622 — A task block's Rerun and its pill

- **Ticket:** LOCAL #622 (feature, prong 1 T4)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/queued/622-a-task-blocks-rerun-and-pill.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`)
- **Status:** open

## Summary
Once a task's run is a block (#621), its Rerun should run the task again through Zed's task machinery rather than type its label at a shell, and the `⏵ Task … finished` line Zed appends is redundant beside the block's pill.

## Acceptance
WHEN the user chooses Rerun on a task block, Marley shall run that task again as Zed's rerun does.
