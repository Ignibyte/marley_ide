---
pipeline_id: d0ec6465-8d89-48f6-849c-1f78b5af261a
ticket: docs/planning/tickets/closed/TICKET-865-an-agent-panel-beside-a-terminal.md
status: Phase 4 — Complete PASS
title: An agent beside a terminal, Apply, and a running block's progress line
type: feature
slice: The Marley look, W2 follow-up (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
A failed block offers Ask Marley, which opens Marley's own agent in the pane at the terminal's
right with the block and "Fix it" in its message box; when that thread's answer proposes a
command in a fenced block, the answer ends with "Apply: <command>", which types the command into
the terminal the block came from and runs it. A running block's header carries a thin moving line
along its top.

## Scope
### In
- `send_block.rs`: the Ask Marley button and `ask_beside`.
- `thread_tab.rs`: `open_beside` gives back the tab it opened.
- `beside.rs` (new): the link from a thread tab to its terminal, the proposed command, and the
  Apply chip.
- `crates/agent_ui/src/conversation_view/thread_view.rs` (Zed crate): `MarleyThreadLook` gains
  `extra_chip`, which Marley answers for the thread.
- `block_headers.rs`: the progress line.

### Out (explicitly deferred)
- An agent column drawn inside the terminal's own view: the thread opens in the pane beside it,
  as #717's Ask Beside does, a split the user can resize and close.

## Reference (§20)
Warp's agent panel on a block (docs/warp_architecture, the AgentTerminal board); upstream Zed's
thread view for the chips.

### Prior art
- `thread_tab::open_beside` (#717): a new thread with text in its message box, in the pane right
  of the active one.
- `send_block` (#826): a failed block's Ask an Agent and Explain.
- `block_headers::header` (#628) draws each block's header.
- #864's `MarleyThreadLook` chips.
- The brain (consultation 753bea9f): nothing on this seam.

## UI proof
Scenario `script/e2e/865-agent-beside-a-terminal-and-progress.sh` (sway): #825's scripted agent
answering with a fenced command; a terminal where `false` fails and `sleep` runs.
- `865-01-running`: `sleep 4` running: the header's moving line (REQ-003).
- `865-02-beside`: the failed block's Ask Marley, then Enter: the thread beside the terminal with
  the block and the answer, and Apply (REQ-001, REQ-002).
- `865-03-applied`: Apply clicked: the terminal ran the command (REQ-002).
- `865-04-day`: Marley Day, the thread beside the terminal (REQ-001).

## Locked-In Decisions
- D1 — Ask Marley fills the message box and sends nothing, as #864's chips and #717 do.
- D2 — The proposed command is the answer's first fenced block of one line; Apply types it and a
  Return into the terminal the block came from, and shows that terminal.
- D3 — The line is indeterminate: a 30% bar sliding across in 1.2 s, still and full width under
  reduced motion.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user clicks Ask Marley on a failed block, Marley shall open its agent in the pane right of the terminal with the block and "Fix it" in the message box, in Marley Night and Marley Day. | `865-02-beside`, `865-04-day` |
| REQ-002 | WHEN that thread's answer proposes a command in a fenced block, the answer shall end with Apply, which runs the command in the block's terminal. | `865-02-beside`, `865-03-applied` |
| REQ-003 | WHILE a block's command runs, its header shall carry a moving line along its top. | `865-01-running` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
