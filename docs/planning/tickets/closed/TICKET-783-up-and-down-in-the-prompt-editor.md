# TICKET-783 — Up and Down in the prompt editor reach the terminal behind it

- **Ticket:** LOCAL #783 (bug, prong 1 T3)
- **Owner:** c70cc71d-da30-4123-80da-1db398eb7911
- **Pipeline doc:** [783-up-and-down-in-the-prompt-editor.spec.md](../../pipeline/completed/783-up-and-down-in-the-prompt-editor.spec.md)
- **Source ticket:** Chad, 2026-10-14: "since we implemented the warp terminal at the bottom that
  is not the native terminakl. if I do an up arrow it does it on the alacritty not that part".
- **Status:** closed

## Summary
In the shell's prompt editor docked under a terminal (#627), Up on an empty editor, or on its
first line once the cursor cannot move, recalls the shell's history on the grid behind the editor
instead of in the editor. Zed's `Editor::move_up` passes the key on when its cursor cannot move,
and the next binding for Up in the editor's context stack is the terminal's
`terminal::SendKeystroke`. Down falls through the same way on the last line. The agent's rich
input leaks both keys to the agent's own input.

## Acceptance
Up in the shell's prompt editor brings the shell's older commands into the editor and Down brings
back newer ones and then the draft, as Warp's input does; neither key reaches the grid. In an
agent's rich input Up and Down move the cursor and never reach the agent.
