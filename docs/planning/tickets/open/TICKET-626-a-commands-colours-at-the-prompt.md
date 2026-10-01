# TICKET-626 — A command's colours at the prompt

- **Ticket:** LOCAL #626 (feature, prong 1 T6)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/queued/626-a-commands-colours-at-the-prompt.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`)
- **Status:** open

## Summary
A typed command reads better in colour: its command words, strings, variables and operators in the theme's syntax colours, as Zed colours a shell script. The prompt editor gets the bash language; the shell's own prompt gets the same colours over the typed range when it is drawn.

## Acceptance
WHEN the user types a command at the prompt, Marley shall draw its words in the theme's syntax colours, in the editor and on the shell's own prompt.
