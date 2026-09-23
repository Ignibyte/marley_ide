# TICKET-473 — Block navigation keys

- **Ticket:** LOCAL #473 (feature, prong 1 T1c)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/473-block-navigation-keys.spec.md
- **Source ticket:** ../../../marley/three-prong-plan.md (T1)
- **Status:** closed

## Summary
Blocks are drawn since #470; moving between them still takes the scrollbar. Two actions scroll
the focused terminal to the start of the previous or the next block, bound to `secondary-up` and
`secondary-down` in the terminal (Ctrl on Linux, Cmd on macOS, as Warp binds its block keys).

## Acceptance
With a terminal focused, the previous-block key puts the start of the block above the
viewport's top at the top, the next-block key the next one's or the live screen; with no block
that way the view stays. The EARS criteria are in the spec.
