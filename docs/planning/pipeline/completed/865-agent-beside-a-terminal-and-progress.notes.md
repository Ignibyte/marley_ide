# An agent beside a terminal, Apply, and a running block's progress line — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-865-an-agent-panel-beside-a-terminal.md
- **Pipeline spec:** 865-agent-beside-a-terminal-and-progress.spec.md

## Phase 1 — Plan
- **Request:** the Queue's next row after #864.
- **Classification / tier:** feature, L. Marley crate, and #864's hook in the thread view grown by
  one function.
- **Recall (§18.3):** #717's `open_beside` already opens a thread at the active pane's right with
  text in its message box; #826's block buttons; #628's block headers are Marley's; #864's chips.
  The ticket's summary named a hook inside `TerminalView`; the pane beside it does the same with
  no new hook (Out). The brain had nothing.
- **Decisions:** D1–D3 in the spec.

## Phase 2 — Code
- `send_block.rs`: Ask Marley beside Ask an Agent and Explain on a failed block; `ask_beside`
  (deferred) puts "This command failed. Fix it, and give the command to run in a code block:" and
  the block in the message box of `assistant::ask_beside`, working in the block's folder, and links
  the tab to the terminal.
- `thread_tab::open_beside` gives back its tab; `assistant::ask_beside` picks the Marley entry's
  agent as `talk_to` does.
- `beside.rs`: `Links`; `apply_chip` finds the terminal of the asking thread view, reads the
  newest answer's first one-line fenced block, and draws "Apply: <command>", which types it with a
  Return into that terminal and shows it.
- `thread_view.rs`: `MarleyThreadLook` gains `extra_chip(view_id, thread)`, drawn after #864's
  chips; `search_looks` sets it to `apply_chip`.
- `block_headers.rs`: `progress_line`, a 2 px accent bar 30% wide sliding across in 1.2 s over a
  running block's header, still and half-bright under reduced motion.
- Found in Test and fixed: the first Ask Marley panicked, "cannot read ThreadView while it is
  already being updated": the chip runs inside the thread view's render and matched the thread by
  reading the view. It matches the view's id now
  (F-claude-865-a-hook-called-from-a-render-read-its-own-view-001).
- Gate: `just gate-diff` 17/17 green.

## Phase 3 — Test
Scenario `script/e2e/865-agent-beside-a-terminal-and-progress.sh` (sway, #825's scripted agent
answering with a fenced `echo fixed`).
- `865-01-running`: `sleep 4` running: the accent line along its header's top, "running" at the
  right (REQ-003).
- `865-02-beside`: after `false` and Ask Marley, then Return: the thread in the pane right of the
  terminal, the block in the prompt, "false always fails. Run this instead:", the fenced `echo
  fixed`, and the chips ending "Apply: echo fixed" (REQ-001, REQ-002).
- `865-03-applied`: the terminal shows `echo fixed` run and `fixed` printed (REQ-002).
- `865-04-day`: Marley Day, the thread beside the terminal with Apply (REQ-001).

## Phase 4 — Complete
- CHANGELOG (Added), `marley_workbench.md`, the ledger row, look-plan W2.
- Knowledge: F-claude-865-a-hook-called-from-a-render-read-its-own-view-001.
- Brain decision recorded on consultation 753bea9f.
