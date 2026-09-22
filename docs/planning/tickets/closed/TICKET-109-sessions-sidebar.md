# TICKET-109 — the sessions sidebar (persistent left) [M5 seq-3]

- **Forge ticket:** #109 `4015210d-112e-451b-8a0b-ae2035922d1b` (feature, M5 seq-3; sprint #16)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `ad701083-ef81-4427-86b3-d571ac75b3a2`
- **Pipeline doc:** ../../pipeline/active/sessions-sidebar.spec.md
- **Status:** closed

## Summary
A persistent left sidebar of sessions: NEW pure `sessions.rs` (`Session`/`SessionRow` + `session_rows`,
cov/MSI 100); the left dock lists the workspace panes (click → focus, active highlighted), above Files.
Promotes the ⌘⇧E Fleet. Deps #68 + #67 + seq-1.

## Acceptance
session_rows at cov/MSI 100 (icon/active/order); the left dock shows the sessions (live capture); FULL gate
GREEN. Full EARS in the spec.
