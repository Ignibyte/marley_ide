# TICKET-706 — Agent Panel threads over MCP

- **Ticket:** LOCAL #706 (feature, Marley's MCP server)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** Chad, 2026-10-09: "yes i would love to have full control over the zed ide" (with a security layer, not approve-every-action); intake: [zed-control-over-mcp.md](../../intake/zed-control-over-mcp.md)
- **Status:** open

## Summary
`thread_list` (stored and live threads, as the rail reads them), `thread_read` (messages and tool calls, redacted), `thread_post` (Act, through the message editor and `ThreadView::send`, so the panel's queue holds) and `thread_answer` (another thread's pending permission, Sensitive, never the caller's own; no answer to a sandbox escalation, and Allow refused while the request carries confusable Unicode, as the panel does).

## Acceptance
An agent lists and reads another thread, posts into it, and answers its pending permission only after the user's per-call Allow; its own thread's permission is refused.
