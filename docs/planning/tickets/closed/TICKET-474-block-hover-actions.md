# TICKET-474 — Copy and rerun on a hovered block

- **Ticket:** LOCAL #474 (feature, prong 1 T1b)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/474-block-hover-actions.spec.md
- **Source ticket:** ../../../marley/three-prong-plan.md (T1)
- **Status:** closed

## Summary
The last of T1's stage-one pieces: while the pointer is over a block, two buttons show beside
its pill, one that copies the block's output and one that runs its command again at the prompt.

## Acceptance
Over a block, Copy puts its output on the clipboard, and Rerun, offered only while the shell is
at a prompt and for a command the shell's own hook reported, sends the command to the
terminal; the buttons stay hidden otherwise and take the clicks on them from the terminal. The
EARS criteria are in the spec.
