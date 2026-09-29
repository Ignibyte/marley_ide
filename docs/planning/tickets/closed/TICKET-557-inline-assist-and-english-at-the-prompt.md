# TICKET-557 — Inline Assist proven, and English at the prompt by local rules

- **Ticket:** LOCAL #557 (feature, prong 1 T3 with prong 2: the Warp blocks note, recommendation 4)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/completed/557-inline-assist-and-english-at-the-prompt.spec.md
- **Source ticket:** docs/planning/design-notes/warp-blocks-and-natural-language-2026-09-25.md (items 1 and 2 of "What Marley would do"; Chad, 2026-09-26: "local first and then jev second")
- **Status:** closed

## Summary
Two halves of Warp's natural-language input, without a network model. First, an e2e scenario
proves Zed's Inline Assist (Ctrl+Enter in a center terminal, already in the tree) works in the
Marley layout against a stand-in model on localhost: the prompt opens, one command streams onto
the prompt line, Ctrl+Enter runs it. Second, English at the prompt by local rules only: while
the shell waits at its prompt, rules read the typed line as a command or as English (a first
word that is no command, Warp's single-word rule, a command followed by English), a dimmed hint
after the cursor says Ctrl+Shift+Enter asks the agent, that key clears the shell's line and
sends the text to the project's agent terminal, and a block that ended with exit 127 on an
English-looking line gets an "Ask the agent" button. Enter always stays with the shell. A System
One second stage is TICKET-573 and needs nothing from this ticket to work alone.

## Acceptance
The Inline Assist scenario shows the prompt, the generated command and its run; an
English-looking line shows the hint and Ctrl+Shift+Enter sends it to the agent; a command shows
no hint and Ctrl+Shift+Enter reaches the shell; an exit-127 block offers "Ask the agent"; the
setting turns the hint off. The full EARS criteria live in the pipeline spec.
