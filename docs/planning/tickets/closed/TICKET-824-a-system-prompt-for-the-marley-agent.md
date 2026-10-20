# TICKET-824 — A system prompt for the Marley agent

- **Ticket:** LOCAL #824 (feature, Marley's own agent #683, Rusty's #696)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** ../../pipeline/completed/824-a-system-prompt-for-the-marley-agent.spec.md
- **Source ticket:** Chad, 2026-10-20: "also mayb we should build a good system prompt for
  marley", then, about the look: "we would design the agent profile and things to be more chat gpt
  like". Asked whether he meant Marley's own agent, the instructions every agent gets, or both, he
  did not answer; under his /goal "finish the remaining tickets we discussed" the default was taken:
  the agent profile Marley owns, the Marley agent's prompt, with Rusty's sharing its answer style.
  The MCP server's tool guide every agent gets stays as it is.
- **Status:** closed

## Summary
The Marley agent's instructions become a full system prompt: who it is and what Marley is, what it
does and with which tools, how it answers (the answer first in plain words, short, one question at
a time, no filler, saying what it does not know), and what it leaves to others. Rusty's gets the
same answering section.

## Acceptance
A new Marley thread's `session/new` carries the new prompt, its answering section included; a
Rusty thread's carries Rusty's with the same section.
