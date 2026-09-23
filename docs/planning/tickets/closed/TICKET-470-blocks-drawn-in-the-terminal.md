# TICKET-470 — Blocks drawn in the terminal, stage one

- **Ticket:** LOCAL #470 (feature, prong 1 T1a)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/470-blocks-drawn-in-the-terminal.spec.md
- **Source ticket:** ../../../marley/three-prong-plan.md (T1; D3, stage one)
- **Status:** closed

## Summary
Zed's terminal keeps each command as a block since #464, and bash and zsh report them since
#463 and #465, but nothing draws them. The plan's T1 is stage-one rendering: decorations over
the terminal's own rows, with no change to its row model. This first slice draws a gutter bar
beside each block's rows, a status pill at the top right of each finished or running block,
and a wash over the running and failed ones. Hover actions (copy, rerun) and block navigation
keys follow as T1b and T1c.

## Acceptance
Every block on screen has a gutter bar over its rows in its status's color; a block whose
first line is on screen has a status pill; a running or failed block has a wash; nothing is
drawn on the alternate screen. The EARS criteria are in the spec.
