---
pipeline_id: a8546eb5-053b-4a9f-b246-5c16a7aacc53
ticket: forge#67 (a12d6977-0e9a-4402-84fb-8f0509caa635) · local docs/planning/tickets/open/TICKET-067-agent-status-live.md
aar_id: 277e763d-5448-4f71-be43-0cebdbe9a70d
status: Phase 5 — Complete PASS
title: drive AgentStatus live from the pane state
type: feature
milestone: M2.C
references:
  - crates/marley_app/src/app.rs (SHIM: refresh_agent_statuses on the pump tick + the agents-leak fix)
---

## Title
Make the #66 agent badge LIVE: each pump tick recomputes an agent pane's status from its session
(`agent_status_from`, #61) — ● while the agent runs a command, ○ at its prompt. The first OBSERVE step.

## Scope
### In (all SHIM, `app.rs`, mutants::skip + cov-excluded — NO new pure surface)
- `refresh_agent_statuses(&mut self)` — for each `self.agents` pane still present, set
  `run.status = agent_status_from(false, is_command_running())`; borrow-safe (collect then write).
- Call it from the live-pump closure each tick.
- LEAK FIX: the pump's dead-pane auto-close (`workspace.close(id)`) also `self.agents.remove(&id)` (the
  #62 close-pane dispatch removes it; the pump path was missed).

### Out
- Detecting the AGENT process (claude) specifically exiting back to the shell (that's Idle here — the
  shell lives). WaitingInput. A distinct ✓ display for a dead-but-kept last pane (dead panes are removed).
  Any new pure fn (the decision is the tested `agent_status_from`).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — the status DECISION stays the pure `marley_agent::agent_status_from` (#61, cov/MSI 100). #67 is
  pure-reuse + masked wiring; NO new unit tests (noted in the test plan).
- D2 — signals: `active` = `session.is_command_running()` (#40); `exited` = false for a live pane (a dead
  pane is removed, not shown Exited). So the live signal is ●(Working) ↔ ○(Idle).
- D3 — all new logic in mutants::skip fns (refresh_agent_statuses skip'd; the pump closure masked) so the
  mutation gate has nothing to flag; app.rs is cov-excluded → coverage unaffected.
- D4 — the leak fix (remove agents entry on pump-close) completes #62's tag lifecycle.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 (visual) | WHEN an agent pane is running a command, its badge shall show ● (Working); at the bare prompt, ○ (Idle). | self-test (cmd-shift-a → ● while claude runs) |
| REQ-002 | WHEN an agent pane is auto-closed on child-exit, its `agents` entry shall be removed (no lingering tag). | code review + no-regression tests |
| REQ-003 | `scripts/gates.sh` GREEN; the app shim is excluded (no new pure lines); the existing `agent_status_from` tests stay green. | gate |

## Phase Plan
- **P2** — the refresh method + the pump-tick call + the leak fix, the borrow-safe pattern, the test plan
  (reuse #61 + self-test).
- **P3** — the app.rs wiring.
- **P3.5** — critic: the borrow safety, the Working/Idle mapping, the leak fix, all-logic-in-skip'd-fns.
- **P4** — `cargo nextest` (no regression; #61's agent_status_from tests) + the SELF-TEST (○→● live) + gate GREEN.
- **P5** — docs, AAR, archive, close #67.
