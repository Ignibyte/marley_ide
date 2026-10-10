# TICKET-741 — A new agent on a remote host

- **Ticket:** LOCAL #741 (feature, the control plane: the harness)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** Chad, 2026-10-10: "Bonus points if we can open on a remote via the harness."
  Plan: [agents-anywhere shelf](../../design-notes/agents-anywhere-2026-10-10.md).
- **Status:** open

## Summary
The New Agent picker's where lists each harness host. Picking one with a CLI agent sets up a seat
there (`seat add`, `seat start`) and opens a terminal attached to it over SSH (`rh attach`), with
the reconnect a remote terminal has (#543, #641). The session tab's Views open over SSH too, which
the guide says they do not yet.

## Acceptance
Through a stand-in harness behind a fake `ssh`, New Agent on that host makes the seat and opens a
terminal running `rh attach` through `ssh`.
