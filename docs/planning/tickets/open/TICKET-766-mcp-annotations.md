# TICKET-766 — MCP annotations on every Marley tool

- **Ticket:** LOCAL #766 (chore, the finish line)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [766-mcp-annotations.spec.md](../../pipeline/queued/766-mcp-annotations.spec.md) (queued; drafted from [756's notes](../../pipeline/completed/756-the-browser-tools-agents-lack.notes.md))
- **Source ticket:** split from #756 (the browser tools agents lack, slice 2) on 2026-10-11, when Chad asked to "spec all" of the survey list. Plan: [finish-line shelf](../../design-notes/finish-line-2026-10-11.md).
- **Status:** open

## Summary
`readOnlyHint`, `destructiveHint`, `idempotentHint` and `openWorldHint` on every tool Marley's MCP server lists and on the IDE server's five, and the guide's tool tables catch up (four tools were missing at the T3 survey).

## Acceptance
`tools/list` carries annotations on every tool, and the guide lists every tool.
