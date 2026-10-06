# TICKET-669 — Test runs list none of the machine's containers

- **Ticket:** LOCAL #669 (chore, the rail's port scan and the e2e harness)
- **Owner:** claude-opus-5-5, 2026-10-06 (found in #668's visual check)
- **Pipeline doc:** none yet
- **Source ticket:** TICKET-668's Phase 3 notes; #614 (container ports in the rail)
- **Status:** open

## Summary
In every e2e run the rail's Containers section lists the containers running on the machine:
`crates/marley_workbench/src/ports.rs` reads `/proc` for `docker-proxy` command lines and asks
the real `docker ps` (or `podman ps`), and a port no project's folder holds is listed apart, under
Containers. `Ports::proc_root` overrides `/proc` for unit tests only. Each such row has a Stop that
runs the engine's `stop`, so a scenario's click that lands on the wrong row could stop one of the
user's real containers, and their number moves every rail row below them: on 2026-10-06 the 13
containers on the dev box pushed #640's Harness rows out of the window and put its tooltip on
`:8143 docker`, and #614's own `APART_Y`, written when the box ran two containers, now puts its
refused-row click on one of the box's (it reaches the scenario's fake `docker`, first on the PATH,
but no longer tests the row it names). #668 kept a run from acting on the user's settings and keys; this is the same
promise for the machine's containers.

## Acceptance
A test run's rail lists no container the run did not start, and no Stop in it can reach one; #614's
scenario, which starts a stand-in `docker-proxy` and a fake `docker`, still shows and stops its
own; #640's rows show in the window again with its `BUILD_Y`.
