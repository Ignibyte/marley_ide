# TICKET-497 — B3b: Open a picked element's listener source in the editor

- **Ticket:** LOCAL #497 (feature, prong 3 B3b; prong 3 wave 2)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/497-pick-source.spec.md
- **Source ticket:** docs/marley/three-prong-plan.md, prong 3 B3b; docs/marley/browser-handoff.md
- **Status:** open

## Summary
The signature move of pillar A: from a picked element to the code that handles it. For each
listener in a pick, Marley follows the script's source map to the original file and line, finds
that file in the workspace, and a click on the listener in the tray opens it in Marley's editor
at the line. The agent's `browser_pick` carries the original locations too.

## Acceptance
A pick's listener shows its original file and line when the script has a source map whose source
is in the workspace; a click opens the file at that line.
