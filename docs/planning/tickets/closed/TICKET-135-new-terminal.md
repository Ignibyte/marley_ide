# TICKET-135 — add a new terminal session (the "+") [M7 seq-4]

- **Forge ticket:** #135 `4f5d3a3a-b1d6-4f16-8d53-8ed924db986d` (feature, M7; sprint #18)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `cc5a58aa-ce4c-4606-8c0c-fd79e2fb3f01`
- **Pipeline doc:** ../../pipeline/active/new-terminal.spec.md
- **Status:** closed

## Summary
A "+" in the top bar spawns a new terminal session — extract `new_terminal_pane()` (reuse `split_focused` +
`spawn_session` + `persist_grid`), DRY the `split-pane` dispatch. The new terminal shows in the sidebar as
"terminal N". Shim-only. Deps #132 + M6 #120/#129.

## Acceptance
The "+" renders in the top bar (live capture); click spawns a terminal (code-reviewed; spawn path tested);
FULL gate GREEN. Full EARS in the spec.
