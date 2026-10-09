# TICKET-704 — Editors over MCP: list, read and open

- **Ticket:** LOCAL #704 (feature, Marley's MCP server)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [704-editors-read-and-open.spec.md](../../pipeline/completed/704-editors-read-and-open.spec.md)
- **Source ticket:** Chad, 2026-10-09: "yes i would love to have full control over the zed ide" (with a security layer, not approve-every-action); intake: [zed-control-over-mcp.md](../../intake/zed-control-over-mcp.md)
- **Status:** closed

## Summary
`editor_list` (every open editor in every window: path, project, dirty, language, cursor and selections), `editor_read` (a buffer's text by path or id, the unsaved text included, paged and redacted, files matching the secret globs refused) and `editor_open` (a file at a line, Act), through #703's layer. It also brings the per-area mode `marley.agent_control.editors` (`off`, `ask_every`, `ask_first`, `allow`; `ask_first` by default) and the once-per-session question (Allow for this session, Always for this project, Deny), keyed on the caller's terminal or client and the project, which `editor_open` is the first to use.

## Acceptance
An agent lists the open editors, reads an unsaved buffer's current text, and opens a file at a line after the one-time Act question.
