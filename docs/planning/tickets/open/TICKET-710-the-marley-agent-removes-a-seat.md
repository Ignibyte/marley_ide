# TICKET-710 — The Marley agent removes a harness seat

- **Ticket:** LOCAL #710 (feature; the counterpart of #692's `seat_add`)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** Chad, 2026-10-09: on Marley's tools for the Manager, "we can communicate still
  via our harness to remove agents and so forth right?"; offered as a small thing, then "lets do
  the small things".
- **Status:** open (waits on the harness)

## Summary
`seat_remove` on Marley's MCP server takes a seat's name and asks the user as `seat_add` does.
On Apply, Marley runs the harness's own seat removal and answers, and the seat leaves the rail's
Harness section.

## Why it waits
The harness has `rh seat add` and `rh seat start` (its TICKET-109) but no command that ends a
seat. `rh stop WORKSPACE` stops a workspace, but the seat's profile
(`ROOT/profiles/NAME.json`) stays, so `seat add` of the same name is refused `seat_exists`, and
supervision's view of a stopped seat isn't settled. Seat lifecycle is the harness's, so Marley
calls the harness's command rather than taking a seat apart itself.

The harness ticket, asked of rustal-harness on 2026-10-09: `rh seat stop NAME` (end the seat's
generation and its supervision; the profile stays) and `rh seat remove NAME` (stop it, then
delete the profile), each printing one JSON object and refusing by name, as `seat add` does.

## Acceptance
With `marley.harness_writes` on:
- `seat_remove` shows the question;
- Apply runs the harness's removal, answers `removed`, and the seat leaves the rail;
- a refusal from the harness comes back with its code.
