# TICKET-811 — Trusted outside clients with their own powers

- **Ticket:** LOCAL #811 (feature, Marley's MCP server)
- **Owner:** b6416b37-a6bf-4172-91e4-8d05bc38cda5
- **Pipeline doc:** ../../pipeline/completed/811-trusted-outside-clients.spec.md
- **Source ticket:** Chad, 2026-10-17: "we need a way to have a trusted device that can perform
  write actions as well. So registering a mcp server to marley needs to be trusted not just open
  and then given permissions (read, write, execute etc)".
- **Status:** closed

## Summary
Outside clients (Browser Clients, #524 and #584) are registered by name and granted the browser
only: read its pages, or also act in them. Chad wants to register a device or program as trusted,
choosing what it may do across Marley's tools (read, write, run), so a client on another machine
can do what an agent in a Marley terminal does, under the same approvals and kill switch. To be
designed with Chad: the powers, how a device proves itself, and where the list lives.

## Acceptance
A client registered as trusted with chosen powers can call exactly those tools, and Cut Off takes
them away at once.
