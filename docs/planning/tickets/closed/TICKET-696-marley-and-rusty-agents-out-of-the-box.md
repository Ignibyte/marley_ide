# TICKET-696 — The Marley and Rusty agents out of the box

- **Ticket:** LOCAL #696 (feature)
- **Owner:** ce546757-1075-466f-a3cf-b6696d2211c9
- **Pipeline doc:** [696-the-marley-and-rusty-agents-out-of-the-box.spec.md](../../pipeline/completed/696-the-marley-and-rusty-agents-out-of-the-box.spec.md)
- **Source ticket:** Chad, 2026-10-08: "marley should come pre configured out of the box … Marley
  is the agent behind the ide, rusty is the agent behind rusty enabled so both will be in the agent
  panel"
- **Status:** closed

## Summary
Chad's answer to the manager plan's decision 3 reverses "offer it once".

**The Marley agent** (#683, #687) is on by default.
- `marley.assistant.agent` gains `auto`, the new default: at start Marley picks Claude Code when
  `claude auth status` says signed in, else Codex when `codex login status` does, else Zed's own
  agent. An agent the user names wins.
- The offer goes, since nothing is left to offer. The switch stays, so off still leaves no trace
  (AD-661).
- The e2e runner's copy keeps it off, so no scenario meets the entry.

**A Rusty entry** joins it while `marley.rusty.enabled` is on. It is built as the Marley agent is:
the same resolved agent (Claude Code or Codex through its ACP adapter, or a `rusty` profile for
Zed's agent), with Rusty's instructions and Rusty's MCP tools (the `rusty` context server). It has
the Marley agent's limits: no file edits and no commands. Rusty works through its tools.

## Acceptance
With no `marley.assistant` in the user's settings, the Agent Panel lists Marley, running on the
first agent found. With Rusty on, it also lists Rusty. Turning either switch off removes its entry.
