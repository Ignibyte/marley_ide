# TICKET-625 — Completions in the prompt editor

- **Ticket:** LOCAL #625 (feature, prong 1 T6)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/completed/625-completions-in-the-prompt-editor.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`)
- **Status:** closed

## Summary
With a Zed editor at the prompt (#624), Zed's completion menu can offer what a command line needs: paths against the prompt's folder, commands from history, and the project's tasks.

## Acceptance
WHEN the user asks for completions in the prompt editor, Marley shall list matching paths of the prompt's folder, history entries and tasks.
