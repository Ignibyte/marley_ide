---
pipeline_id: e1be7458-4534-4b16-a50b-3e0e2af864dc
ticket: docs/planning/tickets/closed/TICKET-694-the-manager-in-the-agent-panel.md
status: Phase 4 — Complete PASS
title: The manager in the Agent Panel
type: feature
slice: phase 2 item 7 of docs/planning/intake/marley-agent-manager-foreman.md
references:
  - docs/planning/pipeline/completed/691-a-harness-seat-in-one-step.spec.md
  - docs/planning/pipeline/completed/683-the-marley-agent-in-the-agent-panel.spec.md
---

## Title
A **Manager** entry in the Agent Panel, running the harness's `rh acp`, while the followed harness
has a manager.

## Scope
### In
- `harness::sync_manager_entry`, run when the fleet or the settings change:
  - while `writes_on` and some session's labels hold `role: manager`, it puts a custom
    `Manager` agent server in the settings' in-memory defaults (#683's way), with the
    `Harness::seat_command` program and its arguments plus `acp`;
  - otherwise it takes the entry out.
- No env and no auth. `rh acp` is the person's side of the thread.

### Out (explicitly deferred)
- The report notice (TICKET-688).
- A manager on another harness than the followed one.

## Reference (§20)
N/A — Marley-specific. The ACP side is the harness's (its docs/ACP.md), and the panel side is
Zed's `agent_servers` custom entry, as #683 adds Marley's.

### Prior art
#683's custom entry in the in-memory defaults, and #691's `seat_command`. The harness's ACP.md
gives the registration (`rh --state ROOT acp`, or `HOST /path/to/rh --state ROOT acp`), and its
MANAGER.md gives the `role` label, which follows the designation.

## UI proof
`script/e2e/694-the-manager-in-the-agent-panel.sh`: the harness's built `rh`, an actor `boss`
designated with `rh manager`, and the actor waiting for a message.
- `694-01-menu`: New Agent Thread's submenu lists Manager.
- `694-02-thread`: the Manager thread after "hello manager". `rh thread show` holds the
  person's message, and the actor took it.

## Locked-In Decisions
- **D1:** the entry follows the role label, so it moves with the designation, and with no manager
  there is no entry.
- **D2:** gated by `marley.harness_writes`, since the thread writes as the person.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the write verbs are on and the fleet has a session labelled `role: manager`, the Agent Panel shall list Manager | shot `694-01-menu` |
| REQ-002 | WHEN a message is sent in the Manager thread, the system shall run `rh acp` through the followed command, and the message shall reach the thread | shot `694-02-thread` and a check of `rh thread show` |
| REQ-003 | WHILE no session is the manager, or the switch is off, the system shall list no Manager entry | review of the diff |

## Phase Plan
- **P1 Plan:** this spec.
- **P2 Code:** `harness.rs`, a review, and the gate.
- **P3 Test:** the scenario.
- **P4 Complete:** docs, ledger, close, archive, commit, push and install.
