# TICKET-864 — The thread's plan, reply chips, terminal peek and checks

- **Ticket:** LOCAL #864 (feature, the Marley look, W2 follow-up)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** split from TICKET-825 while planning it (2026-10-21): the Main board's agent
  answer carries a plan checklist and reply chips, and its details panel a live peek of the
  thread's terminal and its checks, which #825 leaves out.
- **Boards:** Main, ThreadStates, MainDay
- **Size:** M
- **Status:** open

## Summary

In the agent's answer: the thread's plan (`AcpThread::plan`) as a checklist where the answer is,
framed EDIT and RUN labels on the existing edit and terminal cards, and reply chips under a
finished answer (Continue, Explain, Undo the edits) that put their text in the composer. In the
details panel #825 adds: a live peek of the thread's terminal (the last lines of its newest
terminal tool call) and its checks (the diagnostics of the files it changed, and a task run's
result when the thread ran one).

## Acceptance

The plan, the card labels and the chips show in a thread tab as the board shows; the panel's
terminal peek follows the running command and its checks list the changed files' errors. Shot in
Marley Night and Marley Day.
