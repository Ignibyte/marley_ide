---
pipeline_id: 10837fd5-6039-432f-9a82-8089a1bd40b3
ticket: docs/planning/tickets/open/TICKET-880-scenarios-keep-off-real-tmux.md
status: Phase 4 — Complete PASS
title: Scenario runs keep off the user's tmux sessions
type: bug
slice: The e2e harness (CONSTITUTION §7)
references: [docs/planning/tickets/closed/TICKET-877-scenarios-keep-off-real-sessions.md]
---

## Title
Every scenario run gets a tmux server of its own, so the run's Marley never lists, opens or types
into a tmux session of the user's.

## Scope
### In
- `script/e2e.sh`: `TMUX_TMPDIR` in the run's profile folder, `TMUX` and `TMUX_PANE`
  unset, the run's tmux server stopped in `cleanup`.

### Out (explicitly deferred)
- Re-measuring the stale scenarios (the regression at the end).

## Reference (§20)
N/A — Marley-specific: Marley's own e2e harness.

### Prior art
- TICKET-877 (`CLAUDE_CONFIG_DIR` per run), the exports beside it in `script/e2e.sh`.
- `543-remote-terminals-survive-a-drop.sh` already runs its tmux under a `TMUX_TMPDIR` of its
  own; `tmux_sessions.rs` runs plain `tmux`, so the environment decides the server.
- tmux(1): the socket is `$TMUX_TMPDIR/tmux-UID/default`; a Unix socket path is limited to
  about 104 bytes, which a scratchpad `$E2E_WORK` passes.
- The brain (consultation 0ae7f29f).

## UI proof
Scenario `script/e2e/880-no-real-tmux.sh` (sway): no `marley.tmux_hosts` setting, while the
user's tmux server has sessions.
- `880-01-rail`: the rail with the scratch project and no "this machine" tmux row (REQ-001).
- An `expect`: the run's `tmux list-sessions` finds no server (REQ-001).

## Locked-In Decisions
- D1 — The folder is `$E2E_PROFILE/tmux`: short, removed with the profile.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a scenario runs, its Marley and its shell shall reach only a tmux server of the run's own, never the user's. | `880-01-rail`, the `expect` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
