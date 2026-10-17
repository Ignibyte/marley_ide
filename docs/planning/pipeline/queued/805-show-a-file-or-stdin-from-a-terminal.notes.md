# Show a file or stdin from a terminal — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-805-show-a-file-or-stdin-from-a-terminal.md
- **Pipeline spec:** 805-show-a-file-or-stdin-from-a-terminal.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17 (see #803).
- **Discovery (drafted by /spec; re-verified at promotion):** `crates/marley_workbench/bin/marley-agent`;
  `crates/marley_mcp/src/agent_socket.rs` (the credential check); `crates/marley_agent/src/report.rs`
  (`Report`, `Release`, `Approve`); the shell integration's scripts in
  `crates/marley_terminal/shell_integration/`. Escape sequences were ruled out: vte drops unknown
  OSCs, and Marley's DCS hooks carry a nonce the shell keeps from programs, so a printed request
  could only be trusted with a user's confirmation.
- **Open for promotion:** the command's name (`show` may clash with a user's own command).
