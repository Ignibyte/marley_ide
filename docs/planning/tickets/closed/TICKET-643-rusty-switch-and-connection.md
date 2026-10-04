# TICKET-643 — Marley connects to Rusty when Rusty is turned on

- **Ticket:** LOCAL #643 (feature, Rusty in Marley R1: R-D0, R-D1, R-D2, R-D8; prong 2 C2)
- **Owner:** claude-opus-5-5, 2026-10-03 (/spec)
- **Pipeline doc:** ../../pipeline/completed/643-rusty-switch-and-connection.spec.md
- **Source ticket:** `docs/marley/rusty-in-marley.md` (R-D0 to R-D2, R-D8, the slices table's R1);
  Chad, 2026-10-03: "lets make a plan to begin the work and spec out the tickets", confirmed as
  "Queue all five"
- **Status:** closed

## Summary
Rusty in Marley starts with a switch and a connection. `marley.rusty` is one settings block, off
by default: on, Marley starts `rusty-mcp` on stdio (`connection: embedded`) or connects to Rusty's
running service (`service`), keeps that connection, and shows its state on a Rusty section of the
Settings window's Marley page, with Rusty's own embedding provider read and written through
Rusty's tools (`settings_list`, `setting_set`). Off, Marley starts no `rusty-mcp`, opens no
connection and offers Zed's agents no Rusty tools. `marley.rusty_tools` (#633, off by default
since #642) moves into the block as `agent_tools`, and a settings migration in Zed's `migrator`
carries a user's value over. The ticket also creates `crates/marley_rusty`, the pure core the
later slices build on (#644 to #647), holding the typed view of Rusty's settings, fixtures in
`rusty-mcp`'s own answer shape, and the stand-in `rusty-mcp` every scenario runs, so no scenario
reaches the user's Rusty (R-D8).

## Acceptance
With nothing set, Marley runs no `rusty-mcp` and the Rusty section says Rusty is off. A user's
`marley.rusty_tools: true` reads as Rusty on with Rusty Tools for Agents on. Turned on, Marley
connects to the `rusty-mcp` it finds, or to the service, and shows it connected, with Rusty's
embedding provider as Rusty stores it; picking another writes it to Rusty and shows it read back,
and a change Rusty announces shows without a step by the user. A `rusty-mcp` that exits is
started again; one that cannot be found is named with where Marley looked; turning the switch off
ends the process and the connection without a restart.
