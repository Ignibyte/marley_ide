# TICKET-528 — Filter a block's output

- **Ticket:** LOCAL #528 (feature, prong 1 T1: stage-one block actions)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/528-block-filter.spec.md
- **Source ticket:** Chad, 2026-09-25, approving all seven items of the Warp once-over (`docs/planning/design-notes/warp-once-over-2026-09-25.md`, item 5, "Block filtering and the sticky command header")
- **Status:** open

## Summary
A command that prints thousands of lines leaves the one line that matters somewhere in them. Zed's
terminal search highlights matches in place; it hides nothing. Warp's block filter shows only
the lines of a block that match text or a regex, with toggles for case, invert and context lines,
and deletes nothing. Marley's first slice opens the filter as a panel over the terminal: Alt+Shift+F
filters the newest block in view, a Filter button joins Copy and Rerun on a hovered block, and the
panel lists the block's matching lines in a read-only editor, following a running block's output.
Escape closes it and the terminal is as it was. Filtering the rows in place waits for stage two's
display-row map (T5).

## Acceptance
Alt+Shift+F in a terminal opens the filter on the newest block in view; typing `error` lists only
that block's lines holding it, the toggles change the match as their names say, and Escape shows
the terminal unchanged.
