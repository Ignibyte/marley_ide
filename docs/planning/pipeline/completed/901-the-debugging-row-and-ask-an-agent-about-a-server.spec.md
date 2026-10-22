---
pipeline_id: 366c60db-bc3c-46c2-a169-2af2c9b89a97
ticket: docs/planning/tickets/open/TICKET-901-the-debugging-row-and-ask-an-agent-about-a-server.md
status: Phase 4 — Complete PASS
title: The bottom row while debugging, and Ask an agent about a language server
type: feature
slice: The Marley look, W1.4 follow-up (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
While a debug session is paused, the bottom row shows "Paused at main.rs:4" with Continue, Step
over, Step in, Step out and Stop, the paused frame's name, its line and column, and how many
breakpoints are set, as the BottomRow board draws it. A language server with a message (a
failed one, or one that reports its health) offers Ask an Agent in its submenu, which types the
server's name and message at the window's terminal agent.

## Scope
### In
- `debug_row.rs` (Marley, new): `DebugRow`, a status item after the Debug button, from the
  project's `DapStore` sessions and its `BreakpointStore`; its buttons dispatch `debugger_ui`'s
  own actions.
- `lsp_button.rs` (Zed): `MarleyServerAsk`, a global function the server submenu offers as Ask an
  Agent when the server has a message.
- `send_selection.rs` (Marley): `ask_about_server`, as #899's `ask_to_fix`.

### Out (explicitly deferred)
- The bar's yellow frame while paused, and the board's watched value (`name = "thread"`): the
  row is a status item, and Zed's bar draws its own frame.

## Reference (§20)
N/A — Marley-specific over Zed's debugger (`project::debugger`, `debugger_ui`'s actions) and its
language server menu (`language_tools::lsp_button`).

### Prior art
- `status_buttons::DebugButton` (#774) and the bar's left items; `debugger_ui`'s `Continue`,
  `StepOver`, `StepInto`, `StepOut` and `Stop`, registered on the workspace while a session's
  thread is stopped.
- `Session::any_stopped_thread`, `threads`, `stack_frames`; `BreakpointStore::all_source_breakpoints`.
- #838's GDB scenario (`gcc -g`, `.zed/debug.json`, GDB's own DAP); #899's `ask_to_fix` and its
  target picker.
- The brain (consultation 42501ee3): nothing on this seam.

## UI proof
Scenario `script/e2e/901-the-debugging-row-and-ask-an-agent-about-a-server.sh` (sway): #838's C
program under GDB with a breakpoint; `lsp.rust-analyzer.binary.path` pointing at a stand-in
language server that reports its health as an error, and a Rust file open; a stand-in `claude` in a terminal of the window.
- `901-01-paused`: the session paused: the row's pill, controls, frame, line and breakpoint count
  (REQ-001).
- `901-02-stepped`: Step over clicked: the row's line moved on (REQ-001).
- `901-03-ask`: the language servers menu, rust-analyzer's submenu with Ask an Agent (REQ-002).
- `901-04-asked`: it clicked: the stand-in's prompt holds the server's name and message
  (REQ-002).
- `901-05-day`: the paused row in Marley Day (REQ-001).

## Locked-In Decisions
- D1 — The row draws what the debugger reports and acts through `debugger_ui`'s own actions, so
  it does what the panel's buttons do.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a debug session's thread is stopped, the bottom row shall show where it stopped, Continue, Step over, Step in, Step out and Stop, the frame's name, its line and column and the breakpoint count, and its buttons shall act on the session, in Marley Night and Marley Day. | `901-01-paused`, `901-02-stepped`, `901-05-day` |
| REQ-002 | WHEN a language server reports a message, its submenu shall offer Ask an Agent, which types the server's name and message at the window's terminal agent. | `901-03-ask`, `901-04-asked` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
