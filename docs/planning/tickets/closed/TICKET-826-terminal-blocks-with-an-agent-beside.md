# TICKET-826 — Terminal blocks with the agent beside them

- **Ticket:** LOCAL #826 (feature, the Marley layout's look)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** [826-terminal-blocks-with-an-agent-beside.spec.md](../../pipeline/completed/826-terminal-blocks-with-an-agent-beside.spec.md)
- **Source ticket:** Chad, 2026-10-20: "the idea is i think we would design the agent profile and
  things to be more chat gpt like. having panels around the conversations and stuff like that.
  lets go ahead and mock it up. and i feel like warp like". The mockup is the design canvas
  https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn; this ticket is one of its boards. Chad filed the whole plan on 2026-10-20.
- **Boards:** Terminal, TerminalSession, TerminalMore, AgentTerminal
- **Size:** L
- **Status:** closed (queued in wave 2 of [the Marley look plan](../../../marley/look-plan.md), 2026-10-20)

## Summary
Board 2, "Terminal blocks with the agent beside them": each command a framed block with its state color and time; a failed block red with Ask an agent to fix it, Explain, Rerun and Copy output; a running block with a progress line; the prompt bar with folder and branch chips; an agent panel on the right that reads a block and proposes a fix to apply.

## Acceptance
A terminal tab whose blocks and prompt bar look as the board shows, and the agent panel opened from a failed block. Drawn in Marley Night and Marley Day once #828 lands.
