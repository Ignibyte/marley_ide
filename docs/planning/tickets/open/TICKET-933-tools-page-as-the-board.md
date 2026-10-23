# TICKET-933 — MCP Servers and Agent Activity as the board draws them

- **Ticket:** LOCAL #933 (feature, the Marley look, MCP Servers and Agent Activity)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [933-tools-page-as-the-board.spec.md](../../pipeline/queued/933-tools-page-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/F-tools.md, board Tools)
- **Boards:** Tools
- **Size:** M
- **Status:** open

## Summary

MCP Servers has no subtitle. Each agent's card puts its name, version and path on one line with
no tile and no state, and lists its integrations as long rows with the "tested on" text inline.
Marley's own server is one row in a "ZED" card, with no tool count and no word on where its
calls go. Agent Activity draws each call as a table line ending in "done" or "refused: code", and
a call's result or error is never recorded. The Tools board draws a subtitle; agent cards with a
lettered tile, the name over its path and a ready or missing pill; integrations as chips with a
dot; Marley's own card with its tool count and a running pill; and activity rows with the
agent's tile, the tool and time, what it acted on, and how it ended. This ticket brings MCP
Servers and Agent Activity in line with the board, and records each call's result or error.

## Acceptance

MCP Servers shows the subtitle, the agent cards and Marley's card as the board draws them, and
Agent Activity lists each call with its agent's tile and its result or error; shot in Marley
Night and Marley Day.
