# TICKET-129 — the sessions sidebar for the pane world [M6]

- **Forge ticket:** #129 `7102ac4f-ca8e-44e5-a741-c26ef6a0a680` (feature/bugfix, M6; sprint #17)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `1363b465-4f04-4d45-9d56-9d4a174c0be8`
- **Pipeline doc:** ../../pipeline/active/sidebar-sessions.spec.md
- **Status:** closed

## Summary
`panes_of_kind(kind)` (workspace.rs, cov/MSI 100); the sidebar builds sessions from `panes_of_kind(Terminal)`
only, so a session-less files/code/git pane no longer shows as a phantom "terminal N". Deps #120 + M5 #109-112.

## Acceptance
panes_of_kind at cov/MSI 100; no phantom terminal in the sidebar (live capture); FULL gate GREEN. Full EARS
in the spec.
