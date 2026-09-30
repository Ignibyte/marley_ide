# TICKET-610 — The host collector: a script Marley runs over SSH

- **Ticket:** LOCAL #610 (feature, prong 2 D20: the fleet, wave 1)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/queued/610-host-collector-over-ssh.spec.md
- **Source ticket:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones", after the fleet contract (docs/marley/fleet-contract.md, D20) was settled with him.
- **Status:** open

## Summary
Marley reads a host's CPU, memory, disk, network and running agents by running a POSIX sh script there over SSH, base64 in the command as #526's bootstrap does, polls only while shown, and joins the processes to the store's agents.

## Acceptance
A listed SSH host shows its resources and the agent processes running there; an unreachable one says so; this machine shows from the same script; a host name starting with a dash is refused.
