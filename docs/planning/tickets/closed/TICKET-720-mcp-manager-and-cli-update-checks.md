# TICKET-720 — An MCP manager and agent CLI update checks

- **Ticket:** LOCAL #720 (feature; size medium, the first of two slices)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [720-mcp-manager-and-cli-update-checks.spec.md](../../pipeline/completed/720-mcp-manager-and-cli-update-checks.spec.md)
- **Source ticket:** [monocode-findings.md](../../intake/monocode-findings.md), item 7; Chad, 2026-10-09: "lets queue up the MonoCode findings for potential future use"; Chad, 2026-10-11: "17.) build now if we can"
- **Status:** closed

## Summary
A center tab, MCP Servers, that shows in one place every MCP server each agent sees: Claude
Code's (its user, local and project scopes), Codex's, and Zed's own context servers with their
status. Env and header values never show, only their names. At its top, each agent CLI's path and
version and what Marley needs of it; when Claude Code or Codex is older than a part of Marley the
user turned on needs (the shared plugin, #709, wants Claude Code 2.1.287), the tab says so with
the way to update, and a notification says so once per version found.

Chad's 2026-10-11 rule that everything opens as a tab
([tabs-everywhere](../../design-notes/tabs-everywhere-2026-10-11.md)) puts it in a center tab
rather than the Settings page the finding named.

This ticket shows; a second slice edits (add, change and remove a server for Claude Code and
Codex), writing through #755's safe write. Its draft is in this spec's notes. Zed's own servers
are edited on Zed's MCP Servers settings page, which the tab opens.

## Acceptance
With fixture configs for Claude Code, Codex and Zed, the tab lists every server with its scope,
command or URL, and env and header names without values, Zed's with their status; a Claude Code
older than the shared plugin needs shows the warning in the tab and as a notification.
