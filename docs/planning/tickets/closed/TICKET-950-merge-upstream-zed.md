# TICKET-950 — Merge upstream Zed through 2026-10-29

- **Ticket:** LOCAL #950 (chore, upstream sync)
- **Owner:** 9f998252-2a54-4459-b037-d2e59b730208
- **Pipeline doc:** [950-merge-upstream-zed.spec.md](../../pipeline/completed/950-merge-upstream-zed.spec.md)
- **Source ticket:** Chad, 2026-10-29: "one last thing i forgot. lets pull down latest zed release changes so we can stay up to date."
- **Status:** closed

## Summary
The fork carries upstream Zed's main of 2026-10-17 (`56934ec056`, #797). Upstream main has 190
more commits through `0373366f43` (2026-10-29), across 527 files, which hold the v1.26 and v1.27
work (v1.26.2 stable, v1.27.0-pre). A trial merge conflicts in ten files. One upstream commit
takes away an API Marley builds on: `eee7501970` removes agent checkpoints, and with them
`Repository::checkpoint` and `compare_checkpoints`, which Marley's turn record (#509) and turn
undo (#714) call.

## Acceptance
The fork builds on upstream's main of 2026-10-29 with every Marley hunk kept, turns are still
recorded and undone, the gate is green, and the merged release build starts in the Marley layout.
