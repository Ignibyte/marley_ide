# TICKET-917 — The terminal prompt as a card, the sticky header, session chips

- **Ticket:** LOCAL #917 (feature, the Marley look, terminal)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [917-terminal-prompt-and-session.spec.md](../../pipeline/queued/917-terminal-prompt-and-session.spec.md)
- **Source ticket:** the design review of 2026-10-23
  (docs/marley/design-review-2026-10-23/A-core.md, boards Terminal and TerminalSession: "Prompt
  bar as a card", "'N changed' chip at the prompt", "Prompt card extras", "Sticky 'above you'
  banner", "Session chips in the tab row", "'Tell me when done' on a running block")
- **Boards:** Terminal, TerminalSession
- **Size:** M
- **Status:** open

## Summary

At a shell prompt Marley shows a flat editor strip with the folder and branch muted under it. The
boards draw the prompt as a card: coloured chips for the folder, the branch, how many files
changed and the last exit, a `❯` before the line, "→ to accept" after a suggestion, a row of key
hints and a "Start an agent here" button. The sticky header says "<command> is above you ·
running 2 m 04 s" with a "Jump to it" button, where Marley shows the command, a pill and an
arrow. The tab row's right end names the session (bash, a tmux session, an ssh host) with its
folder and branch. A running block offers "Tell me when done". This ticket builds those.

## Acceptance

The prompt is a card with its chips, `❯`, the accept hint, key hints and Start an agent here; the
sticky header reads as the board with Jump to it; the tab row names the session; Tell me when done
posts a notification when that block ends; shot in Marley Night and Marley Day.
