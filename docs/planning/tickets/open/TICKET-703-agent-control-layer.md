# TICKET-703 — The agent-control layer: tiers, consent, an activity log and a kill switch

- **Ticket:** LOCAL #703 (feature, Marley's MCP server)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** Chad, 2026-10-09: "yes i would love to have full control over the zed ide" (with a security layer, not approve-every-action); intake: [zed-control-over-mcp.md](../../intake/zed-control-over-mcp.md)
- **Status:** open

## Summary
Before Marley's MCP server gains tools that drive Zed itself, it gets one layer every write tool goes through: three tiers (Read, Act, Sensitive) set per area in `marley.agent_control`; Act asks once per agent session and project (Allow for this session, Always for this project, Deny); Sensitive asks every time by default; a short list is never allowed. Every Act and Sensitive call lands in an activity log, shown as a card on Home and a tab of its own, with Undo where the call can be undone; `marley: stop agent control` (and a Home button) refuses every Act and Sensitive tool until turned back on; the acting agent's rail row shows a mark while it acts. It is proven on the existing terminal-write and settings tools first, so #704 to #707 plug into it.

## Acceptance
An Act tool's first call from an agent session asks once and later calls run without asking; every Act and Sensitive call shows in the activity log; the kill switch refuses them until it is turned off; reads keep working throughout.
