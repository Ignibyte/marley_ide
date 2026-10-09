# TICKET-705 — Editors over MCP: edit and save

- **Ticket:** LOCAL #705 (feature, Marley's MCP server)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** Chad, 2026-10-09: "yes i would love to have full control over the zed ide" (with a security layer, not approve-every-action); intake: [zed-control-over-mcp.md](../../intake/zed-control-over-mcp.md)
- **Status:** open

## Summary
`editor_edit` applies a change to an open buffer as one undoable transaction (`Buffer::start_transaction`, `edit`, `end_transaction_with_source(Agent)`, as Zed's own agent edits), unsaved and shown in a review view (`git_ui`'s `TextDiffView`), Act; `editor_save` writes it, Sensitive.

## Acceptance
An agent's edit lands in the buffer unsaved as one undo step with a review view of it; a save asks first.
