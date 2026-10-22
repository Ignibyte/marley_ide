# TICKET-880 — Scenario runs keep off the user's tmux sessions

- **Ticket:** LOCAL #880 (bug, the e2e harness)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** [880-scenarios-keep-off-real-tmux.spec.md](../../pipeline/completed/880-scenarios-keep-off-real-tmux.spec.md)
- **Source ticket:** found 2026-10-21 while probing #851 with `510-worktree-agents.sh`: the run's
  Marley listed "this machine · aic", a live agent session on the box's own tmux server, and its
  menu keys opened it. The line the scenario typed next went to the project panel, not the
  session, but the next stale scenario may not be so lucky. Same class as TICKET-877.
- **Size:** S
- **Status:** closed

## Summary

`script/e2e.sh` gives every run a tmux of its own: `TMUX_TMPDIR` in the run's profile folder (a
short path, under tmux's socket length limit), `TMUX` and `TMUX_PANE` unset, and the run's tmux
server stopped at cleanup. A scenario that wants tmux starts it in that same environment.

## Acceptance

A run with no `marley.tmux_hosts` setting lists none of the user's tmux sessions in the rail, and
its `tmux` finds no server until the run starts one.
