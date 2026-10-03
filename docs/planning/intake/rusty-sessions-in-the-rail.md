---
status: superseded
created: 2026-10-01
ticket: <unassigned>
pipeline_spec: <unassigned>
---

# Rusty's agent sessions in the rail (C2's sessions half)

> Superseded 2026-10-02: Rusty's agent host retires once rustal-harness runs Claude Code in its
> own interface (Chad: "Retire it"; `docs/marley/rusty-in-marley.md`, open decision 2). Its
> sessions become harness seats or Zed threads, which the rail already shows.

## What
Rusty's agent sessions (`rusty agent`) listed in Marley's rail beside the harness's, read side:
their title, their state, the permission one waits on, and a tab of the session's transcript.

## Why
Plan C2 and D11: Marley embeds Rusty's sessions in the fleet rail, with Rusty staying Rusty.

## Notes
An Explore read of Rusty (2026-10-01): Rusty's sessions are transient user units with a per-session
registry file and a versioned NDJSON socket (`PROTOCOL = 1`: hello, attach, caught_up, claude,
sent, host, status), but rusty-mcp serves no session or fleet tool, and Rusty's constitution says
"The back end is MCP only … any agent reach the store through `rusty-mcp`'s tools." Reading
Rusty's registry and socket from Marley would go around that, and neither is called a public
contract. The clean path is a Rusty ticket adding `fleet_snapshot` and `fleet_events` (Marley's
`marley_fleet` envelope: Rusty's `starting`, `working`, `idle` and `waiting` map one to one,
`sleeping` reads as idle, `stopped` as done; the waiting permission's tool as the question) to
rusty-mcp, after which Marley follows it as #534 follows the harness, with a setting that can name
more than one MCP fleet. A scenario can then run on a stand-in rusty-mcp, never the user's Rusty.

## Promotion
This is NOT an active pipeline doc — it is a candidate. Promote it via
`/pipeline:plan` when ready: it becomes a ticket (`docs/planning/tickets/open/`) + an active
pipeline doc pair (`docs/planning/pipeline/active/`). On promotion, set
`status: promoted` and fill `ticket:` + `pipeline_spec:`. It waits on the Rusty ticket above, which
is Rusty's repository's to schedule.
