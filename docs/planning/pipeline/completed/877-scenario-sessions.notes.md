# Scenarios keep off the user's Claude Code sessions — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-877-scenarios-keep-off-real-sessions.md

## Phase 1 — Plan
- **Found:** #645's scenario, run to see the Brain page before planning #875, clicked x 42 on the
  rail header for its Brain entry; since #873 that is the Threads icon, and its next click opened
  a Threads row: one of the user's Claude Code sessions, which the run's Marley resumed in a
  terminal (`claude --resume …`). It stopped at Claude Code's first-run theme question; the run's
  sway stopped it with everything else. No process stayed.
- **Cause:** `cli_sessions.rs` reads `CLAUDE_CONFIG_DIR`, else `~/.claude`; the harness set
  `CODEX_HOME` (#725) but not `CLAUDE_CONFIG_DIR`.

## Phase 2 — Code
- `script/e2e.sh`: `export CLAUDE_CONFIG_DIR=$E2E_WORK/claude-config`, with its reason.
- Gate: `--fast` (no Rust).

## Phase 3 — Test
- #873's scenario re-run (`873-02-threads`): the Threads view lists no Claude Code session (before
  the fix it listed the user's, `/srv/stacks/…` folders). REQ-001 met.
- Left as it is, and noted: one row still shows, a Rusty agent's thread from the user's database,
  which the harness copies into each run's profile on purpose (#668's copy). It opens an agent
  thread, not a terminal.

## Phase 4 — Complete
- Knowledge: F-claude-877-a-scenario-resumed-a-real-session-001, L-claude-877-a-rail-header-change-moves-old-clicks-001.
