# TICKET-914 — An agent's answers as the board draws them

- **Ticket:** LOCAL #914 (feature, the Marley look, agent thread)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [914-thread-answers-as-the-board.spec.md](../../pipeline/completed/914-thread-answers-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23
  (docs/marley/design-review-2026-10-23/A-core.md, boards Main and MainDay: "Agent header on the
  answer", "User bubble content", "EDIT and RUN labels", "Reply chips", "Text-width toggle",
  "Send button"; docs/marley/design-review-2026-10-23/C-threads.md, board ThreadStates)
- **Boards:** Main, MainDay, ThreadStates
- **Size:** M
- **Status:** closed

## Summary

#825 and #864 gave the thread its bubbles, card labels and reply chips, but an answer still
starts as bare text, the EDIT and RUN labels are muted frames, the chips are square outlined
buttons, the send button is Zed's muted icon, the width toggle is an icon, and the user's own
message sits in a darker monospace box inside its bubble. The Main board draws each answer under
a header (the agent's tile and name, its model and effort, and a live "Working · 12 min"),
filled EDIT and RUN labels with a result chip, pill chips, a filled accent send button, a
"Full width" text button, and the user's words as plain text in the bubble. This ticket draws
them so.

## Acceptance

Each answer opens with its header and the newest shows its live state; EDIT and RUN labels are
filled with a result chip on RUN and the counts on EDIT; the chips are pills; the send button is
filled; the width toggle reads as words; the user's message is text in its bubble; shot in Marley
Night and Marley Day.
