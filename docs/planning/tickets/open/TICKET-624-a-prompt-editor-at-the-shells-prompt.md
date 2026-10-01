# TICKET-624 — A prompt editor at the shell's prompt, on a key

- **Ticket:** LOCAL #624 (feature, prong 1 T3)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/queued/624-a-prompt-editor-at-the-shells-prompt.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`)
- **Status:** open

## Summary
#481 docks a Zed editor in a terminal's footer for an agent's prompt. At a shell's prompt, Ctrl+G opens the same editor with what is typed so far, and Enter clears the shell's line and runs the editor's text: the first step toward the plan's prompt editor, with Zed's editing, before it becomes the default.

## Acceptance
WHEN the user presses Ctrl+G at a shell prompt, types a command in the editor and presses Enter, the shell shall run that command as a block.
