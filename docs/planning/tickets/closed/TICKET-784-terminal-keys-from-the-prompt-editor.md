# TICKET-784 — Keys only the terminal binds, pressed in the prompt editor

- **Ticket:** LOCAL #784 (bug, prong 1 T3)
- **Owner:** c70cc71d-da30-4123-80da-1db398eb7911
- **Pipeline doc:** [784-terminal-keys-from-the-prompt-editor.spec.md](../../pipeline/completed/784-terminal-keys-from-the-prompt-editor.spec.md)
- **Source ticket:** #783's Complete phase, the sweep of
  `PR-claude-783-an-editor-in-a-terminal-keeps-the-keys-its-actions-pass-on-001`.
- **Status:** closed

## Summary
Zed's Linux keymap binds `ctrl-r`, `ctrl-w`, `ctrl-e`, `ctrl-b`, `ctrl-o`, `ctrl-q`, `alt-b`,
`alt-f`, `ctrl-enter` and others to `terminal::SendKeystroke` in the Terminal context, and the
Editor context binds nothing for them. In the shell's prompt editor (#627) and an agent's rich
input (#481) such a key then likely reaches the program behind the editor: Ctrl-R would start
bash's reverse search on the grid while the editor holds the line. Not yet seen in the app;
#783's sweep found it in the keymap. Reproduce each key first, then decide per key what the
editor does with it (a readline meaning in the editor, Marley's own, or nothing), and keep every
one away from the grid.

## Acceptance
No key pressed in the prompt editor or an agent's rich input reaches the terminal's program unless
the editor sends it on purpose.
