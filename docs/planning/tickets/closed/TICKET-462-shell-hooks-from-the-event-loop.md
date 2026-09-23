# TICKET-462 — Shell hooks from alacritty's event loop

- **Ticket:** LOCAL #462 (feature, prong 1: T0b)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/462-shell-hooks-from-the-event-loop.spec.md
- **Source ticket:** ../../../marley/three-prong-plan.md (T0, D1, D2)
- **Status:** closed

## Summary
The vendored `alacritty_terminal` (#461) reports Marley's shell hooks. Its event loop scans each
read with Marley's DCS scanner, parses the other bytes in stream order, and at each complete
hook emits `Event::ShellHook` with the grid position at that point, taken under the terminal
lock. The scanner moves out of `marley_terminal`, which depends on `alacritty_terminal`, into a
leaf crate. A monotonic count of lines dropped off the top of the grid keeps positions stable
as scrollback rolls. Zed's side of the event is #464.

## Acceptance
A coalesced read carrying `[preexec]` then the output line then `[precmd]` emits the two hooks
in that order, with positions that bracket the output line. A DCS that is not Marley's still
reaches the parser.
