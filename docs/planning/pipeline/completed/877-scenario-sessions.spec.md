---
pipeline_id: 9d0fd87e-cda7-492b-bd50-c594aa37cfbf
ticket: docs/planning/tickets/open/TICKET-877-scenarios-keep-off-real-sessions.md
status: Phase 4 — Complete PASS
title: Scenarios keep off the user's Claude Code sessions
type: bug
slice: The visual check's harness (script/e2e.sh)
references: [CONSTITUTION.md §7]
---

## Title
Every scenario run points `CLAUDE_CONFIG_DIR` at its own folder.

## Scope
### In
- `script/e2e.sh`: `export CLAUDE_CONFIG_DIR=$E2E_WORK/claude-config` beside `CODEX_HOME`.

### Out (explicitly deferred)
- Re-measuring the 13 scenarios' rail-header clicks: the regression at the end (§7).

## Reference (§20)
N/A — Marley-specific: the harness of Marley's visual checks; `cli_sessions.rs` reads
`CLAUDE_CONFIG_DIR`, else `~/.claude`.

### Prior art
- #725 gave each run its own `CODEX_HOME` for the same reason.

## UI proof
`#873`'s scenario's Threads step after the change: the Threads view lists no Claude Code session
(the user's were listed before).

## Locked-In Decisions
- D1 — The run's folder, not an empty temp elsewhere: a scenario that wants Claude Code sessions
  writes its own transcripts there.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a scenario runs, Marley shall list none of the user's Claude Code sessions. | `873-02-threads` re-run |

## Phase Plan
- **P1–P4** together: a one-line harness fix.
