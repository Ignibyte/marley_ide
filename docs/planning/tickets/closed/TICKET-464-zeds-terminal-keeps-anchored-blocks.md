# TICKET-464 — Zed's terminal keeps anchored blocks

- **Ticket:** LOCAL #464 (feature, prong 1: T0b, second half)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/464-zeds-terminal-keeps-anchored-blocks.spec.md
- **Source ticket:** ../../../marley/three-prong-plan.md (T0, D2)
- **Status:** closed

## Summary
Zed's `TerminalBackendEvent` mirrors `Event::ShellHook` (#462): the exhaustive
`From<AlacTermEvent>` in `crates/terminal/src/alacritty.rs` needs its arm. Zed's `Terminal`
decodes each hook with `marley_terminal`'s decoder and keeps a `BlockList` anchored by absolute
line (D2), exposed through `blocks()`, with each block's output read from the grid while it is
still in the scrollback. The gpui era's `BlockList` copies output lines, so the anchored model
is new.

## Acceptance
In a PTY-backed test, a shell that prints Marley's hook frames around `echo hi` leaves a
Finished block with exit 0 whose output is "hi", also when one write carries the whole
command.
