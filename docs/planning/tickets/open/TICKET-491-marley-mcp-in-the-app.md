# TICKET-491 — C0: Marley's MCP server runs in the app

- **Ticket:** LOCAL #491 (feature, prong 2 C0, pulled forward for prong 3)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/491-marley-mcp-in-the-app.spec.md
- **Source ticket:** docs/marley/three-prong-plan.md, prong 2 C0 and prong 3 D17
- **Status:** open

## Summary
`marley_mcp`, ported and compiled but never started, runs inside Marley: a Streamable HTTP
server on 127.0.0.1 with a per-boot bearer, its endpoint written to a 0600 file in Marley's
data directory and removed on quit. Tool calls that need the app are answered on the main
thread. The Marley Claude Code plugin gains a stdio bridge to it, so every Claude Code session
on the machine finds Marley's tools while Marley runs and an empty server when it does not.
The first tools are the terminal's blocks as data, the plan's C0; the browser's follow in
#492.

## Acceptance
A client started in one of Marley's terminals, through the plugin's bridge, lists Marley's
tools and reads that terminal's blocks (commands, exit codes, output); with Marley closed the
bridge answers with no tools and no error.
