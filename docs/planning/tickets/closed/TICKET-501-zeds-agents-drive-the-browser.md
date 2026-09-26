# TICKET-501 — Zed's own agents get Marley's tools, the browser among them

- **Ticket:** LOCAL #501 (feature, prong 3 with prong 2's C0, after wave 2)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/501-zeds-agents-drive-the-browser.spec.md
- **Source ticket:** Chad, 2026-09-25: "the built in zed agents should be extended or made to be able to drive the browser as well if it hasnt already"
- **Status:** closed

## Summary
Marley's MCP server reaches Claude Code in a terminal through the plugin's bridge, but not the
agents of Zed's Agent Panel: the Zed Agent takes its tools from the project's context servers,
and each external (ACP) agent is handed those servers when its session starts, and Marley's
server is not among them. Marley registers it as the context server `marley` among Zed's
defaults, a stdio server running its bridge, so every one of those agents can pull up the
browser and drive it.

## Acceptance
The Zed Agent lists Marley's tools; an external agent started from the Agent Panel is handed
Marley's server and opens a page in a Browser tab through it.
