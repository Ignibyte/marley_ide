# TICKET-584 — Outside clients from other machines

- **Ticket:** LOCAL #584 (feature, prong 3; slice 3 of #524's three)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/584-outside-clients-from-other-machines.spec.md
- **Source ticket:** #524's split (docs/planning/pipeline/completed/524-trusted-outside-browser-access.notes.md, "The split")
- **Status:** closed

## Summary
#524's clients reach Marley's MCP server on 127.0.0.1 only, on a port that changes at each start.
A client on another machine, a Playwright MCP on a second computer or a phone, needs a way in that
never opens the port to the network: a fixed loopback port, reached through `ssh -L` (the house
rule of rustal-harness M9: "Remote clients arrive through M8's authenticated SSH path") or
`tailscale serve` with its certificate (Orca report 04 §3.2 item 3), with the client's own token
as before.

**The plan's finding (2026-09-27).** The fixed port is dropped. A client's token is minted at
each start (AD-524), so a client on another machine that kept a token in its config would be
refused after Marley's next start. Instead its MCP client runs Marley's own bridge on this machine
over SSH, pointed at its endpoint file; the bridge reads each start's token where Marley wrote it,
and nothing new listens. HTTP clients on other machines, the phone among them, wait for the Orca
survey's item 12 and Chad's answer to its question 8.

## Acceptance
A client on another machine lists and calls its grant's tools through an SSH tunnel to a fixed
loopback port; Marley still listens on 127.0.0.1 only; the client's token and Cut Off work as on
the machine itself.

As planned: the spec's REQ-001 to REQ-007 (a client on another machine runs the copied SSH line;
Cut Off and a restart as on the machine itself; no new listening socket).
