# TICKET-707 — Palette actions over MCP

- **Ticket:** LOCAL #707 (feature, Marley's MCP server)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** Chad, 2026-10-09: "yes i would love to have full control over the zed ide" (with a security layer, not approve-every-action); intake: [zed-control-over-mcp.md](../../intake/zed-control-over-mcp.md)
- **Status:** open

## Summary
`action_run` runs a palette action by name with JSON arguments (`App::build_action`, dispatched from the target workspace's focus). An explicit allowlist is Act (navigation, panels, splits, formatting, search); anything else is refused unless the user names it in `marley.agent_control.actions_allowed`; quitting, window closes, tasks and code runs, terminal SendText/SendKeystroke, consent answers, client grants, destructive git, file deletes and installs are refused outright.

## Acceptance
An allowlisted action runs after the one-time Act question; an unlisted one is refused with how to allow it; a refused-outright action can't be allowed.
