# TICKET-583 — Chromium's DevTools off TCP, behind Marley's relay

- **Ticket:** LOCAL #583 (feature, prong 3; slice 2 of #524's three)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet
- **Source ticket:** #524's split (docs/planning/pipeline/completed/524-trusted-outside-browser-access.notes.md, "The split")
- **Status:** open

## Summary
Each project's Chromium listens for DevTools on a TCP port on 127.0.0.1 with no credential, and a
TCP port on loopback checks no user, so every process on the machine, other users' included, can
drive the browser and read its cookies. #524 gave outside clients a front door through Marley's
MCP server; this closes the side door. The project's unit runs a small Marley relay that starts
Chromium with `--remote-debugging-pipe` and serves Marley over a Unix socket in
`$XDG_RUNTIME_DIR` (mode 0600, so the kernel admits only the user), and Playwright clients (#523's
runner, a Playwright MCP, the e2e fixture's stand-in agent) a CDP WebSocket on loopback that takes
a client's token from #524's table. It changes `marley_browser::cdp::connect`, which takes a port
today, #507's per-project endpoint lookups, #523's `MARLEY_CDP_URL`, and amends plan D16.

## Acceptance
No project's Chromium listens on a TCP port; Marley drives its tabs over the relay's Unix socket;
a Playwright client with an allowed client's token attaches over the relay's WebSocket and one
without is refused; #523's scripts run as before.
