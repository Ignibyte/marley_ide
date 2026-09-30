# The host collector: a script Marley runs over SSH — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-610-host-collector-over-ssh.md
- **Pipeline spec:** 610-host-collector-over-ssh.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones", after the fleet contract (docs/marley/fleet-contract.md, D20) was settled with him.
- **Batch:** wave 1 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - BF-claude-ssh-command-leading-dash-host-is-option-smuggling-injection-001: parse_ssh_target refuses a leading dash, and ssh_command puts -- before the destination.
  - L-claude-584: a scenario runs an sshd of its own on localhost; MARLEY_SSH points Marley at a wrapper.
  - F-claude-521: poll only while a surface shows the data.
  - gate:22: process.rs is the workbench's one spawn module; process::output has no stdin, hence the script in the command.
- **Discovery:** one Explore sweep for the wave (2026-09-30): dock panels, center items, gpui
  drawing, marley_fleet, the MCP and HTTP clients, SSH seams, settings, and the Warp, Orca and Zed
  notes. The spec's Prior art cites what applies here; the Plan phase re-verifies each seam at
  promotion.
