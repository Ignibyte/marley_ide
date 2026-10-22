# TICKET-901 — The bottom row while debugging, and Ask an agent about a language server

- **Ticket:** LOCAL #901 (feature, the Marley look, W1.4 follow-up)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [901-the-debugging-row-and-ask-an-agent-about-a-server.spec.md](../../pipeline/completed/901-the-debugging-row-and-ask-an-agent-about-a-server.spec.md)
- **Source ticket:** split from TICKET-863 while planning it (2026-10-22): both are new behaviour;
  the debugging row needs a debug adapter in the scenario's run.
- **Boards:** BottomRow
- **Size:** M
- **Status:** closed

## Summary

While a debug session is paused, the bottom row shows "Paused at main.rs:4" with Continue, Step
over, Step in, Step out and Stop, the selected frame and the breakpoint count. And the language
servers card gains "Ask an agent", which sends a failed server's state and log line to an agent.

## Acceptance

Each works and looks as the board shows; shot in Marley Night and Marley Day.
