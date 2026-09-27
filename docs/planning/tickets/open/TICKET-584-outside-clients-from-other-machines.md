# TICKET-584 — Outside clients from other machines

- **Ticket:** LOCAL #584 (feature, prong 3; slice 3 of #524's three)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet
- **Source ticket:** #524's split (docs/planning/pipeline/completed/524-trusted-outside-browser-access.notes.md, "The split")
- **Status:** open

## Summary
#524's clients reach Marley's MCP server on 127.0.0.1 only, on a port that changes at each start.
A client on another machine, a Playwright MCP on a second computer or a phone, needs a way in that
never opens the port to the network: a fixed loopback port, reached through `ssh -L` (the house
rule of rustal-harness M9: "Remote clients arrive through M8's authenticated SSH path") or
`tailscale serve` with its certificate (Orca report 04 §3.2 item 3), with the client's own token
as before.

## Acceptance
A client on another machine lists and calls its grant's tools through an SSH tunnel to a fixed
loopback port; Marley still listens on 127.0.0.1 only; the client's token and Cut Off work as on
the machine itself.
