---
pipeline_id: 6ef0e346-1cad-4419-96cb-5b2879ae4692
ticket: docs/planning/tickets/closed/TICKET-692-the-marley-agent-sets-up-a-seat.md
status: Phase 4 — Complete PASS
title: The Marley agent sets up a harness seat
type: feature
slice: phase 2 item 6 of docs/planning/intake/marley-agent-manager-foreman.md
references:
  - docs/planning/pipeline/completed/691-a-harness-seat-in-one-step.spec.md
  - docs/planning/pipeline/completed/686-keymap-changes-accepted.spec.md
---

## Title
`seat_add`: an agent proposes a harness seat, the user accepts it, and Marley adds it and starts
it.

## Scope
### In
- **`marley_mcp`:** `Family::Seat` (served), with `seat_add` (write, grant class
  `harness.write`) and its schemas. The server's `INSTRUCTIONS` name it.
- **`mcp.rs`:** grants `harness.write` and routes `seat_add`.
- **`harness_seat::answer_seat_add`:**
  - Refuses `tool_off` unless `writes_on`, `bad_argument` for a missing name, agent or
    folder, and `unavailable` without a seat command.
  - Asks through `settings_change::ask_user`: "<agent> wants to set up a harness seat", with
    the name, agent, folder and role, and the command it runs through.
  - On Apply, runs `seat add`, and answers `{result: "starting", seat, profile, kind}` or the
    harness's refusal by its code.
  - Then runs `seat start`. A failure shows as an app notification.
- **`settings_change`:** the question and its wait move into `ask_user`, which
  `ask_then_write` calls.
- **The Marley agent:** the instructions name `seat_add`, and the Zed profile turns it on.

### Out (explicitly deferred)
- Choosing the harness host (as #691).

## Reference (§20)
N/A — Marley-specific. It is #686's question over #691's commands.

### Prior art
#682 and #686's `ask_then_write` and question card. #691's `run_seat` and `seat_command`.

## UI proof
`script/e2e/692-the-marley-agent-sets-up-a-seat.sh`. It has #691's stand-in `rh`, with the
stand-in MCP client calling `seat_add`.
- `692-01-card`: the question.
- `692-02-started`: the rail lists the seat after Apply.

A refusal comes back with `seat_role_reserved`, checked from the answer.

## Locked-In Decisions
- **D1:** answer after `seat add` and start in the background. The app call's 30 seconds cannot
  hold the user's 25 and a Claude start's 60.
- **D2:** grant class `harness.write`, granted as the other write classes are. The user's Apply
  and `marley.harness_writes` are the checks.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent calls `seat_add` with the write verbs on, the system shall ask the user with the seat's name, agent, folder and role | shot `692-01-card` |
| REQ-002 | WHEN the user applies, the system shall run `seat add`, answer `starting`, then run `seat start` | the answer, the stand-in's log, and shot `692-02-started` |
| REQ-003 | WHEN the harness refuses, the answer shall carry its code | a check of the answer |
| REQ-004 | WHILE `marley.harness_writes` is off, `seat_add` shall be refused with `tool_off` | review of the diff |

## Phase Plan
- **P1 Plan:** this spec.
- **P2 Code:** the registry, `mcp.rs`, `harness_seat.rs`, `settings_change.rs` and
  `assistant.rs`, then the gate.
- **P3 Test:** the scenario.
- **P4 Complete:** docs, ledger, close, archive, commit, push and install.
