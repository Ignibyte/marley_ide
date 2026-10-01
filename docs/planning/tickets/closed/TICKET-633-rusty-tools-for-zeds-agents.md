# TICKET-633 — Rusty's tools for Zed's agents, when Rusty is installed

- **Ticket:** LOCAL #633 (feature, prong 2 C2, plan D11)
- **Owner:** claude-opus-5-5, 2026-10-01
- **Pipeline doc:** ../../pipeline/completed/633-rusty-tools-for-zeds-agents.spec.md
- **Source ticket:** plan C2 ("brain-loop and Rusty tools in the default `context_servers`"), D11 ("Rusty stays Rusty … points agents at `rusty-mcp`"); wave 4
- **Status:** closed

## Summary
When `rusty-mcp` is on the search path, Marley offers it to Zed's agents as a context server named
`rusty`, the way it offers its own `marley` server (#501): a stdio `rusty-mcp`, added to Zed's
defaults, which a user's own `context_servers.rusty` replaces. Zed asks before each call, as it does
for every context server. `marley.rusty_tools` turns it off. Rusty's agent sessions in the rail are
a separate intake (`intake/rusty-sessions-in-the-rail.md`): Rusty serves no session tools yet.

## Acceptance
With a `rusty-mcp` on the search path, Zed's agent settings list a `rusty` server with its tools;
without one, or with the setting off, there is none; a user's own `rusty` entry wins.
