# TICKET-742 — A thread on a remote seat

- **Ticket:** LOCAL #742 (feature, the control plane: the harness)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** Chad, 2026-10-10: "Bonus points if we can open on a remote via the harness."
  Needs the harness's `rh acp` onto any seat (rustal-harness TICKET-116). Plan:
  [agents-anywhere shelf](../../design-notes/agents-anywhere-2026-10-10.md).
- **Status:** open

## Summary
The New Agent picker, on a harness host with "a thread", opens a thread tab whose agent is
`rh acp --seat NAME` run through the harness's command: what you type goes to the seat as a
message, and its replies and reports come back as the thread. The seat works on its own box, so no
file of it passes through Zed's project.

## Acceptance
Through a stand-in harness, a thread tab on a seat sends a message the harness records for that
seat and shows the seat's reply.
