---
pipeline_id: d9613137-90d7-4d6f-bcec-39771cbeaf98
ticket: docs/planning/tickets/open/TICKET-710-the-marley-agent-removes-a-seat.md
status: Phase 4 — Complete PASS
title: The Marley agent stops and removes a harness seat
type: feature
slice: #692's counterpart, on harness TICKET-114
references:
  - docs/planning/pipeline/completed/692-the-marley-agent-sets-up-a-seat.spec.md
---

## Title
`seat_stop` and `seat_remove` on Marley's MCP server: an agent proposes to stop or remove a
harness seat, the user applies, and Marley runs the harness's own command.

## Scope
### In
- **`seat_stop`** (Write, `harness.write`), with `{"name"}`: asks the user as `seat_add` does,
  then runs `seat stop NAME` through `Harness::seat_command`. It answers the harness's object:
  `stopped`, `supervision_ended`, and `result: "stopped"`.
- **`seat_remove`**: the same with `seat remove NAME`, which also answers `removed` (the deleted
  profile's path) and `result: "removed"`.
- **Refusals:** each one comes back by the harness's own code (`seat_name`, `seat_unknown`,
  `seat_runtime`, `seat_stop_failed`, `seat_profile`), through `refusal_of`.
  `marley.harness_writes` off is `tool_off`.
- **The Marley agent** gains both tools (`PROFILE_TOOLS` becomes ten), and its instructions name
  them.
- **Descriptions** in Strict STE with `rustal-ste`'s verbs: "stop" a seat, "remove" a seat.

### Out (explicitly deferred)
- A rail menu item for a seat.
- The harness's own MCP `seat_stop`, which a harness client calls directly.

## Reference (§20)
N/A — Marley-specific. Marley calls rustal-harness's `rh seat stop` and `rh seat remove` (harness
TICKET-114, `22d3157`; rustal-harness `docs/AGENT_SEATS.md#stopping-and-removing-a-seat`, D187).

### Prior art
- #692's `answer_seat_add`: the question, `run_seat` and `refusal_of`.
- #691's `seat_command`.
- harness TICKET-114's JSON and refusal codes.

## UI proof
`script/e2e/710-the-marley-agent-removes-a-seat.sh`, under `compositor sway`, on #692's set-up: a
real runtime and a stand-in `rh` whose `seat stop` and `seat remove` answer harness TICKET-114's
documented JSON and record each call.

Shots:
- `710-01-stop-asked`: `seat_stop builder`, the question asking to stop the seat;
- `710-02-remove-asked`: `seat_remove builder`, the question naming the profile's removal.

The client's replies show:
- after Apply, `"result": "stopped"` with `stopped` and `supervision_ended`;
- `"result": "removed"` with `removed`;
- the stand-in's log shows `seat stop builder`, then `seat remove builder`;
- `seat_stop ghost` comes back as `seat_unknown`.

## Locked-In Decisions
- **D1:** two tools, one per harness command, not a flag: stopping keeps the profile, removing
  deletes it, and the question says which.
- **D2:** every stop or removal waits for the user's Apply, as every seat change does.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent calls `seat_stop` or `seat_remove`, the system shall ask the user, naming the seat and whether its profile is deleted. | Shots 710-01, 710-02 |
| REQ-002 | WHEN the user applies, the system shall run the harness's `seat stop` or `seat remove` and answer its object, with `result`. | The client's replies and the stand-in's log |
| REQ-003 | WHEN the harness refuses, the system shall answer the refusal by the harness's code. | The client's reply (`seat_unknown`) |
| REQ-004 | The Marley agent shall have both tools. | `PROFILE_TOOLS` and the instructions, in the review |
| REQ-005 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan.**
- **P2 Code:** the registry, `harness_seat.rs`, `mcp.rs`, `assistant.rs`, the guide; gate.
- **P3 Test:** the scenario.
- **P4 Complete.**
