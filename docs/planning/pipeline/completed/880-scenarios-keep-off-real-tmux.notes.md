# Scenario runs keep off the user's tmux sessions — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-880-scenarios-keep-off-real-tmux.md
- **Pipeline spec:** 880-scenarios-keep-off-real-tmux.spec.md

## Phase 1 — Plan
- **Request:** found while probing #851: `510-worktree-agents.sh`'s Marley listed and opened
  "this machine · aic", a live agent's tmux session. Its typed line went to the project panel (no
  transcript but this session's holds it; the agent's prompt was empty), so nothing reached the
  agent. Taken ahead of #851.
- **Classification / tier:** bug, S. The harness only.
- **Recall (§18.3):** TICKET-877 and L-claude-877 (a run must not see the user's sessions);
  34 scenarios set `marley.tmux_hosts`, the older ones do not.
- **Decisions:** D1 in the spec.

### Design
- `script/e2e.sh`, beside #877's export: `export TMUX_TMPDIR=$E2E_PROFILE/tmux` (made), `unset
  TMUX TMUX_PANE`; in `cleanup`, before the profile goes, `tmux kill-server` under it.
- **File manifest:** `script/e2e.sh`; the scenario.

## Phase 2 — Code
- **Built:** `script/e2e.sh`: `TMUX_TMPDIR=$E2E_PROFILE/tmux` (made), `TMUX` and `TMUX_PANE`
  unset, beside #877's export; `cleanup` runs `tmux kill-server` (under the run's folder) before
  the profile goes.
- **Gate:** green, 17 of 17.
- **Review:** `543-remote-terminals-survive-a-drop.sh` sets its own `TMUX_TMPDIR` and keeps it;
  scenarios that start tmux in their shell inherit the run's folder, so they and the run's Marley
  agree on the server. The trap is set after the export, so `kill-server` never names the user's.


## Phase 3 — Test
- **Scenario:** `script/e2e/880-no-real-tmux.sh` (sway), with the user's server holding `aic`.
- `880-01-rail`: Default and the scratch project's bash row; no "this machine" tmux row, where
  #510's run showed "this machine · aic". REQ-001 met.
- `expect`: the run's `tmux list-sessions` finds no server: pass. The user's `aic` was still
  listed after the run.


## Phase 4 — Complete
- No CHANGELOG entry: the harness only, as #877.
- Knowledge: F-claude-880-a-scenario-opened-a-live-tmux-session-001,
  PR-claude-880-a-run-names-its-own-copy-of-every-user-store-001. Brain: `brain decide` on
  consultation 0ae7f29f.
- Ticket closed; pipeline archived.
