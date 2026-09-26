# TICKET-509 — Per-turn diffs for Claude Code in a terminal

- **Ticket:** LOCAL #509 (feature, prong 2)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/509-per-turn-diffs.spec.md
- **Source ticket:** Chad, 2026-09-25: "lets do 1 through 8" (item 7 of the list after the browser waves)
- **Status:** open

## Summary
Claude Code in a terminal edits files with nothing in Marley that says what each turn changed. Marley snapshots the tree when a turn starts and when it ends, through the plugin's turn hooks, and shows each turn's diff in a review view, so Chad reviews an agent's work turn by turn.

## Acceptance
Each Claude Code turn in a Marley terminal leaves a turn in a list; opening it shows that turn's diff.
