# TICKET-087 — known-hosts config + a 'connect to…' palette action

- **Forge ticket:** #87 `4bdd76e9-296a-42af-a34f-7a9b125fa516` (feature, M3.A seq-5; BACKLOG — sprint pending)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `08963f50-f4b5-4d26-be9a-d6fd53b43b20`
- **Pipeline doc:** ../../pipeline/active/remote-hosts.spec.md
- **Milestone:** M3.A — The Remote Seam (tickets in the forge backlog tagged M3.A) — **the LAST M3.A ticket**
- **Status:** closed

## Summary
Named ssh hosts in the TOML settings (`[remote] hosts`) + a command-palette "connect: {name}" entry per
host that opens a remote pane (#84). PURE: `RemoteHost` + `remote_palette_actions`. SETTINGS:
`define_setting! RemoteHosts`. SHIM: a shared `open_remote_target` + connect commands (CommandId 1000+) +
the palette dispatch. cov/MSI 100 on the pure surface; the palette dispatch masked. Deps #83 + #84 + M1.B.

## Acceptance
remote_palette_actions at cov/MSI 100 (valid/dropped/empty/mixed); the RemoteHosts setting loads tolerantly;
a configured host → a palette connect entry → opens the pane (self-test, may be env-blocked); FULL gate
GREEN. Full EARS in the spec.
