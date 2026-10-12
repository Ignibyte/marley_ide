# TICKET-768 — Edit each CLI's MCP servers

- **Ticket:** LOCAL #768 (feature, the finish line)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [768-edit-cli-mcp-servers.spec.md](../../pipeline/queued/768-edit-cli-mcp-servers.spec.md) (queued; drafted from [720's notes](../../pipeline/completed/720-mcp-manager-and-cli-update-checks.notes.md))
- **Source ticket:** split from #720 (the MCP manager, slice 2) on 2026-10-11, when Chad asked to "spec all" of the survey list. Plan: [finish-line shelf](../../design-notes/finish-line-2026-10-11.md).
- **Status:** open

## Summary
Add, change and remove an MCP server for Claude Code and Codex from #720's tab, through #755's safe write. Claude Code rewrites its own config while it runs, so the write is checked against a running session first.

## Acceptance
A server added in the tab is in the CLI's config and the CLI lists it at its next start.
