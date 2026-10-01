# TICKET-534 — Harness sessions in the rail (the read side)

- **Ticket:** LOCAL #534 (feature, prong 2, C1 with C3's observer view; deliberate until the harness's M9 exits)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/534-harness-sessions-in-the-rail.spec.md
- **Source ticket:** Chad, 2026-09-25, as the night's brief records his decision: rustal-harness will be embedded in Marley and also run standalone, and its `rh mcp` serves Marley's fleet contract. The plan's prong 2 names the work: C1, "`marley_harness` read side: subscribe, snapshot, events into `marley_fleet`; a fleet rail panel with state chips, question cards and staleness", and D10, a harness session shown in a display-only Zed terminal (`docs/marley/three-prong-plan.md`). The harness's side is its M9 (`/srv/stacks/rustal-harness/docs/ROADMAP.md`, `docs/MCP.md`, `docs/FLEET.md`).
- **Status:** closed

## Summary
The harness keeps agent sessions alive and publishes each one's state in Marley's own envelope, over `rh mcp`; Marley has the envelope's types and nothing that reads a live feed. Marley runs the command a new `marley.harness` setting names (locally `rh --state <root> mcp`, for a harness on another host `ssh <host> rh ... mcp`) through Zed's own MCP client, seeds Marley's `FleetSnapshot` from `fleet_snapshot`, and folds each published change from `fleet_events` with Marley's own reducer. A Harness group in the rail lists every session with its state and, while one waits, its question; a working session that has gone silent says for how long; opening a session shows its recent output, read with `session_read`, in a display-only terminal tab; and a waiting session's question joins the approvals inbox (#508). Read side only: nothing is answered, sent, opened or stopped from Marley yet. The ticket waits, as a deliberate row, until the harness's M9 exits.

## Acceptance
With `marley.harness` set, the rail's Harness group lists the harness's sessions with their states and questions and follows every change within seconds; a session's row opens a read-only view of its output; a waiting question shows in the approvals inbox; a harness that is down or unset reads as such.
