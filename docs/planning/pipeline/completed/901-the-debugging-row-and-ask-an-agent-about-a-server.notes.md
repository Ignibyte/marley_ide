# The bottom row while debugging, and Ask an agent about a language server — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-901-the-debugging-row-and-ask-an-agent-about-a-server.md
- **Pipeline spec:** 901-the-debugging-row-and-ask-an-agent-about-a-server.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "lets finish up the remaining tickets besides the mac issues".
- **Classification / tier:** feature, M. Marley crate plus `language_tools`.
- **Recall (§18.3):** #838 runs GDB's DAP in a run; #863's split note; F-claude-893 (a status or
  toolbar item never reads its workspace in `set_active_pane_item`). `Session::threads` and
  `stack_frames` need the session's context and may send requests, so the row caches what it
  draws, refreshed when the store, a session or the breakpoints notify. The brain had nothing.
- **Manifest:** Marley: `debug_row.rs` (new), `status_buttons.rs`, `send_selection.rs`,
  `search_looks.rs`, `marley_workbench.rs`, `Cargo.toml` (`debugger_ui`, `language_tools`). Zed:
  `crates/language_tools/src/lsp_button.rs` (a ledger row).

## Phase 2 — Code
- `debug_row.rs` (new), `status_buttons.rs` (the row after Debug), `send_selection.rs`
  (`ask_about_server`), `search_looks.rs`, `marley_workbench.rs`; `language_tools` a dependency.
  Zed: `lsp_button.rs`'s `MarleyServerAsk` and its entry (ledger row).
- Deviation: the controls dispatch by name, so `debugger_ui` is no dependency of Marley's.
- Gate: `just gate-diff` 17 passed, 0 failed.

## Phase 3 — Test
- Scenario under sway: a breakpoint on `hello.c`'s line 6, GDB stopped at `main`; the stand-in
  claude in the project's terminal.
- Probe: a rust-analyzer stand-in that exits at start is never listed in the servers menu (it has
  no server to list), so the stand-in became a small language server that answers `initialize` and
  reports `experimental/serverStatus` with health error and the board's message.
- `901-01-paused` (REQ-001): "Paused at hello.c:3", ▶ ↷ ↓ ↑ ■, `main`, "1 breakpoint", `3:0`.
- `901-02-stepped` (REQ-001): ↷ clicked: "Paused at hello.c:4", `4:0`.
- `901-05-day` (REQ-001): the paused row in Marley Day.
- `901-03-ask` (REQ-002): rust-analyzer's submenu: View Message, Ask an Agent, View Logs, Restart,
  Stop, its error and message.
- `901-04-asked` (REQ-002): the stand-in's prompt holds "The language server rust-analyzer
  reports: No Cargo.toml above src/main.rs. Why, and how do I fix it?" (the first run doubled the
  message's full stop; fixed, the gate and the run again).

## Phase 4 — Complete
- CHANGELOG, `marley_workbench.md` (the debugging row, Ask an Agent), look-plan W1.4, ledger row.
- Knowledge: L-claude-901 (a failed language server in a run).
- Brain: decided on consultation 42501ee30b2b44bb902acf52bfc55ca7.
