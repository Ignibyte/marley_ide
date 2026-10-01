# TICKET-619 — A block's path links resolve against the block's own folder

- **Ticket:** LOCAL #619 (feature, prong 1 T2)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/queued/619-block-path-links-at-the-blocks-folder.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`)
- **Status:** open

## Summary
A `path:line:col` printed by a command opens against the folder the terminal is in when the link is clicked, or Zed's per-line guess at it, which gives up once the scrollback is full. A block knows the folder its command ran in (`prompt.pwd`), so a link inside a block opens against that folder, wherever the shell has gone since.

## Acceptance
WHEN the user opens a relative path link inside a finished local block, Marley shall resolve it against the folder that block's command ran in.
