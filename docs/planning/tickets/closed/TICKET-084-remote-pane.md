# TICKET-084 — open a remote (ssh) pane

- **Forge ticket:** #84 `791c60d2-dbea-46c8-8e46-25f796a5eb9c` (feature, M3.A seq-2; BACKLOG — sprint pending)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `a76fd453-2079-436c-a910-8dfa31ca3073`
- **Pipeline doc:** ../../pipeline/active/remote-pane.spec.md
- **Milestone:** M3.A — The Remote Seam (tickets in the forge backlog tagged M3.A; sprint-create erroring)
- **Status:** closed

## Summary
cmd-shift-o opens a pane running `ssh <target>` (composed at the prompt). Thread the ssh argv through the
existing PTY spawn: `SessionOptions.args` + `pty_os` passes it to `Shell::new` (was hardcoded empty). SHIM:
cmd-shift-o → parse_ssh_target (#83) → ssh_command → spawn_remote_session → a pane. Reuses #83's guarded
argv (no injection). cov/MSI 100 on the keymap; the spawn/dispatch masked + self-test-verified. Deps #83 +
#62 + #72 + #77.

## Acceptance
keymap cmd-shift-o→open-remote at cov/MSI 100; compose `localhost` → cmd-shift-o → an ssh pane opens
(self-test + `ps`); FULL gate GREEN. Full EARS in the spec.
