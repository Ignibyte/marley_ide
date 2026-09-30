# TICKET-611 — The marley.work/v1 clients over MCP and HTTP

- **Ticket:** LOCAL #611 (feature, prong 2 D20: the fleet, wave 1)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/completed/611-work-provider-clients.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones", after the fleet contract (docs/marley/fleet-contract.md, D20) was settled with him.
- **Status:** closed

## Summary
Marley reads a real workflow store over MCP (stdio or HTTP, through Zed's context_server) or plain HTTP, polls or follows its change cursor, and shows each source's state honestly: not set up, connecting, ready, unreachable, stale, incompatible.

## Acceptance
Against the stub provider, both an HTTP and an MCP provider list their agents; a killed, silent or wrong-contract provider reads unreachable, stale or incompatible without touching the other; the bearer never reaches the log.
