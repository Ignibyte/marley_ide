# TICKET-476 — The prompt at the bottom of the terminal

- **Ticket:** LOCAL #476 (feature, prong 1: T1d)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/476-prompt-at-the-bottom.spec.md
- **Source ticket:** ../../../marley/three-prong-plan.md (T1)
- **Status:** closed

## Summary
Warp pins its input to the bottom of the pane, and output flows up above it. A terminal draws
from the top instead, so a new terminal, or one just cleared, shows its prompt on the first
row. While a terminal's live screen has empty rows below its content, Marley draws the content
against the bottom edge, so the prompt sits on the last row and each command's output pushes
the rest up. Chad asked for it on 2026-09-23.

## Acceptance
With room to spare, the last used row is drawn on the pane's last row; scrolling back shows the
history above it; the alternate screen draws as before; clicks, selection and the block
decorations follow the shifted rows. The EARS criteria are in the spec.
