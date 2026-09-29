# TICKET-533 — The harness's contract requests answered, and the plan's harness text corrected

- **Ticket:** LOCAL #533 (chore, prong 2, the fleet contract before C1)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/533-harness-contract-alignment.spec.md
- **Source ticket:** Chad, 2026-09-25, as the night's brief records his decision: rustal-harness will be embedded in Marley and also run standalone, and its `rh mcp` serves Marley's fleet contract. The same day the harness filed two requests to Marley from its conformance work (TICKET-053, widened after TICKET-055): MREQ-001, tool names a Claude client can call, and MREQ-002, retry keys on `SendRequest` and `OpenRequest`, returned in each receipt (`/srv/stacks/rustal-harness/docs/planning/MARLEY_REQUESTS.md`).
- **Status:** closed

## Summary
The harness now serves Marley's fleet contract over `rh mcp` and checks it with Marley's own `marley_fleet` crate, and it asks two things of Marley. MREQ-001: Marley's server already serves underscore names (`fleet_snapshot`, `session_surface_to_human`) since #491; the dotted names the harness saw come from the gpui-era repository it pinned (`/srv/stacks/marley` at `0ba2872d8f`), and from `marley_fleet`'s verb docs and Marley's plan, which still spell verbs with dots; those change. MREQ-002: `SendRequest` gains an optional `delivery` id and `OpenRequest` an optional `request` id, each left out of the JSON when absent, and `marley_fleet` gains `SendReceipt` and `OpenReceipt`, the receipt values that return them. The plan says the harness has been paused since 2026-09-14; it resumed on 2026-09-22 and is deep in M9, and the plan gains the decision that the harness is embedded in Marley, run by Marley as its own process over the same protocol a standalone harness speaks on another host, and also runs standalone.

## Acceptance
Every tool name Marley lists is one a Claude client can call; the verb docs and the plan use the wire names; `SendRequest` and `OpenRequest` carry their optional retry ids and `SendReceipt` and `OpenReceipt` return them; the plan states the harness's real status and the embedded-and-standalone decision, and records both requests with their answers.
