# TICKET-694 — The manager in the Agent Panel

- **Ticket:** LOCAL #694 (feature, phase 2 item 7 of the manager plan)
- **Owner:** ce546757-1075-466f-a3cf-b6696d2211c9
- **Pipeline doc:** [694-the-manager-in-the-agent-panel.spec.md](../../pipeline/completed/694-the-manager-in-the-agent-panel.spec.md)
- **Source ticket:** `docs/planning/intake/marley-agent-manager-foreman.md`, phase 2, item 7;
  the harness's TICKET-111 (`rh acp`, closed 2026-10-08) and its MREQ-008
- **Status:** closed

## Summary
When the harness Marley follows has a manager (a session labelled `role: manager`), Marley adds a
**Manager** entry to the Agent Panel. The entry runs `rh acp` through the command it follows the
harness with: the program and its arguments before `mcp`, then `acp`, so a remote root over SSH
gets `ssh HOST rh --state ROOT acp`. Its thread is the manager thread: the person's messages go
in, the manager's posts and reports stream out, and confirmations come up as permission requests.

## Acceptance
With `marley.harness_writes` on and a manager designated, the Agent Panel lists Manager, and a
message typed in its thread reaches the manager. Without a manager, or with the switch off, the
entry is gone.
