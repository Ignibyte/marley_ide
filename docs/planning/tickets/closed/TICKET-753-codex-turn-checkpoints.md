# TICKET-753 — Codex turn checkpoints

- **Ticket:** LOCAL #753 (feature, prong 2: "Codex gets what Claude Code gets", second half; T3 Code survey item 5)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [753-codex-turn-checkpoints.spec.md](../../pipeline/completed/753-codex-turn-checkpoints.spec.md)
- **Source ticket:** [T3 Code survey](../../../t3code_architecture/README.md), item 5; reports 01 (item 5) and 03 (§2.1, item 1). Chad, 2026-10-11, on "Codex gets what Claude Code gets": "3.) yes".
- **Status:** closed

## Summary
#509 keeps each Claude Code turn as a commit of its end on its start and lists it under the
terminal's row; Codex in a terminal gets none. This ticket takes the same checkpoints for a Codex
on its own App Server (#650), at the lead thread's `turn/started` and `turn/completed`, so the
Turns rows and their diffs cover Codex. Undo This Turn (#714, still undecided) would need these
commits to undo a Codex turn; this ticket does not build it.

## Acceptance
A Codex turn that changed files is listed under its terminal's row with its prompt and file count,
and opens in the commit view with that turn's changes alone, shell edits included.
