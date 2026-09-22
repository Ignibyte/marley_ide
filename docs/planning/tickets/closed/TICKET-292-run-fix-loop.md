# TICKET-292 — close the run→fix loop (re-run the failed command from the editor)

- **Forge ticket:** #292 `0db4535b-6237-4318-ae18-a86ba1ce80ca` (feature, M18)
- **Owner:** autonomous /goal run (sprint #31 — M18 Terminal↔Editor Fusion)
- **AAR:** `0e7958f7-089f-4c3c-9279-7ab62656de78`
- **Pipeline doc:** ../../pipeline/active/292-run-fix-loop.spec.md
- **Status:** closed

## Summary
The payoff of the fusion wedge: after you edit+save a file a failed command
referenced, re-run it without leaving the editor, and watch the stale markers
clear on green. A pure `last_failure_block_index(blocks)` locates the last
Failure block; a cockpit palette command "Re-run Last Failed Command" re-runs it
via the shipped #175 `rerun_block`. Clear-on-green is inherent — `rerun_block`
writes a NEW last block, and #289's gutter sources only the LAST block when it is
a Failure, so a green re-run's Success block clears the markers automatically.

## Acceptance
A failing command → edit+save the referenced file → invoke "Re-run Last Failed
Command" from the palette → the command re-runs → on green the gutter markers
clear. Full EARS in the pipeline spec.
