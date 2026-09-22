# TICKET-083 — parse an ssh target + build the ssh command

- **Forge ticket:** #83 `e7b17bf2-3978-4ff4-b6f9-c711ee5e00ee` (feature, M3.A seq-1; BACKLOG — sprint pending)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `2ef9bbd3-8c98-4eb7-9d74-26755c804a5a`
- **Pipeline doc:** ../../pipeline/active/ssh-target.spec.md
- **Milestone:** M3.A — The Remote Seam (the M3.A tickets are in the forge backlog; sprint-create was
  erroring forge-side when they were filed — group them once it recovers)
- **Status:** closed

## Summary
The pure foundation of the remote seam: a new `marley_remote` crate with `SshTarget` +
`parse_ssh_target` (`[user@]host[:port]`, bracketed IPv6 in / bare IPv6 out, all the rejects) +
`ssh_command` (the `ssh` argv — no shell, no injection). Marley spawns the user's ssh; ssh owns all
security. cov/MSI 100. Deps: none (pure).

## Acceptance
parse_ssh_target + ssh_command at cov/MSI 100 (all the split/reject/argv cases); FULL gate GREEN. Full
EARS in the spec.
