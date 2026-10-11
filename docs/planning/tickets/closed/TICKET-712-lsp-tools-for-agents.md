# TICKET-712 — LSP tools for agents

- **Ticket:** LOCAL #712 (feature, Marley's MCP server)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [712-lsp-tools-for-agents.spec.md](../../pipeline/completed/712-lsp-tools-for-agents.spec.md)
- **Source ticket:** the 2026-10-09 intake audit of `mcp-first-class-control-plane.md`: all of it
  shipped but its LSP tools. Chad, 2026-10-11: "6.) yes".
- **Status:** closed

## Summary
Four read tools over Zed's LSP store, in the editors area (#704):
- `editor_diagnostics`: one file's errors and warnings, or a project's;
- `editor_definition`, `editor_references` and `editor_hover`: at a path and a line, with the
  symbol's name on that line or a column.

The file need not be open in an editor: the tool opens it in its project without a tab and asks
its language server. Answers are redacted and paged as `editor_read`'s are, a secret file is
refused, and each call is listed in Agent Activity.

Today no agent outside Zed's own gets diagnostics: #653's IDE link carries `getDiagnostics` to
Claude Code, but its version rows are not yet checked on any release, so the link is off. Zed's
native agent has its own `diagnostics` tool, and `go_to_definition` and `find_references` behind
a feature flag; Codex, Claude Code and the Marley and Rusty agents have nothing.

## Acceptance
In a scenario with a stand-in language server (#653's, taught definition, references and hover),
a client in a Marley terminal gets main.rs's error, the project's diagnostics, a definition, its
references and a hover, and gets them for a file no tab shows; a secret file and a path outside
every project are refused.
