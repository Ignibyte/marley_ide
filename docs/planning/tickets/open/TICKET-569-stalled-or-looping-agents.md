# TICKET-569 — Stalled or looping agents: a flag on the rail row, never a stop

- **Ticket:** LOCAL #569 (feature, prong 2, C1's attention; use 4 of the System One layer, on #565)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/queued/569-stalled-or-looping-agents.spec.md
- **Source ticket:** Chad, 2026-09-26, approving the seven ranked uses of
  `docs/planning/design-notes/jev-system-one-2026-09-25.md` (use 4, "Stalled or looping") on the
  layer TICKET-565 builds, with his rules: "local first and then jev second", and "we need
  probably every aspect of this configurable and turned off / on where the system will use or
  wont use it. Otherwise this becomes a jev required system."
- **Status:** open

## Summary
A working Claude Code row says `working` until #547's thirty quiet minutes make it say `no update
in N m`; between the two, Marley cannot tell a long `cargo test` from a session that froze or a
turn that keeps running the same tool. This ticket adds a flag on the row, and only a flag: `looping?`
when the turn repeats one tool line, `stalled?` with a kind (waiting for input, stuck, frozen) when a
working seat has gone quiet with nothing running under it. Facts computed in code decide first: the
seat's events, the repeats, the CPU of the process tree under the terminal, and Marley's own pending
approvals. A System One question goes through #565's layer only for the quiet case the facts leave
open, at 1, 2, 4 and 8 minutes. The flag is a mark and a word on the row, a label on the seat (as
#566's kind is), and in `act` mode one desktop notification. Marley never stops, interrupts or types
into the agent. Off by default as the use `stall_kind` in `marley.system_one.uses`; the `rules`
provider runs the facts with no model at all.

## Acceptance
On the `rules` provider, a turn that runs the same tool line three times reads `looping?`; a quiet
seat with a tool in flight and CPU under it never flags; on the `replay` provider, a quiet seat the
facts leave open reads `stalled?` with the replayed kind, a `cannot_tell` reading shows nothing, and
`act` posts one notification; the flag rides on the seat's labels and `fleet_snapshot`, and leaves at
the seat's next event; nothing is ever written to the agent's terminal.
