# TICKET-703 — The agent-control layer: tiers, consent, an activity log and a kill switch

- **Ticket:** LOCAL #703 (feature, Marley's MCP server)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [703-agent-control-layer.spec.md](../../pipeline/completed/703-agent-control-layer.spec.md)
- **Source ticket:** Chad, 2026-10-09: "yes i would love to have full control over the zed ide" (with a security layer, not approve-every-action); intake: [zed-control-over-mcp.md](../../intake/zed-control-over-mcp.md)
- **Status:** closed

## Summary
Before Marley's MCP server gains tools that drive Zed itself, every write tool an agent calls
(terminals, the browser, settings, the harness, and every later one) lands in an activity log the
user can browse: who, which tool, what it acted on (redacted), and whether it ran or was refused.
`marley.agent_control.stopped` is a kill switch. On, every write tool refuses with
`agent_control_stopped` while reads keep working. `marley: stop agent control` and `marley: resume
agent control`, a button on Home's AGENT ACTIVITY card and a Settings toggle set it. The Agent
Activity tab, in the Home group, lists the rows. The per-area modes and the once-per-session
question move to #704 with the first tool that uses them; the rail mark is a follow-up.

## Acceptance
An agent's `terminal_run` shows in the Agent Activity tab; after `marley: stop agent control` its
next call is refused with `agent_control_stopped` while `terminal_list` still answers, and the
refusal shows too.
