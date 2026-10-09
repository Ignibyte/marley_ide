---
pipeline_id: 75bcda2e-205e-4877-8288-4f4c3d0c9e05
ticket: docs/planning/tickets/open/TICKET-702-one-rail-row-for-a-thread-in-a-center-tab.md
status: Phase 4 — Complete PASS
title: One rail row for a thread in a center tab
type: bug
slice: the Marley layout (docs/marley/workbench-shell.md), found in #697's visual check
references:
  - docs/planning/pipeline/completed/697-an-agent-thread-in-a-center-tab.spec.md
---

## Title
While an agent thread sits in a center tab (#697), the rail lists it once, as its thread row, not
also as the tab's row; the thread row is the one marked while the tab is in front.

## Scope
### In
- `member_tabs` leaves a `ThreadTab` out of a project's center-tab rows (#674).
- With a `ThreadTab` the displayed workspace's active center item, the rail's focus marks its
  thread's row.
- Clicking the thread row brings the tab forward (already `show_thread` → `activate_for`).

- Found in Test and fixed here (it blocked this scenario's first keys): since #700 a Home or
  Rusty group made behind the shown project at start took the window's focus, so no key reached
  anything, a trust prompt's Enter included. The rail gives focus back once the group is made.

### Out (explicitly deferred)
- Anything about the tab itself.

## Reference (§20)
N/A — Marley-specific: the rail's rows (#674) and Marley's thread tab (#697). Zed's own sidebar
lists threads alone.

### Prior art
- **Marley:** `rail::member_tabs` (#674) skips terminals and Browser tabs, which have rows of their
  own, the same way; `active_rows` and `note_focus` decide the marked row; `Focus::thread` marks a
  thread row (today while the Agent Panel holds focus); `thread_tab::{ThreadTab, activate_for}`;
  `ThreadId::to_key_string`, the thread row's key.
- **Behavior maps:** nothing.

## UI proof
`script/e2e/702-one-rail-row-for-a-thread-in-a-center-tab.sh`, under `compositor sway`, with #697's
scripted agent.

Shots:
- `702-01-one-row`: a thread moved into a center tab: one row for it, the thread row, marked.
- `702-02-from-row`: the terminal shown, then the thread row clicked: the thread's tab in front
  again.

## Locked-In Decisions
- **D1:** the thread row stays (it carries the agent, its status and the unread mark), and the
  tab's row goes (Chad's default, 2026-10-09).
- **D2:** with the tab in front, `Focus::thread` is its thread's key and `Focus::tab` is empty, so
  the thread row is the one marked.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a thread is in a center tab, the rail shall list one row for it, its thread row. | Shot 702-01 |
| REQ-002 | WHILE that tab is the active center item, the rail shall mark the thread row. | Shot 702-01 |
| REQ-003 | WHEN the user clicks that thread row, the system shall bring its tab forward. | Shot 702-02 |
| REQ-004 | The gate shall pass. | `script/gates.sh --diff` exit 0 |
| REQ-005 | WHEN the rail makes a Home or Rusty group behind the shown workspace, the system shall leave the keyboard focus where it was. | The scenario's first keys: Enter on the trust prompt at start, then the palette; shot 702-00 (title without Restricted Mode) |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `rail.rs`, `thread_tab.rs`; a review of the diff; the gate.
- **P3 Test** — the visual check.
- **P4 Complete** — CHANGELOG, architecture docs, ledger, close, archive, commit.
