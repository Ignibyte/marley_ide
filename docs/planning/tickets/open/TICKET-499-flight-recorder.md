# TICKET-499 — B5: Record what already happened in the Browser tab

- **Ticket:** LOCAL #499 (feature, prong 3 B5; prong 3 wave 2)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/499-flight-recorder.spec.md
- **Source ticket:** docs/marley/three-prong-plan.md, prong 3 B5; docs/marley/browser-handoff.md
- **Status:** open

## Summary
Pillar C, the flight recorder. Each page keeps a rolling minute of what happened in it: the input
Marley sent, frames, console messages, requests and navigations, with an accessibility snapshot
at each navigation. "Record this" saves that minute, so a bug Chad just saw is already captured,
and the agent lists and reads recordings through Marley's MCP server. Nothing typed, no cookie,
header or body, and no secret-looking URL value is ever recorded.

## Acceptance
Record this saves the last minute of the page; an agent reads the recording's timeline and frames
through MCP; the recording holds no typed text and no secrets.
