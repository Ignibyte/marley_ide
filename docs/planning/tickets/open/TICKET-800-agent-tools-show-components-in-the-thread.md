# TICKET-800 — Agents show components in the thread

- **Ticket:** LOCAL #800 (feature, agent components)
- **Owner:** b6416b37-a6bf-4172-91e4-8d05bc38cda5
- **Pipeline doc:** ../../pipeline/queued/800-agent-tools-show-components-in-the-thread.spec.md
- **Source ticket:** Chad, 2026-10-17 (as #799)
- **Status:** open

## Summary
Two Marley MCP tools: `component_list` answers with the catalog (each component's description,
input schema and example), and `component_show` takes a component tree. In an agent thread, the
tool call's row draws the component in place of its raw input, for Claude Code, Codex and Zed's
own agent alike, since each keeps the call's input. A call from an agent in a terminal opens the
component in a center tab beside it. The thread view gets one small hook for this; the drawing
is Marley's.

## Acceptance
An agent's `component_show` call draws its table (or any catalog component) inline in the
thread; a tree with an error answers the agent with the error and draws nothing.
