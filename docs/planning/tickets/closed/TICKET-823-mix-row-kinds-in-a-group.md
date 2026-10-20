# TICKET-823 — Any row goes anywhere in its group

- **Ticket:** LOCAL #823 (feature, the Marley layout's rail; #602 drag, #818 thread moves)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** ../../pipeline/completed/823-mix-row-kinds-in-a-group.spec.md
- **Source ticket:** Chad, 2026-10-20: "you can reorder items inside of the groups. You can move
  groups but not reorder there". Told that a row reorders only among rows of its own kind
  (`DraggedRailRow::takes` needs the same `run`: terminals, then Browser tabs, then threads,
  then ports), he picked "mix any order": a thread above a terminal, a Browser tab between two
  agents.
- **Status:** closed

## Summary
A group's rows stop being sections by kind: a terminal, a Browser tab, a thread or a tab drags
to any place among its group's rows, and the rail keeps that order as it keeps #602's.

## Acceptance
In a group with a terminal agent and a thread, the thread's row dropped above the terminal's
stays there, after a restart too.
