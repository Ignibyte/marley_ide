---
pipeline_id: 8c7a8149-f7ac-416e-b860-3e087e41d3fc
ticket: docs/planning/tickets/closed/TICKET-688-a-notice-for-the-managers-reports.md
status: Phase 4 — Complete PASS
title: A notice for the manager's reports
type: feature
slice: phase 2 of docs/planning/intake/marley-agent-manager-foreman.md (TICKET-688)
references:
  - docs/planning/pipeline/completed/694-the-manager-in-the-agent-panel.spec.md
  - docs/planning/pipeline/completed/685-acp-messages-between-turns.spec.md
---

## Title
Marley follows the manager thread. A message, report or confirmation from the manager, while the
Manager thread is not in front, raises a desktop notice and a "Needs you" entry until you look.

## Scope
### In
- **Following.** `harness.rs` reads `thread_read {after}` on the connection Marley already holds,
  each second while the fleet has a manager and the write verbs are on.
  - The first read pages to the end and raises nothing, so old records stay quiet.
  - After that, each new `thread_record` whose author is `manager` is unread.
- **In front.** The active window's workspace shows the Agent Panel with the Manager agent
  selected. A record that arrives then raises nothing, and the unread clear once it is in front.
- **The notice.** A desktop notification (`show_system_notification`), "Manager: Report" (or
  Message, or Confirmation), with the record's first line.
- **The mark.** A "Needs you" inbox entry per unread record: Manager, the kind and the first
  line. A click opens the newest Manager thread the rail lists, or a new one.

### Out (explicitly deferred)
- Answering a confirmation from the inbox. That happens in the thread, as #694 shows.
- A countdown on a confirmation's `expires_ms`.

## Reference (§20)
N/A — Marley-specific. The harness leaves the notice to Marley (its MREQ-008, rewritten). The
inbox and the desktop notice follow #508 and #538.

### Prior art
- `notifications.rs`'s `show_system_notification` (#538) and `rail.rs`'s `harness_entries`
  and `open_thread` (#534, #508).
- `AgentPanel::is_visible` and `selected_agent`.
- The harness's `thread_read` page `{events: [{sequence, kind, change}], next_cursor}`, probed
  2026-10-08.

## UI proof
`script/e2e/688-a-notice-for-the-managers-reports.sh`. Its manager is a harness terminal running a
fixture script: it reports idle, is designated with `rh manager`, and posts a report through
`rh mcp --grant agent`'s `thread_post` when the scenario says so.
- `688-01-needs-you`: after a report with the project in front, the inbox lists it, and the
  log holds the desktop notice.
- `688-02-in-front`: with the Manager thread in front, the entry is gone.

## Locked-In Decisions
- **D1:** every manager record counts, as the harness defines the event, not reports alone. A
  message between turns is news too.
- **D2:** polling `thread_read` each second on the existing connection, as `fleet_events` is
  polled (AD-534), not a resource subscription. Zed's MCP client does not surface resource
  notifications here.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the manager posts a record while the Manager thread is not in front, the system shall show a desktop notice naming the manager and the record's first line | the log's notice line |
| REQ-002 | WHEN that happens, the rail's Needs you shall list the record until the Manager thread is in front | shots `688-01-needs-you`, `688-02-in-front` |
| REQ-003 | WHEN Marley connects to a harness whose thread already holds records, the system shall raise nothing for them | review of the diff |

## Phase Plan
- **P1 Plan:** this spec.
- **P2 Code:** `harness.rs`, `rail.rs` and the notice, then the gate.
- **P3 Test:** the scenario.
- **P4 Complete:** docs, ledger, close, archive, commit, push and install.
