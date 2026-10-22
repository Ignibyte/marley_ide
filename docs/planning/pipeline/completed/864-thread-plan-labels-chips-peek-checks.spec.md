---
pipeline_id: 9894debc-9a79-4e82-ae88-14126dc4dfd1
ticket: docs/planning/tickets/closed/TICKET-864-the-thread-plan-chips-terminal-peek-and-checks.md
status: Phase 4 — Complete PASS
title: The thread's plan, card labels, reply chips, terminal peek and checks
type: feature
slice: The Marley look, W2 follow-up (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
A thread tab's details panel shows the thread's plan as a checklist ("1 of 3"), a live peek of its
newest terminal call's last lines, and its checks (the errors in the files it changed); its edit
and run cards carry framed EDIT and RUN labels; and a finished answer ends with reply chips
(Continue, Explain, Undo the Edits) that put their words in the message box.

## Scope
### In
- `thread_tab.rs`: PLAN, ITS TERMINAL and CHECKS in the details panel (#825).
- `crates/agent_ui/src/conversation_view/thread_view.rs` (Zed crate): `MarleyThreadLook`, a
  global; with it set a tool call's header leads with EDIT or RUN, and the last finished answer
  ends with the chips.

### Out (explicitly deferred)
- The plan inside the answer's flow: Zed's own plan bar above the message box stays, and the
  details panel lists it whole.
- A task run's result among the checks: the thread's tool calls carry no task.

## Reference (§20)
Upstream Zed's agent thread view (`agent_ui::conversation_view::thread_view`) and
`acp_thread::AcpThread` (plan, entries, terminals).

### Prior art
- A probe with a scripted ACP agent: the edit and terminal cards draw unlabeled; the plan is a
  one-line bar over the message box; no chips; the details panel shows agent, context and files.
- `AcpThread::plan`, `ToolCallContent::Terminal`, `terminal::Terminal::last_n_non_empty_lines`.
- The brain (consultation d031d50c): nothing on this seam.

## UI proof
Scenario `script/e2e/864-thread-plan-labels-chips-peek-checks.sh` (sway): #825's set-up with a
scripted agent that sends a plan, an edit and a terminal call, and writes `src/main.rs` with an
error.
- `864-01-thread`: the EDIT and RUN labels, the chips under the answer, and the details panel's
  plan, terminal and checks (REQ-001 to REQ-004).
- `864-02-chip`: Explain clicked: its words in the message box (REQ-003).
- `864-03-day`: Marley Day (all).

## Locked-In Decisions
- D1 — The plan is the details panel's first section, every entry with its state; Zed's bar stays.
- D2 — A chip fills the message box and sends nothing: the user reads and presses Enter.
- D3 — Checks are the errors and warnings Zed's language servers report in the files the thread
  changed.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a thread has a plan, its details panel shall list the plan's entries with their state and how many are done. | `864-01-thread` |
| REQ-002 | A thread's edit and run cards shall lead with framed EDIT and RUN labels. | `864-01-thread` |
| REQ-003 | WHEN an answer has finished, it shall end with Continue, Explain and Undo the Edits chips, each of which puts its words in the message box. | `864-01-thread`, `864-02-chip` |
| REQ-004 | The details panel shall show the newest terminal call's last lines and the errors in the files the thread changed, in Marley Night and Marley Day. | `864-01-thread`, `864-03-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
