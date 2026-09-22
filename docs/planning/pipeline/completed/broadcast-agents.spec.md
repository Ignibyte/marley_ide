---
pipeline_id: 263ea049-e62d-4c4e-b2fe-75f0ea1c9edc
ticket: forge#73 (a466ec24-4271-409f-b932-9f380753edb5) · local docs/planning/tickets/open/TICKET-073-broadcast-agents.md
aar_id: 3f9e50e2-fc4c-4d1d-9a9e-1714a5d65d9d
status: Phase 5 — Complete PASS
title: broadcast a prompt to all agents
type: feature
milestone: M2.D
references:
  - crates/marley_app/src/agent_view.rs (PURE: agent_pane_ids)
  - crates/marley_app/src/keymap.rs (cmd-shift-g → broadcast-to-agents)
  - crates/marley_app/src/app.rs (SHIM: the broadcast dispatch)
---

## Title
cmd-shift-g sends the focused pane's composed line to EVERY running agent — the fleet counterpart of
#72's send-to-one ("all of you: run the tests"). Clears the prompt only on confirmed delivery.

## Scope
### In
- PURE (`agent_view.rs`, cov/MSI 100): `agent_pane_ids(agents: &HashMap<PaneId, AgentRun>) -> Vec<PaneId>`
  = the pane ids sorted by `id.0` (deterministic broadcast targets; mirrors `agent_rows` #68).
- PURE (`keymap.rs`, cov/MSI 100): cmd-shift-g → `broadcast-to-agents` + a keymap test.
- SHIM (`app.rs`, mutants::skip + cov-excluded): the `broadcast-to-agents` dispatch — `send_payload(line)`
  (reused #72) to each `agent_pane_ids` pane's session; clear the focused prompt only if ≥1 delivered.

### Out
- New payload logic (reuses #72's `send_payload`). Targeting a subset of agents (all-or-none). A compose
  overlay (composes at the prompt, like #72).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `agent_pane_ids` gives a deterministic sorted target list (pure, testable), vs iterating the
  HashMap directly (nondeterministic, untestable).
- D2 — clear the prompt ONLY on confirmed delivery to ≥1 agent (the #72 F1 lesson /
  PR-claude-clear-input-only-after-confirmed-delivery-001) — never drop the line if there are no agents.
- D3 — cmd-shift-g (free; "g" = the group/fleet). Reuses #72's `send_payload` (line + `\r`).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `agent_pane_ids(agents)` is called, it shall return the pane ids sorted ascending by `id.0`. | unit (3-scrambled fixture) |
| REQ-002 | WHEN the keymap is queried, cmd-shift-g shall map to `broadcast-to-agents`. | unit |
| REQ-003 (visual) | WHEN a line is composed and cmd-shift-g is pressed, EVERY running agent shall receive it and the prompt shall clear. | self-test (2 agents) |
| REQ-004 | `scripts/gates.sh` GREEN, cov/MSI 100 on agent_pane_ids + keymap; app shim excluded. | gate |

## Phase Plan
- **P2** — `agent_pane_ids` + keymap + the broadcast dispatch, mutation targets, test plan.
- **P3** — agent_pane_ids + keymap + the app.rs shim.
- **P3.5** — critic: agent_pane_ids sort (MSI via 3-scrambled), keymap non-conflict, the deliver-to-all +
  clear-on-≥1 logic, borrow-safety, reuse of send_payload.
- **P4** — agent_pane_ids + keymap unit tests (cov/MSI 100) + the SELF-TEST (2 agents both receive +
  prompt clears) + gate GREEN.
- **P5** — docs, AAR, archive, close #73.
