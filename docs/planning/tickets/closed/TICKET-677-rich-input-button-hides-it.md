# TICKET-677 — The Rich Input button hides it too

- **Ticket:** LOCAL #677 (feature, the agent bar)
- **Owner:** claude-opus-5-5, 2026-10-07 (Chad's request)
- **Pipeline doc:** ../../pipeline/completed/677-rich-input-button-hides-it.spec.md
- **Source ticket:** #481 (Rich Input)
- **Status:** closed

## Summary
Chad, 2026-10-07: "when i click rich input here for the claude session only way to close it is esc.
on warp is said Rich Input and Hide Rich input. mayebe we do something similar?" The agent bar's
pencil opens the agent's editor (`rich_input::open`); only Escape closes it. While the editor is
open the button shows pressed, its tooltip reads Hide Rich Input, and a click closes the editor and
keeps the draft, as Escape does.

## Acceptance
With the agent's editor open, the bar's button is pressed and says Hide Rich Input; a click closes
the editor, keeps its draft, and gives the terminal the focus; the next click opens it with the
draft.
