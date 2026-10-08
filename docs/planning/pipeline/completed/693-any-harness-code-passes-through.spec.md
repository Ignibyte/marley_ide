---
pipeline_id: c47ad7ce-b36b-4a4d-ade2-8867e60433aa
ticket: docs/planning/tickets/closed/TICKET-693-any-harness-code-passes-through.md
status: Phase 4 — Complete PASS
title: Any harness code passes through
type: bug
slice: phase 2 item 6 of docs/planning/intake/marley-agent-manager-foreman.md (follow-up)
references:
  - docs/planning/pipeline/completed/692-the-marley-agent-sets-up-a-seat.spec.md
---

## Title
Marley passes any harness refusal code through by name, not only the twelve it listed.

## Scope
### In
- `marley_mcp::Refusal.code` becomes `Cow<'static, str>`, and `Refusal::new` takes
  `impl Into<Cow<'static, str>>`. Every literal call keeps compiling, and a refusal can carry a
  code it learned at run time.
- `harness_seat::refusal_of` takes the token between `rh: ` and the next `:` as the code when
  it is `[a-z_]+`, and the rest as the reason. Otherwise it is `refused` with the text as
  given. `HARNESS_CODES` goes.
- `run_seat`: exit 2 reads "Marley called the harness wrongly (a usage error): <first line>",
  for the form and `seat_add` alike.

### Out
- Nothing else.

## Reference (§20)
N/A — Marley-specific: the harness's own stderr form (its message of 2026-10-07).

### Prior art
#692's `refusal_of`. `Refusal`'s single serializer, `tools::tool_refusal`.

## UI proof
N/A — no UI delta: the code an agent reads in a refusal's JSON, and the same text in the form.
Its run is `just shot`, plus #692's scenario rerun for `seat_role_reserved`, which must still
pass by name.

## Locked-In Decisions
- **D1:** an owned code in `Refusal`, not a leak and not a list. The type is Marley's own crate.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the harness refuses with `rh: CODE: reason` and CODE is `[a-z_]+`, the answer's code shall be CODE | review of `refusal_of`; #692's scenario still gets `seat_role_reserved` |
| REQ-002 | WHEN the harness says `rh: text` with no code, the answer shall be `refused` with that text | review |
| REQ-003 | WHEN a seat command exits 2, the reason shall say Marley called it wrongly | review |

## Phase Plan
- **P1 Plan:** this spec.
- **P2 Code:** the `Refusal` code, `refusal_of` and `run_seat`, and the gate.
- **P3 Test:** `just shot`, and #692's scenario.
- **P4 Complete:** docs, ledger, close, archive, commit, push and install.
