# TICKET-692 — The Marley agent sets up a harness seat

- **Ticket:** LOCAL #692 (feature, phase 2 item 6's tool, split from #691)
- **Owner:** ce546757-1075-466f-a3cf-b6696d2211c9
- **Pipeline doc:** [692-the-marley-agent-sets-up-a-seat.spec.md](../../pipeline/completed/692-the-marley-agent-sets-up-a-seat.spec.md)
- **Source ticket:** `docs/planning/intake/marley-agent-manager-foreman.md`, phase 2, item 6
- **Status:** closed

## Summary
"Set up a manager working in /srv/work/x" as a request to the Marley agent. `seat_add` on
Marley's MCP server takes a name, the agent, a folder and a role, and asks the user as
`settings_change` does. On Apply, Marley runs #691's `seat add` and answers. It then runs
`seat start` in the background, since a Claude seat may take a minute; a start that fails is a
notification.

## Acceptance
With `marley.harness_writes` on:
- `seat_add` shows the question;
- Apply adds the seat and answers `starting`, and the seat then shows in the rail;
- a refusal from the harness comes back with its code.

Off, the call is refused with `tool_off`.
