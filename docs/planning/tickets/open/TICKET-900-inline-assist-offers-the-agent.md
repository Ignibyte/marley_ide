# TICKET-900 — Inline assist offers to send the selection to an agent

- **Ticket:** LOCAL #900 (feature, the Marley look, W6)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** (not planned yet)
- **Source ticket:** split from TICKET-899 while planning it (2026-10-22): a run has no LLM
  provider, so inline assist shows only "Configure at least one LLM provider" and its prompt
  cannot be shot.
- **Boards:** EditorPopups
- **Size:** S
- **Status:** open

## Summary

Under inline assist's prompt, a line: "Or send lines 20–22 to Claude Code in “Fix the rail
test” · marley: send selection to agent", the agent a terminal in the window runs, as the
EditorPopups board draws it. Needs a stand-in LLM provider in the scenario before it can be seen.

## Acceptance

The line shows under the prompt and names the lines and the agent; shot in Marley Night and
Marley Day.
