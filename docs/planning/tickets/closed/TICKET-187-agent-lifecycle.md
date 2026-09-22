---
id: forge#187 (ec3f3366-5130-4463-9157-eead7052982f)
title: M12 — agent lifecycle: exit code + duration + finish flash
status: closed
milestone: M12 — The Agent Cockpit
pipeline: 6471e72b-23b6-42fb-8ab1-863be04a95c1
---

When an agent's shell exits, its Agents-tab row stays — marked ✓ (exit 0) or ✕ (non-zero) with the run
duration — and a one-shot "{label} finished (exit N, 2m14s)" flash fires. AgentRun gains run_ticks (lifetime)
+ exit_code; the pump captures ChildExited's code and marks the agent Exited (keeping the row); the row
surface (fmt_duration, agent_finish_flash, the ✓/✕ glyph + duration) is pure.
