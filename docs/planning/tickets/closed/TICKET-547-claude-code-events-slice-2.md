# TICKET-547 — Claude Code's events, slice 2: the plugin update, `fleet_snapshot`, and stale rows

- **Ticket:** LOCAL #547 (feature, prong 2 C1, after #519)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/547-claude-code-events-slice-2.spec.md
- **Source ticket:** split from #519 at its Plan, 2026-09-26, to land the rail's states first
- **Status:** closed (2026-09-26)

## Summary
#519 brings Claude Code's hook events into the rail: the plugin's `event.py`, the fold into
`marley_fleet`, and the rows' states and lines. Three parts of its design wait here:
- **The update.** The plugin's version moved with #519, and Claude Code runs an installed plugin
  from its versioned cache, so an existing install needs an update to get the new hook: the agent
  bar's chip reads "Update Marley's plugin" while `installed_plugins.json` lists an older version,
  and a click runs `claude plugin marketplace update marley` and `claude plugin update
  marley@marley` (#519's D9).
- **`fleet_snapshot`.** The MCP server's snapshot is empty today; `mcp.rs` publishes the app's
  snapshot on every change and signals the transport (#519's design, item 7).
- **Stale rows.** A working seat that has had no event for 30 minutes, with Claude Code still in
  the foreground, reads `no update in N m`, with a one-minute timer in the rail (#519's D7).

## Acceptance
With an older plugin installed the chip offers the update and a click runs it; `fleet_snapshot`
answers the rail's seats; a working row with no event for 30 minutes says so. (#519's queued
REQ-010, REQ-011 and REQ-013, moved here.)
