# TICKET-163 — M10: persist the workspace shell

- **Forge ticket:** #163 `657b8a5d-eee0-4c0e-a04b-c43147c33727` (feature, M10; sprint #21)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `f2a6ffe7-388f-4632-b484-39b429f4a96d`
- **Pipeline doc:** ../../pipeline/active/shell-persist.spec.md
- **Status:** closed

## Summary
Projects + tabs survive a relaunch: a pure ShellLayout codec (embedding the per-grid kinds blobs) + a
workspace.shell setting + the guarded boot restore (respawn terminals per root, recreate cockpit/code tabs,
legacy fallback). Deps #150-156, #161/#162, #167.

## Acceptance
The codec + setting at cov/MSI 100; driven — a split + a cockpit tab survive a relaunch in the rail; gate
GREEN.
