# TICKET-627 — The prompt editor by default, with the raw-passthrough ladder

- **Ticket:** LOCAL #627 (feature, prong 1 T3)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/queued/627-the-prompt-editor-by-default.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`)
- **Status:** open

## Summary
The plan's prompt editor: while the shell sits at a prompt, keys go to the docked editor, and the ladder Marley proved sends them raw to the program otherwise (the alternate screen, a running command, bracketed paste, application cursor mode). #484's ghost text and → move into the editor, and #573's reading has its place there.

## Acceptance
WHILE the shell is at a prompt, Marley shall send typed keys to the prompt editor, and WHILE a program runs or the alternate screen is on, it shall send them to the program raw.
