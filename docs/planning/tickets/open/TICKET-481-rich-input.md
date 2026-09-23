# TICKET-481 — Rich input: a Zed editor for an agent's prompt

- **Ticket:** LOCAL #481 (feature, prong 1: T7e)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/481-rich-input.spec.md
- **Source ticket:** ../../../marley/three-prong-plan.md (T7, and T3's prompt editor)
- **Status:** open

## Summary
Warp's rich input puts its own editor in place of a CLI agent's prompt box: you write the
prompt with the mouse, several lines, undo, word motion and Vim keys, then send it to the
agent. Ctrl-G opens it. Marley does the same with a Zed editor docked at the bottom of the
terminal while an agent runs: Enter sends the text to the agent as one paste and a carriage
return, Escape closes it and keeps the draft. It is the agent-first half of T3's prompt editor.

## Acceptance
Ctrl-G or the bar's button opens the editor with the focus; Enter sends the text and closes
it; Escape sends nothing and keeps the draft; without an agent, Ctrl-G is the terminal's as
before. The EARS criteria are in the spec.
