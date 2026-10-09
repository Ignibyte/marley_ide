# TICKET-712 — LSP tools for agents

- **Ticket:** LOCAL #712 (feature, Marley's MCP server)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** the 2026-10-09 intake audit of `mcp-first-class-control-plane.md`: all of it
  shipped but its LSP tools.
- **Status:** open

## Summary
Read-tier tools over Zed's LSP store, in the editors area (#704):
- `editor_diagnostics`: an open file's, or the project's, errors and warnings;
- `editor_definition`, `editor_references` and `editor_hover`: at a path, a line and a column.

They are redacted and paged as `editor_read` is. Diagnostics reach Claude Code today only through
#653's IDE server, so the other agents (Codex, Zed's own, the Marley and Rusty agents) get none.

## Acceptance
Each tool answers in a scenario with a real language server: a Rust file with an error gives its
diagnostic, and a definition, its references and a hover resolve.
