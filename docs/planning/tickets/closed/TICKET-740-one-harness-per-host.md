# TICKET-740 — One harness per host

- **Ticket:** LOCAL #740 (feature, the control plane: the harness)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [740-one-harness-per-host.spec.md](../../pipeline/completed/740-one-harness-per-host.spec.md)
- **Source ticket:** Chad, 2026-10-10: "Bonus points if we can open on a remote via the harness."
  The harness's D164: Marley reaches each box over its own SSH connection. Plan:
  [agents-anywhere shelf](../../design-notes/agents-anywhere-2026-10-10.md).
- **Status:** closed

## Summary
Marley follows a harness on each host the settings list, beside the one it follows today: each with
a name, its host (none for this machine), its root and its `rh`, from which Marley builds
`ssh HOST rh --state ROOT mcp` and the seat and attach commands. The rail's Harness section lists each
host's sessions under its name, and the seat form asks which harness.

## Acceptance
With two harnesses set (one local, one a stand-in reached through a fake `ssh`), the rail lists
both under their names, and a seat made on the second shows there.
