# TICKET-519 — Claude Code's hook events in the rail

- **Ticket:** LOCAL #519 (feature, prong 2, C1's first piece: a terminal's own Claude Code events into `marley_fleet` and the rail)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/519-claude-code-events-in-the-rail.spec.md
- **Source ticket:** Chad, 2026-09-25, on the Orca survey: "we will be taking what it does well and bring it in here". The survey ranks hook-driven agent state first and names it as the ticket that lands before #508 and #509 (`docs/orca_architecture/README.md`, "What Orca does well that Marley lacks, first", item 1, and "New tickets these imply"; reports 01 item 1, 05 item 1, 06 item 3).
- **Status:** closed (2026-09-26, slice 1; the update chip, `fleet_snapshot` and `no update in N m` are TICKET-547)

## Summary
The rail guesses what a terminal's Claude Code is doing from 2 s of quiet, so it cannot tell a long think from a permission prompt and shows nothing of the work. Marley's plugin registers Claude Code's lifecycle hooks and answers each with an OSC 777 under the reserved title `marley-event`, carrying a short base64 JSON summary that Claude Code writes to its own terminal. Marley reads it per terminal, keeps the state in `marley_fleet`, and the agent's row shows working, waiting, idle or failed with the prompt, the tool in flight and the last message. `fleet_snapshot` lists the same seats, and the approvals inbox (#508) and per-turn diffs (#509) read these events.

## Acceptance
With the plugin, a Claude Code row in the rail follows the session's hooks: the prompt and the tool while it works, what it asks while it waits, the last message when the turn ends, `failed` on an API error, and `no update in N m` once a working row has gone 30 minutes without an event; the frames post no notification and ring no bell; `fleet_snapshot` lists the seat.
