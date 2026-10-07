# TICKET-687 — The Marley agent on Codex and on Zed's agent

- **Ticket:** LOCAL #687 (feature, prong 2 C)
- **Owner:** ce546757-1075-466f-a3cf-b6696d2211c9
- **Pipeline doc:** (set at promotion)
- **Source ticket:** `docs/planning/intake/marley-agent-manager-foreman.md`, phase 1, item 5
  (split from TICKET-683 on 2026-10-07)
- **Status:** open

## Summary
TICKET-683 gives the Marley agent to someone signed in to Claude Code. This ticket covers the
others: Codex through the registry's `codex-acp` in its read-only sandbox, found by
`codex login status`; and Zed's own agent as a "Marley" profile with Marley's MCP tools and no
file tools, for someone with an API key, a local model or Zed's plan. Zed's agent takes no system
prompt of its own, so its profile leans on the MCP server's instructions. The `marley.assistant`
setting's `agent` picks among them when more than one is there.

## Acceptance
With Claude Code missing and Codex signed in, the offer names Codex and a Marley thread runs on
Codex read-only; with neither, and a Zed language model configured, the Marley profile appears in
Zed's agent with the docs and settings tools and no file tools.
